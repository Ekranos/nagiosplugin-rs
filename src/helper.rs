use std::error::Error;
use std::process::exit;

use crate::State;

/// Runs the given closure and exits with a State::Critical after printing out
/// the error message if the Result contains an Err.
pub fn safe_run<F, E>(closure: F)
where
    F: Fn() -> Result<(), E>,
    E: Error + Sized,
{
    safe_run_with_state(closure, State::Critical);
}

/// Runs the given closure and exits with the given State after printing out
/// the error message if the Result contains an Err.
pub fn safe_run_with_state<F, E>(closure: F, error_state: State)
where
    F: Fn() -> Result<(), E>,
    E: Error + Sized,
{
    if let Err(e) = closure() {
        println!("{}: {}", error_state.to_string(), e);
        exit(error_state.exit_code());
    }
}
