use std::env::args;

use nagiosplugin::{Resource, State};

// Usage: cargo run --example simple -- haaa
//        cargo run --example simple -- itsfine

fn main() {
    // Grab the first argument
    let arg = args().nth(1).expect("provide an argument");

    // Create a default resource: state is unknown, description is empty
    let mut resource = Resource::new(None, None);

    // Check logic goes here
    match arg.as_ref() {
        "itsfine" => {
            resource.set_state(State::Ok);
            resource.set_description("Eveything is fine :-)");
        }
        "haaa" => {
            resource.set_state(State::Critical);
            resource.set_description("Something went terribly wrong!");
        }
        _ => (), // unexpected argument: the state will remain unknown
    };

    // print the status based on `state` and `description`
    // then exists with appropriate exit code
    resource.print_and_exit();
}
