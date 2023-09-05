use std::sync::mpsc::channel;
use std::thread;

use crate::engine::Engine;
use crate::uci_protocol::UciProtocol;

mod engine;
mod engine_command;
mod heuristic;
mod piece_value;
mod search_options;
mod uci_protocol;

fn main() {
    println!(
        "{} {} by {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let (tx, rx) = channel();
    let mut engine = Engine::new(rx);
    thread::spawn(move || engine.start());

    UciProtocol::new(tx).uci_loop();
}
