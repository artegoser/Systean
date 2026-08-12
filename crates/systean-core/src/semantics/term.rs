use std::collections::BTreeMap;
use std::fmt;

use crate::rational::ExactRational;
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};

use super::{ConstructorId, ContextSlotId, Type, UnitId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Term {
    Const(String),
    Var(String),
    Literal(Literal),
    Call {
        function: String,
        arguments: BTreeMap<String, Term>,
    },
    Bind {
        variable: String,
        variable_type: Type,
        body: Box<Term>,
    },
    Record(BTreeMap<String, Term>),
    Field {
        record: Box<Term>,
        field: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
    Structured(StructuredLiteral),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructuredLiteral {
    pub value: StructuredValue,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructuredValue {
    Number(ExactRational),
    ApproximateNumber {
        value: ExactRational,
        tolerance: Option<ExactRational>,
    },
    Digit(u8),
    DigitSequence(Vec<u8>),
    Unit(UnitId),
    Quantity {
        value: ExactRational,
        unit: UnitId,
        approximate: bool,
        uncertainty: Option<ExactRational>,
    },
    CalendarDate {
        year: i32,
        month: u8,
        day: u8,
    },
    TimeOfDay {
        hour: u8,
        minute: u8,
        second: u8,
    },
    TimeZone {
        offset_minutes: i16,
    },
    Instant {
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        offset_minutes: i16,
    },
    Duration {
        seconds: ExactRational,
    },
    Interval {
        start: Box<StructuredValue>,
        end: Box<StructuredValue>,
    },
    Information {
        mode: ConstructorId,
        status: InformationStatus,
        knower: Option<InformationKnowerValue>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InformationStatus {
    Unknown,
    Unspecified,
    Withheld,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InformationKnowerValue {
    Context(ContextSlotId),
    Value(Box<Term>),
}

impl StructuredLiteral {
    pub fn new(value: StructuredValue, ty: Type) -> Self {
        Self { value, ty }
    }

    pub fn family(&self) -> &'static str {
        self.value.family()
    }
}

impl StructuredValue {
    pub fn family(&self) -> &'static str {
        match self {
            Self::Number(_) => "number",
            Self::ApproximateNumber { .. } => "approximate_number",
            Self::Digit(_) => "digit",
            Self::DigitSequence(_) => "digit_sequence",
            Self::Unit(_) => "unit",
            Self::Quantity { approximate: false, .. } => "quantity",
            Self::Quantity { approximate: true, .. } => "approximate_quantity",
            Self::CalendarDate { .. } => "calendar_date",
            Self::TimeOfDay { .. } => "time_of_day",
            Self::TimeZone { .. } => "timezone",
            Self::Instant { .. } => "instant",
            Self::Duration { .. } => "duration",
            Self::Interval { .. } => "interval",
            Self::Information { .. } => "information",
        }
    }
}

fn rational_to_written(value: &ExactRational) -> String {
    if value.denom().is_one() {
        return value.numer().to_string();
    }
    let negative = value.is_negative();
    let absolute = if negative { -value.clone() } else { value.clone() };
    if let Some((integer, fraction)) = finite_decimal_parts(&absolute) {
        let sign = if negative { "-" } else { "" };
        if fraction.is_empty() {
            format!("{sign}{integer}")
        } else {
            format!("{sign}{integer}.{fraction}")
        }
    } else {
        format!("{}/{}", value.numer(), value.denom())
    }
}

fn finite_decimal_parts(value: &ExactRational) -> Option<(BigInt, String)> {
    if value.is_negative() { return None; }
    let numerator = value.numer().clone();
    let denominator = value.denom().clone();
    let mut reduced = denominator;
    let two = BigInt::from(2u8);
    let five = BigInt::from(5u8);
    let mut twos = 0u32;
    let mut fives = 0u32;
    while (&reduced % &two).is_zero() { reduced /= &two; twos += 1; }
    while (&reduced % &five).is_zero() { reduced /= &five; fives += 1; }
    if !reduced.is_one() { return None; }
    let scale = twos.max(fives);
    let scaled = numerator * two.pow(scale - twos) * five.pow(scale - fives);
    let ten_pow = BigInt::from(10u8).pow(scale);
    let integer = &scaled / &ten_pow;
    let remainder = &scaled % &ten_pow;
    let mut fraction = if scale == 0 {
        String::new()
    } else {
        format!("{:0>width$}", remainder, width = scale as usize)
    };
    while fraction.ends_with('0') { fraction.pop(); }
    Some((integer, fraction))
}

fn temporal_written(value: &StructuredValue) -> Option<String> {
    Some(match value {
        StructuredValue::CalendarDate { year, month, day } => format!("{year:04}-{month:02}-{day:02}"),
        StructuredValue::TimeOfDay { hour, minute, second } => format!("{hour:02}:{minute:02}:{second:02}"),
        StructuredValue::TimeZone { offset_minutes } => {
            if *offset_minutes == 0 { "Z".into() } else {
                let sign = if *offset_minutes < 0 { '-' } else { '+' };
                let absolute = offset_minutes.unsigned_abs();
                format!("{sign}{:02}:{:02}", absolute / 60, absolute % 60)
            }
        }
        StructuredValue::Instant { year, month, day, hour, minute, second, offset_minutes } => {
            let zone = temporal_written(&StructuredValue::TimeZone { offset_minutes: *offset_minutes })?;
            format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{zone}")
        }
        StructuredValue::Duration { seconds } => format!("PT{}S", rational_to_written(seconds)),
        StructuredValue::Interval { start, end } => format!("{}/{}", temporal_written(start)?, temporal_written(end)?),
        _ => return None,
    })
}

impl fmt::Display for StructuredValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(value) => write!(f, "number<{:?}>", rational_to_written(value)),
            Self::ApproximateNumber { value, tolerance } => {
                let canonical = match tolerance {
                    Some(tolerance) => format!("{}±{}", rational_to_written(value), rational_to_written(tolerance)),
                    None => format!("~{}", rational_to_written(value)),
                };
                write!(f, "approximate_number<{canonical:?}>")
            }
            Self::Digit(value) => write!(f, "digit<{:?}>", value.to_string()),
            Self::DigitSequence(values) => {
                let canonical = values.iter().map(u8::to_string).collect::<String>();
                write!(f, "digit_sequence<{canonical:?}>")
            }
            Self::Unit(unit) => write!(f, "unit<\"@{unit}\">") ,
            Self::Quantity { value, unit, approximate, uncertainty } => {
                write!(f, "quantity(value = {}, unit = @{unit}, approximate = {approximate}", rational_to_written(value))?;
                if let Some(uncertainty) = uncertainty {
                    write!(f, ", uncertainty = {}", rational_to_written(uncertainty))?;
                }
                write!(f, ")")
            }
            temporal @ (Self::CalendarDate { .. } | Self::TimeOfDay { .. } | Self::TimeZone { .. } | Self::Instant { .. } | Self::Duration { .. } | Self::Interval { .. }) => {
                write!(f, "{}<{:?}>", temporal.family(), temporal_written(temporal).expect("temporal value"))
            }
            Self::Information { status, knower, .. } => {
                let status = match status {
                    InformationStatus::Unknown => "unknown",
                    InformationStatus::Unspecified => "unspecified",
                    InformationStatus::Withheld => "withheld",
                };
                write!(f, "information(status = {status}")?;
                if let Some(knower) = knower {
                    write!(f, ", knower = {knower}")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl fmt::Display for InformationKnowerValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context(slot) => write!(f, "context(@{slot})"),
            Self::Value(term) => term.fmt(f),
        }
    }
}

impl fmt::Display for StructuredLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Boolean(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value:?}"),
            Self::Structured(value) => value.fmt(f),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Const(name) | Self::Var(name) => f.write_str(name),
            Self::Literal(literal) => literal.fmt(f),
            Self::Call { function, arguments } => {
                write!(f, "{function}(")?;
                for (index, (role, value)) in arguments.iter().enumerate() {
                    if index > 0 { write!(f, ", ")?; }
                    write!(f, "{role} = {value}")?;
                }
                write!(f, ")")
            }
            Self::Bind { variable, variable_type, body } => {
                write!(f, "bind {variable}: {variable_type} => {body}")
            }
            Self::Record(fields) => {
                write!(f, "{{")?;
                for (index, (name, value)) in fields.iter().enumerate() {
                    if index > 0 { write!(f, ", ")?; }
                    write!(f, "{name} = {value}")?;
                }
                write!(f, "}}")
            }
            Self::Field { record, field } => write!(f, "({record}).{field}"),
        }
    }
}
