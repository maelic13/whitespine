use std::io::{self, Write};
use std::sync::Arc;

use crate::bench::BENCH_FENS;
use crate::board::Board;
use crate::engine_command::{EngineCommand, EngineCommandQueue, EngineControl, SearchControl};
use crate::search::{SearchEvent, SearchExit, SearchResult, Searcher};
use crate::search_options::SearchOptions;

pub struct Engine {
    commands: EngineCommandQueue,
    control: Arc<EngineControl>,
    searcher: Searcher,
}

impl Engine {
    pub fn new(commands: EngineCommandQueue, control: Arc<EngineControl>) -> Engine {
        Engine {
            commands,
            control,
            searcher: Searcher::default(),
        }
    }

    pub fn start(&mut self) {
        loop {
            let command = self.commands.wait_pop();

            if self.handle_control_command(&command) {
                break;
            }
            if command.configure.is_some()
                || command.new_game
                || command.ponderhit
                || command.ready.is_some()
            {
                continue;
            }
            if command.stop {
                continue;
            }
            if let Some(depth) = command.bench_depth {
                if self.run_bench(depth, &command.search_options, command.epoch) == SearchExit::Quit
                {
                    break;
                }
                continue;
            }

            if command.epoch != 0 && self.control.current_epoch() != command.epoch {
                continue;
            }
            if !self.control.prepare_search(command.epoch) {
                continue;
            }
            let result = self.search(command.search_options.clone(), true, command.epoch);
            self.control.finish_search_if_current(command.epoch);
            print_bestmove(&result);
            if result.exit == SearchExit::Quit {
                break;
            }
        }
    }

    fn handle_control_command(&mut self, command: &EngineCommand) -> bool {
        if let Some(options) = &command.configure {
            self.searcher.configure(options);
        }
        if command.new_game {
            self.searcher.new_game();
        }
        if command.stop && (command.epoch == 0 || self.control.current_epoch() == command.epoch) {
            self.control.finish_search_if_current(command.epoch);
        }
        if let Some(ready) = &command.ready {
            let _ = ready.send(());
        }
        command.quit
    }

    fn search(&mut self, options: SearchOptions, emit_info: bool, epoch: u64) -> SearchResult {
        let control = Arc::clone(&self.control);
        self.searcher.search(
            options.position.board.clone(),
            &options,
            emit_info,
            || match control.poll_search() {
                SearchControl::Quit => SearchEvent::Quit,
                SearchControl::Stop if epoch == 0 || control.current_epoch() != epoch => {
                    SearchEvent::Stop
                }
                SearchControl::Stop => SearchEvent::Stop,
                SearchControl::PonderHit => SearchEvent::PonderHit,
                SearchControl::None => SearchEvent::None,
            },
        )
    }

    fn run_bench(&mut self, depth: u16, base_options: &SearchOptions, epoch: u64) -> SearchExit {
        if epoch != 0 && self.control.current_epoch() != epoch {
            return SearchExit::Stop;
        }
        if !self.control.prepare_search(epoch) {
            return SearchExit::Stop;
        }
        let mut total_nodes = 0u64;
        let mut total_ms = 0u128;

        println!();
        for (index, fen) in BENCH_FENS.iter().enumerate() {
            if epoch != 0 && self.control.current_epoch() != epoch {
                self.control.finish_search_if_current(epoch);
                return SearchExit::Stop;
            }
            let board = match Board::from_fen(fen) {
                Ok(board) => board,
                Err(err) => {
                    println!(
                        "info string bench position {} failed to parse: {}",
                        index + 1,
                        err
                    );
                    self.control.finish_search_if_current(epoch);
                    return SearchExit::Stop;
                }
            };
            let mut options = SearchOptions::default();
            options.position.board = board;
            options.limits.depth = depth as f64;
            options.engine = base_options.engine;

            let result = self.search(options, false, epoch);
            total_nodes += result.nodes;
            total_ms += result.elapsed_ms;
            let nps = if result.elapsed_ms > 0 {
                result.nodes as u128 * 1000 / result.elapsed_ms
            } else {
                result.nodes as u128
            };

            println!(
                "bench {}/{}  depth {}  score {}  nodes {}  time {}ms  nps {}",
                index + 1,
                BENCH_FENS.len(),
                result.depth,
                result.score,
                result.nodes,
                result.elapsed_ms,
                nps
            );
            flush_stdout();

            if result.exit == SearchExit::Quit {
                self.control.finish_search_if_current(epoch);
                return SearchExit::Quit;
            }
            if epoch != 0 && self.control.current_epoch() != epoch {
                self.control.finish_search_if_current(epoch);
                return SearchExit::Stop;
            }
        }

        let total_nps = if total_ms > 0 {
            total_nodes as u128 * 1000 / total_ms
        } else {
            total_nodes as u128
        };
        println!(
            "\n=========================\nTotal time (ms) : {}\nNodes searched  : {}\nNodes/second    : {}",
            total_ms, total_nodes, total_nps
        );
        flush_stdout();

        self.control.finish_search_if_current(epoch);
        SearchExit::Stop
    }
}

fn print_bestmove(result: &SearchResult) {
    if result.pondermove.is_null() {
        println!("bestmove {}", result.bestmove);
    } else {
        println!("bestmove {} ponder {}", result.bestmove, result.pondermove);
    }
}

fn flush_stdout() {
    io::stdout().flush().expect("stdout flush failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_fixture() -> (Engine, EngineCommandQueue, Arc<EngineControl>) {
        let commands = EngineCommandQueue::default();
        let control = Arc::new(EngineControl::default());
        (
            Engine::new(commands.clone(), Arc::clone(&control)),
            commands,
            control,
        )
    }

    #[test]
    fn handle_control_command_returns_true_only_for_quit() {
        let (mut engine, _commands, control) = engine_fixture();

        assert!(!engine.handle_control_command(&EngineCommand::stop(control.request_stop())));

        let mut options = SearchOptions::default();
        options.engine.hash_mb = 1;
        options.engine.clear_hash = true;
        assert!(!engine.handle_control_command(&EngineCommand::configure(options)));
        assert!(!engine.handle_control_command(&EngineCommand::new_game()));
        assert!(!engine.handle_control_command(&EngineCommand::ponderhit()));

        assert!(engine.handle_control_command(&EngineCommand::quit(control.request_quit())));
    }

    #[test]
    fn search_converts_queued_stop_command_into_search_stop() {
        let (mut engine, _commands, control) = engine_fixture();
        control.request_stop();
        let mut options = SearchOptions::default();
        options.limits.depth = 99.0;

        let result = engine.search(options, false, 0);

        assert_eq!(result.exit, SearchExit::Stop);
        assert!(result.nodes >= 512, "nodes: {}", result.nodes);
        assert!(result.depth < 99);
    }

    #[test]
    fn search_converts_queued_quit_command_into_search_quit() {
        let (mut engine, _commands, control) = engine_fixture();
        control.request_quit();
        let mut options = SearchOptions::default();
        options.limits.depth = 99.0;

        let result = engine.search(options, false, 0);

        assert_eq!(result.exit, SearchExit::Quit);
        assert!(result.nodes >= 512, "nodes: {}", result.nodes);
        assert!(result.depth < 99);
    }
}
