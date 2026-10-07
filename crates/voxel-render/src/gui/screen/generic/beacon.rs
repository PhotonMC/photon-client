//! The beacon screen (`BeaconScreen`): the power buttons of the three tiers, the secondary
//! power and its upgrade, and the buttons that confirm and cancel. The effects are by their
//! registry id (their place in `MOB_EFFECTS`).

use voxel_data::item::mob_effects::MOB_EFFECTS;

use super::{ScreenAction, ScreenDraw};
use voxel_game::container_menu::ContainerMenu;

/// `BeaconBlockEntity.BEACON_EFFECTS`: the effects each tier of levels gives.
const TIERS: [&[&str]; 4] = [
    &["minecraft:speed", "minecraft:haste"],
    &["minecraft:resistance", "minecraft:jump_boost"],
    &["minecraft:strength"],
    &["minecraft:regeneration"],
];

/// A button is 22 by 22.
const SIZE: i32 = 22;

fn effect_id(name: &str) -> i32 {
    MOB_EFFECTS
        .iter()
        .position(|effect| effect.name == name)
        .expect("a beacon effect is a mob effect") as i32
}

/// `BeaconMenu.decodeEffect`: zero is none, otherwise the id plus one.
fn decode(value: i16) -> Option<i32> {
    (value != 0).then(|| i32::from(value) - 1)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Role {
    /// A power of a tier, the primary one or the secondary.
    Power {
        primary: bool,
        tier: i32,
    },
    /// The secondary power's second level (the primary's effect again).
    Upgrade,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone)]
struct Button {
    role: Role,
    /// The place in the image.
    at: (i32, i32),
    effect: i32,
    active: bool,
    selected: bool,
    visible: bool,
}

/// What the screen keeps: its buttons and the effects chosen so far.
#[derive(Debug, Clone)]
pub(super) struct BeaconPanel {
    buttons: Vec<Button>,
    primary: Option<i32>,
    secondary: Option<i32>,
    /// The menu's data when the choice was last read from it: the screen's `dataChanged`.
    seen: Option<(i16, i16, i16)>,
}

impl BeaconPanel {
    /// `BeaconScreen.init`.
    pub(super) fn new() -> Self {
        let mut buttons = Vec::new();
        // A widget is active until told otherwise: the powers are, at once; the others are
        // updated by the first tick.
        let button = |role, at, effect| Button {
            role,
            at,
            effect,
            active: matches!(role, Role::Upgrade | Role::Confirm | Role::Cancel),
            selected: false,
            visible: true,
        };
        for (tier, effects) in TIERS.iter().take(3).enumerate() {
            let count = effects.len() as i32;
            let total = count * 22 + (count - 1) * 2;
            for (c, name) in effects.iter().enumerate() {
                let at = (76 + c as i32 * 24 - total / 2, 22 + tier as i32 * 25);
                buttons.push(button(
                    Role::Power {
                        primary: true,
                        tier: tier as i32,
                    },
                    at,
                    effect_id(name),
                ));
            }
        }
        let count = TIERS[3].len() as i32 + 1;
        let total = count * 22 + (count - 1) * 2;
        for (c, name) in TIERS[3].iter().enumerate() {
            let at = (167 + c as i32 * 24 - total / 2, 47);
            buttons.push(button(
                Role::Power {
                    primary: false,
                    tier: 3,
                },
                at,
                effect_id(name),
            ));
        }
        let mut upgrade = button(
            Role::Upgrade,
            (167 + (count - 1) * 24 - total / 2, 47),
            effect_id(TIERS[0][0]),
        );
        upgrade.visible = false;
        buttons.push(upgrade);
        buttons.push(button(Role::Confirm, (164, 107), 0));
        buttons.push(button(Role::Cancel, (190, 107), 0));
        Self {
            buttons,
            primary: None,
            secondary: None,
            seen: None,
        }
    }

