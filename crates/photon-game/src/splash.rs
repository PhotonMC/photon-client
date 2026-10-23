//! The title screen's yellow line (`SplashManager`): one of the lines of `texts/splashes.txt`, or
//! the line of a special day.

use photon_core::random::RandomSource;

/// `SplashManager.DEFAULT_STYLE`'s colour, `-256`.
pub const SPLASH_COLOUR: u32 = 0xFFFF_FF00;

/// The hash of the one line the reference leaves out of the file.
const LEFT_OUT_HASH: i32 = 125_780_783;

/// `SplashManager.prepare`: the file's lines, trimmed, but the one left out.
pub fn read_splashes(assets_root: &std::path::Path) -> Vec<String> {
    read_splashes_from(&photon_core::resources::FolderResources::new(assets_root))
}

/// The same, from the topmost pack that has the file.
pub fn read_splashes_from(resources: &dyn photon_core::resources::Resources) -> Vec<String> {
    let Ok(id) = photon_core::identifier::Identifier::parse("minecraft:texts/splashes.txt") else {
        return Vec::new();
    };
    resources
        .read(&id)
        .and_then(|bytes| String::from_utf8(bytes).ok())
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

pub use photon_core::calendar::today;
