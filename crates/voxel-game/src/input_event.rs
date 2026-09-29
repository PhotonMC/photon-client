//! Headless keyboard and mouse event records used by the client input layer.

pub const NOT_DIGIT: i32 = -1;
pub const MOD_SHIFT: i32 = 1;
pub const MOD_CONTROL: i32 = 2;
pub const MOD_ALT: i32 = 4;
pub const MOD_SUPER: i32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCodePoint(pub i32);

/// Platform-dependent input constants: macOS edits with the command key,
/// simulates a right click with a long left click, and does not restore key
/// state after the mouse is grabbed.
pub struct InputQuirks;

impl InputQuirks {
    const ON_OSX: bool = cfg!(target_os = "macos");
    pub const REPLACE_CTRL_KEY_WITH_CMD_KEY: bool = Self::ON_OSX;
    pub const EDIT_SHORTCUT_KEY_MODIFIER: i32 = if Self::REPLACE_CTRL_KEY_WITH_CMD_KEY {
        MOD_SUPER
    } else {
        MOD_CONTROL
    };
    pub const SIMULATE_RIGHT_CLICK_WITH_LONG_LEFT_CLICK: bool = Self::ON_OSX;
    pub const RESTORE_KEY_STATE_AFTER_MOUSE_GRAB: bool = !Self::ON_OSX;
}

/// A character callback event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterEvent {
    pub codepoint: i32,
}

impl CharacterEvent {
    pub const fn new(codepoint: i32) -> Self {
        Self { codepoint }
    }

    /// Return the UTF-16 units used by Java's `Character.toString(int)`.
    pub fn codepoint_as_utf16(self) -> Result<Vec<u16>, InvalidCodePoint> {
        let codepoint =
            u32::try_from(self.codepoint).map_err(|_| InvalidCodePoint(self.codepoint))?;
        if codepoint > 0x10_FFFF {
            return Err(InvalidCodePoint(self.codepoint));
        }
        if codepoint <= 0xFFFF {
            return Ok(vec![codepoint as u16]);
        }

        let supplementary = codepoint - 0x1_0000;
        Ok(vec![
            0xD800 | (supplementary >> 10) as u16,
            0xDC00 | (supplementary & 0x3FF) as u16,
        ])
    }

    pub const fn is_allowed_chat_character(self) -> bool {
        self.codepoint >= 32 && self.codepoint != 127 && self.codepoint != 167
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreeditError {
    NegativePreeditSize(i32),
    CodepointBufferUnderflow { required: usize, available: usize },
    InvalidCodePoint(InvalidCodePoint),
    FocusedBlockOutOfBounds { index: i32, block_count: usize },
}

/// Text composition data supplied by an IME pre-edit callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreeditEvent {
    pub full_text: Vec<u16>,
    pub caret_position: i32,
    pub blocks: Vec<Vec<u16>>,
    pub focused_block: i32,
}

impl PreeditEvent {
    pub fn new(
        full_text: Vec<u16>,
        caret_position: i32,
        blocks: Vec<Vec<u16>>,
        focused_block: i32,
    ) -> Result<Self, PreeditError> {
        if focused_block < 0 || focused_block as usize >= blocks.len() {
            return Err(PreeditError::FocusedBlockOutOfBounds {
                index: focused_block,
                block_count: blocks.len(),
            });
        }
        Ok(Self {
            full_text,
            caret_position,
            blocks,
            focused_block,
        })
    }