    /// The menu's data read when it changed (`dataChanged`), then every button's state
    /// (`updateButtons`).
    pub(super) fn tick(&mut self, menu: &ContainerMenu) {
        let data = |index: usize| menu.data.get(index).copied().unwrap_or(0);
        let now = (data(0), data(1), data(2));
        if self.seen != Some(now) {
            self.seen = Some(now);
            self.primary = decode(now.1);
            self.secondary = decode(now.2);
        }
        self.update(
            i32::from(now.0),
            menu.slots.first().is_some_and(Option::is_some),
        );
    }

    /// `BeaconButton.updateStatus` of each.
    fn update(&mut self, levels: i32, has_payment: bool) {
        let (primary, secondary) = (self.primary, self.secondary);
        for button in &mut self.buttons {
            match button.role {
                Role::Power {
                    primary: first,
                    tier,
                } => {
                    button.active = tier < levels;
                    button.selected =
                        Some(button.effect) == if first { primary } else { secondary };
                }
                Role::Upgrade => {
                    if let Some(effect) = primary {
                        button.visible = true;
                        button.effect = effect;
                        button.active = 3 < levels;
                        button.selected = Some(effect) == secondary;
                    } else {
                        button.visible = false;
                    }
                }
                Role::Confirm => button.active = has_payment && primary.is_some(),
                Role::Cancel => {}
            }
        }
    }

    fn over(button: &Button, (left, top): (i32, i32), mouse: (f64, f64)) -> bool {
        let (x, y) = (f64::from(left + button.at.0), f64::from(top + button.at.1));
        button.active
            && button.visible
            && mouse.0 >= x
            && mouse.1 >= y
            && mouse.0 < x + f64::from(SIZE)
            && mouse.1 < y + f64::from(SIZE)
    }

    /// A press of the left button: what the button under the mouse does, `None` when there is
    /// none.
    pub(super) fn click(
        &mut self,
        mouse: (f64, f64),
        origin: (i32, i32),
        menu: &ContainerMenu,
    ) -> Option<Vec<ScreenAction>> {
        let index = self
            .buttons
            .iter()
            .position(|button| Self::over(button, origin, mouse))?;
        let button = self.buttons[index].clone();
        match button.role {
            Role::Power { primary: true, .. } if !button.selected => {
                self.primary = Some(button.effect);
                if self.secondary != Some(button.effect) {
                    self.secondary = None;
                }
            }
            Role::Power { primary: false, .. } | Role::Upgrade if !button.selected => {
                self.secondary = Some(button.effect);
            }
            Role::Confirm => {
                return Some(vec![
                    ScreenAction::SetBeacon {
                        primary: self.primary,
                        secondary: self.secondary,
                    },
                    ScreenAction::Close,
                ]);
            }
            Role::Cancel => return Some(vec![ScreenAction::Close]),
            _ => {}
        }
        let levels = i32::from(menu.data.first().copied().unwrap_or(0));
        self.update(levels, menu.slots.first().is_some_and(Option::is_some));
        Some(Vec::new())
    }

