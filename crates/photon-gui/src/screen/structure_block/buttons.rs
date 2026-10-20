//! The buttons of the structure block's screen.

use super::*;
use crate::widgets::cycle::CycleButton;
use crate::widgets::{Kind, Widget};
use photon_game::text::Run;

/// The buttons, by what they do.
pub struct Buttons {
    pub done: Widget,
    pub cancel: Widget,
    pub save: Widget,
    pub load: Widget,
    pub detect: Widget,
    /// The turn buttons: none, 90, 180 and 270 degrees.
    pub turns: [Widget; 4],
    pub mode: CycleButton<StructureMode>,
    pub entities: CycleButton<bool>,
    pub strict: CycleButton<bool>,
    pub mirror: CycleButton<Mirror>,
    pub air: CycleButton<bool>,
    pub bounding_box: CycleButton<bool>,
}

/// The turns, in the order of the buttons.
pub const TURNS: [(Slot, Rotation, &str); 4] = [
    (Slot::Rot0, Rotation::None, "0"),
    (Slot::Rot90, Rotation::Clockwise90, "90"),
    (Slot::Rot180, Rotation::Clockwise180, "180"),
    (Slot::Rot270, Rotation::Counterclockwise90, "270"),
];

fn mode_text(mode: &StructureMode) -> Component {
    Component::translatable(format!("structure_block.mode.{}", mode.name()))
}

fn mirror_text(mirror: &Mirror) -> Component {
    Component::translatable(format!("mirror.{}", mirror.name()))
}

fn on_off_text(on: &bool) -> Component {
    Component::translatable(if *on { "options.on" } else { "options.off" })
}

/// The slot of the button at a rectangle of the arena: the buttons come first in the order.
fn rect_of(slot: Slot) -> usize {
    ORDER.iter().position(|each| *each == slot).unwrap_or(0)
}

fn button(slot: Slot, label: Vec<Run>) -> Widget {
    Widget {
        rect: rect_of(slot),
        active: true,
        kind: Kind::Button { label },
    }
}

/// The mode button, which cycles through every mode but data.
fn mode_button(lang: &Lang, mode: StructureMode) -> CycleButton<StructureMode> {
    CycleButton::new(
        lang,
        rect_of(Slot::Mode),
        Component::literal("MODE"),
        vec![
            StructureMode::Save,
            StructureMode::Load,
            StructureMode::Corner,
        ],
        mode_text,
        &mode,
    )
    .only_value(lang)
}

fn mirror_button(lang: &Lang, mirror: Mirror) -> CycleButton<Mirror> {
    CycleButton::new(
        lang,
        rect_of(Slot::Mirror),
        Component::literal("MIRROR"),
        Mirror::ALL.to_vec(),
        mirror_text,
        &mirror,
    )
    .only_value(lang)
}

/// The slots of the cycle buttons.
pub const CYCLES: [Slot; 6] = [
    Slot::Mode,
    Slot::Entities,
    Slot::Strict,
    Slot::Mirror,
    Slot::Air,
    Slot::BoundingBox,
];

impl Buttons {
    /// A press of the cycle button of a slot; false when the slot has none.
    pub fn press_cycle(&mut self, slot: Slot, lang: &Lang, shift: bool) -> bool {
        match slot {
            Slot::Mode => drop(self.mode.press(lang, shift)),
            Slot::Entities => drop(self.entities.press(lang, shift)),
            Slot::Strict => drop(self.strict.press(lang, shift)),
            Slot::Mirror => drop(self.mirror.press(lang, shift)),
            Slot::Air => drop(self.air.press(lang, shift)),
            Slot::BoundingBox => drop(self.bounding_box.press(lang, shift)),
            _ => return false,
        }
        true
    }

    /// Whether the mouse is over the cycle button of a slot.
    pub fn is_over(&self, slot: Slot, arena: &Arena, over: (f64, f64)) -> bool {
        match slot {
            Slot::Mode => self.mode.is_mouse_over(arena, over),
            Slot::Entities => self.entities.is_mouse_over(arena, over),
            Slot::Strict => self.strict.is_mouse_over(arena, over),
            Slot::Mirror => self.mirror.is_mouse_over(arena, over),
            Slot::Air => self.air.is_mouse_over(arena, over),
            Slot::BoundingBox => self.bounding_box.is_mouse_over(arena, over),
            _ => false,
        }
    }

    /// The wheel over the cycle button of a slot.
    pub fn scroll_cycle(&mut self, slot: Slot, lang: &Lang, scroll_y: f64) {
        match slot {
            Slot::Mode => drop(self.mode.scrolled(lang, scroll_y)),
            Slot::Entities => drop(self.entities.scrolled(lang, scroll_y)),
            Slot::Strict => drop(self.strict.scrolled(lang, scroll_y)),
            Slot::Mirror => drop(self.mirror.scrolled(lang, scroll_y)),
            Slot::Air => drop(self.air.scrolled(lang, scroll_y)),
            Slot::BoundingBox => drop(self.bounding_box.scrolled(lang, scroll_y)),
            _ => {}
        }
    }

