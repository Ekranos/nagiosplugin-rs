//! The nagiosplugin crate provides some basic utilities to make it easier to write nagios checks.

use std::cmp::Ordering;
use std::process;

#[macro_use]
mod macros;

mod helper;
pub use crate::helper::{safe_run, safe_run_with_state};

/// A Resource basically represents a single service if you view it from the perspective of nagios.
/// If you init it without a state it will determine one from the given metrics.
///
/// You can also create a Resource filled with metrics via the *resource!* macro, which is much
/// like the *vec!* macro.
///
/// ```rust
/// # #[macro_use]
/// # extern crate nagiosplugin;
/// # use nagiosplugin::{SimpleMetric, State};
/// # fn main() {
/// let m1 = SimpleMetric::new("test", Some(State::Ok), 12, None, None, None, None);
/// let m2 = SimpleMetric::new("other", None, true, None, None, None, None);
/// let resource = resource![m1, m2];
/// assert_eq!(&resource.to_nagios_string(), "OK | test=12 other=true");
///
/// // Prints "OK | test=12 other=true" and exits with an exit code of 0 in this case
/// resource.print_and_exit();
/// # }
/// ```
pub struct Resource {
    state: Option<State>,
    metrics: Vec<Box<dyn ResourceMetric>>,
    description: Option<String>,
    name: Option<String>,
}

impl Resource {
    /// If state is set to Some(State) then it will always use this instead of determining it from
    /// the given metrics.
    ///
    /// If you want to create a Resource from some metrics with automatic determination of the
    /// state you can use the *resource!* macro.
    pub fn new(state: Option<State>, description: Option<&str>) -> Resource {
        Resource {
            state,
            metrics: Vec::new(),
            description: description.map(|d| d.to_owned()),
            name: None,
        }
    }

    /// Pushes a single ResourceMetric into the resource.
    pub fn push<M>(&mut self, metric: M)
    where
        M: 'static + ResourceMetric,
    {
        self.metrics.push(Box::new(metric))
    }

    /// Returns a slice of the pushed metrics.
    pub fn metrics(&self) -> &[Box<dyn ResourceMetric>] {
        &self.metrics
    }

    /// Manually set the state for this resource. This disabled the automatic state determination
    /// based on the included metrics of this resource.
    pub fn set_state(&mut self, state: State) {
        self.state = Some(state)
    }

    /// Set the name of this resource. Will be included in the final string output.
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned())
    }

    /// Returns a string which nagios understands to determine the service state.
    ///
    /// This function will automatically determine which service state is appropriate based on the
    /// included metrics. If state has been set manually it will always use the manually set state.
    pub fn to_nagios_string(&self) -> String {
        let mut s = String::new();

        if let Some(ref name) = self.name {
            s.push_str(&format!("{} ", name))
        }

        s.push_str(&self.get_state().to_string());

        if let Some(ref description) = self.description {
            s.push_str(&format!(": {}", description));
        }

        if self.metrics.len() > 0 {
            s.push_str(" |");

            for metric in self.metrics.iter() {
                s.push_str(&format!(" {}", metric.perf_string()));
            }
        }

        s
    }

    /// Will determine a State by the given metrics.
    ///
    /// In case a state is manually set for this resource,
    /// it will return the manually set state instead.
    pub fn get_state(&self) -> State {
        let mut state = State::Unknown;
        if let Some(ref st) = self.state {
            state = st.clone()
        } else {
            for metric in self.metrics.iter() {
                if let Some(st) = metric.state() {
                    if state < st {
                        state = st;
                    }
                }
            }
        }
        state
    }

    /// Get the description of this resource.
    pub fn get_description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Set the description of this resource.
    pub fn set_description(&mut self, description: &str) {
        self.description = Some(description.to_owned());
    }

    /// Will return the exit code of the determined state via Self::state.
    pub fn exit_code(&self) -> i32 {
        self.get_state().exit_code()
    }

    /// Will print Self::to_nagios_string and exit with the exit code from Self::exit_code
    pub fn print_and_exit(&self) {
        println!("{}", self.to_nagios_string());
        process::exit(self.exit_code());
    }
}

impl Default for Resource {
    fn default() -> Self {
        Resource::new(None, None)
    }
}

