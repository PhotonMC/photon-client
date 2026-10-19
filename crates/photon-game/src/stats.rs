//! How the statistics screen writes a number (`StatFormatter`).

use crate::commands::reader::java_double;
use photon_data::stat::Formatter;

/// `StatFormatter.format`: the number as its statistic's formatter writes it.
pub fn format(formatter: Formatter, value: i32) -> String {
    match formatter {
        Formatter::Default => grouped(value),
        Formatter::DivideByTen => decimal(f64::from(value) * 0.1),
        Formatter::Distance => distance(value),
        Formatter::Time => time(value),
    }
}

/// `NumberFormat.getIntegerInstance(Locale.US)`: thousands grouped by a comma.
fn grouped(value: i32) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    if value < 0 { format!("-{out}") } else { out }
}

/// `DECIMAL_FORMAT`, `########0.00`: two decimals, the exact value rounded half to even.
fn decimal(value: f64) -> String {
    format!("{value:.2}")
}

fn distance(cm: i32) -> String {
    let meters = f64::from(cm) / 100.0;
    let kilometers = meters / 1000.0;
    if kilometers > 0.5 {
        format!("{} km", decimal(kilometers))
    } else if meters > 0.5 {
        format!("{} m", decimal(meters))
    } else {
        format!("{cm} cm")
    }
}

fn time(ticks: i32) -> String {
    let seconds = f64::from(ticks) / 20.0;
    let minutes = seconds / 60.0;
    let hours = minutes / 60.0;
    let days = hours / 24.0;
    let years = days / 365.0;
    if years > 0.5 {
        format!("{} y", decimal(years))
    } else if days > 0.5 {
        format!("{} d", decimal(days))
    } else if hours > 0.5 {
        format!("{} h", decimal(hours))
    } else if minutes > 0.5 {
        format!("{} min", decimal(minutes))
    } else {
        format!("{} s", java_double(seconds))
    }
}
