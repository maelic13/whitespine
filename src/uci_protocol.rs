use std::sync::mpsc::Sender;

use cmdr::*;

use crate::engine_command::EngineCommand;
use crate::search_options::SearchOptions;

pub struct UciProtocol {
    search_options: SearchOptions,
    sender: Sender<EngineCommand>,
}

impl UciProtocol {
    pub fn new(sender: Sender<EngineCommand>) -> UciProtocol {
        UciProtocol {
            search_options: SearchOptions::default(),
            sender,
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
        self.sender
            .send(EngineCommand::quit())
            .expect("Stop command could not be sent.");
        Ok(Action::Quit)
    }

    #[cmd]
    fn go(&mut self, args: &[String]) -> CommandResult {
        self.search_options.set_search_parameters(args);
        self.sender
            .send(EngineCommand::go(self.search_options.clone()))
            .expect("Go command could not be sent.");
        Ok(Action::Done)
    }

    #[cmd]
    fn stop(&mut self, _args: &[String]) -> CommandResult {
        self.sender
            .send(EngineCommand::stop())
            .expect("Stop command could not be sent.");
        Ok(Action::Done)
    }

    #[cmd]
    fn setoption(&self, _args: &[String]) -> CommandResult {
        Ok(Action::Done)
    }

    #[cmd]
    fn ucinewgame(&mut self, _args: &[String]) -> CommandResult {
        self.search_options.reset();
        Ok(Action::Done)
    }

    #[cmd]
    fn position(&mut self, args: &[String]) -> CommandResult {
        self.search_options.set_position(args);
        Ok(Action::Done)
    }
}
