#[inline(never)]
pub(crate) fn java_negate(value: f64) -> f64 {
    f64::from_bits(value.to_bits() ^ (1_u64 << 63))
}

pub(super) fn java_f32_add(left: f32, right: f32) -> f32 {
    if right.is_nan() {
        right
    } else if left.is_nan() {
        left
    } else {
        left + right
    }
}

pub(crate) fn java_double_compare(left: f64, right: f64) -> i32 {
    if left < right {
        -1
    } else if left > right {
        1
    } else {
        let left_bits = java_double_to_long_bits(left) as i64;
        let right_bits = java_double_to_long_bits(right) as i64;
        if left_bits < right_bits {
            -1
        } else if left_bits > right_bits {
            1
        } else {
            0
        }
    }
}

pub(super) fn java_double_to_long_bits(value: f64) -> u64 {
    if value.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        value.to_bits()
    }
}

pub(crate) fn java_double_hash(value: f64) -> i32 {
    let bits = java_double_to_long_bits(value);
    (bits ^ (bits >> 32)) as i32
}

pub(super) fn java_float_hash(value: f32) -> i32 {
    let bits = if value.is_nan() {
        0x7fc0_0000_u32
    } else {
        value.to_bits()
    };
    bits as i32
}

pub(crate) fn java_double_to_string(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }

    let raw = format!("{value:?}");
    if value == 0.0 || (value.abs() >= 1.0e-3 && value.abs() < 1.0e7) {
        return raw;
    }

    if let Some((mantissa, exponent)) = raw.split_once('e') {
        return java_scientific_string(
            mantissa,
            exponent.parse().expect("Rust exponent is numeric"),
        );
    }
    java_decimal_to_scientific(&raw)
}

pub(super) fn java_scientific_string(mantissa: &str, exponent: i32) -> String {
    let mantissa = if mantissa.contains('.') {
        mantissa.to_owned()
    } else {
        format!("{mantissa}.0")
    };
    format!("{mantissa}E{exponent}")
}

pub(super) fn java_decimal_to_scientific(raw: &str) -> String {
    let (sign, magnitude) = match raw.strip_prefix('-') {
        Some(magnitude) => ("-", magnitude),
        None => ("", raw),
    };
    let decimal = magnitude.find('.').unwrap_or(magnitude.len());
    let digits = magnitude.replace('.', "");
    let digits = digits.trim_end_matches('0');
    let first = digits
        .bytes()
        .position(|digit| digit != b'0')
        .expect("nonzero finite value has a significant digit");
    let exponent = decimal as i32 - first as i32 - 1;
    let mut mantissa = String::new();
    mantissa.push(digits.as_bytes()[first] as char);
    mantissa.push('.');
    if first + 1 < digits.len() {
        mantissa.push_str(&digits[first + 1..]);
    } else {
        mantissa.push('0');
    }
    format!("{sign}{mantissa}E{exponent}")
}

/// Return Java `Mth.sign(double)` with its NaN branch preserved.
pub fn mth_sign(number: f64) -> i32 {
    if number == 0.0 {
        0
    } else if number > 0.0 {
        1
    } else {
        -1
    }
}

pub(super) fn java_decimal_digit(character: char) -> Option<i64> {
    const DIGIT_RANGES: &[(u32, u32)] = &[
        (0x0030, 0x0039),
        (0x0660, 0x0669),
        (0x06f0, 0x06f9),
        (0x07c0, 0x07c9),
        (0x0966, 0x096f),
        (0x09e6, 0x09ef),
        (0x0a66, 0x0a6f),
        (0x0ae6, 0x0aef),
        (0x0b66, 0x0b6f),
        (0x0be6, 0x0bef),
        (0x0c66, 0x0c6f),
        (0x0ce6, 0x0cef),
        (0x0d66, 0x0d6f),
        (0x0de6, 0x0def),
        (0x0e50, 0x0e59),
        (0x0ed0, 0x0ed9),
        (0x0f20, 0x0f29),
        (0x1040, 0x1049),
        (0x1090, 0x1099),
        (0x17e0, 0x17e9),
        (0x1810, 0x1819),
        (0x1946, 0x194f),
        (0x19d0, 0x19d9),
        (0x1a80, 0x1a89),
        (0x1a90, 0x1a99),
        (0x1b50, 0x1b59),
        (0x1bb0, 0x1bb9),
        (0x1c40, 0x1c49),
        (0x1c50, 0x1c59),
        (0xa620, 0xa629),
        (0xa8d0, 0xa8d9),
        (0xa900, 0xa909),
        (0xa9d0, 0xa9d9),
        (0xa9f0, 0xa9f9),
        (0xaa50, 0xaa59),
        (0xabf0, 0xabf9),
        (0xff10, 0xff19),
    ];
    let codepoint = character as u32;
    DIGIT_RANGES.iter().find_map(|&(first, last)| {
        if (first..=last).contains(&codepoint) {
            Some(i64::from(codepoint - first))
        } else {
            None
        }
    })
}

pub(super) fn parse_java_i32(input: &str) -> Option<i32> {
    let mut characters = input.chars();
    let first = characters.next()?;
    let (negative, first_digit) = match first {
        '-' => (true, None),
        '+' => (false, None),
        character => (false, Some(java_decimal_digit(character)?)),
    };
    let limit = if negative {
        i64::from(i32::MIN)
    } else {
        -i64::from(i32::MAX)
    };
    let mut result = 0_i64;
    let mut digit_count = 0;
    if let Some(digit) = first_digit {
        result = -digit;
        digit_count = 1;
    }
    for character in characters {
        let digit = java_decimal_digit(character)?;
        result = result.checked_mul(10)?.checked_sub(digit)?;
        if result < limit {
            return None;
        }
        digit_count += 1;
    }
    if digit_count == 0 {
        return None;
    }
    if negative {
        Some(result as i32)
    } else {
        Some((-result) as i32)
    }
}
