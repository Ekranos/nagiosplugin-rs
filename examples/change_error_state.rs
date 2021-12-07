use anyhow::anyhow;

use nagiosplugin::{Resource, Runner, ServiceState};

fn main() {
    // Instead of exiting a critical service state, we exit with a service state of "unknown" on error.
    Runner::new()
        .on_error(|e| (ServiceState::Unknown, e))
        .safe_run(do_check)
        .print_and_exit()
}

// This example uses anyhow
fn do_check() -> Result<Resource, anyhow::Error> {
    // Do something which returns an error.
    return Err(anyhow!("something really bad happened"));
}
