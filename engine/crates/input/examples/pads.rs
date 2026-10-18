//! Print how each connected controller resolves through `input::open_gilrs`
//! (card PADM) — the same log line the game prints at startup, without a
//! window: `cargo run --release -p input --example pads`.

fn main() {
    match input::open_gilrs() {
        Ok(g) => {
            if let Some(w) = input::log_pads(&g) {
                eprintln!("(the game shows this on screen)\n{w}");
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("gilrs unavailable: {e}");
            std::process::exit(2);
        }
    }
}
