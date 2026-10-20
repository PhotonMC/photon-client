//! Where the structure block screen's widgets sit, which of them each mode shows, and its labels.

use photon_game::structure_block::StructureMode;

/// Every widget, in the order the screen added them (the order Tab goes through): the buttons
/// first, then the boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Done,
    Cancel,
    Save,
    Load,
    Mode,
    Detect,
    Entities,
    Strict,
    Mirror,
    Air,
    BoundingBox,
    Rot0,
    Rot90,
    Rot180,
    Rot270,
    Name,
    PosX,
    PosY,
    PosZ,
    SizeX,
    SizeY,
    SizeZ,
    Integrity,
    Seed,
    Data,
    /// No widget has the focus, as when the name's box is hidden at the start.
    Nowhere,
}

use Slot::*;

pub const ORDER: [Slot; 25] = [
    Done,
    Cancel,
    Save,
    Load,
    Mode,
    Detect,
    Entities,
    Strict,
    Mirror,
    Air,
    BoundingBox,
    Rot0,
    Rot90,
    Rot180,
    Rot270,
    Name,
    PosX,
    PosY,
    PosZ,
    SizeX,
    SizeY,
    SizeZ,
    Integrity,
    Seed,
    Data,
];

/// The boxes are the last ten.
pub const BOXES: [Slot; 10] = [
    Name, PosX, PosY, PosZ, SizeX, SizeY, SizeZ, Integrity, Seed, Data,
];

/// A widget's place: the offset of its left edge from the middle of the screen, its top and its size.
pub struct Place {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// Where each widget sits: its slot, the offset of its left edge from the middle, its top, width
/// and height.
const PLACES: [(Slot, i32, i32, i32, i32); 25] = [
    (Done, -154, 210, 150, 20),
    (Cancel, 4, 210, 150, 20),
    (Save, 104, 185, 50, 20),
    (Load, 104, 185, 50, 20),
    (Mode, -154, 185, 50, 20),
    (Detect, 104, 120, 50, 20),
    (Strict, 104, 120, 50, 20),
    (Entities, 104, 160, 50, 20),
    (Mirror, -20, 185, 40, 20),
    (Air, 104, 80, 50, 20),
    (BoundingBox, 104, 80, 50, 20),
    (Rot0, -102, 185, 40, 20),
    (Rot90, -61, 185, 40, 20),
    (Rot180, 21, 185, 40, 20),
    (Rot270, 42 + 20, 185, 40, 20),
    (Name, -152, 40, 300, 20),
    (PosX, -152, 80, 80, 20),
    (PosY, -72, 80, 80, 20),
    (PosZ, 8, 80, 80, 20),
    (SizeX, -152, 120, 80, 20),
    (Integrity, -152, 120, 80, 20),
    (SizeY, -72, 120, 80, 20),
    (Seed, -72, 120, 80, 20),
    (SizeZ, 8, 120, 80, 20),
    (Data, -152, 120, 240, 20),
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

/// `updateMode`: whether a mode shows a widget.
pub fn shown(slot: Slot, mode: StructureMode) -> bool {
    let (load, save, data) = (
        mode == StructureMode::Load,
        mode == StructureMode::Save,
        mode == StructureMode::Data,
    );
    match slot {
        Done | Cancel | Mode => true,
        Name => !data,
        PosX | PosY | PosZ | Entities => load || save,
        SizeX | SizeY | SizeZ | Save | Detect | Air => save,
        Integrity | Seed | Load | Strict | Mirror | BoundingBox | Rot0 | Rot90 | Rot180
        | Rot270 => load,
        Data => data,
        Nowhere => false,
    }
}

/// How a label sits against its edge.
pub enum Side {
    /// Its left edge `x` from the middle.
    Left(i32),
    /// Its right edge `x` from the middle.
    Right(i32),
}

/// A label or a box of the part of the screen under the buttons.
pub enum Item {
    Label(&'static str, Side, i32),
    Box(Slot),
}

use Item::{Box, Label};
use Side::{Left, Right};

/// The modes that show a group, and its items.
pub type Group = (fn(StructureMode) -> bool, &'static [Item]);

/// What `extractRenderState` draws after the buttons and the title, in order: the modes that show
/// each group, and its labels (by translation key) and boxes.
pub const GROUPS: [Group; 5] = [
    (
        |mode| mode != StructureMode::Data,
        &[
            Label("structure_block.structure_name", Left(-153), 30),
            Box(Name),
        ],
    ),
    (
        |mode| matches!(mode, StructureMode::Load | StructureMode::Save),
        &[
            Label("structure_block.position", Left(-153), 70),
            Box(PosX),
            Box(PosY),
            Box(PosZ),
            Label("structure_block.include_entities", Right(154), 150),
        ],
    ),
    (
        |mode| mode == StructureMode::Save,
        &[
            Label("structure_block.size", Left(-153), 110),
            Box(SizeX),
            Box(SizeY),
            Box(SizeZ),
            Label("structure_block.detect_size", Right(154), 110),
            Label("structure_block.show_air", Right(154), 70),
        ],
    ),
    (
        |mode| mode == StructureMode::Load,
        &[
            Label("structure_block.integrity", Left(-153), 110),
            Box(Integrity),
            Box(Seed),
            Label("structure_block.strict", Right(154), 110),
            Label("structure_block.show_boundingbox", Right(154), 70),
        ],
    ),
    (
        |mode| mode == StructureMode::Data,
        &[
            Label("structure_block.custom_data", Left(-153), 110),
            Box(Data),
        ],
    ),
];
