use std::error::Error;
use std::process::exit;

use crate::State;
use std::future::Future;

/// Runs the given closure and exits with a State::Critical after printing out
/// the error message if the Result contains an Err.
pub async fn safe_run<F>(future: F)
where
    F: Future<Output = Result<(), Box<dyn Error>>>,
{
    safe_run_with_state(future, State::Critical).await;
}

/// Runs the given closure and exits with a State::Critical after printing out
/// the error message if the Result contains an Err.
pub fn safe_run_sync<F>(f: F)
    where
        F: FnOnce() -> Result<(), Box<dyn Error>>,
{
    safe_run_with_state_sync(f, State::Critical);
}

/// Runs the given closure and exits with the given State after printing out
/// the error message if the Result contains an Err.
pub async fn safe_run_with_state<F>(future: F, error_state: State)
where
    F: Future<Output = Result<(), Box<dyn Error>>>,
{
    if let Err(e) = future.await {
        println!("{}: {}", error_state.to_string(), e);
        exit(error_state.exit_code());
    }
}

/// Runs the given closure and exits with the given State after printing out
/// the error message if the Result contains an Err.
pub fn safe_run_with_state_sync<F>(f: F, error_state: State)
    where
        F: FnOnce() -> Result<(), Box<dyn Error>>,
{
    if let Err(e) = f() {
        println!("{}: {}", error_state.to_string(), e);
        exit(error_state.exit_code());
    }
}
