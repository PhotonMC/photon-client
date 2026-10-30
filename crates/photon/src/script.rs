//! `PHOTON_SCRIPT`: keys held and clicks made on given ticks, so a run can
//! be repeated without a person at the keyboard.

use photon_game::menus::container::ClickType;

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
    /// A scripted press of the pick item key on this tick.
    pub pick: bool,
    /// A scripted press of the drop key on this tick.
    pub drop: bool,
    /// A scripted use click on this tick.
    pub use_click: bool,
    /// The use key held down on this tick.
    pub hold_use: bool,
    /// A scripted press of the inventory key on this tick.
    pub inventory: bool,
    /// A scripted press of the advancements key on this tick.
    pub advancements: bool,
    /// The chat box opened or closed on this tick, and Enter pressed in it.
    pub chat: bool,
    pub enter: bool,
    /// Escape pressed with no screen open: the pause menu opens or closes.
    pub escape: bool,
    /// The pause menu's Disconnect: back to the title (the next `PHOTON_HOPS` address joins).
    pub leave: bool,
    /// Tab pressed in the chat box.
    pub complete: bool,
    /// F1 and F2 pressed on this tick.
    pub toggle_gui: bool,
    pub screenshot: bool,
    /// F3 pressed and let go on this tick: the debug overlay toggles.
    pub debug_overlay: bool,
    /// A letter or digit pressed with F3 held, as the window would send the events.
    pub debug_key: Option<char>,
    /// A scripted click in the open inventory: slot, button, click type.
    pub click: Option<(i32, i32, ClickType)>,
    /// A scripted tab choice on the open creative screen, by table index.
    pub tab: Option<usize>,
    /// A scripted scroll of the creative screen, in hundredths.
    pub scroll: Option<i32>,
    /// A scripted mouse wheel turn over the open screen, in notches (negative is down).
    pub wheel: Option<i32>,
    /// The mouse moved to a place, in GUI units.
    pub mouse: Option<(f64, f64)>,
    /// A mouse press and release at the mouse: button and whether shift is held.
    pub press: Option<(i32, bool)>,
    /// The files named in `PHOTON_DROP` (separated by `:`) dropped on the window.
    pub file_drop: bool,
    /// The packs in use loaded again, as the Resource Packs screen's Done does.
    pub pack_reload: bool,
    /// Text typed into the open screen.
    pub typed: Option<&'static str>,
}

