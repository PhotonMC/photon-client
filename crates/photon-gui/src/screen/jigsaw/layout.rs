//! Where the jigsaw block screen's widgets sit, and what it writes beside them.

/// Every widget, in the order the screen added them (the order Tab goes through): the boxes
/// first, then the buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Pool,
    Name,
    Target,
    FinalState,
    Selection,
    Placement,
    Joint,
    Levels,
    Keep,
    Generate,
    Done,
    Cancel,
}

use Slot::*;

pub const ORDER: [Slot; 12] = [
    Pool, Name, Target, FinalState, Selection, Placement, Joint, Levels, Keep, Generate, Done,
    Cancel,
];

/// The boxes, which are the first six.
pub const BOXES: [Slot; 6] = [Pool, Name, Target, FinalState, Selection, Placement];

/// A widget's place: the offset of its left edge from the middle of the screen, its top and its
/// size.
pub struct Place {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

const PLACES: [(Slot, i32, i32, i32, i32); 12] = [
    (Pool, -153, 20, 300, 20),
    (Name, -153, 55, 300, 20),
    (Target, -153, 90, 300, 20),
    (FinalState, -153, 125, 300, 20),
    (Selection, -153, 160, 98, 20),
    (Placement, -50, 160, 98, 20),
    (Joint, 54, 160, 100, 20),
    (Levels, -154, 185, 100, 20),
    (Keep, -50, 185, 100, 20),
    (Generate, 54, 185, 100, 20),
    (Done, -154, 210, 150, 20),
    (Cancel, 4, 210, 150, 20),
];

pub fn place(slot: Slot) -> Place {
    let (_, x, y, width, height) = PLACES
        .iter()
        .find(|(each, ..)| *each == slot)
        .copied()
        .unwrap_or((slot, 0, 0, 0, 0));
    Place {
        x,
        y,
        width,
        height,
    }
}

/// A label, by translation key, with its left edge from the middle of the screen and its top.
pub type Label = (&'static str, i32, i32);

/// What `extractRenderState` writes after the buttons, in order: each label, then (but for the
/// two priorities, which swap their boxes) the box under it.
pub enum Item {
    Label(Label),
    Box(Slot),
}

pub const ITEMS: [Item; 13] = [
    Item::Label(("jigsaw_block.pool", -153, 10)),
    Item::Box(Pool),
    Item::Label(("jigsaw_block.name", -153, 45)),
    Item::Box(Name),
    Item::Label(("jigsaw_block.target", -153, 80)),
    Item::Box(Target),
    Item::Label(("jigsaw_block.final_state", -153, 115)),
    Item::Box(FinalState),
    Item::Label(("jigsaw_block.selection_priority", -153, 150)),
    Item::Box(Placement),
    Item::Label(("jigsaw_block.placement_priority", -50, 150)),
    Item::Box(Selection),
    Item::Label(("jigsaw_block.joint_label", 53, 150)),
];
