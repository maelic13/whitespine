use std::sync::mpsc::Receiver;
use std::{thread, time};

use crate::engine_command::EngineCommand;
use crate::search_options::SearchOptions;

pub struct Engine {
    best_move: String,
    receiver: Receiver<EngineCommand>,
}

impl Engine {
    pub fn new(receiver: Receiver<EngineCommand>) -> Engine {
        Engine {
            best_move: "bestmove None".to_string(),
            receiver,
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = self.receiver.recv().unwrap();

            if command.quit {
                break;
            } else if command.stop {
                continue;
            }

            self.search(command.search_options);
        }
    }

    fn search(&mut self, search_options: SearchOptions) {
        println!("{:?}", search_options);

        let mut depth = 0;
        loop {
            depth += 1;
            println!(
                "info depth {} seldepth 2 score cp 37 nodes 194 nps 97000 time 2 pv d2d4 d7d5",
                depth
            );
            self.best_move = "bestmove d2d4".to_string();
            thread::sleep(time::Duration::from_secs(2));

            if self.check_stop() {
                println!("{}", self.best_move);
                break;
            }
        }
    }

    fn check_stop(&self) -> bool {
        self.receiver
            .try_recv()
            .unwrap_or(EngineCommand::default())
            .stop
    }
}
