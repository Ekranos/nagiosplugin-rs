use std::error::Error;
use std::process::exit;

use crate::State;

pub fn safe_run<F: Fn() -> Result<(), Box<Error>>>(closure: F) {
    safe_run_with_state(closure, State::Critical);
}

pub fn safe_run_with_state<F: Fn() -> Result<(), Box<Error>>>(closure: F, error_state: State) {
    if let Err(e) = closure() {
        println!("CRITICAL: {}", e.to_string());
        exit(error_state.exit_code());
    }
}
