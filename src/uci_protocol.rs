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

    pub fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            let input: Vec<String> = input.split_whitespace().map(str::to_string).collect();
            let command: &str = &input[0];
            let args: &[String] = &input[1..];

            match command {
                "uci" => self.uci(),
                "isready" => self.isready(),
                "go" => self.go(args),
                "stop" => self.stop(),
                "setoption" => self.setoption(args),
                "ucinewgame" => self.ucinewgame(),
                "position" => self.position(args),
                "quit" => {
                    self.quit();
                    break;
                }
                _ => continue,
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
