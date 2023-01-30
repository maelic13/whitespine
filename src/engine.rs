use std::sync::mpsc::{Receiver, TryRecvError};
use std::{thread, time};

pub struct Engine {
    best_move: String,
    receiver: Receiver<bool>,
}

impl Engine {
    pub fn new(receiver: Receiver<bool>) -> Engine {
        Engine {
            best_move: "bestmove None".to_string(),
            receiver,
        }
    }

    pub fn search(&mut self) {
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
                break;
            }
        }
    }

    fn check_stop(&self) -> bool {
        match self.receiver.try_recv() {
            Err(TryRecvError::Empty) => false,
            Ok(true) | Err(TryRecvError::Disconnected) => {
                println!("{}", self.best_move);
                return true;
            }
            Ok(false) => {
                return true;
            }
        }
    }
}