/// What a plain key name does to a step.
type Flag = (&'static str, fn(&mut HeldKeys));

/// The plain key names of a step.
const FLAGS: &[Flag] = &[
    ("forward", |keys| keys.forward = true),
    ("backward", |keys| keys.backward = true),
    ("left", |keys| keys.left = true),
    ("right", |keys| keys.right = true),
    ("jump", |keys| keys.jump = true),
    ("sneak", |keys| keys.sneak = true),
    ("sprint", |keys| keys.sprint = true),
    ("attack", |keys| keys.attack = true),
    ("mine", |keys| keys.mine = true),
    ("use", |keys| keys.use_click = true),
    ("pick", |keys| keys.pick = true),
    ("drop", |keys| keys.drop = true),
    ("hold_use", |keys| keys.hold_use = true),
    ("inventory", |keys| keys.inventory = true),
    ("advancements", |keys| keys.advancements = true),
    ("chat", |keys| keys.chat = true),
    ("enter", |keys| keys.enter = true),
    ("escape", |keys| keys.escape = true),
    ("leave", |keys| keys.leave = true),
    ("complete", |keys| keys.complete = true),
    ("toggle_gui", |keys| keys.toggle_gui = true),
    ("screenshot", |keys| keys.screenshot = true),
    ("debug_overlay", |keys| keys.debug_overlay = true),
];

impl HeldKeys {
    pub fn input(self) -> photon_world::input::Input {
        photon_world::input::Input::new(
            (self.forward, self.backward, self.left, self.right),
            self.jump,
            self.sneak,
            self.sprint,
        )
    }

    /// A moving mapping (`key.forward`, ...) held down or let go.
    pub fn set_mapping(&mut self, mapping: &str, down: bool) {
        match mapping {
            "key.forward" => self.forward = down,
            "key.back" => self.backward = down,
            "key.left" => self.left = down,
            "key.right" => self.right = down,
            "key.jump" => self.jump = down,
            "key.sneak" => self.sneak = down,
            "key.sprint" => self.sprint = down,
            _ => {}
        }
    }

    /// A step's plain key name, set. Whether it was one.
    fn set_flag(&mut self, name: &str) -> bool {
        FLAGS
            .iter()
            .find(|(flag, _)| *flag == name)
            .map(|(_, set)| set(self))
            .is_some()
    }

    /// One step of a `PHOTON_SCRIPT`, such as `forward,sprint`.
    pub fn parse(step: &str) -> Result<Self, String> {
        let mut keys = Self::default();
        for name in step
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            match name {
                name if keys.set_flag(name) => {}
                "none" => {}
                other => {
                    // `click/SLOT/BUTTON/TYPE`, the type by its id.
                    let parts: Vec<&str> = other.split('/').collect();
                    match parts.as_slice() {
                        ["click", slot, button, kind] => {
                            let number = |text: &str| {
                                text.parse::<i32>()
                                    .map_err(|_| format!("bad click {other} in PHOTON_SCRIPT"))
                            };
                            keys.click =
                                Some((number(slot)?, number(button)?, click_type(number(kind)?)));
                        }
                        ["tab", tab] => {
                            keys.tab = Some(
                                tab.parse()
                                    .map_err(|_| format!("bad tab {other} in PHOTON_SCRIPT"))?,
                            );
                        }
                        ["type", text @ ..] => {
                            // A script is read once: its text lives on; a
                            // slash in it is part of the text, and `%20` is a
                            // space (steps are trimmed).
                            let text = text.join("/").replace("%20", " ");
                            keys.typed = Some(Box::leak(text.into_boxed_str()));
                        }
                        _ if keys.parse_pointer_step(&parts, other)? => {}
                        _ => return Err(format!("unknown key {other} in PHOTON_SCRIPT")),
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

impl HeldKeys {
    /// The steps of the mouse and the keyboard shortcuts: `mouse/X/Y`, `press/B`, `scroll/H`, `wheel/N` (notches over the open screen), `filedrop` (the files of `PHOTON_DROP`), `packreload` and
    /// `debug/K`. Whether `parts` was one of them.
    fn parse_pointer_step(&mut self, parts: &[&str], other: &str) -> Result<bool, String> {
        let keys = self;
        match parts {
            ["debug", key] if key.chars().count() == 1 => {
                keys.debug_key = key.chars().next();
            }
            ["mouse", x, y] => {
                let number = |text: &str| {
                    text.parse::<f64>()
                        .map_err(|_| format!("bad mouse {other} in PHOTON_SCRIPT"))
                };
                keys.mouse = Some((number(x)?, number(y)?));
            }
            ["press", button] | ["press", button, _] => {
                keys.press = Some((
                    button
                        .parse()
                        .map_err(|_| format!("bad press {other} in PHOTON_SCRIPT"))?,
                    parts.len() == 3,
                ));
            }
            ["scroll", hundredths] => {
                keys.scroll = Some(
                    hundredths
                        .parse()
                        .map_err(|_| format!("bad scroll {other} in PHOTON_SCRIPT"))?,
                );
            }
            ["filedrop"] => keys.file_drop = true,
            ["packreload"] => keys.pack_reload = true,
            ["wheel", notches] => {
                keys.wheel = Some(
                    notches
                        .parse()
                        .map_err(|_| format!("bad wheel {other} in PHOTON_SCRIPT"))?,
                );
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
