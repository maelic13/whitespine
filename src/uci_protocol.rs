use std::io;
use std::sync::{Arc, mpsc};

use crate::bench::DEFAULT_BENCH_DEPTH;
use crate::engine_command::{EngineCommand, EngineCommandQueue, EngineControl};
use crate::infra::capitalize_first_letter;
use crate::search_options::SearchOptions;

pub struct UciProtocol {
    search_options: SearchOptions,
    commands: EngineCommandQueue,
    control: Arc<EngineControl>,
}

impl UciProtocol {
    pub fn new(commands: EngineCommandQueue, control: Arc<EngineControl>) -> UciProtocol {
        UciProtocol {
            search_options: SearchOptions::default(),
            commands,
            control,
        }
    }

    pub fn uci_loop(&mut self) {
        loop {
            let mut input = String::new();
            let bytes_read = io::stdin()
                .read_line(&mut input)
                .expect("error: unable to read user input");
            if bytes_read == 0 {
                self.commands.push(EngineCommand::quit(0));
                break;
            }
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
                "bench" => self.bench(args),
                "ponderhit" => self.ponderhit(),
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
        if !self.control.is_searching() {
            let (ready_tx, ready_rx) = mpsc::channel();
            self.commands.push(EngineCommand::ready(ready_tx));
            let _ = ready_rx.recv();
        }
        println!("readyok");
    }

    fn quit(&self) {
        let epoch = self.control.request_quit();
        self.commands.push_priority(EngineCommand::quit(epoch));
    }

    fn go(&mut self, args: &[String]) {
        self.search_options.set_search_parameters(args);
        let epoch = self.control.start_replacing_search();
        self.commands
            .push(EngineCommand::go(self.search_options.clone(), epoch));
    }

    fn stop(&mut self) {
        let epoch = self.control.request_stop();
        self.commands.push(EngineCommand::stop(epoch));
    }

    fn set_option(&mut self, args: &[String]) {
        self.search_options.set_option(args);
        self.commands
            .push(EngineCommand::configure(self.search_options.clone()));
        self.search_options.engine.clear_hash = false;
    }

    fn new_game(&mut self) {
        self.search_options.reset();
        self.commands.push(EngineCommand::new_game());
    }

    fn position(&mut self, args: &[String]) {
        self.search_options.set_position(args);
    }

    fn bench(&mut self, args: &[String]) {
        let depth = args
            .first()
            .and_then(|depth| depth.parse::<u16>().ok())
            .unwrap_or(DEFAULT_BENCH_DEPTH);

        let epoch = self.control.start_replacing_search();
        self.commands.push(EngineCommand::stop(epoch));
        self.commands.push(EngineCommand::bench(
            depth,
            self.search_options.clone(),
            epoch,
        ));
    }

    fn ponderhit(&mut self) {
        self.control.request_ponderhit();
        self.commands.push(EngineCommand::ponderhit());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::board::{Color, Piece, Square};

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_string()).collect()
    }

    fn protocol_fixture() -> (UciProtocol, EngineCommandQueue) {
        let commands = EngineCommandQueue::default();
        let control = Arc::new(EngineControl::default());
        (UciProtocol::new(commands.clone(), control), commands)
    }

    #[test]
    fn go_sends_current_position_with_search_parameters() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.position(&args(&["startpos", "moves", "e2e4"]));
        protocol.go(&args(&["depth", "3", "nodes", "123"]));

        let command = commands.wait_pop();
        assert!(!command.stop);
        assert!(!command.quit);
        assert!(command.epoch > 0);
        assert_eq!(command.search_options.limits.depth, 3.0);
        assert_eq!(command.search_options.limits.nodes, 123);
        assert_eq!(
            command.search_options.position.board.side_to_move(),
            Color::Black
        );
        assert_eq!(
            command.search_options.position.board.piece_at(Square::E4),
            Some((Color::White, Piece::Pawn))
        );
    }

    #[test]
    fn setoption_sends_configure_and_clears_clear_hash_button_state() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.set_option(&args(&["name", "Hash", "value", "8"]));
        let hash_command = commands.wait_pop();
        assert_eq!(
            hash_command
                .configure
                .expect("hash command must configure engine")
                .engine
                .hash_mb,
            8
        );

        protocol.set_option(&args(&["name", "Clear", "Hash"]));
        let clear_command = commands.wait_pop();
        assert!(
            clear_command
                .configure
                .expect("clear hash command must configure engine")
                .engine
                .clear_hash
        );
        assert!(!protocol.search_options.engine.clear_hash);
    }

    #[test]
    fn bench_sends_stop_before_bench_command() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.bench(&args(&["5"]));

        let stop = commands.wait_pop();
        assert!(stop.stop);
        assert!(stop.bench_depth.is_none());

        let bench = commands.wait_pop();
        assert!(!bench.stop);
        assert_eq!(bench.bench_depth, Some(5));
        assert_eq!(bench.epoch, stop.epoch);
        assert_eq!(bench.search_options.engine.threads, 1);
    }

    #[test]
    fn newgame_resets_position_and_sends_marker_command() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.position(&args(&["startpos", "moves", "e2e4"]));
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E4),
            Some((Color::White, Piece::Pawn))
        );

        protocol.new_game();

        let command = commands.wait_pop();
        assert!(command.new_game);
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E2),
            Some((Color::White, Piece::Pawn))
        );
        assert_eq!(
            protocol.search_options.position.board.piece_at(Square::E4),
            None
        );
    }

    #[test]
    fn control_commands_are_forwarded_to_engine_thread() {
        let (mut protocol, commands) = protocol_fixture();

        protocol.stop();
        protocol.ponderhit();
        protocol.quit();

        let quit = commands.wait_pop();
        assert!(quit.stop);
        assert!(quit.quit);
        assert!(commands.wait_pop().stop);
        assert!(commands.wait_pop().ponderhit);
    }
}
