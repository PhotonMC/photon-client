//! Key binding state and event matching for the pinned Java client.

use std::collections::HashMap;

/// The identity of a keyboard, scancode, or mouse input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKey {
    KeySym(i32),
    ScanCode(i32),
    Mouse(i32),
}

impl InputKey {
    pub const UNKNOWN: Self = Self::KeySym(-1);

    pub const fn key_sym(value: i32) -> Self {
        Self::KeySym(value)
    }

    pub const fn scan_code(value: i32) -> Self {
        Self::ScanCode(value)
    }

    pub const fn mouse(value: i32) -> Self {
        Self::Mouse(value)
    }

    /// Return the serialized name used by `InputConstants.Key`.
    pub fn name(self) -> String {
        match self {
            Self::KeySym(-1) => "key.keyboard.unknown".to_owned(),
            Self::KeySym(value) => keysym_name(value),
            Self::ScanCode(value) => format!("scancode.{value}"),
            Self::Mouse(value) => mouse_name(value),
        }
    }
}

fn keysym_name(value: i32) -> String {
    let known = match value {
        39 => Some("key.keyboard.apostrophe"),
        44 => Some("key.keyboard.comma"),
        45 => Some("key.keyboard.minus"),
        46 => Some("key.keyboard.period"),
        47 => Some("key.keyboard.slash"),
        59 => Some("key.keyboard.semicolon"),
        61 => Some("key.keyboard.equal"),
        91 => Some("key.keyboard.left.bracket"),
        92 => Some("key.keyboard.backslash"),
        93 => Some("key.keyboard.right.bracket"),
        96 => Some("key.keyboard.grave.accent"),
        161 => Some("key.keyboard.world.1"),
        162 => Some("key.keyboard.world.2"),
        256 => Some("key.keyboard.escape"),
        257 => Some("key.keyboard.enter"),
        258 => Some("key.keyboard.tab"),
        259 => Some("key.keyboard.backspace"),
        260 => Some("key.keyboard.insert"),
        261 => Some("key.keyboard.delete"),
        262 => Some("key.keyboard.right"),
        263 => Some("key.keyboard.left"),
        264 => Some("key.keyboard.down"),
        265 => Some("key.keyboard.up"),
        266 => Some("key.keyboard.page.up"),
        267 => Some("key.keyboard.page.down"),
        268 => Some("key.keyboard.home"),
        269 => Some("key.keyboard.end"),
        280 => Some("key.keyboard.caps.lock"),
        281 => Some("key.keyboard.scroll.lock"),
        282 => Some("key.keyboard.num.lock"),
        283 => Some("key.keyboard.print.screen"),
        284 => Some("key.keyboard.pause"),
        330 => Some("key.keyboard.keypad.decimal"),
        331 => Some("key.keyboard.keypad.divide"),
        332 => Some("key.keyboard.keypad.multiply"),
        333 => Some("key.keyboard.keypad.subtract"),
        334 => Some("key.keyboard.keypad.add"),
        335 => Some("key.keyboard.keypad.enter"),
        336 => Some("key.keyboard.keypad.equal"),
        340 => Some("key.keyboard.left.shift"),
        341 => Some("key.keyboard.left.control"),
        342 => Some("key.keyboard.left.alt"),
        343 => Some("key.keyboard.left.win"),
        344 => Some("key.keyboard.right.shift"),
        345 => Some("key.keyboard.right.control"),
        346 => Some("key.keyboard.right.alt"),
        347 => Some("key.keyboard.right.win"),
        348 => Some("key.keyboard.menu"),
        _ => None,
    };
    if let Some(name) = known {
        return name.to_owned();
    }
    if (48..=57).contains(&value) {
        return format!("key.keyboard.{}", value - 48);
    }
    if (65..=90).contains(&value) {
        return format!(
            "key.keyboard.{}",
            (value as u8 as char).to_ascii_lowercase()
        );
    }
    if (290..=314).contains(&value) {
        return format!("key.keyboard.f{}", value - 289);
    }
    format!("key.keyboard.{value}")
}

