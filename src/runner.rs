use std::fmt::Debug;

use crate::{Resource, ServiceState};

/// The runner is a helper to run a function that returns a [Result] with a [Resource] and maps the error
/// case to a [ServiceState] and a message. This is to avoid boilerplate in every plugin.
///
/// ## Example
///
/// ```no_run
/// use std::error::Error;
///
/// use nagiosplugin::{Metric, Resource, Runner, TriggerIfValue};
///
/// fn main() {
///     Runner::new().safe_run(do_check).print_and_exit()
/// }
///
/// fn do_check() -> Result<Resource, Box<dyn Error>> {
///    // The first metric will not issue an alarm, the second one will.
///    let resource = Resource::new("foo")
///         .with_description("This is a simple test plugin")
///         .with_result(Metric::new("test", 15).with_thresholds(20, 50, TriggerIfValue::Greater))
///         .with_result(Metric::new("alerting", 42).with_thresholds(40, 50, TriggerIfValue::Greater));
///
///     Ok(resource)
/// }
/// ```

pub struct Runner<E> {
    #[allow(clippy::type_complexity)]
    on_error: Option<Box<dyn FnOnce(E) -> (ServiceState, E)>>,
}

impl<E: Debug> Runner<E> {
    pub fn new() -> Self {
        Self { on_error: None }
    }

    /// This will set a custom error handler. The is mostly useful to provide better plugin output.
    pub fn on_error(mut self, f: impl FnOnce(E) -> (ServiceState, E) + 'static) -> Self {
        self.on_error = Some(Box::new(f));
        self
    }

    /// This will run either the default `on_error` handler or the one specified by calling
    /// [Self::on_error]. It will use the given ([ServiceState], message) tuple and exit with these.
    pub fn safe_run(self, f: impl FnOnce() -> Result<Resource, E>) -> RunnerResult<E> {
        match f() {
            Ok(resource) => RunnerResult::Ok(resource),
            Err(err) => {
                let (state, msg) = match self.on_error {
                    None => (ServiceState::Critical, err),
                    Some(f) => f(err),
                };

                RunnerResult::Err(state, msg)
            }
        }
    }
}

impl<E: Debug> Default for Runner<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of a runner execution.
pub enum RunnerResult<E> {
    /// The run was successful and it contains the returned [Resource].
    Ok(Resource),
    /// The run was not successful and it contains the [ServiceState] and the error.
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
                (ServiceState::Unknown, EmptyError)
            })
            .safe_run(|| Ok(Resource::new("test")));

        matches!(result, RunnerResult::Ok(_));
    }

    #[test]
    fn test_runner_error() {
        let result = Runner::<EmptyError>::new()
            .on_error(|_| (ServiceState::Unknown, EmptyError))
            .safe_run(|| Err(EmptyError {}));

        matches!(result, RunnerResult::Err(_, _));
    }
}
