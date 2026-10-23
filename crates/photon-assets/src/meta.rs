//! What a pack's `pack.mcmeta` says that the stack needs (`PackFormat`, `OverlayMetadataSection`,
//! `ResourceFilterSection`): the overlays that apply to the running game, and the patterns that
//! hide what packs below it hold.

use photon_core::json::{JsonObject, JsonValue};
use regex::Regex;

/// A pack format version (`PackFormat`): a major and a minor number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackFormat {
    pub major: i32,
    pub minor: i32,
}

impl PackFormat {
    pub const fn of(major: i32, minor: i32) -> Self {
        Self { major, minor }
    }
}

fn number(value: &JsonValue) -> Option<i32> {
    value.as_i32()
}

/// A format as `min_format` and `max_format` write it: a major number, or `[major, minor]`; a
/// missing minor is `default_minor`.
fn format_of(value: &JsonValue, default_minor: i32) -> Option<PackFormat> {
    match value {
        JsonValue::Array(parts) => {
            let major = number(parts.first()?)?;
            let minor = parts.get(1).and_then(number).unwrap_or(default_minor);
            Some(PackFormat::of(major, minor))
        }
        other => Some(PackFormat::of(number(other)?, default_minor)),
    }
}

/// An inclusive range of numbers written as one number, `[min, max]` or an object with
/// `min_inclusive` and `max_inclusive`.
fn int_range(value: &JsonValue) -> Option<(i32, i32)> {
    match value {
        JsonValue::Array(parts) => Some((number(parts.first()?)?, number(parts.get(1)?)?)),
        JsonValue::Object(object) => Some((
            number(object.get_str("min_inclusive")?)?,
            number(object.get_str("max_inclusive")?)?,
        )),
        other => number(other).map(|single| (single, single)),
    }
}

/// The range of formats an overlay entry names (`IntermediaryFormat.OVERLAY_CODEC`): the new
/// `min_format` and `max_format` pair, or the old `formats` range.
pub fn format_range(entry: &JsonObject) -> Option<(PackFormat, PackFormat)> {
    let min = entry.get_str("min_format");
    let max = entry.get_str("max_format");
    match (min, max) {
        (Some(min), Some(max)) => Some((format_of(min, 0)?, format_of(max, i32::MAX)?)),
        (None, None) => {
            let (low, high) = int_range(entry.get_str("formats")?)?;
            Some((PackFormat::of(low, 0), PackFormat::of(high, 0)))
        }
        _ => None,
    }
}

/// The formats a pack declares in its `pack` section (`PackFormat.packCodec`): the
/// `min_format` and `max_format` pair, the old `supported_formats` range, or the single
/// `pack_format`. `None` when it declares none, which `PackCompatibility` calls unknown.
fn declared_formats(section: &JsonObject) -> Option<(PackFormat, PackFormat)> {
    if section.has("min_format") || section.has("max_format") {
        return format_range(section);
    }
    if let Some(range) = section.get_str("supported_formats") {
        let (low, high) = int_range(range)?;
        return Some((PackFormat::of(low, 0), PackFormat::of(high, 0)));
    }
    let single = PackFormat::of(number(section.get_str("pack_format")?)?, 0);
    Some((single, single))
}

/// `PackCompatibility.forVersion(..) == COMPATIBLE` for the pack metadata `meta`.
pub(crate) fn is_compatible(meta: &JsonObject, current: PackFormat) -> bool {
    meta.object("pack")
        .and_then(declared_formats)
        .is_some_and(|(low, high)| high >= current && current >= low)
}

/// The directories of the overlays that apply to `current` (`overlaysForVersion`), in the order
/// the file lists them.
pub(crate) fn overlays_for(meta: &JsonObject, current: PackFormat) -> Vec<String> {
    let Some(entries) = meta.object("overlays").and_then(|o| o.array("entries")) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| {
            let JsonValue::Object(entry) = entry else {
                return None;
            };
            let (low, high) = format_range(entry)?;
            let directory = entry.text("directory")?;
            let valid = !directory.is_empty()
                && directory
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
            (valid && low <= current && current <= high).then_some(directory)
        })
        .collect()
}

/// `IdentifierPattern`: a regular expression for the namespace and one for the path; a missing
/// one matches everything. Like `Pattern.asPredicate`, a pattern matches anywhere in the text.
pub(crate) struct Pattern {
    namespace: Option<Regex>,
    path: Option<Regex>,
}

impl Pattern {
    fn namespace_matches(&self, namespace: &str) -> bool {
        self.namespace
            .as_ref()
            .is_none_or(|re| re.is_match(namespace))
    }

    fn path_matches(&self, path: &str) -> bool {
        self.path.as_ref().is_none_or(|re| re.is_match(path))
    }
}

/// `ResourceFilterSection`: what the pack hides from the packs below it.
#[derive(Default)]
pub(crate) struct Filter {
    block: Vec<Pattern>,
}

impl Filter {
    pub(crate) fn is_namespace_filtered(&self, namespace: &str) -> bool {
        self.block.iter().any(|p| p.namespace_matches(namespace))
    }

    /// The path alone is tested: a pattern's namespace has no say here.
    pub(crate) fn is_path_filtered(&self, path: &str) -> bool {
        self.block.iter().any(|p| p.path_matches(path))
    }
}

fn pattern_of(entry: &JsonValue) -> Option<Pattern> {
    let JsonValue::Object(entry) = entry else {
        return None;
    };
    let compile = |key: &str| match entry.text(key) {
        Some(text) => Regex::new(&text).map(Some).ok(),
        None => Some(None),
    };
    Some(Pattern {
        namespace: compile("namespace")?,
        path: compile("path")?,
    })
}

/// The `filter` section of a `pack.mcmeta`, if it has a valid one.
pub(crate) fn filter_of(meta: &JsonObject) -> Option<Filter> {
    let block = meta.object("filter")?.array("block")?;
    let block: Option<Vec<Pattern>> = block.iter().map(pattern_of).collect();
    block.map(|block| Filter { block })
}
