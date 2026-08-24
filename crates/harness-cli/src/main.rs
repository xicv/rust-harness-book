#![forbid(unsafe_code)]

mod render;

use std::env;
use std::process::ExitCode;

use harness_core::run_turn;

fn main() -> ExitCode {
    let prompt = env::args().skip(1).collect::<Vec<_>>().join(" ");

    match run_turn(&prompt) {
        Ok(outcome) => {
            for event in outcome.events() {
                println!("{}", render::render_event(event));
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
