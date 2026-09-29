//! Text components, and the language they are put into words with.
//!
//! A component is a piece of content (literal text, or a translation key
//! with arguments) under a style, followed by siblings that inherit the
//! style. It comes as the data JSON writes it or as the tag the network
//! carries; the language turns it into runs of text, each with the style it
//! ends up in. Not read: keybinds, scores, selectors, NBT contents, click
//! and hover events, fonts and the shadow colour.

use std::collections::HashMap;

use voxel_core::json::{JsonObject, JsonValue};
use voxel_core::nbt::FieldValue;

/// `ChatFormatting`'s colours, by name.
pub fn named_colour(name: &str) -> Option<u32> {
    Some(match name {
        "black" => 0x00_0000,
        "dark_blue" => 0x00_00AA,
        "dark_green" => 0x00_AA00,
        "dark_aqua" => 0x00_AAAA,
        "dark_red" => 0xAA_0000,
        "dark_purple" => 0xAA_00AA,
        "gold" => 0xFF_AA00,
        "gray" => 0xAA_AAAA,
        "dark_gray" => 0x55_5555,
        "blue" => 0x55_55FF,
        "green" => 0x55_FF55,
        "aqua" => 0x55_FFFF,
        "red" => 0xFF_5555,
        "light_purple" => 0xFF_55FF,
        "yellow" => 0xFF_FF55,
        "white" => 0xFF_FFFF,
        _ => return None,
    })
}

/// `Style`, as far as the text is drawn by it. A field left unset takes
/// what the parent has.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Style {
    /// `0xRRGGBB`.
    pub colour: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
}

impl Style {
    /// A style of just a colour.
    pub fn coloured(name: &str) -> Self {
        Self {
            colour: named_colour(name),
            ..Self::default()
        }
    }

    /// `Style.applyTo`: this style over a parent's.
    pub fn apply_to(&self, parent: &Style) -> Style {
        Style {
            colour: self.colour.or(parent.colour),
            bold: self.bold.or(parent.bold),
            italic: self.italic.or(parent.italic),
            underlined: self.underlined.or(parent.underlined),
            strikethrough: self.strikethrough.or(parent.strikethrough),
            obfuscated: self.obfuscated.or(parent.obfuscated),
        }
    }
}

/// What a component says.
#[derive(Debug, Clone, PartialEq)]
pub enum Content {
    Literal(String),
    Translatable {
        key: String,
        fallback: Option<String>,
        args: Vec<Component>,
    },
}

/// `Component`.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub content: Content,
    pub style: Style,
    pub siblings: Vec<Component>,
}

impl Component {
    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            content: Content::Literal(text.into()),
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    pub fn empty() -> Self {
        Self::literal("")
    }

    pub fn translatable(key: impl Into<String>) -> Self {
        Self::translatable_with(key, Vec::new())
    }

    pub fn translatable_with(key: impl Into<String>, args: Vec<Component>) -> Self {
        Self {
            content: Content::Translatable {
                key: key.into(),
                fallback: None,
                args,
            },
            style: Style::default(),
            siblings: Vec::new(),
        }
    }

    /// `withStyle(ChatFormatting)` for a colour.
    pub fn with_colour(mut self, name: &str) -> Self {
        self.style.colour = named_colour(name);
        self
    }

    /// `withStyle` under a style: what this sets, over what it had.
    pub fn with_style(mut self, style: &Style) -> Self {
        self.style = style.apply_to(&self.style);
        self
    }

    pub fn italic(mut self) -> Self {
        self.style.italic = Some(true);
        self
    }

    pub fn append(mut self, sibling: Component) -> Self {
        self.siblings.push(sibling);
        self
    }

    /// A component from data JSON: a string, a list (its first element with
    /// the others as siblings) or an object.
    pub fn from_json(value: &JsonValue) -> Option<Self> {
        match value {
            JsonValue::String(units) => Some(Self::literal(String::from_utf16_lossy(units))),
            JsonValue::Number(text) => Some(Self::literal(text.clone())),
            JsonValue::Bool(value) => Some(Self::literal(value.to_string())),
            JsonValue::Array(items) => {
                let mut items = items.iter().filter_map(Self::from_json);
                let mut first = items.next()?;
                first.siblings.extend(items);
                Some(first)
            }
            JsonValue::Object(object) => Self::from_json_object(object),
            JsonValue::Null => None,
        }
    }