    /// Build the record from the arrays read by Java's native callback bridge.
    pub fn create_from_callback(
        preedit_size: i32,
        codepoints: &[i32],
        block_sizes: &[i32],
        focused_block: i32,
        caret: i32,
    ) -> Option<Result<Self, PreeditError>> {
        if preedit_size == 0 {
            return None;
        }
        if preedit_size < 0 {
            return Some(Err(PreeditError::NegativePreeditSize(preedit_size)));
        }

        let required = preedit_size as usize;
        if codepoints.len() < required {
            return Some(Err(PreeditError::CodepointBufferUnderflow {
                required,
                available: codepoints.len(),
            }));
        }

        let codepoints = &codepoints[..required];
        let mut full_text = Vec::new();
        let mut blocks = Vec::with_capacity(block_sizes.len());
        let mut offset = 0usize;
        let mut converted_caret = 0i32;

        for &block_size in block_sizes {
            let count = if block_size < 0 {
                0
            } else {
                block_size as usize
            };
            if count > codepoints.len().saturating_sub(offset) {
                return Some(Err(PreeditError::CodepointBufferUnderflow {
                    required: offset.saturating_add(count),
                    available: codepoints.len(),
                }));
            }

            let mut block = Vec::new();
            for &codepoint in &codepoints[offset..offset + count] {
                if offset as i32 == caret {
                    converted_caret = (full_text.len() + block.len()) as i32;
                }
                let units = match CharacterEvent::new(codepoint).codepoint_as_utf16() {
                    Ok(units) => units,
                    Err(error) => {
                        return Some(Err(PreeditError::InvalidCodePoint(error)));
                    }
                };
                block.extend(units);
                offset += 1;
            }
            full_text.extend_from_slice(&block);
            blocks.push(block);
        }

        if offset as i32 == caret {
            converted_caret = full_text.len() as i32;
        }

        Some(PreeditEvent::new(
            full_text,
            converted_caret,
            blocks,
            focused_block,
        ))
    }
}

/// Shared classification and modifier helpers for client input records.
pub trait InputWithModifiers {
    fn input(&self) -> i32;

    fn modifiers(&self) -> i32;

    fn is_selection(&self) -> bool {
        matches!(self.input(), 257 | 32 | 335)
    }

    fn is_confirmation(&self) -> bool {
        matches!(self.input(), 257 | 335)
    }

    fn is_escape(&self) -> bool {
        self.input() == 256
    }

    fn is_left(&self) -> bool {
        self.input() == 263
    }

    fn is_right(&self) -> bool {
        self.input() == 262
    }

    fn is_up(&self) -> bool {
        self.input() == 265
    }

    fn is_down(&self) -> bool {
        self.input() == 264
    }

    fn is_cycle_focus(&self) -> bool {
        self.input() == 258
    }

    fn get_digit(&self) -> i32 {
        let value = self.input().wrapping_sub(48);
        if (0..=9).contains(&value) {
            value
        } else {
            NOT_DIGIT
        }
    }

    fn has_alt_down(&self) -> bool {
        self.modifiers() & MOD_ALT != 0
    }

    fn has_shift_down(&self) -> bool {
        self.modifiers() & MOD_SHIFT != 0
    }

    fn has_control_down(&self) -> bool {
        self.modifiers() & MOD_CONTROL != 0
    }

    fn has_control_down_with_quirk(&self) -> bool {
        self.modifiers() & InputQuirks::EDIT_SHORTCUT_KEY_MODIFIER != 0
    }

    fn is_select_all(&self) -> bool {
        self.input() == 65
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_copy(&self) -> bool {
        self.input() == 67
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_paste(&self) -> bool {
        self.input() == 86
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_cut(&self) -> bool {
        self.input() == 88
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }
}

/// A keyboard event as delivered by GLFW.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: i32,
    pub scancode: i32,
    pub modifiers: i32,
}

impl KeyEvent {
    pub const fn new(key: i32, scancode: i32, modifiers: i32) -> Self {
        Self {
            key,
            scancode,
            modifiers,
        }
    }
}

impl InputWithModifiers for KeyEvent {
    fn input(&self) -> i32 {
        self.key
    }

    fn modifiers(&self) -> i32 {
        self.modifiers
    }
}

/// A mouse button and its modifier snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseButtonInfo {
    pub button: i32,
    pub modifiers: i32,
}

impl MouseButtonInfo {
    pub const fn new(button: i32, modifiers: i32) -> Self {
        Self { button, modifiers }
    }
}

impl InputWithModifiers for MouseButtonInfo {
    fn input(&self) -> i32 {
        self.button
    }

    fn modifiers(&self) -> i32 {
        self.modifiers
    }
}

/// A mouse button event with its window coordinates.
#[derive(Debug, Clone, Copy)]
pub struct MouseButtonEvent {
    pub x: f64,
    pub y: f64,
    pub button_info: MouseButtonInfo,
}

impl MouseButtonEvent {
    pub const fn new(x: f64, y: f64, button_info: MouseButtonInfo) -> Self {
        Self { x, y, button_info }
    }

    pub const fn button(self) -> i32 {
        self.button_info.button
    }
}

impl InputWithModifiers for MouseButtonEvent {
    fn input(&self) -> i32 {
        self.button()
    }

    fn modifiers(&self) -> i32 {
        self.button_info.modifiers
    }
}
