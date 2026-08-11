use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Signed, ToPrimitive, Zero};

use super::NumberConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactNumber(pub BigRational);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberParse {
    pub value: ExactNumber,
    pub consumed: usize,
}

impl ExactNumber {
    pub fn from_integer(value: BigInt) -> Self {
        Self(BigRational::from_integer(value))
    }

    pub fn parse_written(source: &str, max_exponent: u32) -> Result<Self, String> {
        let source = source.trim();
        if source.is_empty() {
            return Err("empty number".into());
        }
        if let Some((left, right)) = split_once_unique(source, '/') {
            let numerator = parse_bigint(left)?;
            let denominator = parse_bigint(right)?;
            if denominator.is_zero() {
                return Err("rational denominator cannot be zero".into());
            }
            return Ok(Self(BigRational::new(numerator, denominator)));
        }

        let (mantissa, exponent) = match source.find(|character| matches!(character, 'e' | 'E')) {
            Some(index) => {
                if source[index + 1..].contains(|character| matches!(character, 'e' | 'E')) {
                    return Err("number contains more than one exponent marker".into());
                }
                let exponent = source[index + 1..]
                    .parse::<i64>()
                    .map_err(|_| "invalid decimal exponent".to_owned())?;
                if exponent.unsigned_abs() > u64::from(max_exponent) {
                    return Err(format!(
                        "decimal exponent {exponent} exceeds configured limit {max_exponent}"
                    ));
                }
                (&source[..index], exponent)
            }
            None => (source, 0),
        };
        let mut value = parse_decimal_mantissa(mantissa)?;
        if exponent != 0 {
            let scale = pow10(exponent.unsigned_abs() as u32);
            value = if exponent > 0 {
                value * BigRational::from_integer(scale)
            } else {
                value / BigRational::from_integer(scale)
            };
        }
        Ok(Self(value))
    }

    pub fn canonical_written(&self) -> String {
        rational_to_decimal_or_fraction(&self.0)
    }

    pub fn canonical_fraction(&self) -> String {
        if self.0.denom().is_one() {
            self.0.numer().to_string()
        } else {
            format!("{}/{}", self.0.numer(), self.0.denom())
        }
    }

    pub fn is_integer(&self) -> bool {
        self.0.denom().is_one()
    }

    pub fn integer(&self) -> Option<&BigInt> {
        self.is_integer().then(|| self.0.numer())
    }
}

