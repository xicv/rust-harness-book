#![forbid(unsafe_code)]

mod doctor;
mod render;

use std::env;
use std::process::ExitCode;

use doctor::ToolchainReport;
use harness_core::run_turn;

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();

    if matches!(arguments.as_slice(), [argument] if argument == "--doctor") {
        return run_doctor();
    }

    run_prompt(&arguments.join(" "))
}

fn run_prompt(prompt: &str) -> ExitCode {
    match run_turn(prompt) {
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

fn run_doctor() -> ExitCode {
    match ToolchainReport::collect() {
        Ok(report) => {
            print!("{report}");
            if report.is_match() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("doctor error: {error}");
            ExitCode::from(2)
        }
    }
}
