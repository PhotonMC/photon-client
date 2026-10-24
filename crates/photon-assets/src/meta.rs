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

/// The last pack format whose packs carry no minor version (`PackFormat.lastPreMinorVersion` for
/// client resources).
const LAST_PRE_MINOR: i32 = 64;

/// What a `pack` section writes about its formats (`PackFormat.IntermediaryFormat`).
struct Declared {
    min: Option<PackFormat>,
    max: Option<PackFormat>,
    supported: Option<(i32, i32)>,
    format: Option<i32>,
}

impl Declared {
    /// The main format has to sit inside the range and not be below 15
    /// (`validatePackFormatForRange`).
    fn main_in(&self, low: i32, high: i32) -> bool {
        self.format
            .is_some_and(|f| f >= low && f <= high && f >= 15)
    }

    /// `validateNewFormat` for a pack whose `min_format` and `max_format` are both written.
    fn new_range(&self, min: PackFormat, max: PackFormat) -> bool {
        if min > max {
            return false;
        }
        if min.major > LAST_PRE_MINOR {
            return self.supported.is_none()
                && self
                    .format
                    .is_none_or(|_| self.main_in(min.major, max.major));
        }
        let old = self.supported.is_some_and(|(low, high)| {
            low == min.major && (high == max.major || high == LAST_PRE_MINOR)
        });
        old && self.main_in(min.major, max.major)
    }

    /// `validate` with the `pack_format` field allowed and the old field not required.
    fn range(&self) -> Option<(PackFormat, PackFormat)> {
        match (self.min, self.max) {
            (Some(min), Some(max)) => self.new_range(min, max).then_some((min, max)),
            (None, None) => self.old_range(),
            _ => None,
        }
    }

    /// `validateOldFormat`, or a lone `pack_format`.
    fn old_range(&self) -> Option<(PackFormat, PackFormat)> {
        if let Some((low, high)) = self.supported {
            let valid = high <= LAST_PRE_MINOR && self.main_in(low, high);
            return valid.then_some((PackFormat::of(low, 0), PackFormat::of(high, 0)));
        }
        let single = self.format.filter(|f| *f <= LAST_PRE_MINOR)?;
        Some((PackFormat::of(single, 0), PackFormat::of(single, 0)))
    }
}

/// The formats a pack declares in its `pack` section (`PackFormat.packCodec`, with the
/// validation of `IntermediaryFormat.validate`). `None` when the section is invalid or declares
/// none, which `PackCompatibility` calls unknown.
fn declared_formats(section: &JsonObject) -> Option<(PackFormat, PackFormat)> {
    Declared {
        min: section.get_str("min_format").and_then(|v| format_of(v, 0)),
        max: section
            .get_str("max_format")
            .and_then(|v| format_of(v, i32::MAX)),
        supported: section.get_str("supported_formats").and_then(int_range),
        format: section.get_str("pack_format").and_then(number),
    }
    .range()
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
pub struct IdentifierPattern {
    namespace: Option<Regex>,
    path: Option<Regex>,
}

impl IdentifierPattern {
    /// An `IdentifierPattern` of a JSON object (`namespace` and `path`, both optional); none if
    /// a regular expression does not compile.
    pub fn from_json(entry: &JsonObject) -> Option<Self> {
        let compile = |key: &str| match entry.text(key) {
            Some(text) => Regex::new(&text).map(Some).ok(),
            None => Some(None),
        };
        Some(Self {
            namespace: compile("namespace")?,
            path: compile("path")?,
        })
    }

    /// `locationPredicate`: both parts match the identifier.
    pub fn matches(&self, namespace: &str, path: &str) -> bool {
        self.namespace_matches(namespace) && self.path_matches(path)
    }

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
    block: Vec<IdentifierPattern>,
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

fn pattern_of(entry: &JsonValue) -> Option<IdentifierPattern> {
    let JsonValue::Object(entry) = entry else {
        return None;
    };
    IdentifierPattern::from_json(entry)
}

/// The `filter` section of a `pack.mcmeta`, if it has a valid one.
pub(crate) fn filter_of(meta: &JsonObject) -> Option<Filter> {
    let block = meta.object("filter")?.array("block")?;
    let block: Option<Vec<IdentifierPattern>> = block.iter().map(pattern_of).collect();
    block.map(|block| Filter { block })
}