pub fn parse_spoken_number(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<Option<NumberParse>, String> {
    if start >= tokens.len() {
        return Ok(None);
    }
    let negative = tokens.get(start).is_some_and(|token| token == &config.minus);
    let explicit_plus = tokens.get(start).is_some_and(|token| token == &config.plus);
    let core_start = start + usize::from(negative || explicit_plus);
    if tokens.get(core_start).is_some_and(|token| token == &config.rational) {
        let mut parsed = parse_spoken_rational(tokens, core_start, config, scope_open, scope_close)?;
        if negative { parsed.value.0 = -parsed.value.0; }
        parsed.consumed += core_start - start;
        return Ok(Some(parsed));
    }
    if tokens.get(core_start).is_some_and(|token| token == &config.exponent) {
        let mut parsed = parse_spoken_exponent(tokens, core_start, config, scope_open, scope_close)?;
        if negative { parsed.value.0 = -parsed.value.0; }
        parsed.consumed += core_start - start;
        return Ok(Some(parsed));
    }

    let mut index = core_start;

    let Some((integer, consumed)) = parse_spoken_integer(tokens, index, config)? else {
        return Ok(None);
    };
    index += consumed;
    let mut value = BigRational::from_integer(integer);

    if tokens.get(index).is_some_and(|token| token == &config.decimal) {
        index += 1;
        let fraction_start = index;
        let mut digits = String::new();
        while let Some(token) = tokens.get(index) {
            let Some(digit) = config.digits.get(token) else { break };
            digits.push(char::from(b'0' + *digit));
            index += 1;
        }
        if index == fraction_start {
            return Err(format!("`{}` requires at least one following digit", config.decimal));
        }
        let denominator = pow10(digits.len() as u32);
        let numerator = parse_bigint(&digits)?;
        value += BigRational::new(numerator, denominator);
    }

    if negative {
        value = -value;
    }
    Ok(Some(NumberParse {
        value: ExactNumber(value),
        consumed: index - start,
    }))
}

pub fn parse_spoken_integer(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
) -> Result<Option<(BigInt, usize)>, String> {
    let Some(first) = tokens.get(start) else { return Ok(None) };
    if config.digits.get(first) == Some(&0) {
        return Ok(Some((BigInt::zero(), 1)));
    }

    let mut index = start;
    let mut value = BigInt::zero();
    let mut previous_exponent: Option<u32> = None;
    let mut had_outer = false;

    loop {
        let Some((exponent, chain_len)) = parse_outer_chain(tokens, index, config)? else { break };
        if previous_exponent.is_some_and(|previous| exponent >= previous) {
            return Err("numeric outer magnitude terms must descend strictly".into());
        }
        index += chain_len;
        let Some((coefficient, coefficient_len)) = parse_coefficient(tokens, index, config, false)? else {
            return Err("outer magnitude requires a non-zero coefficient".into());
        };
        value += BigInt::from(coefficient) * pow10(exponent);
        index += coefficient_len;
        previous_exponent = Some(exponent);
        had_outer = true;
    }

    if let Some((coefficient, coefficient_len)) = parse_coefficient(tokens, index, config, false)? {
        value += BigInt::from(coefficient);
        index += coefficient_len;
    } else if !had_outer {
        return Ok(None);
    }

    Ok(Some((value, index - start)))
}

pub fn canonical_spoken(
    number: &ExactNumber,
    config: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<Vec<String>, String> {
    let mut output = Vec::new();
    let mut value = number.0.clone();
    if value.is_negative() {
        output.push(config.minus.clone());
        value = -value;
    }

    if value.denom().is_one() {
        output.extend(integer_to_spoken(value.numer(), config)?);
        return Ok(output);
    }

    if let Some(decimal) = finite_decimal_parts(&value) {
        let (integer, fraction) = decimal;
        let leading_zeros = if integer.is_zero() {
            fraction.chars().take_while(|character| *character == '0').count()
        } else {
            0
        };
        if fraction.len() > config.max_inline_fraction_digits
            || leading_zeros > config.max_inline_leading_fraction_zeros
        {
            return scientific_spoken(
                &value,
                config,
                scope_open,
                scope_close,
                output,
            );
        }
        output.extend(integer_to_spoken(&integer, config)?);
        if !fraction.is_empty() {
            output.push(config.decimal.clone());
            for character in fraction.chars() {
                let digit = character.to_digit(10).unwrap() as u8;
                output.push(digit_surface(digit, config)?.to_owned());
            }
        }
        return Ok(output);
    }

    output.push(config.rational.clone());
    output.push(scope_open.to_owned());
    output.extend(integer_to_spoken(value.numer(), config)?);
    output.push(scope_close.to_owned());
    output.push(scope_open.to_owned());
    output.extend(integer_to_spoken(value.denom(), config)?);
    output.push(scope_close.to_owned());
    Ok(output)
}

fn scientific_spoken(
    value: &BigRational,
    config: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
    mut prefix: Vec<String>,
) -> Result<Vec<String>, String> {
    let canonical = rational_to_decimal_or_fraction(value);
    if canonical.contains('/') {
        return Err("scientific spoken form requires a finite decimal value".into());
    }
    let unsigned = canonical.strip_prefix('-').unwrap_or(&canonical);
    let mut digits = unsigned.chars().filter(|character| *character != '.').collect::<String>();
    let decimal_index = unsigned.find('.').unwrap_or(unsigned.len());
    let first_nonzero = digits.find(|character| character != '0')
        .ok_or_else(|| "zero does not require scientific notation".to_owned())?;
    let exponent = decimal_index as i64 - first_nonzero as i64 - 1;
    digits.drain(..first_nonzero);
    while digits.ends_with('0') { digits.pop(); }
    let significand = if digits.len() == 1 {
        digits
    } else {
        format!("{}.{}", &digits[..1], &digits[1..])
    };
    let significand = ExactNumber::parse_written(&significand, config.max_explicit_exponent)?;

    prefix.push(config.exponent.clone());
    prefix.push(scope_open.to_owned());
    prefix.extend(inline_decimal_spoken(&significand, config)?);
    prefix.push(scope_close.to_owned());
    prefix.push(scope_open.to_owned());
    prefix.extend(integer_to_spoken(&BigInt::from(exponent), config)?);
    prefix.push(scope_close.to_owned());
    Ok(prefix)
}

fn inline_decimal_spoken(number: &ExactNumber, config: &NumberConfig) -> Result<Vec<String>, String> {
    let mut value = number.0.clone();
    let mut output = Vec::new();
    if value.is_negative() {
        output.push(config.minus.clone());
        value = -value;
    }
    let (integer, fraction) = finite_decimal_parts(&value)
        .ok_or_else(|| "inline decimal spoken form requires a finite decimal".to_owned())?;
    output.extend(integer_to_spoken(&integer, config)?);
    if !fraction.is_empty() {
        output.push(config.decimal.clone());
        for character in fraction.chars() {
            output.push(digit_surface(character.to_digit(10).unwrap() as u8, config)?.to_owned());
        }
    }
    Ok(output)
}

pub fn integer_to_spoken(value: &BigInt, config: &NumberConfig) -> Result<Vec<String>, String> {
    if value.is_negative() {
        let mut result = vec![config.minus.clone()];
        result.extend(integer_to_spoken(&(-value), config)?);
        return Ok(result);
    }
    if value.is_zero() {
        return Ok(vec![digit_surface(0, config)?.to_owned()]);
    }

    let decimal = value.to_string();
    let mut groups = Vec::<(u32, u16)>::new();
    let mut end = decimal.len();
    let mut group_index = 0u32;
    while end > 0 {
        let start = end.saturating_sub(3);
        let group = decimal[start..end]
            .parse::<u16>()
            .map_err(|_| "cannot decompose integer into decimal groups".to_owned())?;
        if group != 0 {
            groups.push((group_index * 3, group));
        }
        group_index += 1;
        end = start;
    }
    groups.reverse();

    let mut output = Vec::new();
    for (exponent, coefficient) in groups {
        if exponent > 0 {
            output.extend(canonical_outer_chain(exponent, config)?);
        }
        output.extend(coefficient_to_spoken(coefficient, config)?);
    }
    Ok(output)
}

fn parse_spoken_rational(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<NumberParse, String> {
    let mut index = start + 1;
    require_token(tokens, &mut index, scope_open, "rational numerator scope")?;
    let Some((numerator, consumed)) = parse_signed_spoken_integer(tokens, index, config)? else {
        return Err("rational numerator must be an integer".into());
    };
    index += consumed;
    require_token(tokens, &mut index, scope_close, "rational numerator scope")?;
    require_token(tokens, &mut index, scope_open, "rational denominator scope")?;
    let Some((denominator, consumed)) = parse_spoken_integer(tokens, index, config)? else {
        return Err("rational denominator must be a positive integer".into());
    };
    index += consumed;
    require_token(tokens, &mut index, scope_close, "rational denominator scope")?;
    if denominator.is_zero() {
        return Err("rational denominator cannot be zero".into());
    }
    Ok(NumberParse {
        value: ExactNumber(BigRational::new(numerator, denominator)),
        consumed: index - start,
    })
}

fn parse_spoken_exponent(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
    scope_open: &str,
    scope_close: &str,
) -> Result<NumberParse, String> {
    let mut index = start + 1;
    require_token(tokens, &mut index, scope_open, "exponent significand scope")?;
    let Some(significand) = parse_spoken_number(tokens, index, config, scope_open, scope_close)? else {
        return Err("exponent notation requires a numeric significand".into());
    };
    index += significand.consumed;
    require_token(tokens, &mut index, scope_close, "exponent significand scope")?;
    require_token(tokens, &mut index, scope_open, "exponent value scope")?;
    let Some((exponent, consumed)) = parse_signed_spoken_integer(tokens, index, config)? else {
        return Err("exponent notation requires an integer exponent".into());
    };
    index += consumed;
    require_token(tokens, &mut index, scope_close, "exponent value scope")?;
    let exponent = exponent
        .to_i64()
        .ok_or_else(|| "explicit exponent is too large for configured numeric codec".to_owned())?;
    if exponent.unsigned_abs() > u64::from(config.max_explicit_exponent) {
        return Err(format!(
            "explicit exponent {exponent} exceeds configured limit {}",
            config.max_explicit_exponent
        ));
    }
    let scale = BigRational::from_integer(pow10(exponent.unsigned_abs() as u32));
    let value = if exponent >= 0 {
        significand.value.0 * scale
    } else {
        significand.value.0 / scale
    };
    Ok(NumberParse { value: ExactNumber(value), consumed: index - start })
}

fn parse_signed_spoken_integer(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
) -> Result<Option<(BigInt, usize)>, String> {
    let negative = tokens.get(start).is_some_and(|token| token == &config.minus);
    let positive = tokens.get(start).is_some_and(|token| token == &config.plus);
    let index = start + usize::from(negative || positive);
    let Some((mut value, consumed)) = parse_spoken_integer(tokens, index, config)? else {
        return Ok(None);
    };
    if negative { value = -value; }
    Ok(Some((value, consumed + usize::from(negative || positive))))
}

fn parse_outer_chain(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
) -> Result<Option<(u32, usize)>, String> {
    let mut index = start;
    let mut surfaces = Vec::new();
    let mut exponent = 0u32;
    while let Some(token) = tokens.get(index) {
        let Some(value) = config.outer_magnitudes.get(token) else { break };
        exponent = exponent.checked_add(*value)
            .ok_or_else(|| "outer magnitude exponent overflow".to_owned())?;
        surfaces.push(token.clone());
        index += 1;
    }
    if surfaces.is_empty() {
        return Ok(None);
    }
    let canonical = canonical_outer_chain(exponent, config)?;
    if canonical != surfaces {
        return Err(format!(
            "non-canonical outer magnitude chain `{}`; canonical is `{}`",
            surfaces.join(" "), canonical.join(" ")
        ));
    }
    Ok(Some((exponent, index - start)))
}

fn canonical_outer_chain(exponent: u32, config: &NumberConfig) -> Result<Vec<String>, String> {
    if exponent == 0 || exponent % 3 != 0 {
        return Err(format!("outer magnitude exponent {exponent} is not a positive multiple of three"));
    }
    let mut choices = config.outer_magnitudes.iter()
        .map(|(surface, exponent)| (*exponent, surface.as_str()))
        .collect::<Vec<_>>();
    choices.sort_by(|left, right| right.0.cmp(&left.0));
    let mut remaining = exponent;
    let mut output = Vec::new();
    while remaining > 0 {
        let Some((value, surface)) = choices.iter().find(|(value, _)| *value <= remaining) else {
            return Err(format!("cannot represent outer magnitude exponent {exponent}"));
        };
        output.push((*surface).to_owned());
        remaining -= *value;
    }
    Ok(output)
}

fn parse_coefficient(
    tokens: &[String],
    start: usize,
    config: &NumberConfig,
    allow_zero: bool,
) -> Result<Option<(u16, usize)>, String> {
    let Some(token) = tokens.get(start) else { return Ok(None) };
    if let Some(digit) = config.digits.get(token) {
        if *digit == 0 && !allow_zero { return Ok(None) }
        return Ok(Some((*digit as u16, 1)));
    }
    if config.coefficient_magnitudes.get(token) == Some(&2) {
        let Some(hundreds) = tokens.get(start + 1).and_then(|token| config.digits.get(token)).copied() else {
            return Err(format!("`{token}` requires a coefficient digit 1..9"));
        };
        if hundreds == 0 { return Err(format!("`{token}` coefficient cannot be zero")); }
        let mut value = u16::from(hundreds) * 100;
        let mut index = start + 2;
        if tokens.get(index).and_then(|token| config.coefficient_magnitudes.get(token)) == Some(&1) {
            let tens_surface = tokens[index].clone();
            let Some(tens) = tokens.get(index + 1).and_then(|token| config.digits.get(token)).copied() else {
                return Err(format!("`{tens_surface}` requires a coefficient digit 1..9"));
            };
            if tens == 0 { return Err(format!("`{tens_surface}` coefficient cannot be zero")); }
            value += u16::from(tens) * 10;
            index += 2;
        }
        if let Some(units) = tokens.get(index).and_then(|token| config.digits.get(token)).copied() {
            if units != 0 {
                value += u16::from(units);
                index += 1;
            }
        }
        return Ok(Some((value, index - start)));
    }
    if config.coefficient_magnitudes.get(token) == Some(&1) {
        let Some(tens) = tokens.get(start + 1).and_then(|token| config.digits.get(token)).copied() else {
            return Err(format!("`{token}` requires a coefficient digit 1..9"));
        };
        if tens == 0 { return Err(format!("`{token}` coefficient cannot be zero")); }
        let mut value = u16::from(tens) * 10;
        let mut index = start + 2;
        if let Some(units) = tokens.get(index).and_then(|token| config.digits.get(token)).copied() {
            if units != 0 {
                value += u16::from(units);
                index += 1;
            }
        }
        return Ok(Some((value, index - start)));
    }
    Ok(None)
}

fn coefficient_to_spoken(value: u16, config: &NumberConfig) -> Result<Vec<String>, String> {
    if value == 0 || value > 999 {
        return Err(format!("coefficient {value} is outside 1..999"));
    }
    let mut output = Vec::new();
    let hundreds = value / 100;
    let tens = (value % 100) / 10;
    let units = value % 10;
    if hundreds > 0 {
        output.push(coefficient_magnitude_surface(2, config)?.to_owned());
        output.push(digit_surface(hundreds as u8, config)?.to_owned());
    }
    if tens > 0 {
        output.push(coefficient_magnitude_surface(1, config)?.to_owned());
        output.push(digit_surface(tens as u8, config)?.to_owned());
    }
    if units > 0 {
        output.push(digit_surface(units as u8, config)?.to_owned());
    }
    Ok(output)
}

fn digit_surface(digit: u8, config: &NumberConfig) -> Result<&str, String> {
    config.digits.iter()
        .find_map(|(surface, value)| (*value == digit).then_some(surface.as_str()))
        .ok_or_else(|| format!("no spoken form configured for digit {digit}"))
}

fn coefficient_magnitude_surface(exponent: u32, config: &NumberConfig) -> Result<&str, String> {
    config.coefficient_magnitudes.iter()
        .find_map(|(surface, value)| (*value == exponent).then_some(surface.as_str()))
        .ok_or_else(|| format!("no coefficient magnitude configured for 10^{exponent}"))
}

fn parse_decimal_mantissa(source: &str) -> Result<BigRational, String> {
    let negative = source.starts_with('-');
    let positive = source.starts_with('+');
    let unsigned = if negative || positive { &source[1..] } else { source };
    if unsigned.is_empty() {
        return Err("number has no digits".into());
    }
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap();
    let fraction = parts.next();
    if parts.next().is_some() {
        return Err("number contains more than one decimal point".into());
    }
    if whole.is_empty() || !whole.chars().all(|character| character.is_ascii_digit()) {
        return Err("invalid decimal integer part".into());
    }
    let mut numerator = parse_bigint(whole)?;
    let mut denominator = BigInt::one();
    if let Some(fraction) = fraction {
        if fraction.is_empty() || !fraction.chars().all(|character| character.is_ascii_digit()) {
            return Err("invalid decimal fractional part".into());
        }
        denominator = pow10(fraction.len() as u32);
        numerator = numerator * &denominator + parse_bigint(fraction)?;
    }
    if negative { numerator = -numerator; }
    Ok(BigRational::new(numerator, denominator))
}

fn rational_to_decimal_or_fraction(value: &BigRational) -> String {
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

fn finite_decimal_parts(value: &BigRational) -> Option<(BigInt, String)> {
    if value.is_negative() {
        return None;
    }
    let numerator = value.numer().clone();
    let denominator = value.denom().clone();
    let mut reduced = denominator.clone();
    let two = BigInt::from(2u8);
    let five = BigInt::from(5u8);
    let mut twos = 0u32;
    let mut fives = 0u32;
    while (&reduced % &two).is_zero() {
        reduced /= &two;
        twos += 1;
    }
    while (&reduced % &five).is_zero() {
        reduced /= &five;
        fives += 1;
    }
    if !reduced.is_one() {
        return None;
    }
    let scale = twos.max(fives);
    let scaled = numerator * pow_bigint(&two, scale - twos) * pow_bigint(&five, scale - fives);
    let ten_pow = pow10(scale);
    let integer = &scaled / &ten_pow;
    let remainder = &scaled % &ten_pow;
    let mut fraction = if scale == 0 {
        String::new()
    } else {
        format!("{:0>width$}", remainder, width = scale as usize)
    };
    while fraction.ends_with('0') {
        fraction.pop();
    }
    Some((integer, fraction))
}

fn parse_bigint(source: &str) -> Result<BigInt, String> {
    if source.is_empty() {
        return Err("empty integer".into());
    }
    BigInt::parse_bytes(source.as_bytes(), 10).ok_or_else(|| format!("invalid integer `{source}`"))
}

fn pow10(exponent: u32) -> BigInt {
    BigInt::from(10u8).pow(exponent)
}

fn pow_bigint(base: &BigInt, exponent: u32) -> BigInt {
    base.pow(exponent)
}

fn require_token(tokens: &[String], index: &mut usize, expected: &str, context: &str) -> Result<(), String> {
    match tokens.get(*index) {
        Some(token) if token == expected => {
            *index += 1;
            Ok(())
        }
        Some(token) => Err(format!("{context} expected `{expected}`, found `{token}`")),
        None => Err(format!("{context} expected `{expected}`, found end of input")),
    }
}

fn split_once_unique(source: &str, delimiter: char) -> Option<(&str, &str)> {
    let index = source.find(delimiter)?;
    if source[index + delimiter.len_utf8()..].contains(delimiter) {
        return None;
    }
    Some((&source[..index], &source[index + delimiter.len_utf8()..]))
}
