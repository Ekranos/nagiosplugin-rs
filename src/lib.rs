//! This crate provides utilities to write Icinga/Nagios checks/plugins.
//! If you want to use this library only for compatible output take a look at the [Resource].
//! If you also want error handling, take a look at [safe_run].
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

use crate::ServiceState::{Critical, Warning};
use std::str::FromStr;

#[cfg(feature = "clap")]
pub mod config_generator;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
/// Represents the state of a service / resource.
pub enum ServiceState {
    Ok,
    Warning,
    Critical,
    #[default]
    Unknown,
}

impl ServiceState {
    /// Returns the corresponding exit code for this state.
    pub fn exit_code(&self) -> i32 {
        match self {
            ServiceState::Ok => 0,
            ServiceState::Warning => 1,
            ServiceState::Critical => 2,
            ServiceState::Unknown => 3,
        }
    }

    /// Returns a number for ordering purposes. Ordering is Ok < Unknown < Warning < Critical.
    /// So if you order you get the best to worst state.
    fn order_number(&self) -> u8 {
        match self {
            ServiceState::Ok => 0,
            ServiceState::Unknown => 1,
            ServiceState::Warning => 2,
            ServiceState::Critical => 3,
        }
    }
}

impl PartialOrd for ServiceState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.order_number().partial_cmp(&other.order_number())
    }
}

impl Ord for ServiceState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order_number().cmp(&other.order_number())
    }
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            ServiceState::Ok => "OK",
            ServiceState::Warning => "WARNING",
            ServiceState::Critical => "CRITICAL",
            ServiceState::Unknown => "UNKNOWN",
        };

        f.write_str(s)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("expected one of: ok, warning, critical, unknown")]
/// This error is returned by the [FromStr] implementation of [ServiceState].
pub struct ServiceStateFromStrError;

