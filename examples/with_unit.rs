use std::error::Error;

use nagiosplugin::{safe_run, Metric, Resource, ServiceState, TriggerIfValue, Unit, UnitString};

fn main() {
    safe_run(do_check, ServiceState::Critical).print_and_exit()
}

fn do_check() -> Result<Resource, Box<dyn Error>> {
    // UnitString::new will check if the given string is valid
    let custom_unit = Unit::Other(UnitString::new("km")?);

    // Use new_unchecked only if you know what you are doing
    let _custom_unit = Unit::Other(UnitString::new_unchecked("km"));

    let resource = Resource::new("foo")
        .with_description("This is a simple test plugin")
        .with_result(
            Metric::new("test", 15)
                .with_thresholds(20, 50, TriggerIfValue::Greater)
                .with_unit(Unit::Megabytes), // Use built-in unit megabytes
        )
        .with_result(Metric::new("bar", 10).with_unit(custom_unit));

    Ok(resource)
}
