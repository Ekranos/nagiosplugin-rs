use std::error::Error;

use nagiosplugin::{safe_run, Resource, ServiceState};

fn main() {
    safe_run(do_check, ServiceState::Critical).print_and_exit()
}

fn do_check() -> Result<Resource, Box<dyn Error>> {
    // The first metric will not issue an alarm, the second one will.
    let resource = Resource::new("foo").with_description("This is a simple test plugin");

    Ok(resource)
}
