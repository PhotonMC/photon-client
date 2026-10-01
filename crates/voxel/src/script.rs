//! `VOXEL_SCRIPT`: keys held and clicks made on given ticks, so a run can
//! be repeated without a person at the keyboard.

use voxel_game::container_menu::ClickType;

/// Held movement keys, as the key mappings see them.
#[derive(Default, Clone, Copy)]
pub struct HeldKeys {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub sneak: bool,
    pub sprint: bool,
    /// A scripted attack click on this tick.
    pub attack: bool,
    /// The attack key held down on this tick.
    pub mine: bool,
    /// A scripted use click on this tick.
    pub use_click: bool,
    /// The use key held down on this tick.
    pub hold_use: bool,
    /// A scripted press of the inventory key on this tick.
    pub inventory: bool,
    /// The chat box opened or closed on this tick, and Enter pressed in it.
    pub chat: bool,
    pub enter: bool,
    /// Escape pressed with no screen open: the pause menu opens or closes.
    pub escape: bool,
    /// Tab pressed in the chat box.
    pub complete: bool,
    /// A scripted click in the open inventory: slot, button, click type.
    pub click: Option<(i32, i32, ClickType)>,
    /// A scripted tab choice on the open creative screen, by table index.
    pub tab: Option<usize>,
    /// A scripted scroll of the creative screen, in hundredths.
    pub scroll: Option<i32>,
    /// The mouse moved to a place, in GUI units.
    pub mouse: Option<(f64, f64)>,
    /// A mouse press and release at the mouse: button and whether shift is held.
    pub press: Option<(i32, bool)>,
    /// Text typed into the open screen.
    pub typed: Option<&'static str>,
}

impl HeldKeys {
    pub fn input(self) -> voxel_world::input::Input {
        voxel_world::input::Input::new(
            self.forward,
            self.backward,
            self.left,
            self.right,
            self.jump,
            self.sneak,
            self.sprint,
        )
    }

    pub fn set(&mut self, key: glfw::Key, down: bool) {
        match key {
            glfw::Key::W => self.forward = down,
            glfw::Key::S => self.backward = down,
            glfw::Key::A => self.left = down,
            glfw::Key::D => self.right = down,
            glfw::Key::Space => self.jump = down,
            glfw::Key::LeftShift => self.sneak = down,
            glfw::Key::LeftControl => self.sprint = down,
            _ => {}
        }
    }

    /// One step of a `VOXEL_SCRIPT`, such as `forward,sprint`.
    pub fn parse(step: &str) -> Result<Self, String> {
        let mut keys = Self::default();
        for name in step
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match name {
                "forward" => keys.forward = true,
                "backward" => keys.backward = true,
                "left" => keys.left = true,
                "right" => keys.right = true,
                "jump" => keys.jump = true,
                "sneak" => keys.sneak = true,
                "sprint" => keys.sprint = true,
                "attack" => keys.attack = true,
                "mine" => keys.mine = true,
                "use" => keys.use_click = true,
                "hold_use" => keys.hold_use = true,
                "inventory" => keys.inventory = true,
                "chat" => keys.chat = true,
                "enter" => keys.enter = true,
                "escape" => keys.escape = true,
                "complete" => keys.complete = true,
                "none" => {}
                other => {
                    // `click/SLOT/BUTTON/TYPE`, the type by its id.
                    let parts: Vec<&str> = other.split('/').collect();
                    match parts.as_slice() {
                        ["click", slot, button, kind] => {
                            let number = |text: &str| {
                                text.parse::<i32>()
                                    .map_err(|_| format!("bad click {other} in VOXEL_SCRIPT"))
                            };
                            keys.click =
                                Some((number(slot)?, number(button)?, click_type(number(kind)?)));
                        }
                        ["tab", tab] => {
                            keys.tab = Some(
                                tab.parse()
                                    .map_err(|_| format!("bad tab {other} in VOXEL_SCRIPT"))?,
                            );
                        }
                        ["type", text @ ..] => {
                            // A script is read once: its text lives on; a
                            // slash in it is part of the text.
                            keys.typed = Some(Box::leak(text.join("/").into_boxed_str()));
                        }
                        ["mouse", x, y] => {
                            let number = |text: &str| {
                                text.parse::<f64>()
                                    .map_err(|_| format!("bad mouse {other} in VOXEL_SCRIPT"))
                            };
                            keys.mouse = Some((number(x)?, number(y)?));
                        }
                        ["press", button] | ["press", button, _] => {
                            keys.press = Some((
                                button
                                    .parse()
                                    .map_err(|_| format!("bad press {other} in VOXEL_SCRIPT"))?,
                                parts.len() == 3,
                            ));
                        }
                        ["scroll", hundredths] => {
                            keys.scroll = Some(
                                hundredths
                                    .parse()
                                    .map_err(|_| format!("bad scroll {other} in VOXEL_SCRIPT"))?,
                            );
                        }
                        _ => return Err(format!("unknown key {other} in VOXEL_SCRIPT")),
                    }
                }
            }
        }
        Ok(keys)
    }
}

/// A scripted run: keys held for so many ticks, and turns applied between.
pub struct Script {
    steps: Vec<(HeldKeys, u32, f64, f64)>,
    index: usize,
    ticks_in_step: u32,
}

impl Script {
    /// `forward=40;forward,jump=10;turn:90;none=20` holds the keys for the
    /// counted ticks, and `turn:D` turns the view by D degrees.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut steps = Vec::new();
        for step in text
            .split(';')
            .map(str::trim)
            .filter(|step| !step.is_empty())
        {
            if let Some(degrees) = step.strip_prefix("turn:") {
                let degrees: f64 = degrees.parse().map_err(|_| format!("bad turn {degrees}"))?;
                steps.push((HeldKeys::default(), 0, degrees, 0.0));
                continue;
            }
            if let Some(degrees) = step.strip_prefix("pitch:") {
                let degrees: f64 = degrees
                    .parse()
                    .map_err(|_| format!("bad pitch {degrees}"))?;
                steps.push((HeldKeys::default(), 0, 0.0, degrees));
                continue;
            }
            let (keys, count) = step
                .split_once('=')
                .ok_or(format!("{step}: expected keys=ticks"))?;
            let count: u32 = count
                .parse()
                .map_err(|_| format!("bad tick count {count}"))?;
            steps.push((HeldKeys::parse(keys)?, count, 0.0, 0.0));
        }
        Ok(Self {
            steps,
            index: 0,
            ticks_in_step: 0,
        })
    }

    /// The keys for the next tick and a turn and pitch to apply first, or
    /// nothing once done.
    pub fn next(&mut self) -> Option<(HeldKeys, f64, f64)> {
        let mut turn = 0.0;
        let mut pitch = 0.0;
        loop {
            let (keys, count, degrees, tilt) = *self.steps.get(self.index)?;
            if count == 0 {
                turn += degrees;
                pitch += tilt;
                self.index += 1;
                continue;
            }
            self.ticks_in_step += 1;
            if self.ticks_in_step >= count {
                self.index += 1;
                self.ticks_in_step = 0;
            }
            return Some((keys, turn, pitch));
        }
    }
}

/// A click type by its id, clamped into the ones there are.
fn click_type(id: i32) -> ClickType {
    [
        ClickType::Pickup,
        ClickType::QuickMove,
        ClickType::Swap,
        ClickType::Clone,
        ClickType::Throw,
        ClickType::QuickCraft,
        ClickType::PickupAll,
    ][id.clamp(0, 6) as usize]
}