/// Represents a single metric of a resource. You shouldn't need to implement this by yourself
/// since the crate provided types already implement this.
pub trait ResourceMetric {
    fn perf_string(&self) -> String;
    fn name(&self) -> &str;
    fn state(&self) -> Option<State>;
}

impl<T, O> ResourceMetric for T
where
    O: ToPerfString,
    T: Metric<Output = O> + ToPerfString,
{
    fn perf_string(&self) -> String {
        self.to_perf_string()
    }

    fn name(&self) -> &str {
        self.name()
    }

    fn state(&self) -> Option<State> {
        self.state()
    }
}

/// Represents a service state from nagios.
#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Ok,
    Warning,
    Critical,
    Unknown,
}

impl State {
    /// Returns the corresponding nagios exit code to signal the service state of self.
    pub fn exit_code(&self) -> i32 {
        match self {
            &State::Ok => 0,
            &State::Warning => 1,
            &State::Critical => 2,
            &State::Unknown => 3,
        }
    }
}

impl ToString for State {
    fn to_string(&self) -> String {
        match self {
            State::Ok => "OK".to_owned(),
            State::Warning => "WARNING".to_owned(),
            State::Critical => "CRITICAL".to_owned(),
            State::Unknown => "UNKNOWN".to_owned(),
        }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        let f = |state| match state {
            &State::Unknown => 0,
            &State::Ok => 1,
            &State::Warning => 2,
            &State::Critical => 3,
        };

        f(self).partial_cmp(&f(other))
    }
}

/// The purpose of ToPerfString is only so one can define custom representations of custom types
/// without using the ToString trait so we don't interfere with that.
///
/// Also used internally for generation of the final output.
///
/// It's already implemented for some basic types.
pub trait ToPerfString {
    fn to_perf_string(&self) -> String;
}

impl_to_perf_string_on_to_string!(bool, usize);
impl_to_perf_string_on_to_string!(u8, u16, u32, u64, u128);
impl_to_perf_string_on_to_string!(i8, i16, i32, i64, i128);
impl_to_perf_string_on_to_string!(f32, f64);
impl_to_perf_string_on_to_string!(String);

impl<'a> ToPerfString for &'a str {
    fn to_perf_string(&self) -> String {
        self.to_string()
    }
}

impl<T, O> ToPerfString for T
where
    O: ToPerfString,
    T: Metric<Output = O>,
{
    fn to_perf_string(&self) -> String {
        // replace `=`
        let name = self.name().replace('=', "_");

        // quote `'`
        let name = name.replace('\'', "''");

        // quote if contains spaces
        let name = if name.contains(' ') {
            format!("'{}'", self.name())
        } else {
            name.to_string()
        };

        metric_string!(
            name,
            format!(
                "{}{}",
                self.value().to_perf_string(),
                self.unit_of_measurement().to_string()
            ),
            //            self.value(),
            self.warning(),
            self.critical(),
            self.min(),
            self.max()
        )
    }
}

impl<T> ToPerfString for Option<T>
where
    T: ToPerfString,
{
    fn to_perf_string(&self) -> String {
        match self {
            Some(ref s) => s.to_perf_string(),
            None => String::new(),
        }
    }
}

#[derive(Clone)]
pub enum Unit {
    None,
    Seconds,
    Milliseconds,
    Microseconds,
    Percentage,
    Bytes,
    KiloBytes,
    MegaBytes,
    TeraBytes,
    Counter,
    Other(String),
}

impl ToString for Unit {
    fn to_string(&self) -> String {
        match self {
            &Unit::None => "".to_owned(),
            &Unit::Seconds => "s".to_owned(),
            &Unit::Milliseconds => "ms".to_owned(),
            &Unit::Microseconds => "us".to_owned(),
            &Unit::Percentage => "%".to_owned(),
            &Unit::Bytes => "B".to_owned(),
            &Unit::KiloBytes => "KB".to_owned(),
            &Unit::MegaBytes => "MB".to_owned(),
            &Unit::TeraBytes => "TB".to_owned(),
            &Unit::Counter => "c".to_owned(),
            &Unit::Other(ref str) => str.to_owned(),
        }
    }
}

/// This trait can be implemented for any kind of metric and will be used to generate the final
/// string output for nagios. Calls to the functions should return immediately and not query the
/// service every time.
pub trait Metric {
    type Output: ToPerfString;

