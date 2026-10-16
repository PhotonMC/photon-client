//! The game mode switcher (`GameModeSwitcherScreen`, F3+F4): four slots with an icon each, the one
//! picked shown by its name; F4 steps to the next, the mouse picks by hovering once it has moved,
//! and letting go of F3 switches to the one picked.

use crate::{FontRuns, FrameRuns};
use photon_game::text::{Component, Lang, Run, Style};
use photon_world::ClientPlayerGameMode;

use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame, ItemDrawer};

const SLOT_AREA: i32 = 26;
const SLOT_AREA_PADDED: i32 = 31;
/// The slots side by side with the padding between them (`ALL_SLOTS_WIDTH`).
const ALL_SLOTS_WIDTH: i32 = 4 * SLOT_AREA_PADDED - 5;

/// The slots, in the order the reference lays them out (`GameModeIcon.values`).
const MODES: [ClientPlayerGameMode; 4] = [
    ClientPlayerGameMode::Creative,
    ClientPlayerGameMode::Survival,
    ClientPlayerGameMode::Adventure,
    ClientPlayerGameMode::Spectator,
];

/// The item each slot shows and the key of the mode's name.
fn icon(mode: ClientPlayerGameMode) -> (&'static str, &'static str) {
    match mode {
        ClientPlayerGameMode::Creative => ("minecraft:grass_block", "gameMode.creative"),
        ClientPlayerGameMode::Survival => ("minecraft:iron_sword", "gameMode.survival"),
        ClientPlayerGameMode::Adventure => ("minecraft:map", "gameMode.adventure"),
        ClientPlayerGameMode::Spectator => ("minecraft:ender_eye", "gameMode.spectator"),
    }
}

/// `GameModeIcon.getNext`.
fn next(mode: ClientPlayerGameMode) -> ClientPlayerGameMode {
    match mode {
        ClientPlayerGameMode::Creative => ClientPlayerGameMode::Survival,
        ClientPlayerGameMode::Survival => ClientPlayerGameMode::Adventure,
        ClientPlayerGameMode::Adventure => ClientPlayerGameMode::Spectator,
        ClientPlayerGameMode::Spectator => ClientPlayerGameMode::Creative,
    }
}

pub struct GameModeSwitcherScreen {
    size: (i32, i32),
    hovered: ClientPlayerGameMode,
    /// Where the mouse was when the screen first drew: the picking by the mouse starts when it
    /// has moved from there.
    first_mouse: Option<(i32, i32)>,
    /// `GameModeSlot.isSelected` of each slot, which the slot draws by one pass after it was set.
    selected: [bool; 4],
}

impl GameModeSwitcherScreen {
    /// `GameModeSwitcherScreen()`: the mode picked first is `default_selected`, which the caller
    /// takes as `getDefaultSelected` does.
    pub fn new(default_selected: ClientPlayerGameMode, size: (i32, i32)) -> Self {
        Self {
            size,
            hovered: default_selected,
            first_mouse: None,
            selected: [false; 4],
        }
    }

    /// `getDefaultSelected`: the mode played in before, or creative (survival from creative).
    pub fn default_selected(
        previous: Option<ClientPlayerGameMode>,
        current: ClientPlayerGameMode,
    ) -> ClientPlayerGameMode {
        previous.unwrap_or(if current == ClientPlayerGameMode::Creative {
            ClientPlayerGameMode::Survival
        } else {
            ClientPlayerGameMode::Creative
        })
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.size = size;
    }

    /// The mode picked now, which letting go of F3 switches to.
    pub fn selected(&self) -> ClientPlayerGameMode {
        self.hovered
    }

    /// The switch key pressed: the next mode, and the mouse is waited on to move again.
    pub fn next_mode(&mut self) {
        self.first_mouse = None;
        self.hovered = next(self.hovered);
    }

    /// `extractBackground` and `extractRenderState`, in the order the screen draws them.
    /// `select_key` is the name of the switch key.
    pub fn draw(
        &mut self,
        frame: &mut GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        item: &mut ItemDrawer<'_>,
        mouse: (i32, i32),
        select_key: &str,
    ) {
        let (width, height) = self.size;
        frame.blit_texture(
            atlas,
            "container/gamemode_switcher",
            (128, 128),
            (0, 0),
            width / 2 - 62,
            height / 2 - 31 - 27,
            125,
            75,
        );
        let name = lang.text(icon(self.hovered).1);
        frame.text_with_shadow(
            atlas,
            font,
            &name,
            width / 2 - font.width(&name) / 2,
            height / 2 - 31 - 20,
            0xFFFF_FFFF,
        );
        let key = Component::literal(select_key).with_colour("aqua");
        let text: Vec<Run> = lang.runs(
            &Component::translatable_with("debug.gamemodes.select_next", vec![key]),
            &Style::default(),
        );
        frame.styled_text(
            atlas,
            font,
            &text,
            width / 2 - font.width_of_runs(&text) / 2,
            height / 2 + 5,
            0xFFFF_FFFF,
            true,
        );
        let first = *self.first_mouse.get_or_insert(mouse);
        let same_as_first = first == mouse;
        for (index, mode) in MODES.into_iter().enumerate() {
            let x = width / 2 - ALL_SLOTS_WIDTH / 2 + index as i32 * SLOT_AREA_PADDED;
            let y = height / 2 - 31;
            // `AbstractWidget.extractRenderState` decides hovering before it draws; the slot's
            // selection is what the previous slots of this pass left.
            let over =
                mouse.0 >= x && mouse.1 >= y && mouse.0 < x + SLOT_AREA && mouse.1 < y + SLOT_AREA;
            frame.sprite(atlas, "gamemode_switcher/slot", x, y, SLOT_AREA, SLOT_AREA);
            if self.selected[index] {
                frame.sprite(
                    atlas,
                    "gamemode_switcher/selection",
                    x,
                    y,
                    SLOT_AREA,
                    SLOT_AREA,
                );
            }
            if let Some(id) = photon_game::item::id_of(icon(mode).0) {
                frame.item(
                    item,
                    &photon_world::ClientItemStackState::new(1, id),
                    x + 5,
                    y + 5,
                );
            }
            self.selected[index] = self.hovered == mode;
            if !same_as_first && (over || self.selected[index]) {
                self.hovered = mode;
            }
        }
    }
}
