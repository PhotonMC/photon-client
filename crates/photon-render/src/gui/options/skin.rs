//! The skin customization screen (`SkinCustomizationScreen`): which parts of the skin the player
//! shows, and the main hand. Both are told to the server.

use photon_game::options::{MODEL_PART_IDS, MainHand, Options};
use photon_game::text::{Component, Lang};

use crate::gui::font::Font;
use crate::gui::options::list::{SubScreen, SubScreenInput, boolean, cycle};
use crate::gui::widgets::cycle::CycleButton;
use crate::gui::widgets::slider::SliderKey;
use crate::gui::{GuiAtlas, GuiFrame};

/// The tag of the main hand's button; the parts' are their bits.
const MAIN_HAND: u8 = 7;

/// The caption key of each part (`PlayerModelPart.name`).
const CAPTIONS: [&str; 7] = [
    "options.modelPart.cape",
    "options.modelPart.jacket",
    "options.modelPart.left_sleeve",
    "options.modelPart.right_sleeve",
    "options.modelPart.left_pants_leg",
    "options.modelPart.right_pants_leg",
    "options.modelPart.hat",
];

fn main_hand(value: &MainHand) -> Component {
    Component::translatable(value.caption_key())
}

/// What the screen's buttons say now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SkinChoices {
    pub model_parts: i32,
    pub main_hand: MainHand,
}

pub struct SkinCustomizationScreen {
    screen: SubScreen,
}

impl SkinCustomizationScreen {
    /// `init`, with the options as they are.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        options: &Options,
    ) -> Self {
        debug_assert_eq!(CAPTIONS.len(), MODEL_PART_IDS.len());
        let screen = SubScreen::new(
            font,
            lang,
            size,
            "options.skinCustomisation.title",
            in_world,
            |list| {
                let mut widgets: Vec<_> = CAPTIONS
                    .iter()
                    .enumerate()
                    .map(|(bit, caption)| {
                        (
                            bit as u8,
                            boolean(caption, options.is_model_part_enabled(bit)),
                        )
                    })
                    .collect();
                widgets.push((
                    MAIN_HAND,
                    cycle(
                        "options.mainHand",
                        MainHand::ALL.to_vec(),
                        main_hand,
                        options.main_hand,
                    ),
                ));
                list.add_small(lang, widgets);
            },
        );
        Self { screen }
    }

    /// What the buttons say now, for the options to take.
    pub fn choices(&self) -> SkinChoices {
        let list = &self.screen.list;
        let mut model_parts = 0;
        for bit in 0..CAPTIONS.len() {
            let shown = list
                .find(bit as u8)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
                .is_some_and(|button| *button.value());
            if shown {
                model_parts |= 1 << bit;
            }
        }
        let hand = list
            .find(MAIN_HAND)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<MainHand>>())
            .map_or(MainHand::Right, |button| *button.value());
        SkinChoices {
            model_parts,
            main_hand: hand,
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    pub fn clicked(&mut self, lang: &Lang, mouse: (f64, f64), shift: bool) -> SubScreenInput {
        self.screen.clicked(lang, mouse, shift)
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        self.screen.dragged(lang, mouse, dy);
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
    }

    /// An arrow, Enter or Space.
    pub fn key_pressed(&mut self, lang: &Lang, key: SliderKey, shift: bool) -> SubScreenInput {
        self.screen.key_pressed(lang, key, shift)
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
    }

    pub fn scrolled(&mut self, mouse: (f64, f64), scroll_y: f64) {
        self.screen.list.scrolled(mouse, scroll_y);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }
}