impl FromStr for ServiceState {
    type Err = ServiceStateFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ok" => Ok(ServiceState::Ok),
            "warning" => Ok(ServiceState::Warning),
            "critical" => Ok(ServiceState::Critical),
            "unknown" => Ok(ServiceState::Unknown),
            _ => Err(ServiceStateFromStrError),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// This represents the unit for a metric. It can be one of the predefined units or a custom one.
/// See [Nagios Plugin Development Guidelines](https://nagios-plugins.org/doc/guidelines.html#AEN200) for more information.
pub enum Unit {
    #[default]
    None,
    Seconds,
    Milliseconds,
    Microseconds,
    Percentage,
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Counter,
    Other(UnitString),
}

impl Unit {
    fn as_str(&self) -> &str {
        match self {
            Unit::None => "",
            Unit::Seconds => "s",
            Unit::Milliseconds => "ms",
            Unit::Microseconds => "us",
            Unit::Percentage => "%",
            Unit::Bytes => "B",
            Unit::Kilobytes => "KB",
            Unit::Megabytes => "MB",
            Unit::Gigabytes => "GB",
            Unit::Terabytes => "TB",
            Unit::Counter => "c",
            Unit::Other(s) => &s.0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
/// This error is returned if a [UnitString] is created with an invalid string.
pub enum UnitStringCreateError {
    // TODO: Maybe even whitespace?
    #[error("expected string to not include numbers, semicolons or quotes")]
    InvalidCharacters,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
/// Newtype wrapper around a string to ensure only valid strings end up in the performance data.
pub struct UnitString(String);

impl UnitString {
    pub fn new(s: impl Into<String>) -> Result<Self, UnitStringCreateError> {
        let s = s.into();
        if ('0'..='9').chain(['"', ';']).any(|c| s.contains(c)) {
            Err(UnitStringCreateError::InvalidCharacters)
        } else {
            Ok(UnitString::new_unchecked(s))
        }
    }

    pub fn new_unchecked(s: impl Into<String>) -> Self {
        UnitString(s.into())
    }
}

impl FromStr for UnitString {
    type Err = UnitStringCreateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        UnitString::new(s)
    }
}

/// Defines if a metric triggers if value is greater or less than the thresholds.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TriggerIfValue {
    Greater,
    Less,
}

impl From<&TriggerIfValue> for Ordering {
    fn from(v: &TriggerIfValue) -> Self {
        match v {
            TriggerIfValue::Greater => Ordering::Greater,
            TriggerIfValue::Less => Ordering::Less,
        }
    }
}

/// Defines a metric with a required name and value. Also takes optional thresholds (warning, critical)
/// minimum, maximum. Can also be set to ignore thresholds and have a fixed [ServiceState].
#[derive(Debug, Clone)]
pub struct Metric<T> {
    name: String,
    value: T,
    unit: Unit,
    thresholds: Option<(Option<T>, Option<T>, TriggerIfValue)>,
    min: Option<T>,
    max: Option<T>,
    fixed_state: Option<ServiceState>,
}

impl<T> Metric<T> {
    pub fn new(name: impl Into<String>, value: T) -> Self {
        Self {
            name: name.into(),
            value,
            unit: Default::default(),
            thresholds: Default::default(),
            min: Default::default(),
            max: Default::default(),
            fixed_state: Default::default(),
        }
    }

    pub fn with_thresholds(
        mut self,
        warning: impl Into<Option<T>>,
        critical: impl Into<Option<T>>,
        trigger_if_value: TriggerIfValue,
    ) -> Self {
        self.thresholds = Some((warning.into(), critical.into(), trigger_if_value));
        self
    }

    pub fn with_minimum(mut self, minimum: T) -> Self {
        self.min = Some(minimum);
        self
    }

    pub fn with_maximum(mut self, maximum: T) -> Self {
        self.max = Some(maximum);
        self
    }

    /// If a fixed state is set, this metric will always report the given state if turned in to a
    /// [CheckResult].
    pub fn with_fixed_state(mut self, state: ServiceState) -> Self {
        self.fixed_state = Some(state);
        self
    }

    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }
}

/// Represents a single performance metric.
#[derive(Debug, Clone)]
pub struct PerfData<T> {
    name: String,
    value: T,
    unit: Unit,
    warning: Option<T>,
    critical: Option<T>,
    minimum: Option<T>,
    maximum: Option<T>,
}

impl<T: ToPerfString> PerfData<T> {
    pub fn new(name: impl Into<String>, value: T) -> Self {
        Self {
            name: name.into(),
            value,
            unit: Default::default(),
            warning: Default::default(),
            critical: Default::default(),
            minimum: Default::default(),
            maximum: Default::default(),
        }
    }

    pub fn with_thresholds(mut self, warning: Option<T>, critical: Option<T>) -> Self {
        self.warning = warning;
        self.critical = critical;
        self
    }

    pub fn with_minimum(mut self, minimum: T) -> Self {
        self.minimum = Some(minimum);
        self
    }

    pub fn with_maximum(mut self, maximum: T) -> Self {
        self.maximum = Some(maximum);
        self
    }

    pub fn with_unit(mut self, unit: Unit) -> Self {
        self.unit = unit;
        self
    }
}

impl<T: ToPerfString> From<PerfData<T>> for PerfString {
    fn from(perf_data: PerfData<T>) -> Self {
        let s = PerfString::new(
            &perf_data.name,
            &perf_data.value,
            perf_data.unit,
            perf_data.warning.as_ref(),
            perf_data.critical.as_ref(),
            perf_data.minimum.as_ref(),
            perf_data.maximum.as_ref(),
        );
        s
    }
}

/// Newtype wrapper around a string to ensure only valid strings end up in the final output.
/// This is used for the performance data / metric part of the output.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PerfString(String);

impl PerfString {
    pub fn new<T>(
        name: &str,
        value: &T,
        unit: Unit,
        warning: Option<&T>,
        critical: Option<&T>,
        minimum: Option<&T>,
        maximum: Option<&T>,
    ) -> Self
    where
        T: ToPerfString,
    {
        // TODO: Sanitize name
        let value = value.to_perf_string();
        let warning = warning.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let critical = critical.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let minimum = minimum.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let maximum = maximum.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        PerfString(format!(
            "'{}'={}{};{};{};{};{}",
            name,
            value,
            unit.as_str(),
            warning,
            critical,
            minimum,
            maximum
        ))
    }
}

/// Represents a single item of a check. Multiple of these are used to form a [Resource].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    state: Option<ServiceState>,
    message: Option<String>,
    perf_string: Option<PerfString>,
}

impl CheckResult {
    /// Creates an empty instance.
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            message: Default::default(),
            perf_string: Default::default(),
        }
    }

    pub fn with_state(mut self, state: ServiceState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Sets the performance data of this result. Takes anything that implements [`Into<PerfString>`].
    /// This includes [`PerfData`].
    pub fn with_perf_data(mut self, perf_data: impl Into<PerfString>) -> Self {
        self.perf_string = Some(perf_data.into());
        self
    }
}

