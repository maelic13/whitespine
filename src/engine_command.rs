use std::collections::VecDeque;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc::Sender,
};

use crate::search_options::SearchOptions;

#[derive(Default)]
pub struct EngineControl {
    stop: AtomicBool,
    quit: AtomicBool,
    ponderhit: AtomicBool,
    searching: AtomicBool,
    epoch: AtomicU64,
}

impl EngineControl {
    pub fn request_stop(&self) -> u64 {
        let epoch = self.next_epoch();
        self.stop.store(true, Ordering::Release);
        epoch
    }

    pub fn request_quit(&self) -> u64 {
        let epoch = self.next_epoch();
        self.quit.store(true, Ordering::Release);
        self.stop.store(true, Ordering::Release);
        epoch
    }

    pub fn request_ponderhit(&self) {
        self.ponderhit.store(true, Ordering::Release);
    }

    pub fn start_replacing_search(&self) -> u64 {
        let epoch = self.next_epoch();
        if self.searching.swap(true, Ordering::AcqRel) {
            self.stop.store(true, Ordering::Release);
        }
        epoch
    }

    pub fn prepare_search(&self, epoch: u64) -> bool {
        if epoch != 0 && self.current_epoch() != epoch {
            return false;
        }
        self.stop.store(false, Ordering::Release);
        self.ponderhit.store(false, Ordering::Release);
        self.searching.store(true, Ordering::Release);
        if epoch != 0 && self.current_epoch() != epoch {
            self.stop.store(true, Ordering::Release);
            self.searching.store(false, Ordering::Release);
            return false;
        }
        true
    }

    pub fn finish_search_if_current(&self, epoch: u64) {
        if epoch == 0 || self.current_epoch() == epoch {
            self.searching.store(false, Ordering::Release);
        }
    }

    pub fn current_epoch(&self) -> u64 {
        self.epoch.load(Ordering::Acquire)
    }

    pub fn is_searching(&self) -> bool {
        self.searching.load(Ordering::Acquire)
    }

    pub fn poll_search(&self) -> SearchControl {
        if self.quit.load(Ordering::Acquire) {
            SearchControl::Quit
        } else if self.stop.load(Ordering::Acquire) {
            SearchControl::Stop
        } else if self.ponderhit.swap(false, Ordering::AcqRel) {
            SearchControl::PonderHit
        } else {
            SearchControl::None
        }
    }

    fn next_epoch(&self) -> u64 {
        self.epoch.fetch_add(1, Ordering::AcqRel) + 1
    }
}

pub enum SearchControl {
    None,
    Stop,
    Quit,
    PonderHit,
}

#[derive(Clone, Default)]
pub struct EngineCommandQueue {
    inner: Arc<QueueInner>,
}

#[derive(Default)]
struct QueueInner {
    commands: Mutex<VecDeque<EngineCommand>>,
    available: Condvar,
}

impl EngineCommandQueue {
    pub fn push(&self, command: EngineCommand) {
        {
            let mut commands = self.inner.commands.lock().expect("command queue poisoned");
            commands.push_back(command);
        }
        self.inner.available.notify_one();
    }

    pub fn push_priority(&self, command: EngineCommand) {
        {
            let mut commands = self.inner.commands.lock().expect("command queue poisoned");
            commands.push_front(command);
        }
        self.inner.available.notify_one();
    }

    pub fn wait_pop(&self) -> EngineCommand {
        let mut commands = self.inner.commands.lock().expect("command queue poisoned");
        loop {
            if let Some(command) = commands.pop_front() {
                return command;
            }
            commands = self
                .inner
                .available
                .wait(commands)
                .expect("command queue poisoned");
        }
    }
}

pub struct EngineCommand {
    pub search_options: SearchOptions,
    pub stop: bool,
    pub quit: bool,
    pub bench_depth: Option<u16>,
    pub configure: Option<SearchOptions>,
    pub new_game: bool,
    pub ponderhit: bool,
    pub ready: Option<Sender<()>>,
    pub epoch: u64,
}

impl EngineCommand {
    pub fn go(options: SearchOptions, epoch: u64) -> EngineCommand {
        EngineCommand {
            search_options: options,
            stop: false,
            quit: false,
            bench_depth: None,
            configure: None,
            new_game: false,
            ponderhit: false,
            ready: None,
            epoch,
        }
    }

    pub fn stop(epoch: u64) -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: false,
            bench_depth: None,
            configure: None,
            new_game: false,
            ponderhit: false,
            ready: None,
            epoch,
        }
    }

    pub fn quit(epoch: u64) -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: true,
            quit: true,
            bench_depth: None,
            configure: None,
            new_game: false,
            ponderhit: false,
            ready: None,
            epoch,
        }
    }

    pub fn bench(depth: u16, options: SearchOptions, epoch: u64) -> EngineCommand {
        EngineCommand {
            search_options: options,
            stop: false,
            quit: false,
            bench_depth: Some(depth),
            configure: None,
            new_game: false,
            ponderhit: false,
            ready: None,
            epoch,
        }
    }

    pub fn configure(options: SearchOptions) -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            bench_depth: None,
            configure: Some(options),
            new_game: false,
            ponderhit: false,
            ready: None,
            epoch: 0,
        }
    }

    pub fn new_game() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            bench_depth: None,
            configure: None,
            new_game: true,
            ponderhit: false,
            ready: None,
            epoch: 0,
        }
    }

    pub fn ponderhit() -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            bench_depth: None,
            configure: None,
            new_game: false,
            ponderhit: true,
            ready: None,
            epoch: 0,
        }
    }

    pub fn ready(ready: Sender<()>) -> EngineCommand {
        EngineCommand {
            search_options: SearchOptions::default(),
            stop: false,
            quit: false,
            bench_depth: None,
            configure: None,
            new_game: false,
            ponderhit: false,
            ready: Some(ready),
            epoch: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_preparation_rejects_stale_epochs() {
        let control = EngineControl::default();
        let stale_epoch = control.start_replacing_search();
        let stop_epoch = control.request_stop();

        assert_ne!(stale_epoch, stop_epoch);
        assert!(!control.prepare_search(stale_epoch));
        assert!(control.is_searching());
        control.finish_search_if_current(stop_epoch);
        assert!(!control.is_searching());

        let current_epoch = control.start_replacing_search();
        assert!(control.prepare_search(current_epoch));
        assert!(control.is_searching());
        control.finish_search_if_current(current_epoch);
        assert!(!control.is_searching());
    }
}
