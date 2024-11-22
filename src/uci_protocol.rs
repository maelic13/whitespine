use std::io;
use std::sync::mpsc::Sender;

use crate::engine_command::EngineCommand;
use crate::infra::capitalize_first_letter;
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
            if input.is_empty() {
                continue;
            }
            let command: &str = &input[0];
            let args: &[String] = &input[1..];

            match command {
                "uci" => self.uci(),
                "isready" => self.is_ready(),
                "go" => self.go(args),
                "stop" => self.stop(),
                "setoption" => self.set_option(args),
                "ucinewgame" => self.new_game(),
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
        println!(
            "id name {} {}",
            capitalize_first_letter(env!("CARGO_PKG_NAME")),
            env!("CARGO_PKG_VERSION")
        );
        println!("id author {}", env!("CARGO_PKG_AUTHORS").replace(':', ", "));
        for option in SearchOptions::get_uci_options() {
            println!("{}", option);
        }
        println!("uciok");
    }

    fn is_ready(&self) {
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

    fn set_option(&mut self, args: &[String]) {
        self.search_options.set_option(args);
    }

    fn new_game(&mut self) {
        self.search_options.reset();
    }

    fn position(&mut self, args: &[String]) {
        self.search_options.set_position(args);
    }
}
