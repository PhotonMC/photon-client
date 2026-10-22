//! What a test instance block's screen shows and sends
//! (`TestInstanceBlockEditScreen`, `ServerboundTestInstanceBlockActionPacket`).

use crate::structure_block::Rotation;

/// What the screen shows of a block: the test it names, the size of its structure, the rotation
/// and whether entities are left out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceContents {
    /// The test's identifier as written, `None` for a block that names none.
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: Rotation,
    pub ignore_entities: bool,
}

/// `ServerboundTestInstanceBlockActionPacket.Action`, in the order the wire counts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl InstanceAction {
    pub fn name(self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Query => "query",
            Self::Set => "set",
            Self::Reset => "reset",
            Self::Save => "save",
            Self::Export => "export",
            Self::Run => "run",
        }
    }
}

/// One action sent to the server with the values the screen holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRequest {
    pub action: InstanceAction,
    /// The test's identifier, normalised (`Identifier.toString`), `None` when the box did not
    /// parse.
    pub test: Option<String>,
    pub size: (i32, i32, i32),
    pub rotation: Rotation,
    pub ignore_entities: bool,
}

impl Rotation {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::Clockwise90,
        Self::Clockwise180,
        Self::Counterclockwise90,
    ];

    /// The enum constant's name in lower case.
    pub fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Clockwise90 => "clockwise_90",
            Self::Clockwise180 => "clockwise_180",
            Self::Counterclockwise90 => "counterclockwise_90",
        }
    }
}
