//! Text components, and the language they are put into words with.
//!
//! A component is a piece of content (literal text, or a translation key
//! with arguments) under a style, followed by siblings that inherit the
//! style. It comes as the data JSON writes it or as the tag the network
//! carries; the language turns it into runs of text, each with the style it
//! ends up in. Not read: keybinds, scores, selectors, NBT contents, the
//! dialog click event, an item's components in a hover, fonts and the
//! shadow colour of a component's JSON.

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
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    /// `0xRRGGBB`.
    pub colour: Option<u32>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    /// `shadow_color`, `0xAARRGGBB`: zero alpha draws no shadow (`Style.withoutShadow`). Not read
    /// from a component; set by the screens that style their text.
    pub shadow: Option<u32>,
    /// What a click on the text does (`clickEvent`).
    pub click: Option<Box<ClickEvent>>,
    /// What the mouse over the text shows (`hoverEvent`).
    pub hover: Option<Box<HoverEvent>>,
    /// What a Shift click on the text types into the chat box.
    pub insertion: Option<Box<str>>,
}

/// `ClickEvent`, as far as the client acts on it. The dialog action, which
/// needs the registry of dialogs, is not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickEvent {
    OpenUrl(String),
    OpenFile(String),
    RunCommand(String),
    SuggestCommand(String),
    ChangePage(i32),
    CopyToClipboard(String),
    /// `custom`: the identifier, for the server's own handling.
    Custom(String),
}

/// `HoverEvent`.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverEvent {
    ShowText(Box<Component>),
    /// An item by its identifier and count; its components are not read.
    ShowItem {
        id: String,
        count: i32,
    },
    ShowEntity {
        id: String,
        uuid: String,
        name: Option<Box<Component>>,
    },
}

impl Style {
    /// A style of just a colour.
    pub fn coloured(name: &str) -> Self {
        Self {
            colour: named_colour(name),
            ..Self::default()
        }
    }

    /// A style from the tag the network carries: a compound with the
    /// style's fields beside any others.
    pub fn from_nbt(value: &FieldValue) -> Option<Self> {
        match value {
            FieldValue::Compound(fields) => Some(Self::from_fields(fields)),
            _ => None,
        }
    }

