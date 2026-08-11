use num_bigint::BigInt;
use num_traits::ToPrimitive;

use super::{integer_to_spoken, parse_spoken_integer, NumberConfig};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemporalLiteralValue {
    CalendarDate { year: i32, month: u8, day: u8 },
    TimeOfDay { hour: u8, minute: u8, second: u8 },
    TimeZone { offset_minutes: i16 },
    Instant {
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        offset_minutes: i16,
    },
    DurationSeconds { seconds_canonical: String },
    Interval { start: String, end: String },
}

impl TemporalLiteralValue {
    pub fn canonical_written(&self) -> String {
        match self {
            Self::CalendarDate { year, month, day } => format!("{year:04}-{month:02}-{day:02}"),
            Self::TimeOfDay { hour, minute, second } => format!("{hour:02}:{minute:02}:{second:02}"),
            Self::TimeZone { offset_minutes } => format_timezone(*offset_minutes),
            Self::Instant { year, month, day, hour, minute, second, offset_minutes } => format!(
                "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{}",
                format_timezone(*offset_minutes)
            ),
            Self::DurationSeconds { seconds_canonical } => format!("PT{seconds_canonical}S"),
            Self::Interval { start, end } => format!("{start}/{end}"),
        }
    }
}

pub fn parse_written_temporal(token: &str) -> Result<Option<TemporalLiteralValue>, String> {
    if token.len() >= 4
        && token.get(..2).is_some_and(|prefix| prefix.eq_ignore_ascii_case("PT"))
        && token.chars().last().is_some_and(|last| matches!(last, 'S' | 's'))
    {
        let value = &token[2..token.len() - 1];
        if value.is_empty() {
            return Err("duration literal requires a numeric seconds value".into());
        }
        return Ok(Some(TemporalLiteralValue::DurationSeconds {
            seconds_canonical: value.to_owned(),
        }));
    }
    if let Some((left, right)) = token.split_once('/') {
        let left_value = parse_written_instant(left)?;
        let right_value = parse_written_instant(right)?;
        if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
            return Ok(Some(TemporalLiteralValue::Interval {
                start: left_value.canonical_written(),
                end: right_value.canonical_written(),
            }));
        }
    }
    if let Some(value) = parse_written_instant(token)? {
        return Ok(Some(value));
    }
    if let Some((year, month, day)) = parse_date(token) {
        validate_date(year, month, day)?;
        return Ok(Some(TemporalLiteralValue::CalendarDate { year, month, day }));
    }
    if let Some((hour, minute, second)) = parse_time(token) {
        validate_time(hour, minute, second)?;
        return Ok(Some(TemporalLiteralValue::TimeOfDay { hour, minute, second }));
    }
    if let Some(offset) = parse_timezone(token) {
        return Ok(Some(TemporalLiteralValue::TimeZone { offset_minutes: offset }));
    }
    Ok(None)
}

pub fn parse_spoken_date(
    tokens: &[String],
    start: usize,
    number: &NumberConfig,
    marker: &str,
    scope_open: &str,
    scope_close: &str,
) -> Result<Option<(TemporalLiteralValue, usize)>, String> {
    if tokens.get(start).is_none_or(|token| token != marker) {
        return Ok(None);
    }
    let mut index = start + 1;
    let Some(year) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Ok(None);
    };
    let Some(month) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Ok(None);
    };
    let Some(day) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Ok(None);
    };
    let year = bigint_i32(&year, "calendar year")?;
    let month = bigint_u8(&month, "calendar month")?;
    let day = bigint_u8(&day, "calendar day")?;
    validate_date(year, month, day)?;
    Ok(Some((TemporalLiteralValue::CalendarDate { year, month, day }, index - start)))
}

pub fn parse_spoken_time(
    tokens: &[String],
    start: usize,
    number: &NumberConfig,
    marker: &str,
    scope_open: &str,
    scope_close: &str,
) -> Result<Option<(TemporalLiteralValue, usize)>, String> {
    if tokens.get(start).is_none_or(|token| token != marker) {
        return Ok(None);
    }
    let mut index = start + 1;
    let Some(hour) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Ok(None);
    };
    if tokens.get(index).is_none_or(|token| token != &number.decimal) {
        return Ok(None);
    }
    index += 1;
    let Some(minute) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Err("time-of-day literal requires scoped minutes".into());
    };
    if tokens.get(index).is_none_or(|token| token != &number.decimal) {
        return Err("time-of-day literal requires second separator".into());
    }
    index += 1;
    let Some(second) = parse_scoped_integer(tokens, &mut index, number, scope_open, scope_close)? else {
        return Err("time-of-day literal requires scoped seconds".into());
    };
    let hour = bigint_u8(&hour, "hour")?;
    let minute = bigint_u8(&minute, "minute")?;
    let second = bigint_u8(&second, "second")?;
    validate_time(hour, minute, second)?;
    Ok(Some((TemporalLiteralValue::TimeOfDay { hour, minute, second }, index - start)))
}

