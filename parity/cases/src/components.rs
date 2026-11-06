//! Item stacks as the wire holds them: data component patches, tags and
//! the registry ids the scenarios use, all as hex.

use crate::bits::{f32_bits, f64_bits, hex, varint};

// Data component type ids.
pub const CUSTOM_DATA: i64 = 0;
pub const MAX_STACK_SIZE: i64 = 1;
pub const MAX_DAMAGE: i64 = 2;
pub const DAMAGE: i64 = 3;
pub const UNBREAKABLE: i64 = 4;
pub const CUSTOM_MODEL_DATA: i64 = 17;
pub const CUSTOM_NAME: i64 = 6;
pub const ITEM_NAME: i64 = 9;
pub const LORE: i64 = 11;
pub const RARITY: i64 = 12;
pub const ENCHANTMENTS: i64 = 13;
pub const CAN_PLACE_ON: i64 = 14;
pub const CAN_BREAK: i64 = 15;
pub const REPAIR_COST: i64 = 19;
pub const GLINT: i64 = 21;
pub const STORED_ENCHANTMENTS: i64 = 45;
pub const DYED_COLOR: i64 = 47;
pub const POTION_CONTENTS: i64 = 53;
pub const MAP_ID: i64 = 48;
pub const MAP_POST_PROCESSING: i64 = 50;
pub const BUNDLE_CONTENTS: i64 = 52;
pub const WRITTEN_BOOK_CONTENT: i64 = 57;
pub const INSTRUMENT: i64 = 63;
pub const OMINOUS_BOTTLE_AMPLIFIER: i64 = 65;
pub const RECIPES: i64 = 68;
pub const BANNER_PATTERNS: i64 = 74;
pub const CONTAINER: i64 = 77;

// The vanilla enchantments' network ids.
pub const BINDING_CURSE: i64 = 2;
pub const EFFICIENCY: i64 = 8;
pub const MENDING: i64 = 23;
pub const PROTECTION: i64 = 28;
pub const RIPTIDE: i64 = 32;
pub const SHARPNESS: i64 = 33;
pub const SMITE: i64 = 35;
pub const UNBREAKING: i64 = 40;
pub const VANISHING_CURSE: i64 = 41;

static CASE_VERSION: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Names the version the cases are generated for: the ids below are the newest version's, and the
/// version before it numbers some of them differently.
pub fn set_version(version: &str) {
    if let Ok(mut current) = CASE_VERSION.write() {
        *current = Some(version.to_owned());
    }
}

pub fn is_26_2() -> bool {
    CASE_VERSION.read().is_ok_and(|current| current.as_deref() == Some("26.2"))
}

/// A component type id (as the newest version numbers it) as the version under test does.
pub fn kind_number(kind: i64) -> i64 {
    if is_26_2() { component_26_2(kind) } else { kind }
}

/// `lodestone_tracker`'s type id as a hex byte.
pub fn lodestone_tracker_hex() -> &'static str {
    if is_26_2() { "43" } else { "45" }
}

/// The data component type ids that 26.2 numbers differently: (26.3's, 26.2's).
const COMPONENTS_26_2: [(i64, i64); 40] = [
    (42, 41),
    (45, 42),
    (46, 43),
    (47, 44),
    (48, 46),
    (49, 47),
    (50, 48),
    (51, 49),
    (52, 50),
    (53, 51),
    (54, 52),
    (55, 53),
    (56, 54),
    (57, 55),
    (58, 56),
    (59, 57),
    (60, 58),
    (61, 59),
    (62, 60),
    (63, 61),
    (64, 62),
    (65, 63),
    (66, 64),
    (67, 65),
    (68, 66),
    (69, 67),
    (70, 68),
    (71, 69),
    (72, 70),
    (73, 71),
    (74, 72),
    (75, 73),
    (76, 74),
    (77, 75),
    (78, 76),
    (79, 77),
    (80, 78),
    (81, 79),
    (82, 80),
    (83, 81),
];

