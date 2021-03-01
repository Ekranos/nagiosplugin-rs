use std::error::Error;
use std::process::exit;

use crate::State;
use std::future::Future;

/// Runs the given closure and exits with a State::Critical after printing out
/// the error message if the Result contains an Err.
pub async fn safe_run<F, E, R>(closure: F)
where
    F: Fn() -> R,
    E: Error + Sized,
    R: Future<Output=Result<(), E>>,
{
    safe_run_with_state(closure, State::Critical).await;
}

/// Runs the given closure and exits with the given State after printing out
/// the error message if the Result contains an Err.
pub async fn safe_run_with_state<F, E, R>(closure: F, error_state: State)
where
    F: Fn() -> R,
    E: Error + Sized,
    R: Future<Output=Result<(), E>>,
{
    if let Err(e) = closure().await {
        println!("{}: {}", error_state.to_string(), e);
        exit(error_state.exit_code());
    }
}
