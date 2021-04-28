use std::fmt::Debug;

use crate::{Resource, ServiceState};

pub struct Runner<E> {
    on_error: Option<Box<dyn FnOnce(&E) -> (ServiceState, E)>>,
}

impl<E: Debug> Runner<E> {
    pub fn new() -> Self {
        Self { on_error: None }
    }

    pub fn on_error(mut self, f: impl FnOnce(&E) -> (ServiceState, E) + 'static) -> Self {
        self.on_error = Some(Box::new(f));
        self
    }

    /// This will run either the default `on_error` handler or the one specified by calling
    /// [on_error]. It will use the given ([ServiceState], message) tuple and exit with these.
    pub fn safe_run(self, f: impl FnOnce() -> Result<Resource, E>) -> RunnerResult<E> {
        match f() {
            Ok(resource) => RunnerResult::Ok(resource),
            Err(err) => {
                let (state, msg) = self
                    .on_error
                    .map(|f| f(&err))
                    .unwrap_or_else(|| (ServiceState::Critical, err));

                RunnerResult::Err(state, msg)
            }
        }
    }
}

pub enum RunnerResult<E> {
    Ok(Resource),
    Err(ServiceState, E),
}

impl<E: Debug> RunnerResult<E> {
    pub fn print_and_exit(self) -> ! {
        match self {
            RunnerResult::Ok(resource) => resource.print_and_exit(),
            RunnerResult::Err(state, msg) => {
                println!("{}: {:?}", state, msg);
                std::process::exit(state.exit_code());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    #[error("woops")]
    struct EmptyError;

    #[test]
    fn test_runner_ok() {
        let result = Runner::<EmptyError>::new()
            .on_error(|_| {
                assert!(false);
                (ServiceState::Unknown, EmptyError {})
            })
            .safe_run(|| Ok(Resource::new("test")));

        matches!(result, RunnerResult::Ok(_));
    }

    #[test]
    fn test_runner_error() {
        let result = Runner::<EmptyError>::new()
            .on_error(|_| (ServiceState::Unknown, EmptyError {}))
            .safe_run(|| Err(EmptyError {}));

        matches!(result, RunnerResult::Err(_, _));
    }
}