    fn from_json_object(object: &JsonObject) -> Option<Self> {
        let content = if let Some(text) = object.get_str("text") {
            Content::Literal(match text {
                JsonValue::String(units) => String::from_utf16_lossy(units),
                JsonValue::Number(text) => text.clone(),
                JsonValue::Bool(value) => value.to_string(),
                _ => return None,
            })
        } else if let Some(key) = object.text("translate") {
            Content::Translatable {
                key,
                fallback: object.text("fallback"),
                args: object
                    .array("with")
                    .unwrap_or(&[])
                    .iter()
                    .filter_map(Self::from_json)
                    .collect(),
            }
        } else {
            Content::Literal(String::new())
        };
        let flag = |key: &str| object.bool(key);
        Some(Self {
            content,
            style: Style {
                colour: object.text("color").and_then(|name| parse_colour(&name)),
                bold: flag("bold"),
                italic: flag("italic"),
                underlined: flag("underlined"),
                strikethrough: flag("strikethrough"),
                obfuscated: flag("obfuscated"),
            },
            siblings: object
                .array("extra")
                .unwrap_or(&[])
                .iter()
                .filter_map(Self::from_json)
                .collect(),
        })
    }

    /// A component from the tag the network carries.
    pub fn from_nbt(value: &FieldValue) -> Option<Self> {
        match value {
            FieldValue::Str(text) => Some(Self::literal(text.clone())),
            FieldValue::Byte(number) => Some(Self::literal(number.to_string())),
            FieldValue::Short(number) => Some(Self::literal(number.to_string())),
            FieldValue::Int(number) => Some(Self::literal(number.to_string())),
            FieldValue::Long(number) => Some(Self::literal(number.to_string())),
            FieldValue::Float(number) => Some(Self::literal(number.to_string())),
            FieldValue::Double(number) => Some(Self::literal(number.to_string())),
            FieldValue::List(items) => {
                let mut items = items.iter().filter_map(Self::from_nbt);
                let mut first = items.next()?;
                first.siblings.extend(items);
                Some(first)
            }
            FieldValue::Compound(fields) => {
                let get = |key: &str| {
                    fields
                        .iter()
                        .find(|field| field.key == key)
                        .map(|f| &f.value)
                };
                let string = |key: &str| match get(key) {
                    Some(FieldValue::Str(text)) => Some(text.clone()),
                    _ => None,
                };
                let flag = |key: &str| match get(key) {
                    Some(FieldValue::Byte(value)) => Some(*value != 0),
                    _ => None,
                };
                let content = if let Some(text) = get("text") {
                    match Self::from_nbt(text)?.content {
                        content @ Content::Literal(_) => content,
                        _ => return None,
                    }
                } else if let Some(key) = string("translate") {
                    Content::Translatable {
                        key,
                        fallback: string("fallback"),
                        args: match get("with") {
                            Some(FieldValue::List(items)) => {
                                items.iter().filter_map(Self::from_nbt).collect()
                            }
                            _ => Vec::new(),
                        },
                    }
                } else {
                    Content::Literal(String::new())
                };
                Some(Self {
                    content,
                    style: Style {
                        colour: string("color").and_then(|name| parse_colour(&name)),
                        bold: flag("bold"),
                        italic: flag("italic"),
                        underlined: flag("underlined"),
                        strikethrough: flag("strikethrough"),
                        obfuscated: flag("obfuscated"),
                    },
                    siblings: match get("extra") {
                        Some(FieldValue::List(items)) => {
                            items.iter().filter_map(Self::from_nbt).collect()
                        }
                        _ => Vec::new(),
                    },
                })
            }
            _ => None,
        }
    }
}

/// A colour by name or as `#RRGGBB`.
fn parse_colour(name: &str) -> Option<u32> {
    match name.strip_prefix('#') {
        Some(hex) => u32::from_str_radix(hex, 16).ok().filter(|_| hex.len() == 6),
        None => named_colour(name),
    }
}

