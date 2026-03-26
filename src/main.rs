use std::sync::mpsc::channel;
use std::thread;

use crate::engine::Engine;
use crate::infra::capitalize_first_letter;
use crate::uci_protocol::UciProtocol;
use crate::version::display_version;

mod engine;
mod engine_command;
mod heuristic;
mod infra;
mod piece_value;
mod search_options;
mod uci_protocol;
mod version;

fn main() {
    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        display_version(),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let (tx, rx) = channel();
    let mut engine = Engine::new(rx);
    thread::spawn(move || engine.start());

    UciProtocol::new(tx).uci_loop();
}