/// A data component type id as 26.2 numbers it (the constants are 26.3's).
fn component_26_2(kind: i64) -> i64 {
    COMPONENTS_26_2.iter().find(|(newest, _)| *newest == kind).map_or(kind, |(_, older)| *older)
}

/// Item registry ids of the items a stack holds here: (name, 26.3's id, 26.2's id).
const ITEMS: [(&str, i64, i64); 5] =
    [("stone", 1, 1), ("dirt", 55, 55), ("iron_sword", 1045, 959), ("bundle", 1153, 1065), ("ender_pearl", 1265, 1144)];

/// The registry id of an item of the version under test.
pub fn item_id(item: &str) -> i64 {
    let (_, newest, older) =
        ITEMS.iter().find(|(name, ..)| *name == item).unwrap_or_else(|| panic!("no registry id for {item}"));
    if is_26_2() { *older } else { *newest }
}

/// A protocol VarInt as hex.
pub fn vi(value: i64) -> String {
    hex(&varint(value))
}

/// `ByteBufCodecs.STRING_UTF8`.
pub fn text(value: &str) -> String {
    format!("{}{}", vi(value.len() as i64), hex(value.as_bytes()))
}

/// A network tag.
#[derive(Clone, Debug)]
pub enum Tag {
    Text(String),
    Compound(Vec<(String, Tag)>),
    List(Vec<Tag>),
    Byte(i64),
    Short(i64),
    Int(i64),
    Long(i64),
    Float(f64),
    Double(f64),
    Bytes(Vec<i64>),
    Ints(Vec<i64>),
    Longs(Vec<i64>),
}

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

/// A compound of `(name, tag)` pairs in the order given.
pub fn compound(pairs: Vec<(&str, Tag)>) -> Tag {
    Tag::Compound(pairs.into_iter().map(|(name, tag)| (name.to_owned(), tag)).collect())
}

fn big_endian(value: i64, size: usize) -> String {
    hex(&value.to_be_bytes()[8 - size..])
}

fn array(kind: u8, size: usize, items: &[i64]) -> (u8, String) {
    let body: String = items.iter().map(|item| big_endian(*item, size)).collect();
    (kind, format!("{:08x}{body}", items.len()))
}

/// The tag's type id and its payload.
fn encode(tag: &Tag) -> (u8, String) {
    match tag {
        Tag::Bytes(items) => array(7, 1, items),
        Tag::Ints(items) => array(11, 4, items),
        Tag::Longs(items) => array(12, 8, items),
        Tag::Compound(pairs) => encode_compound(pairs),
        Tag::List(items) => encode_list(items),
        scalar => encode_scalar(scalar),
    }
}

/// A tag that is a single value.
fn encode_scalar(tag: &Tag) -> (u8, String) {
    match tag {
        Tag::Text(value) => (8, format!("{:04x}{}", value.len(), hex(value.as_bytes()))),
        Tag::Byte(value) => (1, big_endian(*value, 1)),
        Tag::Short(value) => (2, big_endian(*value, 2)),
        Tag::Int(value) => (3, big_endian(*value, 4)),
        Tag::Long(value) => (4, big_endian(*value, 8)),
        Tag::Float(value) => (5, f32_bits(*value)),
        Tag::Double(value) => (6, f64_bits(*value)),
        _ => unreachable!("encode handles the other kinds"),
    }
}

fn encode_compound(pairs: &[(String, Tag)]) -> (u8, String) {
    let mut payload = String::new();
    for (name, inner) in pairs {
        let (kind, body) = encode(inner);
        payload.push_str(&format!("{kind:02x}{:04x}{}{body}", name.len(), hex(name.as_bytes())));
    }
    (10, payload + "00")
}

/// A list: a mix of types is wrapped, each element in a compound under an
/// empty name.
fn encode_list(items: &[Tag]) -> (u8, String) {
    let mut element = items.first().map_or(0, |first| encode(first).0);
    let mixed = items.iter().any(|item| encode(item).0 != element);
    let wrapped: Vec<Tag>;
    let items = if mixed {
        element = 10;
        wrapped = items
            .iter()
            .map(|item| match item {
                Tag::Compound(_) => item.clone(),
                other => Tag::Compound(vec![(String::new(), other.clone())]),
            })
            .collect();
        &wrapped[..]
    } else {
        items
    };
    let body: String = items.iter().map(|item| encode(item).1).collect();
    (9, format!("{element:02x}{:08x}{body}", items.len()))
}

