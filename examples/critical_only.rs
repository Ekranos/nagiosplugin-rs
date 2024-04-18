use std::error::Error;

use nagiosplugin::{safe_run, Metric, Resource, ServiceState, TriggerIfValue};

fn main() {
    safe_run(do_check, ServiceState::Critical).print_and_exit()
}

fn do_check() -> Result<Resource, Box<dyn Error>> {
    // The first metric will not issue an alarm, the second one will.
    let resource = Resource::new("foo")
        .with_description("This is a simple test plugin")
        .with_result(Metric::new("test", 15).with_thresholds(None, 50, TriggerIfValue::Greater))
        .with_result(Metric::new("alerting", 52).with_thresholds(
            None,
            50,
            TriggerIfValue::Greater,
        ));

    Ok(resource)
}
