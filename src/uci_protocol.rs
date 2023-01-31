use std::sync::mpsc::{channel, Sender};
use std::thread;

use cmdr::*;

use crate::engine::Engine;
use crate::uci_options::UciOptions;

pub struct UciProtocol {
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<bool>>,
    options: UciOptions,
}

impl UciProtocol {
    pub fn default() -> UciProtocol {
        UciProtocol {
            thread: None,
            sender: None,
            options: UciOptions::default(),
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
        let options = self.options.clone();

        self.thread = Some(thread::spawn(move || engine.search(options)));
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
    fn setoption(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Done)
    }

    #[cmd]
    fn ucinewgame(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Done)
    }

    #[cmd]
    fn position(&mut self, args: &[String]) -> CommandResult {
        self.options.reset_position();

        if args[0] == "fen" {
            let mut fen = args[1].to_string();
            for partial in args[2..].as_ref() {
                if partial == "moves" {
                    break;
                }
                fen += &*String::from(" ");
                fen += partial;
            }
            self.options.fen = fen;
        }

        let moves_start_index = args
            .iter()
            .position(|r| r == "moves")
            .unwrap_or(args.len() - 1)
            + 1;
        self.options.played_moves = args[moves_start_index..].to_vec();
        Ok(Action::Done)
    }
}