pub fn canonical_spoken_temporal(
    value: &TemporalLiteralValue,
    number: &NumberConfig,
    date_marker: &str,
    timezone_marker: &str,
    scope_open: &str,
    scope_close: &str,
) -> Result<Vec<String>, String> {
    match value {
        TemporalLiteralValue::CalendarDate { year, month, day } => {
            let mut out = vec![date_marker.to_owned()];
            push_scoped_integer(&mut out, &BigInt::from(*year), number, scope_open, scope_close)?;
            push_scoped_integer(&mut out, &BigInt::from(*month), number, scope_open, scope_close)?;
            push_scoped_integer(&mut out, &BigInt::from(*day), number, scope_open, scope_close)?;
            Ok(out)
        }
        TemporalLiteralValue::TimeOfDay { hour, minute, second } => {
            let mut out = vec![date_marker.to_owned()];
            push_scoped_integer(&mut out, &BigInt::from(*hour), number, scope_open, scope_close)?;
            out.push(number.decimal.clone());
            push_scoped_integer(&mut out, &BigInt::from(*minute), number, scope_open, scope_close)?;
            out.push(number.decimal.clone());
            push_scoped_integer(&mut out, &BigInt::from(*second), number, scope_open, scope_close)?;
            Ok(out)
        }
        TemporalLiteralValue::TimeZone { offset_minutes } => {
            let mut out = vec![timezone_marker.to_owned()];
            if *offset_minutes == 0 {
                out.extend(integer_to_spoken(&BigInt::from(0u8), number)?);
                return Ok(out);
            }
            let negative = *offset_minutes < 0;
            out.push(if negative { number.minus.clone() } else { number.plus.clone() });
            let absolute = offset_minutes.unsigned_abs();
            let hours = absolute / 60;
            let minutes = absolute % 60;
            out.extend(integer_to_spoken(&BigInt::from(hours), number)?);
            out.push(number.decimal.clone());
            out.extend(integer_to_spoken(&BigInt::from(minutes), number)?);
            Ok(out)
        }
        TemporalLiteralValue::Instant {
            year,
            month,
            day,
            hour,
            minute,
            second,
            offset_minutes,
        } => {
            let mut out = vec![date_marker.to_owned()];
            for value in [
                BigInt::from(*year),
                BigInt::from(*month),
                BigInt::from(*day),
                BigInt::from(*hour),
                BigInt::from(*minute),
                BigInt::from(*second),
            ] {
                push_scoped_integer(&mut out, &value, number, scope_open, scope_close)?;
            }
            out.extend(canonical_spoken_temporal(
                &TemporalLiteralValue::TimeZone { offset_minutes: *offset_minutes },
                number,
                date_marker,
                timezone_marker,
                scope_open,
                scope_close,
            )?);
            Ok(out)
        }
        TemporalLiteralValue::DurationSeconds { seconds_canonical } => Ok(vec![
            date_marker.to_owned(),
            scope_open.to_owned(),
            seconds_canonical.clone(),
            scope_close.to_owned(),
        ]),
        TemporalLiteralValue::Interval { start, end } => {
            let start_value = parse_written_instant(start)?
                .ok_or_else(|| "interval start is not a canonical instant".to_owned())?;
            let end_value = parse_written_instant(end)?
                .ok_or_else(|| "interval end is not a canonical instant".to_owned())?;
            let mut out = vec![date_marker.to_owned(), scope_open.to_owned()];
            out.extend(canonical_spoken_temporal(
                &start_value, number, date_marker, timezone_marker, scope_open, scope_close,
            )?);
            out.push(scope_close.to_owned());
            out.push(scope_open.to_owned());
            out.extend(canonical_spoken_temporal(
                &end_value, number, date_marker, timezone_marker, scope_open, scope_close,
            )?);
            out.push(scope_close.to_owned());
            Ok(out)
        },
    }
}

