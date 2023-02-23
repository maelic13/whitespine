use std::io;
use std::sync::mpsc::Sender;

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

impl UciProtocol {
    pub fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            let input: Vec<String> = input.split_whitespace().map(str::to_string).collect();
            let command: &String = &input[0];
            let args: &[String] = &input[1..];

            if command == "uci" {
                self.uci();
            }
            if command == "isready" {
                self.isready();
            }
            if command == "quit" {
                self.quit();
                break;
            }
            if command == "go" {
                self.go(args);
            }
            if command == "stop" {
                self.stop();
            }
            if command == "setoption" {
                self.setoption(args);
            }
            if command == "ucinewgame" {
                self.isready();
            }
            if command == "position" {
                self.position(args);
            }
        }
    }

    fn uci(&self) {
        println!("id name {}", env!("CARGO_PKG_NAME"));
        println!("id author {}", env!("CARGO_PKG_AUTHORS").replace(':', ", "));
        println!("uciok");
    }

    fn isready(&self) {
        println!("readyok");
    }

    fn quit(&self) {
        self.sender
            .send(EngineCommand::quit())
            .expect("Stop command could not be sent.");
    }

    fn go(&mut self, args: &[String]) {
        self.search_options.set_search_parameters(args);
        self.sender
            .send(EngineCommand::go(self.search_options.clone()))
            .expect("Go command could not be sent.");
    }

    fn stop(&mut self) {
        self.sender
            .send(EngineCommand::stop())
            .expect("Stop command could not be sent.");
    }

    fn setoption(&self, _args: &[String]) {
        println!("No engine options currently supported.");
    }

    fn ucinewgame(&mut self) {
        self.search_options.reset();
    }

    fn position(&mut self, args: &[String]) {
        self.search_options.set_position(args);
    }
}