impl Default for CheckResult {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: PartialOrd + ToPerfString> From<Metric<T>> for CheckResult {
    fn from(metric: Metric<T>) -> Self {
        let state = if let Some(state) = metric.fixed_state {
            Some(state)
        } else if let Some((warning, critical, trigger)) = &metric.thresholds {
            let ord: Ordering = trigger.into();
            let warning_cmp = warning.as_ref().and_then(|w| metric.value.partial_cmp(w));
            let critical_cmp = critical.as_ref().and_then(|w| metric.value.partial_cmp(w));

            [(critical_cmp, Critical), (warning_cmp, Warning)]
                .iter()
                .filter_map(|(cmp, state)| cmp.as_ref().map(|cmp| (cmp, state)))
                .filter_map(|(&cmp, &state)| {
                    if cmp == ord || cmp == Ordering::Equal {
                        Some(state)
                    } else {
                        None
                    }
                })
                .next()
        } else {
            None
        };

        let message = match state {
            Some(state) if state != ServiceState::Ok => {
                let (warning, critical, _) = metric.thresholds.as_ref().unwrap();
                let threshold = match state {
                    ServiceState::Warning => warning.as_ref().unwrap(),
                    ServiceState::Critical => critical.as_ref().unwrap(),
                    _ => unreachable!(),
                };
                Some(format!(
                    "metric '{}' is {}: value '{}' has exceeded threshold of '{}'",
                    &metric.name,
                    state,
                    metric.value.to_perf_string(),
                    threshold.to_perf_string(),
                ))
            }
            _ => None,
        };

        let perf_string = {
            let (warning, critical) = if let Some((warning, critical, _)) = &metric.thresholds {
                (warning.as_ref(), critical.as_ref())
            } else {
                (None, None)
            };

            PerfString::new(
                &metric.name,
                &metric.value,
                metric.unit,
                warning,
                critical,
                metric.min.as_ref(),
                metric.max.as_ref(),
            )
        };

        CheckResult {
            state,
            message,
            perf_string: Some(perf_string),
        }
    }
}

/// Implement this if you have a value which can be converted to a performance metric value.
pub trait ToPerfString {
    fn to_perf_string(&self) -> String;
}

macro_rules! impl_to_perf_string {
    ($t:ty) => {
        impl ToPerfString for $t {
            fn to_perf_string(&self) -> String {
                self.to_string()
            }
        }
    };
}

impl_to_perf_string!(usize);
impl_to_perf_string!(isize);
impl_to_perf_string!(u8);
impl_to_perf_string!(u16);
impl_to_perf_string!(u32);
impl_to_perf_string!(u64);
impl_to_perf_string!(u128);
impl_to_perf_string!(i8);
impl_to_perf_string!(i16);
impl_to_perf_string!(i32);
impl_to_perf_string!(i64);
impl_to_perf_string!(i128);
impl_to_perf_string!(f32);
impl_to_perf_string!(f64);

/// Represents a single service / resource from the perspective of Icinga.
#[derive(Debug, PartialEq, Eq)]
pub struct Resource {
    name: String,
    results: Vec<CheckResult>,
    fixed_state: Option<ServiceState>,
    description: Option<String>,
}

impl Resource {
    /// Creates a new instance with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Default::default(),
            fixed_state: Default::default(),
            description: Default::default(),
        }
    }

    /// If a fixed state is set, the coressponding [Resource] will always report the given state regardless of the
    /// actual state of the [CheckResult]s.
    pub fn with_fixed_state(mut self, state: ServiceState) -> Self {
        self.fixed_state = Some(state);
        self
    }

    pub fn with_result(mut self, result: impl Into<CheckResult>) -> Self {
        self.push_result(result);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.set_description(description);
        self
    }

    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    pub fn push_result(&mut self, result: impl Into<CheckResult>) {
        self.results.push(result.into());
    }

    /// Calculates the state and message of this resource
    pub fn nagios_result(self) -> (ServiceState, String) {
        let (state, messages, perf_string) = {
            let mut final_state = ServiceState::Ok;

            let mut messages = String::new();
            let mut perf_string = String::new();

            for result in self.results {
                if let Some(state) = result.state {
                    if final_state < state {
                        final_state = state;
                    }
                }

                if let Some(message) = result.message {
                    messages.push_str(message.trim());
                    messages.push('\n');
                }

                if let Some(s) = result.perf_string {
                    perf_string.push(' ');
                    perf_string.push_str(s.0.trim());
                }
            }

            if let Some(state) = self.fixed_state {
                final_state = state;
            }

            (final_state, messages, perf_string)
        };

        let description = {
            let mut s = String::new();
            s.push_str(&self.name);
            s.push_str(" is ");
            s.push_str(&state.to_string());

            if let Some(description) = self.description {
                s.push_str(": ");
                s.push_str(description.trim());
            }
            s
        };

        let pad = if messages.is_empty() { "" } else { "\n\n" };

        (
            state,
            format!("{}{}{}|{}", description, pad, messages.trim(), perf_string),
        )
    }

    /// Calls [Self::nagios_result] and prints the result to stdout. It will also exit with the
    /// corresponding exit code based on the state.
    fn print_and_exit(self) -> ! {
        let (state, s) = self.nagios_result();
        println!("{}", &s);
        std::process::exit(state.exit_code());
    }
}

