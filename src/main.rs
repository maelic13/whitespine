mod engine;
mod uci_protocol;

use uci_protocol::UciProtocol;

use cmdr::{cmd_loop, Result};

fn main() -> Result<()> {
    println!(
        "{} {} by {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );
    cmd_loop(&mut UciProtocol::default())?;
    Ok(())
}
