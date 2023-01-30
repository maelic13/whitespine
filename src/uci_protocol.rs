use crate::engine::Engine;

use std::sync::mpsc::{channel, Sender};
use std::thread;

use cmdr::*;

pub struct UciProtocol {
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<bool>>,
}

impl UciProtocol {
    pub fn default() -> UciProtocol {
        UciProtocol {
            thread: None,
            sender: None,
        }
    }
}

#[cmdr]
impl UciProtocol {
    #[cmd]
    fn uci(&self, _args: &[String]) -> CommandResult {
        println!("id name {}", env!("CARGO_PKG_NAME"));
        println!("id author {}", env!("CARGO_PKG_AUTHORS").replace(':', ", "));
        println!("uciok");
        Ok(Action::Done)
    }

    #[cmd]
    fn isready(&self, _args: &[String]) -> CommandResult {
        println!("readyok");
        Ok(Action::Done)
    }

    #[cmd]
    fn quit(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Quit)
    }

    #[cmd]
    fn go(&mut self, _args: &[String]) -> CommandResult {
        if self.sender.is_some() {
            self.sender.to_owned().unwrap().send(false).unwrap_or(());
        }

        let (tx, rx) = channel();
        let mut engine = Engine::new(rx);
        self.thread = Some(thread::spawn(move || engine.search()));
        self.sender = Some(tx);

        Ok(Action::Done)
    }

    #[cmd]
    fn stop(&mut self, _args: &[String]) -> CommandResult {
        if self.sender.is_none() {
            return Ok(Action::Done);
        }

        self.sender.to_owned().unwrap().send(true).unwrap_or(());
        self.thread = None;
        self.sender = None;

        Ok(Action::Done)
    }

    #[cmd]
    fn ucinewgame(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Done)
    }

    #[cmd]
    fn position(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Done)
    }
}