/// Helper function to safely run a check with a defined [ServiceState] on error and return a [RunResult] which can be used to print and exit.
///
/// ## Example
///
/// ```no_run
/// use std::error::Error;
///
/// use nagiosplugin::{safe_run, Metric, Resource, ServiceState, TriggerIfValue};
///
/// fn main() {
///     safe_run(do_check, ServiceState::Critical).print_and_exit()
/// }
///
/// fn do_check() -> Result<Resource, Box<dyn Error>> {
///    // The first metric will not issue an alarm, the second one will.
///    let resource = Resource::new("foo")
///         .with_description("This is a simple test plugin")
///         .with_result(Metric::new("test", 15).with_thresholds(20, 50, TriggerIfValue::Greater))
///         .with_result(Metric::new("alerting", 42).with_thresholds(40, 50, TriggerIfValue::Greater));
///
///     Ok(resource)
/// }
/// ```
pub fn safe_run<E>(
    f: impl FnOnce() -> Result<Resource, E>,
    error_state: ServiceState,
) -> RunResult<E> {
    match f() {
        Ok(resource) => RunResult::Ok(resource),
        Err(err) => RunResult::Err(error_state, err),
    }
}

/// The result of a runner execution.
#[derive(Debug)]
pub enum RunResult<E> {
    /// The run was successful and it contains the returned [Resource].
    Ok(Resource),
    /// The run was not successful and it contains the [ServiceState] and the error.
    Err(ServiceState, E),
}

