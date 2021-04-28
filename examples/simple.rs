use std::error::Error;

use nagiosplugin::{Metric, Resource, Runner, TriggerIfValue};

fn main() {
    Runner::new().safe_run(do_check).print_and_exit()
}

fn do_check() -> Result<Resource, Box<dyn Error>> {
    // The first metric will not issue an alarm, the second one will.
    let resource = Resource::new("foo")
        .with_description("This is a simple test plugin")
        .with_result(Metric::new("test", 15).with_thresholds(20, 50, TriggerIfValue::Greater))
        .with_result(Metric::new("alerting", 42).with_thresholds(40, 50, TriggerIfValue::Greater));

    Ok(resource)
}