    fn name(&self) -> &str;
    fn state(&self) -> Option<State>;
    fn value(&self) -> Self::Output;
    fn warning(&self) -> Option<Self::Output>;
    fn critical(&self) -> Option<Self::Output>;
    fn min(&self) -> Option<Self::Output>;
    fn max(&self) -> Option<Self::Output>;
    fn unit_of_measurement(&self) -> &Unit;
}

/// A PartialOrdMetric is a metric which will automatically calculate the State
/// based on the given value and warning and/or critical value.
///
/// It doesn't matter if you provide warning or critical or both of none of these. Even though
/// you should choose SimpleMetric if you aren't providing any warning or critical value.
///
/// The state function of the implemented Metric trait will always be one of: Ok, Warning, Critical
///
/// ```rust
/// # extern crate nagiosplugin;
/// # use nagiosplugin::{Metric, State, PartialOrdMetric};
/// let metric = PartialOrdMetric::new("test", 15, Some(15), Some(30), None, None, false);
/// assert_eq!(metric.state(), Some(State::Warning));
/// assert_eq!(metric.value(), 15);
/// ```
pub struct PartialOrdMetric<T>
where
    T: PartialOrd + ToPerfString + Clone,
{
    name: String,
    value: T,
    warning: Option<T>,
    critical: Option<T>,
    min: Option<T>,
    max: Option<T>,
    lower_is_critical: bool,
    unit_of_measurement: Unit,
}

impl<T> PartialOrdMetric<T>
where
    T: PartialOrd + ToPerfString + Clone,
{
    /// Creates a new PartialOrdMetric from the given values.
    ///
    /// *In debug builds this will panic if you pass incorrect values for warning and critical.*
    pub fn new(
        name: &str,
        value: T,
        warning: Option<T>,
        critical: Option<T>,
        min: Option<T>,
        max: Option<T>,
        lower_is_critical: bool,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            if warning.is_some() && critical.is_some() {
                let warning = warning.clone().unwrap();
                let critical = critical.clone().unwrap();

                if lower_is_critical && warning < critical {
                    panic!("lower_is_critical is set to true while warning is lower than critical, this is not correct");
                } else if !lower_is_critical && warning > critical {
                    panic!("lower_is_critical is set to false while warning is lower than critical, this is not correct");
                }
            }

            if min.is_some() && max.is_some() {
                let min = min.clone().unwrap();
                let max = max.clone().unwrap();

                assert!(min < max, "minimum value is not smaller than maximum value")
            }
        }

        PartialOrdMetric {
            name: name.to_owned(),
            value: value.clone(),
            warning: warning.map(|w| w.clone()),
            critical: critical.map(|c| c.clone()),
            min: min.map(|m| m.clone()),
            max: max.map(|m| m.clone()),
            lower_is_critical,
            unit_of_measurement: Unit::None,
        }
    }

    pub fn set_unit_of_measurement(&mut self, unit_of_measurement: Unit) {
        self.unit_of_measurement = unit_of_measurement
    }
}

impl<T> Metric for PartialOrdMetric<T>
where
    T: PartialOrd + ToPerfString + Clone,
{
    type Output = T;

    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> Option<State> {
        if let Some(ref critical) = self.critical {
            if self.lower_is_critical {
                if &self.value <= critical {
                    return Some(State::Critical);
                }
            } else {
                if &self.value >= critical {
                    return Some(State::Critical);
                }
            }
        }

        if let Some(ref warning) = self.warning {
            if self.lower_is_critical {
                if &self.value <= warning {
                    return Some(State::Warning);
                }
            } else {
                if &self.value >= warning {
                    return Some(State::Warning);
                }
            }
        }

        Some(State::Ok)
    }

    fn value(&self) -> <Self as Metric>::Output {
        self.value.clone()
    }

    fn warning(&self) -> Option<<Self as Metric>::Output> {
        self.warning.clone()
    }

    fn critical(&self) -> Option<<Self as Metric>::Output> {
        self.critical.clone()
    }

    fn min(&self) -> Option<<Self as Metric>::Output> {
        self.min.clone()
    }

    fn max(&self) -> Option<<Self as Metric>::Output> {
        self.max.clone()
    }

    fn unit_of_measurement(&self) -> &Unit {
        &self.unit_of_measurement
    }
}

