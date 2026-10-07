use super::*;

pub(crate) const VANILLA_ITEM_COMPONENT_CONFORMANCE: &[InventoryComponentProbeSpec] = &[
    InventoryComponentProbeSpec {
        component_path: "custom_model_data",
        component_type_id: 17,
        codec: InventoryComponentCodec::CustomModelData,
        component_hash: 0x77a4_a24f,
        item_registry_id: 1_046,
        expected_positive_component_type_ids: &[17],
        carried_components: &[(17, 0x77a4_a24f)],
    },
    InventoryComponentProbeSpec {
        component_path: "food",
        component_type_id: 23,
        codec: InventoryComponentCodec::Food,
        component_hash: 0xfac6_9b4d_u32 as i32,
        item_registry_id: 1_089,
        expected_positive_component_type_ids: &[23],
        carried_components: &[(23, 0xfac6_9b4d_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "damage",
        component_type_id: 3,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[3],
        carried_components: &[(3, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "tool",
        component_type_id: 28,
        codec: InventoryComponentCodec::Tool,
        component_hash: 0x37de_0062,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[28],
        carried_components: &[(28, 0x37de_0062)],
    },
    InventoryComponentProbeSpec {
        component_path: "weapon",
        component_type_id: 29,
        codec: InventoryComponentCodec::Weapon,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[29],
        carried_components: &[(29, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "max_stack_size",
        component_type_id: 1,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[1],
        carried_components: &[(1, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "max_damage",
        component_type_id: 2,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[1, 2],
        carried_components: &[(1, 0x5d50_d71c), (2, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "unbreakable",
        component_type_id: 4,
        codec: InventoryComponentCodec::Unit,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[4],
        carried_components: &[(4, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "minimum_attack_charge",
        component_type_id: 7,
        codec: InventoryComponentCodec::Float,
        component_hash: 0x9ec7_d769_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[7],
        carried_components: &[(7, 0x9ec7_d769_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "repair_cost",
        component_type_id: 19,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[19],
        carried_components: &[(19, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "enchantment_glint_override",
        component_type_id: 21,
        codec: InventoryComponentCodec::Byte,
        component_hash: 0xc336_cec2_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[21],
        carried_components: &[(21, 0xc336_cec2_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "rarity",
        component_type_id: 12,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xab53_df3a_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[12],
        carried_components: &[(12, 0xab53_df3a_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "item_model",
        component_type_id: 10,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0x75c9_3192_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[10],
        carried_components: &[(10, 0x75c9_3192_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "use_effects",
        component_type_id: 5,
        codec: InventoryComponentCodec::UseEffects,
        component_hash: 0x9619_e224_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[5],
        carried_components: &[(5, 0x9619_e224_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "damage_type",
        component_type_id: 8,
        codec: InventoryComponentCodec::RegistryId,
        component_hash: 0x55e3_3a50_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[8],
        carried_components: &[(8, 0x55e3_3a50_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "glider",
        component_type_id: 34,
        codec: InventoryComponentCodec::Unit,
        component_hash: 0xc574_b4c8_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[34],
        carried_components: &[(34, 0xc574_b4c8_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tooltip_style",
        component_type_id: 35,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0xabab_f3db_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[35],
        carried_components: &[(35, 0xabab_f3db_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "enchantable",
        component_type_id: 31,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xdeed_7aaa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[31],
        carried_components: &[(31, 0xdeed_7aaa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "dye",
        component_type_id: 43,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[43],
        carried_components: &[(43, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "dyed_color",
        component_type_id: 44,
        codec: InventoryComponentCodec::Int,
        component_hash: 0x23f6_5927_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[44],
        carried_components: &[(44, 0x23f6_5927_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "potion_duration_scale",
        component_type_id: 52,
        codec: InventoryComponentCodec::Float,
        component_hash: 0x9a13_eb94_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[52],
        carried_components: &[(52, 0x9a13_eb94_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "map_color",
        component_type_id: 45,
        codec: InventoryComponentCodec::Int,
        component_hash: 0x23f6_5927_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[45],
        carried_components: &[(45, 0x23f6_5927_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "map_id",
        component_type_id: 46,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x5d50_d71c,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[46],
        carried_components: &[(46, 0x5d50_d71c)],
    },
    InventoryComponentProbeSpec {
        component_path: "base_color",
        component_type_id: 73,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[73],
        carried_components: &[(73, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "note_block_sound",
        component_type_id: 71,
        codec: InventoryComponentCodec::Identifier,
        component_hash: 0xda54_5935_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[71],
        carried_components: &[(71, 0xda54_5935_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "ominous_bottle_amplifier",
        component_type_id: 63,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x3f72_5e25_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[63],
        carried_components: &[(63, 0x3f72_5e25_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "break_sound",
        component_type_id: 81,
        codec: InventoryComponentCodec::RegistryId,
        component_hash: 0x9662_f221_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[81],
        carried_components: &[(81, 0x9662_f221_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "wolf/collar",
        component_type_id: 85,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[85],
        carried_components: &[(85, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/base_color",
        component_type_id: 90,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[90],
        carried_components: &[(90, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/pattern",
        component_type_id: 89,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0xe196_874b_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[89],
        carried_components: &[(89, 0xe196_874b_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "tropical_fish/pattern_color",
        component_type_id: 91,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[91],
        carried_components: &[(91, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "cat/collar",
        component_type_id: 108,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[108],
        carried_components: &[(108, 0x8c64_52aa_u32 as i32)],
    },
    InventoryComponentProbeSpec {
        component_path: "sheep/color",
        component_type_id: 109,
        codec: InventoryComponentCodec::VarInt,
        component_hash: 0x8c64_52aa_u32 as i32,
        item_registry_id: VANILLA_STONE_ITEM_REGISTRY_ID,
        expected_positive_component_type_ids: &[109],
        carried_components: &[(109, 0x8c64_52aa_u32 as i32)],
    },
];

pub(crate) const VANILLA_MILK_BUCKET_ITEM_REGISTRY_ID: i32 = 1_046;

pub(crate) const VANILLA_SNOWBALL_ITEM_REGISTRY_ID: i32 = 1_044;

// DataComponents registers CUSTOM_MODEL_DATA after the first 17 built-in component types.

pub(crate) const PLAYER_RELATIVE_X_FLAG: i32 = 1 << 0;

pub(crate) const PLAYER_RELATIVE_Y_FLAG: i32 = 1 << 1;

pub(crate) const PLAYER_RELATIVE_Z_FLAG: i32 = 1 << 2;

pub(crate) const PLAYER_RELATIVE_Y_ROT_FLAG: i32 = 1 << 3;

pub(crate) const PLAYER_RELATIVE_X_ROT_FLAG: i32 = 1 << 4;

pub(crate) const MAX_PACKET_UNCOMPRESSED_LENGTH: usize = 8 * 1024 * 1024;

pub(crate) const MAX_CHUNK_DATA_LENGTH: usize = 2 * 1024 * 1024;

pub(crate) const MAX_KNOWN_PACKS: usize = 64;

pub(crate) const MAX_READABLE_FRAME_LENGTH: usize = MAX_FRAME_LENGTH;

pub(crate) const WORLD_READY_DRAIN_TIMEOUT: Duration = Duration::from_millis(250);

pub(crate) const DELAYED_DESTROY_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_200);

pub(crate) const WATER_BUCKET_DRAIN_TIMEOUT: Duration = Duration::from_millis(1_000);

pub(crate) const CONSUMABLE_DRAIN_TIMEOUT: Duration = Duration::from_millis(2_500);

pub(crate) const COMPONENT_PREPARATION_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const WORLD_LIFECYCLE_DRAIN_TIMEOUT: Duration = Duration::from_secs(15);

pub(crate) const DIMENSION_RESPAWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const LIGHT_UPDATE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const BLOCK_ENTITY_UPDATE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const ENTITY_ADD_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const ENTITY_REMOVE_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const ENTITY_METADATA_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) const ENTITY_POSITION_SYNC_DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors raised while driving the initial real-server client session.
#[derive(Debug)]
pub enum ConnectionError {
    Io(io::Error),
    Frame(FrameError),
    Wire(wire::ReadError),
    Compression(String),
    Codec(&'static str),
    UnsupportedPacket { phase: ConnectionProtocol, id: i32 },
    ServerDisconnected,
    InvalidArgument(&'static str),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Frame(error) => write!(formatter, "frame error: {error:?}"),
            Self::Wire(error) => write!(formatter, "wire error: {error:?}"),
            Self::Compression(error) => write!(formatter, "compression error: {error}"),
            Self::Codec(name) => write!(formatter, "packet codec failed: {name}"),
            Self::UnsupportedPacket { phase, id } => {
                write!(formatter, "unsupported packet id {id} in {phase:?}")
            }
            Self::ServerDisconnected => formatter.write_str("server disconnected"),
            Self::InvalidArgument(name) => write!(formatter, "invalid argument: {name}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<FrameError> for ConnectionError {
    fn from(error: FrameError) -> Self {
        Self::Frame(error)
    }
}

impl From<wire::ReadError> for ConnectionError {
    fn from(error: wire::ReadError) -> Self {
        Self::Wire(error)
    }
}

/// One received packet after the outer frame and optional compression were removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketObservation {
    pub phase: ConnectionProtocol,
    pub id: i32,
    pub body_length: usize,
}
