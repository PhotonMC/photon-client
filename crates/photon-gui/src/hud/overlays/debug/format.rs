//! `String.format("%.Nf", value)` as Java prints it: the shortest decimal digits of the number
//! (the ones `Double.toString` gives), rounded half up at the precision. The binary value's exact
//! expansion is not what rounds, so `0.15` prints as `0.2` at one digit.

pub fn fixed(value: f64, decimals: usize) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value.is_infinite() {
        return format!("{sign}Infinity");
    }
    let (digits, integer_len) = rounded_digits(value.abs(), decimals);
    let text = |slice: &[u8]| {
        slice
            .iter()
            .map(|d| char::from(b'0' + d))
            .collect::<String>()
    };
    let integer = if integer_len == 0 {
        "0".to_owned()
    } else {
        text(&digits[..integer_len])
    };
    match decimals {
        0 => format!("{sign}{integer}"),
        _ => format!("{sign}{integer}.{}", text(&digits[integer_len..])),
    }
}

/// The shortest digits of a finite, non-negative number, cut at `decimals` places and rounded half
/// up: the digits of the integer part then of the fraction, and how many belong to the integer part.
fn rounded_digits(value: f64, decimals: usize) -> (Vec<u8>, usize) {
    let scientific = format!("{value:e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa
        .bytes()
        .filter(u8::is_ascii_digit)
        .map(|d| d - b'0')
        .collect();
    // The value is 0.d1d2d3... times ten to the `point`.
    let point = exponent + 1;
    if point <= 0 {
        let mut padded = vec![0; (-point) as usize];
        padded.append(&mut digits);
        digits = padded;
    }
    let mut integer_len = point.max(0) as usize;
    let keep = integer_len + decimals;
    digits.resize(digits.len().max(keep + 1), 0);
    let round_up = digits[keep] >= 5;
    digits.truncate(keep);
    if round_up && carry(&mut digits) {
        digits.insert(0, 1);
        integer_len += 1;
    }
    (digits, integer_len)
}

/// Adds one in the last place; whether it carried out of the first digit.
fn carry(digits: &mut [u8]) -> bool {
    for digit in digits.iter_mut().rev() {
        if *digit == 9 {
            *digit = 0;
        } else {
            *digit += 1;
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::fixed;

    #[test]
    fn it_rounds_the_shortest_digits_half_up() {
        assert_eq!(fixed(0.15, 1), "0.2");
        assert_eq!(fixed(22.25, 1), "22.3");
        assert_eq!(fixed(-22.25, 1), "-22.3");
        assert_eq!(fixed(64.0625, 3), "64.063");
        assert_eq!(fixed(0.0, 3), "0.000");
        assert_eq!(fixed(-0.0, 1), "-0.0");
        assert_eq!(fixed(-0.04, 1), "-0.0");
        assert_eq!(fixed(9.9996, 3), "10.000");
        assert_eq!(fixed(0.9996, 3), "1.000");
        assert_eq!(fixed(1234.5, 0), "1235");
        assert_eq!(fixed(0.000_001_2, 5), "0.00000");
        assert_eq!(fixed(1.0e10, 2), "10000000000.00");
    }
}
