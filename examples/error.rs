use anyhow::anyhow;

use nagiosplugin::{safe_run, Resource, ServiceState};

fn main() {
    safe_run(do_check, ServiceState::Critical).print_and_exit()
}

// This example uses anyhow
fn do_check() -> Result<Resource, anyhow::Error> {
    // Do something which returns an error.
    return Err(anyhow!("something really bad happened"));
}