impl<E: std::fmt::Display> RunResult<E> {
    pub fn print_and_exit(self) -> ! {
        match self {
            RunResult::Ok(resource) => resource.print_and_exit(),
            RunResult::Err(state, msg) => {
                println!("{}: {}", state, msg);
                std::process::exit(state.exit_code());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_nagios_result() {
        let (state, s) = Resource::new("foo")
            .with_description("i am bar")
            .with_result(
                CheckResult::new()
                    .with_state(ServiceState::Warning)
                    .with_message("flubblebar"),
            )
            .with_result(CheckResult::new().with_state(ServiceState::Critical))
            .nagios_result();

        assert_eq!(state, ServiceState::Critical);
        assert!(s.contains("i am bar"));
        assert!(s.contains("flubblebar"));
        assert!(s.contains(&ServiceState::Critical.to_string()));
    }

    #[test]
    fn test_resource_with_fixed_state() {
        let (state, _) = Resource::new("foo")
            .with_fixed_state(ServiceState::Critical)
            .nagios_result();
        assert_eq!(state, ServiceState::Critical);
    }

    #[test]
    fn test_resource_with_ok_result() {
        let (state, msg) = Resource::new("foo")
            .with_result(
                CheckResult::new()
                    .with_message("test")
                    .with_state(ServiceState::Ok),
            )
            .nagios_result();

        assert_eq!(ServiceState::Ok, state);
        assert!(msg.contains("test"));
    }

    #[test]
    fn test_perf_string_new() {
        let s = PerfString::new("foo", &12, Unit::None, Some(&42), None, None, Some(&60));
        assert_eq!(&s.0, "'foo'=12;42;;;60")
    }

    #[test]
    fn test_metric_into_check_result_complete() {
        let metric = Metric::new("test", 42)
            .with_minimum(0)
            .with_maximum(100)
            .with_thresholds(40, 50, TriggerIfValue::Greater);

        let result: CheckResult = metric.into();
        assert_eq!(result.state, Some(ServiceState::Warning));

        let message = result.message.expect("no message set");
        assert!(message.contains(&ServiceState::Warning.to_string()));
        assert!(message.contains("test"));
        assert!(message.contains("threshold"));
    }

    #[test]
    fn test_metric_into_check_result_threshold_less() {
        let result: CheckResult = Metric::new("test", 40)
            .with_thresholds(50, 30, TriggerIfValue::Less)
            .into();

        assert_eq!(result.state, Some(ServiceState::Warning));
    }

    #[test]
    fn test_metric_into_check_result_threshold_greater() {
        let result: CheckResult = Metric::new("test", 40)
            .with_thresholds(30, 50, TriggerIfValue::Greater)
            .into();

        assert_eq!(result.state, Some(ServiceState::Warning));
    }

    #[test]
    fn test_metric_into_check_result_threshold_equal_to_val() {
        let result: CheckResult = Metric::new("foo", 30)
            .with_thresholds(30, 40, TriggerIfValue::Greater)
            .into();

        assert_eq!(result.state, Some(ServiceState::Warning));
    }

    #[test]
    fn test_metric_into_check_result_threshold_only_warning() {
        let result: CheckResult = Metric::new("foo", 30)
            .with_thresholds(25, None, TriggerIfValue::Greater)
            .into();

        assert_eq!(result.state, Some(ServiceState::Warning));

        let result: CheckResult = Metric::new("foo", 30)
            .with_thresholds(35, None, TriggerIfValue::Greater)
            .into();

        assert_eq!(result.state, None);
    }

    #[test]
    fn test_metric_into_check_result_with_unit() {
        let result: CheckResult = Metric::new("foo", 20)
            .with_thresholds(25, None, TriggerIfValue::Greater)
            .with_unit(Unit::Megabytes)
            .into();

        result.perf_string.unwrap().0.contains("MB");

        assert_eq!(result.state, None);
    }

    #[derive(Debug, thiserror::Error)]
    #[error("woops")]
    struct EmptyError;

    fn do_check(success: bool) -> Result<Resource, EmptyError> {
        if success {
            Ok(Resource::new("test"))
        } else {
            Err(EmptyError {})
        }
    }

    #[test]
    fn test_safe_run_ok() {
        let result = safe_run(|| do_check(true), ServiceState::Critical);

        matches!(result, RunResult::Ok(_));
    }

    #[test]
    fn test_safe_run_error() {
        let result = safe_run(|| do_check(false), ServiceState::Critical);

        matches!(result, RunResult::Err(_, _));
    }
}