fn parse_scoped_integer(
    tokens: &[String],
    index: &mut usize,
    number: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<Option<BigInt>, String> {
    if tokens.get(*index).is_none_or(|token| token != scope_open) {
        return Ok(None);
    }
    *index += 1;
    let Some((value, consumed)) = parse_spoken_integer(tokens, *index, number)? else {
        return Err("scoped temporal field requires an integer".into());
    };
    *index += consumed;
    match tokens.get(*index) {
        Some(token) if token == scope_close => *index += 1,
        Some(token) => return Err(format!("temporal field expected `{scope_close}`, found `{token}`")),
        None => return Err(format!("temporal field expected `{scope_close}`, found end of input")),
    }
    Ok(Some(value))
}

fn push_scoped_integer(
    output: &mut Vec<String>,
    value: &BigInt,
    number: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<(), String> {
    output.push(scope_open.to_owned());
    output.extend(integer_to_spoken(value, number)?);
    output.push(scope_close.to_owned());
    Ok(())
}

fn parse_written_instant(token: &str) -> Result<Option<TemporalLiteralValue>, String> {
    let Some(t_index) = token.find(|character| matches!(character, 'T' | 't')) else {
        return Ok(None);
    };
    let date = &token[..t_index];
    let rest = &token[t_index + 1..];
    // `t` is also a perfectly ordinary letter in Systean roots (for example
    // `pent`, `rat`, and `keta`).  An arbitrary token containing `t` must not
    // become an instant candidate merely because the written ISO form uses
    // `T` as its separator.  Only commit to instant parsing after the prefix
    // is itself a syntactically valid written calendar date.
    let Some((year, month, day)) = parse_date(date) else {
        return Ok(None);
    };
    validate_date(year, month, day)?;
    let (time_text, zone_text) = split_time_zone(rest)
        .ok_or_else(|| "instant requires explicit timezone".to_owned())?;
    let Some((hour, minute, second)) = parse_time(time_text) else {
        return Err("invalid instant time".into());
    };
    validate_time(hour, minute, second)?;
    let offset_minutes = parse_timezone(zone_text).ok_or_else(|| "invalid instant timezone".to_owned())?;
    Ok(Some(TemporalLiteralValue::Instant {
        year,
        month,
        day,
        hour,
        minute,
        second,
        offset_minutes,
    }))
}

fn parse_date(source: &str) -> Option<(i32, u8, u8)> {
    let mut parts = source.split('-');
    let year = parts.next()?.parse().ok()?;
    let month = parts.next()?.parse().ok()?;
    let day = parts.next()?.parse().ok()?;
    (parts.next().is_none()).then_some((year, month, day))
}

fn parse_time(source: &str) -> Option<(u8, u8, u8)> {
    let mut parts = source.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let second = match parts.next() {
        Some(value) => value.parse().ok()?,
        None => 0,
    };
    (parts.next().is_none()).then_some((hour, minute, second))
}

fn split_time_zone(source: &str) -> Option<(&str, &str)> {
    if source.chars().last().is_some_and(|last| matches!(last, 'Z' | 'z')) {
        return Some((&source[..source.len() - 1], "Z"));
    }
    let index = source
        .char_indices()
        .skip(1)
        .find_map(|(index, ch)| matches!(ch, '+' | '-').then_some(index))?;
    Some((&source[..index], &source[index..]))
}

fn parse_timezone(source: &str) -> Option<i16> {
    if source.eq_ignore_ascii_case("z") {
        return Some(0);
    }
    let sign = match source.as_bytes().first().copied()? {
        b'+' => 1i16,
        b'-' => -1i16,
        _ => return None,
    };
    let mut parts = source[1..].split(':');
    let hour: i16 = parts.next()?.parse().ok()?;
    let minute: i16 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || hour > 14 || minute > 59 || (hour == 14 && minute != 0) {
        return None;
    }
    Some(sign * (hour * 60 + minute))
}

fn format_timezone(offset_minutes: i16) -> String {
    if offset_minutes == 0 {
        return "Z".into();
    }
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let absolute = offset_minutes.unsigned_abs();
    format!("{sign}{:02}:{:02}", absolute / 60, absolute % 60)
}

fn validate_date(year: i32, month: u8, day: u8) -> Result<(), String> {
    if year < 1 {
        return Err("calendar year must be >= 1".into());
    }
    if !(1..=12).contains(&month) {
        return Err(format!("calendar month {month} is outside 1..12"));
    }
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!(),
    };
    if day == 0 || day > max_day {
        return Err(format!("calendar day {day} is invalid for {year:04}-{month:02}"));
    }
    Ok(())
}

fn validate_time(hour: u8, minute: u8, second: u8) -> Result<(), String> {
    if hour > 23 || minute > 59 || second > 59 {
        return Err(format!("invalid time-of-day {hour:02}:{minute:02}:{second:02}"));
    }
    Ok(())
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn bigint_i32(value: &BigInt, label: &str) -> Result<i32, String> {
    value.to_i32().ok_or_else(|| format!("{label} is outside supported range"))
}

fn bigint_u8(value: &BigInt, label: &str) -> Result<u8, String> {
    value.to_u8().ok_or_else(|| format!("{label} is outside supported range"))
}
