//! The advancements the server tells of (`ClientboundUpdateAdvancementsPacket`): each one's place in
//! the tree, what it shows and what it asks for, and how far the player is.

use photon_core::ids::ItemId;

use crate::item_components::Patch;
use crate::text::Component;

/// `AdvancementType`, in the order the wire counts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvancementType {
    Task,
    Challenge,
    Goal,
}

impl AdvancementType {
    /// From the wire's ordinal.
    pub fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Task),
            1 => Some(Self::Challenge),
            2 => Some(Self::Goal),
            _ => None,
        }
    }
}

/// `DisplayInfo` as the client has it (the chat announcement is not sent).
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayInfo {
    pub title: Component,
    pub description: Component,
    pub icon: (ItemId, i32, Patch),
    pub frame: AdvancementType,
    pub show_toast: bool,
    pub hidden: bool,
    /// The background texture of a root, such as `minecraft:gui/advancements/backgrounds/stone`.
    pub background: Option<String>,
    pub x: f32,
    pub y: f32,
}

/// One `AdvancementHolder`: the id, and the advancement without its criteria (the client is not
/// sent them) and rewards.
#[derive(Debug, Clone, PartialEq)]
pub struct Advancement {
    pub id: String,
    pub parent: Option<String>,
    pub display: Option<DisplayInfo>,
    /// `AdvancementRequirements`: every group must have one criterion done.
    pub requirements: Vec<Vec<String>>,
    pub sends_telemetry_event: bool,
}

/// `AdvancementProgress` on the wire: each criterion and when it was obtained (milliseconds since
/// the epoch), if it was.
pub type Progress = Vec<(String, Option<i64>)>;

/// `ClientboundUpdateAdvancementsPacket`.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateAdvancements {
    /// The client forgets everything before applying the rest.
    pub reset: bool,
    pub added: Vec<Advancement>,
    pub removed: Vec<String>,
    pub progress: Vec<(String, Progress)>,
    pub show_advancements: bool,
}

mod client;
pub use client::{AdvancementProgress, ClientAdvancements, Updated};