    fn from_fields(fields: &[voxel_core::nbt::Field]) -> Self {
        let get = |key: &str| {
            fields
                .iter()
                .find(|field| field.key == key)
                .map(|f| &f.value)
        };
        let flag = |key: &str| match get(key) {
            Some(FieldValue::Byte(value)) => Some(*value != 0),
            _ => None,
        };
        Self {
            colour: match get("color") {
                Some(FieldValue::Str(name)) => parse_colour(name),
                _ => None,
            },
            bold: flag("bold"),
            italic: flag("italic"),
            underlined: flag("underlined"),
            strikethrough: flag("strikethrough"),
            obfuscated: flag("obfuscated"),
            shadow: None,
            click: match get("click_event") {
                Some(FieldValue::Compound(event)) => click_from_nbt(event).map(Box::new),
                _ => None,
            },
            hover: match get("hover_event") {
                Some(FieldValue::Compound(event)) => hover_from_nbt(event).map(Box::new),
                _ => None,
            },
            insertion: match get("insertion") {
                Some(FieldValue::Str(text)) => Some(text.as_str().into()),
                _ => None,
            },
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
            shadow: self.shadow.or(parent.shadow),
            click: self.click.clone().or_else(|| parent.click.clone()),
            hover: self.hover.clone().or_else(|| parent.hover.clone()),
            insertion: self.insertion.clone().or_else(|| parent.insertion.clone()),
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

/// `ClickEvent.CODEC` from JSON: by its `action`.
fn click_from_json(event: &JsonObject) -> Option<ClickEvent> {
    Some(match event.text("action")?.as_str() {
        "open_url" => ClickEvent::OpenUrl(event.text("url")?),
        "open_file" => ClickEvent::OpenFile(event.text("path")?),
        "run_command" => ClickEvent::RunCommand(event.text("command")?),
        "suggest_command" => ClickEvent::SuggestCommand(event.text("command")?),
        "change_page" => ClickEvent::ChangePage(event.i32("page").filter(|page| *page > 0)?),
        "copy_to_clipboard" => ClickEvent::CopyToClipboard(event.text("value")?),
        "custom" => ClickEvent::Custom(event.text("id")?),
        _ => return None,
    })
}

/// `HoverEvent.CODEC` from JSON.
fn hover_from_json(event: &JsonObject) -> Option<HoverEvent> {
    Some(match event.text("action")?.as_str() {
        "show_text" => {
            HoverEvent::ShowText(Box::new(Component::from_json(event.get_str("value")?)?))
        }
        "show_item" => HoverEvent::ShowItem {
            id: event.text("id")?,
            count: event.i32("count").unwrap_or(1),
        },
        "show_entity" => HoverEvent::ShowEntity {
            id: event.text("id")?,
            uuid: match event.get_str("uuid")? {
                JsonValue::String(units) => String::from_utf16_lossy(units),
                JsonValue::Array(parts) => uuid_text(
                    &parts
                        .iter()
                        .filter_map(|part| part.as_i32())
                        .collect::<Vec<_>>(),
                )?,
                _ => return None,
            },
            name: event
                .get_str("name")
                .and_then(Component::from_json)
                .map(Box::new),
        },
        _ => return None,
    })
}

/// A UUID of four integers, as text.
fn uuid_text(parts: &[i32]) -> Option<String> {
    let [a, b, c, d] = parts else {
        return None;
    };
    let (a, b, c, d) = (*a as u32, *b as u32, *c as u32, *d as u32);
    Some(format!(
        "{a:08x}-{:04x}-{:04x}-{:04x}-{:04x}{d:08x}",
        b >> 16,
        b & 0xFFFF,
        c >> 16,
        c & 0xFFFF
    ))
}

/// `ClickEvent.CODEC` from the tag.
fn click_from_nbt(event: &[voxel_core::nbt::Field]) -> Option<ClickEvent> {
    let get = |key: &str| {
        event
            .iter()
            .find(|field| field.key == key)
            .map(|f| &f.value)
    };
    let text = |key: &str| match get(key) {
        Some(FieldValue::Str(text)) => Some(text.clone()),
        _ => None,
    };
    Some(match text("action")?.as_str() {
        "open_url" => ClickEvent::OpenUrl(text("url")?),
        "open_file" => ClickEvent::OpenFile(text("path")?),
        "run_command" => ClickEvent::RunCommand(text("command")?),
        "suggest_command" => ClickEvent::SuggestCommand(text("command")?),
        "change_page" => match get("page")? {
            FieldValue::Int(page) if *page > 0 => ClickEvent::ChangePage(*page),
            _ => return None,
        },
        "copy_to_clipboard" => ClickEvent::CopyToClipboard(text("value")?),
        "custom" => ClickEvent::Custom(text("id")?),
        _ => return None,
    })
}

/// `HoverEvent.CODEC` from the tag.
fn hover_from_nbt(event: &[voxel_core::nbt::Field]) -> Option<HoverEvent> {
    let get = |key: &str| {
        event
            .iter()
            .find(|field| field.key == key)
            .map(|f| &f.value)
    };
    let text = |key: &str| match get(key) {
        Some(FieldValue::Str(text)) => Some(text.clone()),
        _ => None,
    };
    Some(match text("action")?.as_str() {
        "show_text" => HoverEvent::ShowText(Box::new(Component::from_nbt(get("value")?)?)),
        "show_item" => HoverEvent::ShowItem {
            id: text("id")?,
            count: match get("count") {
                Some(FieldValue::Int(count)) => *count,
                _ => 1,
            },
        },
        "show_entity" => HoverEvent::ShowEntity {
            id: text("id")?,
            uuid: match get("uuid")? {
                FieldValue::IntArray(parts) => uuid_text(parts)?,
                FieldValue::Str(text) => text.clone(),
                _ => return None,
            },
            name: get("name").and_then(Component::from_nbt).map(Box::new),
        },
        _ => return None,
    })
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
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub text: String,
    pub style: Style,
}

/// The language: translation keys to their format strings.
#[derive(Debug, Clone, Default)]
pub struct Lang {
    texts: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(json: &str) -> Component {
        Component::from_json(&voxel_core::json::parse(json).expect("json")).expect("component")
    }

    #[test]
    fn click_and_hover_events_are_read_and_inherited() {
        let parent = component(
            r#"{"text":"a","click_event":{"action":"run_command","command":"/say hi"},
                "hover_event":{"action":"show_text","value":"tip"},"insertion":"ins",
                "extra":[{"text":"b"}]}"#,
        );
        assert_eq!(
            parent.style.click.as_deref(),
            Some(&ClickEvent::RunCommand("/say hi".to_owned()))
        );
        assert_eq!(parent.style.insertion.as_deref(), Some("ins"));
        let Some(HoverEvent::ShowText(tip)) = parent.style.hover.as_deref() else {
            panic!("a text hover");
        };
        assert_eq!(**tip, Component::literal("tip"));
        // The sibling takes the parent's events.
        let runs = Lang::default().runs(&parent, &Style::default());
        assert_eq!(runs.len(), 2);
        assert!(runs.iter().all(|run| run.style.click == parent.style.click));
        // An action that is not known is no event, and the rest is kept.
        let odd = component(r#"{"text":"x","click_event":{"action":"nope"},"bold":true}"#);
        assert_eq!((odd.style.click, odd.style.bold), (None, Some(true)));
        // A change of page needs a positive page.
        let page = component(r#"{"text":"x","click_event":{"action":"change_page","page":0}}"#);
        assert_eq!(page.style.click, None);
    }

    #[test]
    fn a_hovered_entity_has_its_uuid_written_out() {
        let entity = component(
            r#"{"text":"x","hover_event":{"action":"show_entity","id":"minecraft:pig",
                "uuid":[1,2,3,4]}}"#,
        );
        let Some(HoverEvent::ShowEntity { uuid, .. }) = entity.style.hover.as_deref() else {
            panic!("an entity hover");
        };
        assert_eq!(uuid, "00000001-0000-0002-0000-000300000004");
    }

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

mod component_translatable;

mod lang_texts;
