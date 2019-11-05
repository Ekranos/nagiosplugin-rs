use std::env::args;

use nagiosplugin::{Resource, State, SimpleMetric};

// Usage: cargo run --example simple -- <value>


fn main() {
    // Grab the first argument
    let arg: u32 = args().nth(1).expect("provide an argument").parse().expect("argument should be a number");

    // Create a new resource
    let mut resource = Resource::new(None, None);

    // create a metric
    let metric = SimpleMetric::new(
        &"size of thing",  // label
        Some(State::Ok),   // state
        arg,               // current value
        None,              // warn
        None,              // crit
        None,              // min
        None,              // max
        );

    // add the metric to the resource
    resource.push(metric);

    // print status, perfdata, and exit
    resource.print_and_exit();
}
