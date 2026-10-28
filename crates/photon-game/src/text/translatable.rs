use super::*;

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

    /// A translatable that reads `fallback` when no language file has `key`: a text of ours that
    /// the reference's language files do not carry.
    pub fn translatable_or(key: impl Into<String>, fallback: impl Into<String>) -> Self {
        Self {
            content: Content::Translatable {
                key: key.into(),
                fallback: Some(fallback.into()),
                args: Vec::new(),
            },
            style: Style::default(),
            siblings: Vec::new(),
        }
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

    pub(super) fn from_json_object(object: &JsonObject) -> Option<Self> {
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
                shadow: None,
                click: object
                    .object("click_event")
                    .and_then(click_from_json)
                    .map(Box::new),
                hover: object
                    .object("hover_event")
                    .and_then(hover_from_json)
                    .map(Box::new),
                insertion: object.text("insertion").map(String::into_boxed_str),
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
                    style: Style::from_fields(fields),
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
