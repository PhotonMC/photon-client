//! The date, as `SpecialDates` and the splash texts read it.

/// Today's month and day, in UTC (the reference reads the local zone's; a few hours either side of
/// midnight may name the day before or after).
pub fn today() -> (u32, u32) {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    // Days since 1970-01-01 to a civil date (Howard Hinnant's `civil_from_days`).
    let days = i64::try_from(seconds / 86_400).unwrap_or(0) + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    (month as u32, day as u32)
}

/// `SpecialDates.isExtendedChristmas`: the twenty-fourth to the twenty-sixth of December (in UTC
/// here, where the reference reads the local zone).
pub fn is_extended_christmas() -> bool {
    matches!(today(), (12, 24..=26))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_is_a_real_month_and_day() {
        let (month, day) = today();
        assert!((1..=12).contains(&month) && (1..=31).contains(&day));
    }
}
