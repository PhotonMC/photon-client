use super::*;

/// Wire-level fields decoded from one clientbound entity-position-sync packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityPositionSyncObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// Wire-level fields decoded from one clientbound entity-teleport packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityTeleportObservation {
    pub entity_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub delta_x: f64,
    pub delta_y: f64,
    pub delta_z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
    pub relative_flags: i32,
    pub on_ground: bool,
    pub motion_before: Option<(f64, f64, f64)>,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
    pub applied_motion: Option<(f64, f64, f64)>,
}

/// Wire-level fields decoded from one clientbound relative entity-move packet.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMoveObservation {
    pub entity_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub y_rot: Option<f32>,
    pub x_rot: Option<f32>,
    pub has_position: bool,
    pub has_rotation: bool,
    pub on_ground: bool,
    pub applied_to_world: bool,
    pub applied_on_ground: Option<bool>,
    pub applied_position: Option<(f64, f64, f64)>,
    pub applied_rotation: Option<(f32, f32, f32)>,
}

/// Wire-level fields decoded from one clientbound container slot update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetSlotObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound cursor item update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorItemObservation {
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one direct local-player inventory update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayerInventoryObservation {
    pub slot: i32,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container content update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetContentObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub item_count: usize,
    pub items: Vec<Option<ClientItemStackState>>,
    pub hotbar_slot_zero_item_count: Option<i32>,
    pub hotbar_slot_zero_item_registry_id: Option<i32>,
    pub hotbar_slot_zero_positive_component_type_ids: Vec<i32>,
    pub hotbar_slot_zero_negative_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_item_count: Option<i32>,
    pub main_inventory_slot_zero_item_registry_id: Option<i32>,
    pub main_inventory_slot_zero_positive_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_negative_component_type_ids: Vec<i32>,
    pub carried_item_count: Option<i32>,
    pub carried_item_registry_id: Option<i32>,
    pub carried_positive_component_type_ids: Vec<i32>,
    pub carried_negative_component_type_ids: Vec<i32>,
    pub carried_item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound mob-effect removal packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectRemoveObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound mob-effect update packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectUpdateObservation {
    pub entity_id: i32,
    pub effect_registry_id: i32,
    pub amplifier: i32,
    pub duration_ticks: i32,
    pub flags: u8,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound system-chat packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemChatObservation {
    pub overlay: bool,
    pub translation_key: Option<String>,
}

/// Structural fields decoded from one clientbound level-chunk-with-light packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPacketObservation {
    pub x: i32,
    pub z: i32,
    pub heightmap_count: usize,
    pub heightmap_observations: Vec<HeightmapObservation>,
    pub section_data_length: usize,
    pub section_observations: Vec<ChunkSectionObservation>,
    pub block_entity_count: usize,
    pub block_entities: Vec<BlockEntityObservation>,
    pub sky_mask_word_count: usize,
    pub block_mask_word_count: usize,
    pub empty_sky_mask_word_count: usize,
    pub empty_block_mask_word_count: usize,
    pub sky_update_count: usize,
    pub sky_update_lengths: Vec<usize>,
    pub block_update_count: usize,
    pub block_update_lengths: Vec<usize>,
    pub(crate) light_data: LightUpdateData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LightUpdateData {
    pub(crate) sky_mask: BitSet,
    pub(crate) block_mask: BitSet,
    pub(crate) empty_sky_mask: BitSet,
    pub(crate) empty_block_mask: BitSet,
    pub(crate) sky_updates: Vec<Vec<u8>>,
    pub(crate) block_updates: Vec<Vec<u8>>,
}

/// Wire-level fields observed from one chunk heightmap entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightmapObservation {
    pub type_id: i32,
    pub word_count: usize,
}

/// Structural fields decoded from one serialized 16-by-16 chunk section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkSectionObservation {
    pub non_empty_block_count: i16,
    pub fluid_count: i16,
    pub block_state_bits_per_entry: u8,
    pub block_state_palette_entry_count: usize,
    pub block_state_palette_ids: Vec<i32>,
    pub block_state_storage_word_count: usize,
    pub block_state_first_values: Vec<i32>,
    pub block_state_values: Vec<i32>,
    pub block_state_distinct_value_count: usize,
    pub biome_bits_per_entry: u8,
    pub biome_palette_entry_count: usize,
    pub biome_palette_ids: Vec<i32>,
    pub biome_storage_word_count: usize,
    pub biome_first_values: Vec<i32>,
    pub biome_values: Vec<i32>,
    pub biome_distinct_value_count: usize,
}

/// The observable result of a status and ping session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusConnectionTrace {
    pub packets: Vec<PacketObservation>,
    pub status_response: String,
    pub pong_time: i64,
}

/// One clientbound container-data packet: a menu's `ContainerData` value
/// (a furnace's burn time, an enchanting table's cost).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerDataObservation {
    pub container_id: i32,
    pub id: i16,
    pub value: i16,
}