    /// `init`: the buttons, with the rectangles they sit in for a screen `size` big.
    pub fn new(lang: &Lang, size: (i32, i32), contents: &StructureBlockContents) -> (Arena, Self) {
        let mut arena = Arena::default();
        for slot in &ORDER[..15] {
            let at = place(*slot);
            let rect = arena.add(at.width, at.height);
            arena.0[rect].x = size.0 / 2 + at.x;
            arena.0[rect].y = at.y;
        }
        let words = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let literal = |text: &str| lang.runs(&Component::literal(text), &Style::default());
        let on_off = |slot: Slot, name: &str, value: bool| {
            CycleButton::new(
                lang,
                rect_of(slot),
                Component::translatable(name),
                vec![true, false],
                on_off_text,
                &value,
            )
            .only_value(lang)
        };
        let mut turns = TURNS.map(|(slot, _, text)| button(slot, literal(text)));
        update_turns(&mut turns, contents.rotation);
        let buttons = Self {
            done: button(Slot::Done, words("gui.done")),
            cancel: button(Slot::Cancel, words("gui.cancel")),
            save: button(Slot::Save, words("structure_block.button.save")),
            load: button(Slot::Load, words("structure_block.button.load")),
            detect: button(Slot::Detect, words("structure_block.button.detect_size")),
            turns,
            mode: mode_button(lang, contents.mode),
            entities: on_off(
                Slot::Entities,
                "structure_block.include_entities",
                !contents.ignore_entities,
            ),
            strict: on_off(Slot::Strict, "structure_block.strict", contents.strict),
            mirror: mirror_button(lang, contents.mirror),
            air: on_off(Slot::Air, "structure_block.show_air", contents.show_air),
            bounding_box: on_off(
                Slot::BoundingBox,
                "structure_block.show_boundingbox",
                contents.show_bounding_box,
            ),
        };
        (arena, buttons)
    }

    /// The plain button of a slot, if it is one.
    pub fn plain(&self, slot: Slot) -> Option<&Widget> {
        Some(match slot {
            Slot::Done => &self.done,
            Slot::Cancel => &self.cancel,
            Slot::Save => &self.save,
            Slot::Load => &self.load,
            Slot::Detect => &self.detect,
            Slot::Rot0 => &self.turns[0],
            Slot::Rot90 => &self.turns[1],
            Slot::Rot180 => &self.turns[2],
            Slot::Rot270 => &self.turns[3],
            _ => return None,
        })
    }

    /// The focus moved: the cycle buttons light to show it.
    pub fn set_focus(&mut self, focus: Slot) {
        self.mode.set_focused(focus == Slot::Mode);
        self.entities.set_focused(focus == Slot::Entities);
        self.strict.set_focused(focus == Slot::Strict);
        self.mirror.set_focused(focus == Slot::Mirror);
        self.air.set_focused(focus == Slot::Air);
        self.bounding_box.set_focused(focus == Slot::BoundingBox);
    }

    /// Whether a widget is active: the turn to the rotation the block has is not.
    pub fn is_active(&self, slot: Slot) -> bool {
        match slot {
            Slot::Rot0 | Slot::Rot90 | Slot::Rot180 | Slot::Rot270 => {
                let at = TURNS
                    .iter()
                    .position(|(each, _, _)| *each == slot)
                    .unwrap_or(0);
                self.turns[at].active
            }
            _ => true,
        }
    }
}

/// `updateDirectionButtons`: every turn is active but the one the block has.
pub fn update_turns(turns: &mut [Widget; 4], rotation: Rotation) {
    for (turn, (_, each, _)) in turns.iter_mut().zip(TURNS) {
        turn.active = each != rotation;
    }
}

impl StructureBlockScreen {
    /// The widgets the mode shows, in the order they were added; a button is lit while the mouse
    /// is on it or it has the focus.
    pub(super) fn draw_buttons(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let buttons = &self.buttons;
        let arena = &self.arena;
        for slot in &ORDER[..15] {
            if !self.is_shown(*slot) {
                continue;
            }
            if let Some(widget) = buttons.plain(*slot) {
                let highlighted = widget.contains(arena, mouse) || self.focus == *slot;
                widget.draw(frame, atlas, font, arena, highlighted);
                continue;
            }
            match slot {
                Slot::Mode => buttons.mode.draw(frame, atlas, font, arena, mouse),
                Slot::Entities => buttons.entities.draw(frame, atlas, font, arena, mouse),
                Slot::Strict => buttons.strict.draw(frame, atlas, font, arena, mouse),
                Slot::Mirror => buttons.mirror.draw(frame, atlas, font, arena, mouse),
                Slot::Air => buttons.air.draw(frame, atlas, font, arena, mouse),
                _ => buttons.bounding_box.draw(frame, atlas, font, arena, mouse),
            }
        }
    }
}