/// Represents a simple metric where no logic is performed. You give some values in and the same
/// get out.
#[derive(Clone)]
pub struct SimpleMetric<T>
where
    T: ToPerfString + Clone,
{
    name: String,
    state: Option<State>,
    value: T,
    warning: Option<T>,
    critical: Option<T>,
    min: Option<T>,
    max: Option<T>,
    unit_of_measurement: Unit,
}

impl<T> SimpleMetric<T>
where
    T: ToPerfString + Clone,
{
    pub fn new(
        name: &str,
        state: Option<State>,
        value: T,
        warning: Option<T>,
        critical: Option<T>,
        min: Option<T>,
        max: Option<T>,
    ) -> Self {
        SimpleMetric {
            name: name.to_owned(),
            state,
            value,
            warning,
            critical,
            min,
            max,
            unit_of_measurement: Unit::None,
        }
    }

    pub fn set_unit_of_measurement(&mut self, unit_of_measurement: Unit) {
        self.unit_of_measurement = unit_of_measurement
    }
}

impl<T> Metric for SimpleMetric<T>
where
    T: ToPerfString + Clone,
{
    type Output = T;

    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> Option<State> {
        self.state.clone()
    }

    fn value(&self) -> <Self as Metric>::Output {
        self.value.clone()
    }

    fn warning(&self) -> Option<<Self as Metric>::Output> {
        self.warning.clone()
    }

    fn critical(&self) -> Option<<Self as Metric>::Output> {
        self.critical.clone()
    }

    fn min(&self) -> Option<<Self as Metric>::Output> {
        self.min.clone()
    }

    fn max(&self) -> Option<<Self as Metric>::Output> {
        self.max.clone()
    }

    fn unit_of_measurement(&self) -> &Unit {
        &self.unit_of_measurement
    }
}

#[cfg(test)]
mod tests {
    use crate::{Metric, PartialOrdMetric, Resource, SimpleMetric, State, ToPerfString, Unit};

    #[test]
    fn test_partial_ord_metric() {
        let metric = PartialOrdMetric::new("test", 12, None, None, None, None, false);
        assert_eq!(metric.name(), "test");
        assert_eq!(metric.state(), Some(State::Ok));
        assert_eq!(metric.value(), 12);
        assert_eq!(metric.warning(), None);
        assert_eq!(metric.critical(), None);

        // Cases with lower_is_critical = false

        let metric = PartialOrdMetric::new("test", 12, Some(15), Some(30), None, None, false);
        assert_eq!(metric.state(), Some(State::Ok));
        assert_eq!(metric.value(), 12);
        assert_eq!(metric.warning(), Some(15));
        assert_eq!(metric.critical(), Some(30));

        let metric = PartialOrdMetric::new("test", 15, Some(15), Some(30), None, None, false);
        assert_eq!(metric.state(), Some(State::Warning));
        assert_eq!(metric.value(), 15);

        let metric = PartialOrdMetric::new("test", 18, Some(15), Some(30), None, None, false);
        assert_eq!(metric.state(), Some(State::Warning));
        assert_eq!(metric.value(), 18);

        let metric = PartialOrdMetric::new("test", 30, Some(15), Some(30), None, None, false);
        assert_eq!(metric.state(), Some(State::Critical));
        assert_eq!(metric.value(), 30);

        let metric = PartialOrdMetric::new("test", 35, Some(15), Some(30), None, None, false);
        assert_eq!(metric.state(), Some(State::Critical));
        assert_eq!(metric.value(), 35);

        // Cases with lower_is_critical = true

        let metric = PartialOrdMetric::new("test", 35, Some(30), Some(15), None, None, true);
        assert_eq!(metric.state(), Some(State::Ok));
        assert_eq!(metric.value(), 35);
        assert_eq!(metric.warning(), Some(30));
        assert_eq!(metric.critical(), Some(15));

        let metric = PartialOrdMetric::new("test", 30, Some(30), Some(15), None, None, true);
        assert_eq!(metric.state(), Some(State::Warning));
        assert_eq!(metric.value(), 30);

        let metric = PartialOrdMetric::new("test", 20, Some(30), Some(15), None, None, true);
        assert_eq!(metric.state(), Some(State::Warning));
        assert_eq!(metric.value(), 20);

        let metric = PartialOrdMetric::new("test", 15, Some(30), Some(15), None, None, true);
        assert_eq!(metric.state(), Some(State::Critical));
        assert_eq!(metric.value(), 15);

        let metric = PartialOrdMetric::new("test", 10, Some(30), Some(15), None, None, true);
        assert_eq!(metric.state(), Some(State::Critical));
        assert_eq!(metric.value(), 10);
    }

