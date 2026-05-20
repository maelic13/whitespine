use std::sync::mpsc::channel;
use std::thread;

fn main() {
    use whitespine::engine::Engine;
    use whitespine::infra::capitalize_first_letter;
    use whitespine::uci_protocol::UciProtocol;

    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let (tx, rx) = channel();
    let mut engine = Engine::new(rx);
    let engine_thread = thread::spawn(move || engine.start());

    UciProtocol::new(tx).uci_loop();
    engine_thread.join().expect("Engine thread failed.");
}