/// A tag with its type id first.
pub fn nbt(tag: &Tag) -> String {
    let (kind, payload) = encode(tag);
    format!("{kind:02x}{payload}")
}

/// A tag of a string.
pub fn nbt_text(value: &str) -> String {
    nbt(&Tag::from(value))
}

/// `DataComponentPatch.STREAM_CODEC`: (type id, value hex) pairs, then
/// removed type ids.
pub fn patch_of(added: &[(i64, String)], removed: &[i64]) -> String {
    let mut out = format!("{}{}", vi(added.len() as i64), vi(removed.len() as i64));
    let number = |kind: i64| if is_26_2() { component_26_2(kind) } else { kind };
    for (kind, value) in added {
        out.push_str(&vi(number(*kind)));
        out.push_str(value);
    }
    for kind in removed {
        out.push_str(&vi(number(*kind)));
    }
    out
}

/// A patch that adds components and removes none.
pub fn patch(added: &[(i64, String)]) -> String {
    patch_of(added, &[])
}

/// `custom_model_data`: the float bits (hex, so a NaN can be spelled), flags, strings and colours,
/// each a counted list.
pub fn custom_model_data(floats: &[&str], flags: &[bool], strings: &[&str], colors: &[i64]) -> String {
    let floats = format!("{}{}", vi(floats.len() as i64), floats.concat());
    let flags = format!(
        "{}{}",
        vi(flags.len() as i64),
        flags.iter().map(|flag| format!("{:02x}", u8::from(*flag))).collect::<String>()
    );
    let strings =
        format!("{}{}", vi(strings.len() as i64), strings.iter().map(|value| text(value)).collect::<String>());
    let colors = format!(
        "{}{}",
        vi(colors.len() as i64),
        colors.iter().map(|color| format!("{:08x}", *color as i32 as u32)).collect::<String>()
    );
    format!("{floats}{flags}{strings}{colors}")
}

/// `enchantments`: a level for each enchantment.
pub fn enchantments(levels: &[(i64, i64)]) -> String {
    let body: String = levels.iter().map(|(kind, level)| format!("{}{}", vi(*kind), vi(*level))).collect();
    format!("{}{body}", vi(levels.len() as i64))
}

/// A stack in a bundle or a container: its item, count and patch hex.
pub type Stack = (&'static str, i64, String);

/// A stack with no components.
pub fn stack(item: &'static str, count: i64) -> Stack {
    (item, count, patch(&[]))
}

/// A `bundle_contents` patch: a stack per item, first first.
pub fn bundle(items: &[Stack]) -> String {
    let stacks: String =
        items.iter().map(|(item, count, inner)| format!("{}{}{inner}", vi(item_id(item)), vi(*count))).collect();
    patch(&[(BUNDLE_CONTENTS, format!("{}{stacks}", vi(items.len() as i64)))])
}

/// `can_break` or `can_place_on` naming blocks by registry id: a list of one predicate with only
/// the block set (`blocks` present; no state, nbt or component tests).
pub fn adventure_blocks(blocks: &[i64]) -> String {
    let ids: String = blocks.iter().map(|id| vi(*id)).collect();
    format!("{}01{}{ids}00000000", vi(1), vi(blocks.len() as i64 + 1))
}

/// A `can_break` predicate naming one block and one state property: `matcher` is the value
/// matcher's hex (an exact value, or a range of two optional bounds).
pub fn adventure_state(block: i64, property: &str, matcher: &str) -> String {
    format!("{}01{}{}01{}{}{matcher}000000", vi(1), vi(2), vi(block), vi(1), text(property))
}

/// A predicate naming a block tag.
pub fn adventure_tag(tag: &str) -> String {
    format!("{}01{}{}00000000", vi(1), vi(0), text(tag))
}