    #[test]
    fn test_simple_metric() {
        let metric = SimpleMetric::new("test", Some(State::Ok), 12, None, None, None, None);
        assert_eq!(metric.state(), Some(State::Ok));
        assert_eq!(metric.value(), 12);
        assert_eq!(metric.warning(), None);
        assert_eq!(metric.critical(), None);

        let metric = SimpleMetric::new(
            "test",
            Some(State::Unknown),
            22,
            Some(15),
            Some(30),
            None,
            None,
        );
        assert_eq!(metric.state(), Some(State::Unknown));
        assert_eq!(metric.value(), 22);
        assert_eq!(metric.warning(), Some(15));
        assert_eq!(metric.critical(), Some(30));

        let metric = SimpleMetric::new("test", Some(State::Ok), "test", None, None, None, None);
        assert_eq!(metric.state(), Some(State::Ok));
        assert_eq!(metric.value(), "test");
        assert_eq!(metric.warning(), None);
        assert_eq!(metric.critical(), None);
    }

    #[test]
    fn test_simple_metric_unit_of_measurement() {
        let mut metric = SimpleMetric::new("foo", None, 12, None, None, None, None);
        metric.set_unit_of_measurement(Unit::Microseconds);
        assert_eq!(&metric.to_perf_string(), "foo=12us");

        metric.set_unit_of_measurement(Unit::Other("bar".to_owned()));
        assert_eq!(&metric.to_perf_string(), "foo=12bar");
    }

    #[test]
    fn test_resource() {
        let m1 = SimpleMetric::new("test", Some(State::Ok), 12, None, None, None, None);
        let m2 = SimpleMetric::new("other", None, true, None, None, None, None);

        let resource = resource![m1, m2];

        assert_eq!(&resource.to_nagios_string(), "OK | test=12 other=true");

        let m1 = SimpleMetric::new("test", Some(State::Ok), 12, Some(14), None, Some(0), None);
        let m2 = SimpleMetric::new("other", None, true, None, None, None, None);

        let resource = resource![m1, m2];

        assert_eq!(
            &resource.to_nagios_string(),
            "OK | test=12;14;;0 other=true"
        );

        let m1 = SimpleMetric::new("test", Some(State::Ok), 12, Some(14), None, Some(0), None);
        let m2 = SimpleMetric::new("other", None, true, None, None, None, None);

        let mut resource: Resource = resource![m1, m2];
        resource.set_description("A test description");

        assert_eq!(
            &resource.to_nagios_string(),
            "OK: A test description | test=12;14;;0 other=true"
        );

        let test_data = [
            ("test", "OK | test=0"),
            ("test=a", "OK | test_a=0"),
            ("te'st", "OK | te''st=0"),
            ("te st", "OK | 'te st'=0"),
        ];
        for (label, expected_string) in &test_data {
            let metric = SimpleMetric::new(label, Some(State::Ok), 0, None, None, None, None);
            let resource: Resource = resource![metric];

            assert_eq!(&resource.to_nagios_string(), expected_string,);
        }
    }

    #[test]
    fn test_resource_with_name() {
        let mut resource = Resource::new(Some(State::Ok), None);
        resource.set_name("foo");
        assert_eq!(&resource.to_nagios_string(), "foo OK")
    }

    #[test]
    fn test_state() {
        assert_eq!(State::Ok.exit_code(), 0);
        assert_eq!(State::Warning.exit_code(), 1);
        assert_eq!(State::Critical.exit_code(), 2);
        assert_eq!(State::Unknown.exit_code(), 3);

        assert_eq!(&State::Ok.to_string(), "OK");
        assert_eq!(&State::Warning.to_string(), "WARNING");
        assert_eq!(&State::Critical.to_string(), "CRITICAL");
        assert_eq!(&State::Unknown.to_string(), "UNKNOWN");
    }
}
