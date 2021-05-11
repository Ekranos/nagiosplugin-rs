/// This crate provides utilities to write Icinga/Nagios checks/plugins.
/// If you want to use this library only for compatible output take a look at the [Resource].
/// If you also want error handling, take a look at the [Runner].
use std::cmp::Ordering;
use std::fmt;
use std::fmt::Formatter;

pub use runner::*;

use crate::ServiceState::{Critical, Warning};
use std::str::FromStr;

mod runner;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ServiceState {
    Ok,
    Warning,
    Critical,
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

/// Defines if a metric triggers if value is greater or less than the thresholds.
#[derive(Debug, Copy, Clone)]
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
    thresholds: Option<(T, T, TriggerIfValue)>,
    min: Option<T>,
    max: Option<T>,
    fixed_state: Option<ServiceState>,
}

impl<T> Metric<T> {
    pub fn new(name: impl Into<String>, value: T) -> Self {
        Self {
            name: name.into(),
            value,
            thresholds: Default::default(),
            min: Default::default(),
            max: Default::default(),
            fixed_state: Default::default(),
        }
    }

    pub fn with_thresholds(
        mut self,
        warning: T,
        critical: T,
        trigger_if_value: TriggerIfValue,
    ) -> Self {
        self.thresholds = Some((warning, critical, trigger_if_value));
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
}

/// Represents a single performance metric.
pub struct PerfData<T> {
    name: String,
    value: T,
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
            warning: Default::default(),
            critical: Default::default(),
            minimum: Default::default(),
            maximum: Default::default(),
        }
    }

    pub fn with_thresholds(mut self, warning: T, critical: T) -> Self {
        self.warning = Some(warning);
        self.critical = Some(critical);
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
}

impl<T: ToPerfString> From<PerfData<T>> for PerfString {
    fn from(perf_data: PerfData<T>) -> Self {
        let s = PerfString::new(
            &perf_data.name,
            &perf_data.value,
            perf_data.warning.as_ref(),
            perf_data.critical.as_ref(),
            perf_data.minimum.as_ref(),
            perf_data.maximum.as_ref(),
        );
        s
    }
}

/// Newtype wrapper around a string to "ensure" only valid conversions happen.
/// If you want to create
pub struct PerfString(String);

impl PerfString {
    pub fn new<T>(
        name: &str,
        value: &T,
        warning: Option<&T>,
        critical: Option<&T>,
        minimum: Option<&T>,
        maximum: Option<&T>,
    ) -> Self
    where
        T: ToPerfString,
    {
        let value = value.to_perf_string();
        let warning = warning.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let critical = critical.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let minimum = minimum.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        let maximum = maximum.map_or_else(|| "".to_owned(), |v| v.to_perf_string());
        PerfString(format!(
            "'{}'={};{};{};{};{}",
            name, value, warning, critical, minimum, maximum
        ))
    }
}

/// Represents a single item of a check. Multiple of these are used to form a [Resource].
pub struct CheckResult {
    state: Option<ServiceState>,
    message: Option<String>,
    perf_string: Option<PerfString>,
}

impl CheckResult {
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
        let state = if let Some((warning, critical, trigger)) = &metric.thresholds {
            let ord: Ordering = trigger.into();
            let warning_cmp = metric.value.partial_cmp(warning);
            let critical_cmp = metric.value.partial_cmp(critical);

            [(critical_cmp, Critical), (warning_cmp, Warning)]
                .iter()
                .filter_map(|(cmp, state)| {
                    if let Some(cmp) = cmp {
                        Some((cmp, state))
                    } else {
                        None
                    }
                })
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
        }
        .or(Some(ServiceState::Ok));

        let message = match state {
            Some(state) if state != ServiceState::Ok => {
                let (warning, critical, _) = metric.thresholds.as_ref().unwrap();
                let threshold = match state {
                    ServiceState::Warning => warning,
                    ServiceState::Critical => critical,
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
                (Some(warning), Some(critical))
            } else {
                (None, None)
            };

            PerfString::new(
                &metric.name,
                &metric.value,
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
pub struct Resource {
    name: String,
    results: Vec<CheckResult>,
    fixed_state: Option<ServiceState>,
    description: Option<String>,
}

impl Resource {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            results: Default::default(),
            fixed_state: Default::default(),
            description: Default::default(),
        }
    }

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

    fn print_and_exit(self) -> ! {
        let (state, s) = self.nagios_result();
        println!("{}", &s);
        std::process::exit(state.exit_code());
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

    // #[test]
    // fn test_resource_empty() {
    //     let (state, msg) =
    // }

    #[test]
    fn test_perf_string_new() {
        let s = PerfString::new("foo", &12, Some(&42), None, None, Some(&60));
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

        assert_eq!(result.state, Some(Warning));
    }
}