/// What a clientbound update-recipes packet carries: the item sets by name
/// (the items a furnace, a blast furnace or a smoker accepts, and so on, as
/// registry ids) and the stonecutter's entries.
pub(crate) struct RecipeUpdate {
    pub sets: Vec<(String, Vec<i32>)>,
    pub stonecutter: Vec<voxel_world::StonecutterEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedItemStackObservation {
    pub(super) item_count: Option<i32>,
    pub(super) item_registry_id: Option<i32>,
    pub(super) positive_component_type_ids: Vec<i32>,
    pub(super) negative_component_type_ids: Vec<i32>,
    pub(super) patch_bytes: Vec<u8>,
}

impl ParsedItemStackObservation {
    pub(super) fn client_state(&self) -> Option<ClientItemStackState> {
        Some(
            ClientItemStackState::from_wire(
                self.item_count?,
                self.item_registry_id?,
                self.positive_component_type_ids.clone(),
                self.negative_component_type_ids.clone(),
            )
            .with_patch_bytes(self.patch_bytes.clone()),
        )
    }
}

/// What the title packets, the action bar and the tab list ask the HUD to show.
#[derive(Debug, Clone, PartialEq)]
pub enum HudEvent {
    Title(voxel_game::text::Component),
    Subtitle(voxel_game::text::Component),
    /// `ClientboundSetTitlesAnimationPacket`, in ticks.
    Times {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    Clear {
        reset_times: bool,
    },
    ActionBar(voxel_game::text::Component),
    TabList {
        header: voxel_game::text::Component,
        footer: voxel_game::text::Component,
    },
    /// `ClientboundBossEventPacket`.
    Boss {
        id: u128,
        operation: BossOperation,
    },
}

/// What a boss event packet does to the bar `id`.
#[derive(Debug, Clone, PartialEq)]
pub enum BossOperation {
    Add {
        name: voxel_game::text::Component,
        progress: f32,
        /// `BossBarColor` ordinal: pink, blue, red, green, yellow, purple, white.
        color: u8,
        /// `BossBarOverlay` ordinal: progress, then notched 6, 10, 12, 20.
        overlay: u8,
        flags: u8,
    },
    Remove,
    Progress(f32),
    Name(voxel_game::text::Component),
    Style {
        color: u8,
        overlay: u8,
    },
    Properties(u8),
}

/// A `PlayerInfo` of the tab list: what the client keeps of a player.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayerInfo {
    pub name: String,
    /// 0 survival, 1 creative, 2 adventure, 3 spectator.
    pub game_mode: i32,
    pub latency: i32,
    pub listed: bool,
    pub display_name: Option<voxel_game::text::Component>,
    pub list_order: i32,
}

/// One `ClientboundPlayerInfoUpdatePacket` action of a player.
#[derive(Debug, Clone, PartialEq)]
pub enum InfoAction {
    Add { name: String },
    GameMode(i32),
    Listed(bool),
    Latency(i32),
    DisplayName(Option<voxel_game::text::Component>),
    ListOrder(i32),
}

/// A player's id and what a player info packet does to it.
pub type PlayerActions = ((u64, u64), Vec<InfoAction>);

/// `ClientboundCommandSuggestionsPacket`: the request it answers, the range
/// of the typed text the matches replace, and the matches.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandSuggestions {
    pub id: i32,
    pub start: usize,
    pub length: usize,
    pub matches: Vec<String>,
}

/// `ClientboundLevelEventPacket`: an event of the level at a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEvent {
    pub event: i32,
    pub pos: (i32, i32, i32),
    pub data: i32,
    pub global: bool,
}

/// Position, event id and event data of a block event.
pub(crate) type BlockEvent = ((i32, i32, i32), u8, u8);

/// How a chat type is named on the wire (`Holder<ChatType>`): by its
/// registry id, or written out.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatHolder {
    Registry(i32),
    Direct(voxel_game::chat::ChatDecoration),
}

/// `ChatType.Bound`: the type, the sender's name and the target's.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatBound {
    pub holder: ChatHolder,
    pub name: voxel_game::text::Component,
    pub target: Option<voxel_game::text::Component>,
}

/// A `MessageSignature`: 256 bytes.
pub type ChatSignature = Vec<u8>;

/// `MessageSignature.Packed`: an id in the signature cache, or the signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedSignature {
    Cached(i32),
    Full(ChatSignature),
}

/// `FilterMask`: what the server hid of a message.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FilterMask {
    #[default]
    PassThrough,
    FullyFiltered,
    /// The bit set of hidden characters, as the words the network writes.
    Partial(Vec<u64>),
}

impl FilterMask {
    /// Whether character `index` is hidden.
    pub fn hides(&self, index: usize) -> bool {
        match self {
            Self::Partial(words) => words
                .get(index / 64)
                .is_some_and(|word| word >> (index % 64) & 1 == 1),
            _ => false,
        }
    }
}

/// `ClientboundPlayerChatPacket` as it arrives, its signatures not yet
/// looked up in the cache.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerChatPacket {
    pub global_index: i32,
    pub sender: (u64, u64),
    pub signature: Option<ChatSignature>,
    pub signed_text: String,
    pub timestamp_millis: i64,
    pub last_seen: Vec<PackedSignature>,
    pub unsigned: Option<voxel_game::text::Component>,
    pub filter: FilterMask,
    pub bound: ChatBound,
}

/// A chat packet as the client shows it.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatEvent {
    /// `ClientboundSystemChatPacket`: the action bar when `overlay`.
    System {
        content: voxel_game::text::Component,
        overlay: bool,
    },
    /// `ClientboundDisguisedChatPacket`.
    Disguised {
        content: voxel_game::text::Component,
        bound: ChatBound,
    },
    /// `ClientboundPlayerChatPacket`, its signatures looked up; whether the
    /// sender is a player the client knows and is the local player.
    Player {
        packet: PlayerChatPacket,
        sender_known: bool,
        sender_is_local: bool,
    },
    /// `ClientboundDeleteChatPacket`: the signature of the message to delete.
    Delete(ChatSignature),
}

/// `MessageSignatureCache`: the last 128 signatures, by place, which the
/// packets refer to by number.
#[derive(Debug, Clone)]
pub struct SignatureCache {
    pub(super) entries: Vec<Option<ChatSignature>>,
}