/// The strings of a JSON object of strings, in order: what a language file
/// is. The general JSON reader is far too slow for its thousands of keys.
fn flat_strings(text: &str) -> Option<Vec<(String, String)>> {
    let bytes = text.as_bytes();
    let mut at = 0;
    let skip_space = |at: &mut usize| {
        while *at < bytes.len() && bytes[*at].is_ascii_whitespace() {
            *at += 1;
        }
    };
    let string = |at: &mut usize| -> Option<String> {
        if bytes.get(*at) != Some(&b'"') {
            return None;
        }
        *at += 1;
        let mut units: Vec<u16> = Vec::new();
        let mut plain_start = *at;
        let mut out = String::new();
        loop {
            match *bytes.get(*at)? {
                b'"' => {
                    out.push_str(&String::from_utf16_lossy(&units));
                    out.push_str(&text[plain_start..*at]);
                    *at += 1;
                    return Some(out);
                }
                b'\\' => {
                    out.push_str(&text[plain_start..*at]);
                    *at += 1;
                    let escape = *bytes.get(*at)?;
                    *at += 1;
                    let unit = match escape {
                        b'"' => u16::from(b'"'),
                        b'\\' => u16::from(b'\\'),
                        b'/' => u16::from(b'/'),
                        b'b' => 8,
                        b'f' => 12,
                        b'n' => u16::from(b'\n'),
                        b'r' => u16::from(b'\r'),
                        b't' => u16::from(b'\t'),
                        b'u' => {
                            let hex = text.get(*at..*at + 4)?;
                            *at += 4;
                            u16::from_str_radix(hex, 16).ok()?
                        }
                        _ => return None,
                    };
                    units.push(unit);
                    // Units gather until a plain stretch follows.
                    if bytes.get(*at) != Some(&b'\\') {
                        out.push_str(&String::from_utf16_lossy(&units));
                        units.clear();
                    }
                    plain_start = *at;
                }
                _ => *at += 1,
            }
        }
    };
    skip_space(&mut at);
    if bytes.get(at) != Some(&b'{') {
        return None;
    }
    at += 1;
    let mut entries = Vec::new();
    loop {
        skip_space(&mut at);
        match bytes.get(at)? {
            b'}' => return Some(entries),
            b',' => {
                at += 1;
                continue;
            }
            _ => {}
        }
        let key = string(&mut at)?;
        skip_space(&mut at);
        if bytes.get(at) != Some(&b':') {
            return None;
        }
        at += 1;
        skip_space(&mut at);
        let value = string(&mut at)?;
        entries.push((key, value));
    }
}

/// A stretch of text in one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub style: Style,
}

/// The language: translation keys to their format strings.
#[derive(Debug, Clone, Default)]
pub struct Lang {
    texts: HashMap<String, String>,
}

impl Lang {
    /// A language of given texts.
    pub fn new(texts: impl IntoIterator<Item = (String, String)>) -> Self {
        Self {
            texts: texts.into_iter().collect(),
        }
    }

    /// The default language from `assets/minecraft/lang/en_us.json`.
    pub fn load(assets_root: &std::path::Path) -> Self {
        let Ok(text) = std::fs::read_to_string(assets_root.join("minecraft/lang/en_us.json"))
        else {
            return Self::default();
        };
        Self::new(flat_strings(&text).unwrap_or_default())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.texts.get(key).map(String::as_str)
    }

    /// `Language.getOrDefault`: the text, or the key itself.
    pub fn text(&self, key: &str) -> String {
        self.get(key).unwrap_or(key).to_owned()
    }

    /// A component in words: its runs, each in the style it ends up in
    /// under `parent`.
    pub fn runs(&self, component: &Component, parent: &Style) -> Vec<Run> {
        let mut out = Vec::new();
        self.push_runs(component, parent, &mut out);
        out
    }

    fn push_runs(&self, component: &Component, parent: &Style, out: &mut Vec<Run>) {
        let style = component.style.apply_to(parent);
        match &component.content {
            Content::Literal(text) => {
                if !text.is_empty() {
                    out.push(Run {
                        text: text.clone(),
                        style: style.clone(),
                    });
                }
            }
            Content::Translatable {
                key,
                fallback,
                args,
            } => {
                let template = self
                    .get(key)
                    .or(fallback.as_deref())
                    .unwrap_or(key)
                    .to_owned();
                self.push_template(&template, args, &style, out);
            }
        }
        for sibling in &component.siblings {
            self.push_runs(sibling, &style, out);
        }
    }

