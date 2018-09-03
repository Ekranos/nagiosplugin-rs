initSidebarItems({"enum":[["State","Represents a service state from nagios."]],"macro":[["resource","Let's you simply create a resource from multiple metrics. It's a bit like the vec! macro. `rust # #[macro_use] # extern crate nagiosplugin; # # use nagiosplugin::{SimpleMetric, State}; # # fn main() { let m1 = SimpleMetric::new(\"test\", Some(State::Ok), 12, None, None, None, None); let m2 = SimpleMetric::new(\"other\", None, true, None, None, None, None); let resource = resource![m1, m2]; # }`"]],"struct":[["PartialOrdMetric","A PartialOrdMetric is a metric which will automatically calculate the State based on the given value and warning and/or critical value."],["Resource","A Resource basically represents a single service if you view it from the perspective of nagios. If you init it without a state it will determine one from the given metrics."],["SimpleMetric","Represents a simple metric where no logic is performed. You give some values in and the same get out."]],"trait":[["Metric","This trait can be implemented for any kind of metric and will be used to generate the final string output for nagios. Calls to the functions should return immediately and not query the service every time."],["ResourceMetric","Represents a single metric of a resource. You shouldn't need to implement this by yourself since the crate provided types already implement this."],["ToPerfString","The purpose of ToPerfString is only so one can define custom representations of custom types without using the ToString trait so we don't interfere with that."]]});