    /// The buttons, in the order they were added, and the translation key (and what follows the
    /// translation) of the one the mouse is on.
    pub(super) fn draw(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        origin: (i32, i32),
        mouse: (f64, f64),
    ) -> Option<(String, &'static str)> {
        let mut hovered = None;
        for button in self.buttons.iter().filter(|button| button.visible) {
            let (x, y) = (origin.0 + button.at.0, origin.1 + button.at.1);
            let over = Self::over(button, origin, mouse);
            let sprite = if !button.active {
                "container/beacon/button_disabled"
            } else if button.selected {
                "container/beacon/button_selected"
            } else if over {
                "container/beacon/button_highlighted"
            } else {
                "container/beacon/button"
            };
            draw.frame.sprite(draw.atlas, sprite, x, y, SIZE, SIZE);
            let icon = match button.role {
                Role::Confirm => "container/beacon/confirm".to_owned(),
                Role::Cancel => "container/beacon/cancel".to_owned(),
                _ => format!(
                    "mob_effect/{}",
                    MOB_EFFECTS[button.effect as usize]
                        .name
                        .trim_start_matches("minecraft:")
                ),
            };
            draw.frame.sprite(draw.atlas, &icon, x + 2, y + 2, 18, 18);
            if over {
                let key = |effect: i32| MOB_EFFECTS[effect as usize].description_id();
                hovered = Some(match button.role {
                    Role::Confirm => ("gui.done".to_owned(), ""),
                    Role::Cancel => ("gui.cancel".to_owned(), ""),
                    Role::Upgrade => (key(button.effect), " II"),
                    _ => (key(button.effect), ""),
                });
            }
        }
        hovered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_game::container_menu::MenuKind;
    use voxel_world::ClientItemStackState as Stack;

    fn menu(levels: i16, primary: i16, secondary: i16, payment: bool) -> ContainerMenu {
        let mut menu = ContainerMenu::for_menu(MenuKind::Beacon, 3, Vec::new(), None, 0);
        menu.data = vec![levels, primary, secondary];
        if payment {
            menu.slots[0] = Some(Stack::new(
                1,
                voxel_game::item::id_of("minecraft:emerald").unwrap(),
            ));
        }
        menu
    }

    /// The middle of the first button of a tier, on a screen at the origin.
    fn on(panel: &BeaconPanel, index: usize) -> (f64, f64) {
        let (x, y) = panel.buttons[index].at;
        (f64::from(x + 11), f64::from(y + 11))
    }

    #[test]
    fn the_powers_a_level_allows_are_active_and_a_chosen_one_is_selected() {
        let mut panel = BeaconPanel::new();
        panel.tick(&menu(2, effect_id("minecraft:speed") as i16 + 1, 0, false));
        // Speed and haste (tier 0) and the resistance tier (1) are open; strength (2) is not.
        assert!(panel.buttons[0].active && panel.buttons[0].selected);
        assert!(panel.buttons[1].active && !panel.buttons[1].selected);
        assert!(panel.buttons[2].active);
        assert!(!panel.buttons[4].active);
        // The upgrade shows once a primary is chosen, and waits for the fourth level.
        let upgrade = panel
            .buttons
            .iter()
            .find(|b| b.role == Role::Upgrade)
            .unwrap();
        assert!(upgrade.visible && !upgrade.active);
    }

    #[test]
    fn choosing_a_new_primary_drops_a_secondary_that_is_not_the_same_effect() {
        let mut panel = BeaconPanel::new();
        let open = menu(4, 0, 0, true);
        panel.tick(&open);
        panel.click(on(&panel, 0), (0, 0), &open).unwrap();
        assert_eq!(panel.primary, Some(effect_id("minecraft:speed")));
        // The secondary power, regeneration, then another primary.
        let regeneration = panel
            .buttons
            .iter()
            .position(|b| matches!(b.role, Role::Power { primary: false, .. }))
            .unwrap();
        panel
            .click(on(&panel, regeneration), (0, 0), &open)
            .unwrap();
        assert_eq!(panel.secondary, Some(effect_id("minecraft:regeneration")));
        panel.click(on(&panel, 1), (0, 0), &open).unwrap();
        assert_eq!(panel.primary, Some(effect_id("minecraft:haste")));
        assert_eq!(panel.secondary, None);
    }

    #[test]
    fn confirming_sends_the_choice_and_closes_cancelling_only_closes() {
        let mut panel = BeaconPanel::new();
        let open = menu(1, 0, 0, true);
        panel.tick(&open);
        // No choice yet: the confirm button is not active, so it takes no click.
        let confirm = panel.buttons.len() - 2;
        assert!(panel.click(on(&panel, confirm), (0, 0), &open).is_none());
        panel.click(on(&panel, 0), (0, 0), &open).unwrap();
        let actions = panel.click(on(&panel, confirm), (0, 0), &open).unwrap();
        assert_eq!(
            actions,
            vec![
                ScreenAction::SetBeacon {
                    primary: Some(effect_id("minecraft:speed")),
                    secondary: None
                },
                ScreenAction::Close
            ]
        );
        let cancel = panel.buttons.len() - 1;
        assert_eq!(
            panel.click(on(&panel, cancel), (0, 0), &open).unwrap(),
            vec![ScreenAction::Close]
        );
    }
}
