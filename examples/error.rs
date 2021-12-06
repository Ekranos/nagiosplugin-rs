use anyhow::anyhow;

use nagiosplugin::{Resource, Runner};

fn main() {
    Runner::new().safe_run(do_check).print_and_exit()
}

// This example uses anyhow
fn do_check() -> Result<Resource, anyhow::Error> {
    // Do something which returns an error.
    return Err(anyhow!("something really bad happened"));
}
