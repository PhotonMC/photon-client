//! The numbers of the structure block's boxes as the game reads and writes them: `Integer.parseInt`,
//! `Long.valueOf`, `Float.valueOf` and `DecimalFormat("0.0###")`.

/// `parseCoordinate`: a plain integer, or 0.
pub fn coordinate(text: &str) -> i32 {
    parse_integer(text, i64::from(i32::MIN), i64::from(i32::MAX)).map_or(0, |value| value as i32)
}

/// `parseSeed`: a plain integer, or 0.
pub fn seed(text: &str) -> i64 {
    parse_integer(text, i64::MIN, i64::MAX).unwrap_or(0)
}

/// `Integer.parseInt` and `Long.parseLong`: an optional sign and ASCII digits, no spaces.
fn parse_integer(text: &str, low: i64, high: i64) -> Option<i64> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let magnitude: i128 = digits.parse().ok()?;
    let value = if negative { -magnitude } else { magnitude };
    let (low, high) = (i128::from(low), i128::from(high));
    (low..=high).contains(&value).then_some(value as i64)
}

/// `parseIntegrity`: what `Float.valueOf` reads, or 1.
pub fn integrity(text: &str) -> f32 {
    parse_float(text).unwrap_or(1.0)
}

/// `Float.valueOf`: spaces and control characters are trimmed from both ends, then a sign, `NaN`
/// or `Infinity`, or decimal digits with a point and an exponent, and a trailing `f` or `d`.
fn parse_float(text: &str) -> Option<f32> {
    let trimmed = text.trim_matches(|each: char| each <= ' ');
    let (sign, body) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    match body {
        "NaN" => return Some(f32::NAN),
        "Infinity" => return Some(sign * f32::INFINITY),
        _ => {}
    }
    let number = body.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(body);
    let (mantissa, exponent) = match number.find(['e', 'E']) {
        Some(at) => (&number[..at], Some(&number[at + 1..])),
        None => (number, None),
    };
    if !is_decimal(mantissa) || exponent.is_some_and(|exponent| !is_exponent(exponent)) {
        return None;
    }
    number.parse::<f32>().ok().map(|value| sign * value)
}

/// Digits with at most one point and at least one digit.
fn is_decimal(mantissa: &str) -> bool {
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
    whole.len() + fraction.len() > 0 && digits(whole) && digits(fraction)
}

/// The digits of an exponent, after an optional sign.
fn is_exponent(exponent: &str) -> bool {
    let unsigned = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
    !unsigned.is_empty() && unsigned.bytes().all(|byte| byte.is_ascii_digit())
}

/// `DecimalFormat("0.0###").format(integrity)`: at least one and at most four decimals.
pub fn format_integrity(value: f32) -> String {
    let value = f64::from(value);
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-\u{221e}" } else { "\u{221e}" }.to_owned();
    }
    let text = format!("{:.4}", value.abs());
    let trimmed = text.trim_end_matches('0');
    let trimmed = if trimmed.ends_with('.') {
        format!("{trimmed}0")
    } else {
        trimmed.to_owned()
    };
    // The format keeps the sign of a value that rounds to nothing: `-0.0`.
    if value.is_sign_negative() {
        format!("-{trimmed}")
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_are_plain_or_nothing() {
        assert_eq!(coordinate("12"), 12);
        assert_eq!(coordinate("+7"), 7);
        assert_eq!(coordinate("-3"), -3);
        assert_eq!(coordinate(" 3"), 0);
        assert_eq!(coordinate("2147483648"), 0);
        assert_eq!(coordinate("1.5"), 0);
        assert_eq!(seed("-9223372036854775808"), i64::MIN);
        assert_eq!(seed(""), 0);
    }

    #[test]
    fn floats_follow_the_games_grammar() {
        assert_eq!(integrity(" 0.5 "), 0.5);
        assert_eq!(integrity("1e-1"), 0.1);
        assert_eq!(integrity(".5f"), 0.5);
        assert_eq!(integrity("inf"), 1.0);
        assert_eq!(integrity("abc"), 1.0);
        assert_eq!(integrity("-Infinity"), f32::NEG_INFINITY);
        assert!(integrity("NaN").is_nan());
    }

    #[test]
    fn integrity_is_written_with_one_to_four_decimals() {
        assert_eq!(format_integrity(1.0), "1.0");
        assert_eq!(format_integrity(0.5), "0.5");
        assert_eq!(format_integrity(0.123456), "0.1235");
        assert_eq!(format_integrity(0.0), "0.0");
        assert_eq!(format_integrity(-0.00001), "-0.0");
    }
}