    /// `TranslatableContents.decomposeTemplate`: literal text, and an
    /// argument where `%s`, `%d` or `%n$s` stands.
    fn push_template(&self, template: &str, args: &[Component], style: &Style, out: &mut Vec<Run>) {
        let mut literal = String::new();
        let mut next_arg = 0;
        let chars: Vec<char> = template.chars().collect();
        let mut at = 0;
        let flush = |literal: &mut String, out: &mut Vec<Run>| {
            if !literal.is_empty() {
                out.push(Run {
                    text: std::mem::take(literal),
                    style: style.clone(),
                });
            }
        };
        while at < chars.len() {
            if chars[at] != '%' {
                literal.push(chars[at]);
                at += 1;
                continue;
            }
            // `%(?:(\d+)\$)?([A-Za-z%]|$)`
            let mut end = at + 1;
            let digits_start = end;
            while end < chars.len() && chars[end].is_ascii_digit() {
                end += 1;
            }
            let position = if end > digits_start && chars.get(end) == Some(&'$') {
                let number: String = chars[digits_start..end].iter().collect();
                end += 1;
                number.parse::<usize>().ok()
            } else {
                end = digits_start;
                None
            };
            match chars.get(end) {
                Some('%') => {
                    literal.push('%');
                    at = end + 1;
                }
                Some(kind) if kind.is_ascii_alphabetic() && matches!(kind, 's' | 'd') => {
                    let index = position.map_or_else(
                        || {
                            let index = next_arg;
                            next_arg += 1;
                            index
                        },
                        |position| position.saturating_sub(1),
                    );
                    flush(&mut literal, out);
                    if let Some(arg) = args.get(index) {
                        self.push_runs(arg, style, out);
                    }
                    at = end + 1;
                }
                _ => {
                    literal.extend(&chars[at..end.min(chars.len())]);
                    if end >= chars.len() {
                        at = chars.len();
                    } else {
                        literal.push(chars[end]);
                        at = end + 1;
                    }
                }
            }
        }
        flush(&mut literal, out);
    }

    /// `Component.getString`: the text without its styles.
    pub fn plain(&self, component: &Component) -> String {
        self.runs(component, &Style::default())
            .into_iter()
            .map(|run| run.text)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lang() -> Lang {
        Lang::new([
            ("item.a".to_owned(), "Apple".to_owned()),
            ("enchantment.level.3".to_owned(), "III".to_owned()),
            ("two".to_owned(), "%2$s then %1$s, %s%%".to_owned()),
        ])
    }

    #[test]
    fn a_translation_with_arguments_is_put_in_words() {
        let component = Component::translatable_with(
            "two",
            vec![Component::literal("a"), Component::literal("b")],
        );
        assert_eq!(lang().plain(&component), "b then a, a%");
    }

    #[test]
    fn a_missing_key_reads_as_itself() {
        assert_eq!(lang().plain(&Component::translatable("nothing")), "nothing");
    }

    #[test]
    fn siblings_inherit_the_style() {
        let component = Component::literal("a")
            .with_colour("red")
            .append(Component::literal("b"))
            .append(Component::literal("c").with_colour("blue"));
        let runs = lang().runs(&component, &Style::default());
        assert_eq!(runs.len(), 3);
        assert_eq!(runs[1].style.colour, named_colour("red"));
        assert_eq!(runs[2].style.colour, named_colour("blue"));
    }

    #[test]
    fn a_language_file_reads_with_its_escapes() {
        let entries = flat_strings(
            "{\n \"a\": \"x\\u00e9y\",\n \"b\": \"q\\\"r\\n\", \"c\": \"\\ud83d\\ude00!\" }",
        )
        .expect("a language");
        assert_eq!(entries[0], ("a".to_owned(), "x\u{e9}y".to_owned()));
        assert_eq!(entries[1], ("b".to_owned(), "q\"r\n".to_owned()));
        assert_eq!(entries[2], ("c".to_owned(), "\u{1f600}!".to_owned()));
    }

    #[test]
    fn json_and_nbt_components_read_alike() {
        let json = voxel_core::json::parse(
            r##"{"translate":"item.a","color":"#FF0000","italic":true,"extra":["x"]}"##,
        )
        .expect("json");
        let from_json = Component::from_json(&json).expect("a component");
        assert_eq!(from_json.style.colour, Some(0xFF_0000));
        assert_eq!(from_json.style.italic, Some(true));
        assert_eq!(lang().plain(&from_json), "Applex");
        let nbt = FieldValue::Compound(vec![
            voxel_core::nbt::Field {
                key: "translate".into(),
                value: FieldValue::Str("item.a".into()),
            },
            voxel_core::nbt::Field {
                key: "italic".into(),
                value: FieldValue::Byte(1),
            },
        ]);
        let from_nbt = Component::from_nbt(&nbt).expect("a component");
        assert_eq!(from_nbt.style.italic, Some(true));
        assert_eq!(lang().plain(&from_nbt), "Apple");
    }
}
