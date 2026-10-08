//! The title screen's yellow line (`SplashManager`): one of the lines of `texts/splashes.txt`, or
//! the line of a special day.

use photon_core::random::RandomSource;

/// `SplashManager.DEFAULT_STYLE`'s colour, `-256`.
pub const SPLASH_COLOUR: u32 = 0xFFFF_FF00;

/// The hash of the one line the reference leaves out of the file.
const LEFT_OUT_HASH: i32 = 125_780_783;

/// `SplashManager.prepare`: the file's lines, trimmed, but the one left out.
pub fn read_splashes(assets_root: &std::path::Path) -> Vec<String> {
    std::fs::read_to_string(assets_root.join("minecraft/texts/splashes.txt"))
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|line| photon_core::identifier::java_string_hash(line) != LEFT_OUT_HASH)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// `SplashManager.getSplash` on the day `(month, day)`: Christmas Eve, New Year's Day and
/// Halloween have their own; else a line drawn at random, which one time in the file's length
/// (when the draw is 42) is the player's name.
pub fn pick(
    splashes: &[String],
    (month, day): (u32, u32),
    user: &str,
    random: &mut RandomSource,
) -> Option<String> {
    match (month, day) {
        (12, 24) => return Some("Merry X-mas!".to_owned()),
        (1, 1) => return Some("Happy new year!".to_owned()),
        (10, 31) => return Some("OOoooOOOoooo! Spooky!".to_owned()),
        _ => {}
    }
    let count = i32::try_from(splashes.len())
        .ok()
        .filter(|count| *count > 0)?;
    if random.next_int_bound(count).ok()? == 42 {
        return Some(format!("{} IS YOU", user.to_uppercase()));
    }
    let index = random.next_int_bound(count).ok()?;
    splashes.get(usize::try_from(index).ok()?).cloned()
}

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