fn mouse_name(value: i32) -> String {
    match value {
        0 => "key.mouse.left".to_owned(),
        1 => "key.mouse.right".to_owned(),
        2 => "key.mouse.middle".to_owned(),
        _ => format!("key.mouse.{}", value.wrapping_add(1)),
    }
}

/// One Java `KeyMapping` state holder.
#[derive(Debug, Clone)]
pub struct KeyMapping {
    name: String,
    default_key: InputKey,
    key: InputKey,
    is_down: bool,
    click_count: i32,
}

impl KeyMapping {
    pub fn new(name: impl Into<String>, key: InputKey) -> Self {
        Self {
            name: name.into(),
            default_key: key,
            key,
            is_down: false,
            click_count: 0,
        }
    }

    pub const fn is_down(&self) -> bool {
        self.is_down
    }

    pub const fn key(&self) -> InputKey {
        self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn default_key(&self) -> InputKey {
        self.default_key
    }

    pub fn set_down(&mut self, down: bool) {
        self.is_down = down;
    }

    pub fn consume_click(&mut self) -> bool {
        if self.click_count == 0 {
            return false;
        }
        self.click_count -= 1;
        true
    }

    pub fn set_key(&mut self, key: InputKey) {
        self.key = key;
    }

    pub fn is_default(&self) -> bool {
        self.key == self.default_key
    }

    pub fn is_unbound(&self) -> bool {
        self.key == InputKey::UNKNOWN
    }

    pub fn save_string(&self) -> String {
        self.key.name()
    }

    pub fn same(&self, other: &Self) -> bool {
        self.key == other.key
    }

    pub fn matches_key(&self, key: InputKey) -> bool {
        self.key == key
    }

    pub fn matches_key_event(&self, key: i32, scancode: i32) -> bool {
        match self.key {
            InputKey::ScanCode(expected) if key == InputKey::UNKNOWN.value() => {
                expected == scancode
            }
            InputKey::KeySym(expected) if key != InputKey::UNKNOWN.value() => expected == key,
            _ => false,
        }
    }

    pub fn matches_mouse(&self, button: i32) -> bool {
        matches!(self.key, InputKey::Mouse(expected) if expected == button)
    }

    fn click(&mut self) {
        self.click_count = self.click_count.wrapping_add(1);
    }

    fn release(&mut self) {
        self.click_count = 0;
        self.is_down = false;
    }
}

impl InputKey {
    const fn value(self) -> i32 {
        match self {
            Self::KeySym(value) | Self::ScanCode(value) | Self::Mouse(value) => value,
        }
    }
}

/// Registry that models Java `KeyMapping`'s static binding table.
#[derive(Debug, Default)]
pub struct KeyMappingRegistry {
    mappings: Vec<KeyMapping>,
    by_key: HashMap<InputKey, Vec<usize>>,
}

impl KeyMappingRegistry {
    pub fn register(&mut self, name: impl Into<String>, key: InputKey) -> usize {
        let id = self.mappings.len();
        self.mappings.push(KeyMapping::new(name, key));
        self.by_key.entry(key).or_default().push(id);
        id
    }

    pub fn mapping(&self, id: usize) -> Option<&KeyMapping> {
        self.mappings.get(id)
    }

    pub fn mapping_mut(&mut self, id: usize) -> Option<&mut KeyMapping> {
        self.mappings.get_mut(id)
    }

    pub fn set(&mut self, key: InputKey, down: bool) {
        if let Some(ids) = self.by_key.get(&key) {
            for &id in ids {
                self.mappings[id].set_down(down);
            }
        }
    }

    pub fn click(&mut self, key: InputKey) {
        if let Some(ids) = self.by_key.get(&key) {
            for &id in ids {
                self.mappings[id].click();
            }
        }
    }

    pub fn release_all(&mut self) {
        for mapping in &mut self.mappings {
            mapping.release();
        }
    }
}
