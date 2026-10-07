use super::*;

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
