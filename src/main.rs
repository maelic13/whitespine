use std::sync::mpsc::channel;
use std::thread;

use crate::engine::Engine;
use crate::infra::capitalize_first_letter;
use crate::uci_protocol::UciProtocol;

mod engine;
mod engine_command;
mod heuristic;
mod infra;
mod piece_value;
mod search_options;
mod uci_protocol;

fn main() {
    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let (tx, rx) = channel();
    let mut engine = Engine::new(rx);
    thread::spawn(move || engine.start());

    UciProtocol::new(tx).uci_loop();
}
