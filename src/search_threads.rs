use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    mpsc::{self, Sender},
};
use std::thread::{self, JoinHandle};

use crate::board::{Board, Move};
use crate::search::{SearchEvent, SearchResult, Searcher};
use crate::search_options::{EngineOptions, SearchLimits};
use crate::tt::TranspositionTable;

pub(crate) const STOP_NONE: u8 = 0;
pub(crate) const STOP_SEARCH: u8 = 1;
pub(crate) const STOP_QUIT: u8 = 2;

pub(crate) struct WorkerJob {
    pub root: Board,
    pub root_moves: Arc<[Move]>,
    pub limits: SearchLimits,
    pub engine_options: EngineOptions,
    pub tt: TranspositionTable,
    pub hash_mb: usize,
    pub root_move_offset: usize,
    pub stop_state: Arc<AtomicU8>,
    pub result_tx: Sender<SearchResult>,
}

enum WorkerMessage {
    Search(WorkerJob),
    NewGame,
    Shutdown,
}

struct SearchWorkerHandle {
    sender: Sender<WorkerMessage>,
    handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(crate) struct WorkerPool {
    workers: Vec<SearchWorkerHandle>,
}

impl WorkerPool {
    pub(crate) fn set_helper_count(&mut self, helper_count: usize) {
        while self.workers.len() > helper_count {
            if let Some(mut worker) = self.workers.pop() {
                let _ = worker.sender.send(WorkerMessage::Shutdown);
                if let Some(handle) = worker.handle.take() {
                    let _ = handle.join();
                }
            }
        }
        while self.workers.len() < helper_count {
            if let Some(worker) = spawn_search_worker(self.workers.len()) {
                self.workers.push(worker);
            } else {
                println!(
                    "info string Unable to create helper search thread {}; using {} search threads.",
                    self.workers.len() + 1,
                    self.workers.len() + 1
                );
                break;
            }
        }
    }

    pub(crate) fn new_game(&self) {
        for worker in &self.workers {
            let _ = worker.sender.send(WorkerMessage::NewGame);
        }
    }

    pub(crate) fn send_search(&self, index: usize, job: WorkerJob) -> bool {
        self.workers
            .get(index)
            .is_some_and(|worker| worker.sender.send(WorkerMessage::Search(job)).is_ok())
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.set_helper_count(0);
    }
}

fn spawn_search_worker(index: usize) -> Option<SearchWorkerHandle> {
    let (sender, receiver) = mpsc::channel();
    let handle = thread::Builder::new()
        .name(format!("whitespine-search-{index}"))
        .spawn(move || {
            let mut worker = Searcher::worker_default();
            while let Ok(message) = receiver.recv() {
                match message {
                    WorkerMessage::Search(job) => {
                        let result_tx = job.result_tx.clone();
                        let stop_state = Arc::clone(&job.stop_state);
                        let mut helper_poll = || match stop_state.load(Ordering::Relaxed) {
                            STOP_QUIT => SearchEvent::Quit,
                            STOP_SEARCH => SearchEvent::Stop,
                            _ => SearchEvent::None,
                        };
                        let result = worker.run_worker_job(job, &mut helper_poll);
                        let _ = result_tx.send(result);
                    }
                    WorkerMessage::NewGame => worker.reset_worker_state_for_new_game(),
                    WorkerMessage::Shutdown => break,
                }
            }
        })
        .ok()?;
    Some(SearchWorkerHandle {
        sender,
        handle: Some(handle),
    })
}
