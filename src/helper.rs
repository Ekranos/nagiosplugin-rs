use std::error::Error;
use std::process::exit;

use crate::State;

/// Runs the given closure and exits with a State::Critical after printing out
/// the error message if the Result contains an Err.
pub fn safe_run<F: Fn() -> Result<(), Box<Error>>>(closure: F) {
    safe_run_with_state(closure, State::Critical);
}

/// Runs the given closure and exits with the given State after printing out
/// the error message if the Result contains an Err.
pub fn safe_run_with_state<F: Fn() -> Result<(), Box<Error>>>(closure: F, error_state: State) {
    if let Err(e) = closure() {
        println!("CRITICAL: {}", e.to_string());
        exit(error_state.exit_code());
    }
}
