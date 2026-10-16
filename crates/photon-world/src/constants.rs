//! `ClientWorld` constants layer split out of the former single-file module.

use photon_core::ids::EntityTypeId;

pub const PLAYER_RESPAWN_KEEP_ATTRIBUTE_MODIFIERS: u8 = 1;
pub const PLAYER_RESPAWN_KEEP_ENTITY_DATA: u8 = 2;
pub(crate) const VANILLA_PLAYER_MOVEMENT_SPEED_ATTRIBUTE_ID: i32 = 26;
pub(crate) const VANILLA_PLAYER_DEFAULT_MOVEMENT_SPEED: f32 = 0.1_f32;
pub(crate) const VANILLA_PLAYER_SNEAKING_SPEED_ATTRIBUTE_ID: i32 = 31;
pub(crate) const VANILLA_PLAYER_DEFAULT_SNEAKING_SPEED: f64 = 0.3;
pub(crate) const VANILLA_PLAYER_SPRINTING_MODIFIER_ID: &str = "minecraft:sprinting";
pub(crate) const VANILLA_PLAYER_SPRINTING_MODIFIER_AMOUNT: f32 = 0.3_f32;
pub(crate) const VANILLA_PLAYER_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:player");
pub(crate) const VANILLA_ITEM_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:item");
pub(crate) const VANILLA_BREEZE_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:breeze");
pub(crate) const VANILLA_SPIDER_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:spider");
pub(crate) const VANILLA_ZOMBIE_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:zombie");
pub(crate) const VANILLA_SHULKER_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:shulker");
pub(crate) const VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:happy_ghast");
pub const VANILLA_COW_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:cow");
pub const VANILLA_SNOWBALL_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:snowball");
pub const VANILLA_ARROW_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:arrow");
pub const VANILLA_EXPERIENCE_ORB_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:experience_orb");
pub const VANILLA_EGG_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:egg");
pub const VANILLA_ENDER_PEARL_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:ender_pearl");
pub const VANILLA_EXPERIENCE_BOTTLE_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:experience_bottle");
pub const VANILLA_SPLASH_POTION_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:splash_potion");
pub const VANILLA_LINGERING_POTION_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:lingering_potion");
pub const VANILLA_SPECTRAL_ARROW_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:spectral_arrow");
pub const VANILLA_TRIDENT_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:trident");
pub(crate) const VANILLA_LIVING_ENTITY_TYPE_IDS: &[i32] = &[
    2, 4, 5, 7, 10, 11, 14, 16, 17, 19, 20, 21, 22, 26, 27, 28, 30, 31, 32, 35, 36, 38, 40, 41, 42,
    43, 46, 54, 55, 57, 58, 59, 61, 62, 63, 64, 66, 67, 68, 70, 78, 80, 83, 86, 87, 88, 91, 96, 97,
    98, 99, 100, 101, 102, 103, 104, 107, 108, 109, 110, 111, 112, 114, 115, 116, 117, 119, 121,
    124, 127, 128, 129, 130, 131, 135, 137, 138, 139, 140, 141, 142, 143, 145, 146, 147, 149, 150,
    151, 152, 153, 154, 155, 156,
];
pub(crate) const VANILLA_ANIMAL_ENTITY_TYPE_IDS: &[i32] = &[
    4, 7, 11, 19, 20, 21, 26, 30, 36, 54, 55, 58, 62, 64, 66, 78, 86, 87, 88, 91, 96, 98, 100, 104,
    108, 111, 116, 119, 129, 135, 138, 149, 152, 153,
];
pub(crate) const VANILLA_CHEST_BOAT_ENTITY_TYPE_IDS: &[i32] =
    &[1, 8, 13, 24, 34, 75, 82, 90, 95, 126];
pub(crate) const VANILLA_MINECART_ENTITY_TYPE_IDS: &[i32] = &[25, 29, 56, 65, 85, 122, 134];
pub(crate) const VANILLA_BOAT_RIDE_HEIGHT: f32 = 0.5625_f32 / 3.0_f32;
pub(crate) const VANILLA_RAFT_RIDE_HEIGHT: f32 = 0.5625_f32 * 0.8888889_f32;
pub(crate) const VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET: f32 = 0.15_f32;
pub(crate) const VANILLA_BOAT_HALF_WIDTH: f64 = 0.6875;
pub(crate) const VANILLA_BOAT_HEIGHT: f64 = 0.5625;
pub(crate) const VANILLA_ARROW_AIR_FRICTION: f32 = 0.99_f32;
pub(crate) const VANILLA_ARROW_GRAVITY: f64 = 0.05;
pub(crate) const PROJECTILE_DEGREES_PER_RADIAN: f64 = 57.2957763671875;
pub(crate) const VANILLA_EXPERIENCE_ORB_AIR_FRICTION: f32 = 0.98_f32;
pub(crate) const VANILLA_EXPERIENCE_ORB_GRAVITY: f64 = 0.03;
pub const VANILLA_PRIMED_TNT_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:tnt");
pub(crate) const VANILLA_PRIMED_TNT_AIR_FRICTION: f32 = 0.98_f32;
pub(crate) const VANILLA_PRIMED_TNT_GRAVITY: f64 = 0.04;
pub const VANILLA_FALLING_BLOCK_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:falling_block");
pub(crate) const VANILLA_FALLING_BLOCK_AIR_FRICTION: f32 = 0.98_f32;
pub(crate) const VANILLA_FALLING_BLOCK_GRAVITY: f64 = 0.04;
pub const VANILLA_FIREWORK_ROCKET_ENTITY_TYPE_ID: EntityTypeId =
    photon_data::entity_type::named("minecraft:firework_rocket");
pub(crate) const VANILLA_FIREWORK_ROCKET_HORIZONTAL_ACCELERATION: f64 = 1.15;
pub(crate) const VANILLA_FIREWORK_ROCKET_VERTICAL_ACCELERATION: f64 = 0.04;
pub(crate) const VANILLA_SPIDER_VEHICLE_ATTACHMENT_Y: f64 = 0.3125;
pub(crate) const VANILLA_BOAT_PASSENGER_GEOMETRY: &[(i32, f32, f32)] = &[
    (0, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        1,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (
        8,
        VANILLA_RAFT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (9, VANILLA_RAFT_RIDE_HEIGHT, 0.0),
    (12, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        13,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (23, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        24,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (33, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        34,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (74, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        75,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (81, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        82,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (89, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        90,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (94, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        95,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
    (125, VANILLA_BOAT_RIDE_HEIGHT, 0.0),
    (
        126,
        VANILLA_BOAT_RIDE_HEIGHT,
        VANILLA_CHEST_BOAT_PASSENGER_Z_OFFSET,
    ),
];
pub(crate) const VANILLA_PLAYER_VEHICLE_ATTACHMENT_Y: f64 = 0.6;
pub(crate) const VANILLA_AIR_BLOCK_STATE_ID: i32 = 0;
pub(crate) const VANILLA_WATER_BLOCK_STATE_ID: i32 = 86;
pub(crate) const ENTITY_SHARED_FLAGS_DATA_ID: u8 = 0;
pub(crate) const ENTITY_AIR_SUPPLY_DATA_ID: u8 = 1;
pub(crate) const ENTITY_CUSTOM_NAME_DATA_ID: u8 = 2;
pub(crate) const ENTITY_CUSTOM_NAME_VISIBLE_DATA_ID: u8 = 3;
pub(crate) const ENTITY_SILENT_DATA_ID: u8 = 4;
pub(crate) const ENTITY_NO_GRAVITY_DATA_ID: u8 = 5;
pub(crate) const ENTITY_POSE_DATA_ID: u8 = 6;
pub(crate) const ENTITY_TICKS_FROZEN_DATA_ID: u8 = 7;
pub(crate) const ENTITY_LIVING_FLAGS_DATA_ID: u8 = 8;
pub(crate) const ENTITY_HEALTH_DATA_ID: u8 = 9;
pub(crate) const ENTITY_EFFECT_PARTICLES_DATA_ID: u8 = 10;
pub(crate) const ENTITY_EFFECT_AMBIENCE_DATA_ID: u8 = 11;
pub(crate) const ENTITY_ARROW_COUNT_DATA_ID: u8 = 12;
pub(crate) const ENTITY_STINGER_COUNT_DATA_ID: u8 = 13;
pub(crate) const ENTITY_SLEEPING_POS_DATA_ID: u8 = 14;
pub(crate) const ENTITY_MOB_FLAGS_DATA_ID: u8 = 15;
pub(crate) const VANILLA_ZOMBIE_BABY_DATA_ID: u8 = 16;
/// Where a pig's and a cow's variant sit among their data, and the
/// serializers that carry them.
pub const PIG_VARIANT_DATA_ID: u8 = 19;
pub const PIG_VARIANT_SERIALIZER_ID: i32 = 28;
pub const COW_VARIANT_DATA_ID: u8 = 18;
pub const COW_VARIANT_SERIALIZER_ID: i32 = 23;
/// Where a piglin's baby and dancing flags sit among its data (booleans).
pub const PIGLIN_BABY_DATA_ID: u8 = 17;
pub const PIGLIN_CHARGING_CROSSBOW_DATA_ID: u8 = 18;
pub const PIGLIN_DANCING_DATA_ID: u8 = 19;
/// Where an avatar's main arm sits among its data, and the serializer that
/// carries it (`HumanoidArm`, an id: left is zero).
pub const AVATAR_MAIN_HAND_DATA_ID: u8 = 15;
pub const HUMANOID_ARM_SERIALIZER_ID: i32 = 42;
/// Where a sheep's wool byte and a chicken's variant sit among their data,
/// and the serializer that carries the chicken's.
pub const SHEEP_WOOL_DATA_ID: u8 = 18;
pub const CHICKEN_VARIANT_DATA_ID: u8 = 18;
pub const CHICKEN_VARIANT_SERIALIZER_ID: i32 = 30;
/// Where an armor stand's flags (a byte: small, arms, no base plate, marker)
/// and its six rotations sit among its data, and the serializer of the
/// rotations (`Rotations`).
pub const ARMOR_STAND_FLAGS_DATA_ID: u8 = 15;
pub const ARMOR_STAND_HEAD_POSE_DATA_ID: u8 = 16;
pub const ROTATIONS_SERIALIZER_ID: i32 = 9;
/// Where a villager's unhappy counter (an int) and its type, profession and
/// level sit among its data, and the serializer that carries the latter.
pub const VILLAGER_UNHAPPY_DATA_ID: u8 = 18;
pub const VILLAGER_DATA_DATA_ID: u8 = 19;
pub const VILLAGER_DATA_SERIALIZER_ID: i32 = 18;
/// Where an enderman's carried block (an optional block state, which is zero
/// when it carries none) and creepy flag sit among its data, and the
/// serializer of the former; where a zombie villager's villager data sits.
pub const ENDERMAN_CARRY_DATA_ID: u8 = 16;
pub const OPTIONAL_BLOCK_STATE_SERIALIZER_ID: i32 = 15;
pub const ENDERMAN_CREEPY_DATA_ID: u8 = 17;
pub const ZOMBIE_VILLAGER_DATA_DATA_ID: u8 = 20;
/// Where an abstract horse's flags (a byte: eating 16, standing 32, mouth open
/// 64), a horse's variant data (an int) and a chested horse's chest (a
/// boolean) sit among their data.
pub const HORSE_FLAGS_DATA_ID: u8 = 18;
pub const HORSE_VARIANT_DATA_ID: u8 = 19;
pub const HORSE_CHEST_DATA_ID: u8 = 19;
/// Where a llama's variant (an int: creamy 0, white 1, brown 2, gray 3) sits
/// among its data.
pub const LLAMA_VARIANT_DATA_ID: u8 = 21;
/// Where a camel's dash flag (a boolean) and the tick its pose last changed
/// (a long, negative while it sits) sit among its data.
pub const CAMEL_DASH_DATA_ID: u8 = 19;
pub const CAMEL_LAST_POSE_CHANGE_DATA_ID: u8 = 20;
/// Where a bee's flags (a byte: rolling, stung, nectar) and the time its anger
/// ends (a long) sit among its data, and the serializer of the latter.
pub const BEE_FLAGS_DATA_ID: u8 = 18;
pub const BEE_ANGER_END_DATA_ID: u8 = 19;
/// Where a tamable animal's flags (1 sitting, 4 tame) and a wolf's own data
/// sit: whether it is interested, its collar's dye, the game time its anger
/// ends and its variant (a registry id).
pub const TAMABLE_FLAGS_DATA_ID: u8 = 18;
pub const WOLF_INTERESTED_DATA_ID: u8 = 20;
pub const WOLF_COLLAR_DATA_ID: u8 = 21;
pub const WOLF_ANGER_END_DATA_ID: u8 = 22;
pub const WOLF_VARIANT_DATA_ID: u8 = 23;
pub const WOLF_VARIANT_SERIALIZER_ID: i32 = 25;
/// Where a cat's variant (a registry id), whether it lies down, whether it is
/// relaxed and the dye of its collar sit among its data (it shares the
/// tamable flags with the wolf).
pub const CAT_VARIANT_DATA_ID: u8 = 20;
/// Where a frog's variant sits among its data, and the serializer that carries it.
pub const FROG_VARIANT_DATA_ID: u8 = 18;
pub const FROG_VARIANT_SERIALIZER_ID: i32 = 27;
/// The same for a zombie nautilus's variant.
pub const ZOMBIE_NAUTILUS_VARIANT_DATA_ID: u8 = 21;
pub const ZOMBIE_NAUTILUS_VARIANT_SERIALIZER_ID: i32 = 32;
/// Where a polar bear's standing flag sits among its data.
pub const POLAR_BEAR_STANDING_DATA_ID: u8 = 18;
/// Where a fox's variant (an int) and its flags sit among its data.
pub const FOX_VARIANT_DATA_ID: u8 = 18;
/// Where a panda's counters (unhappy, sneeze, eat: ints), genes (main, hidden)
/// and flags (bytes) sit among its data.
/// Where a strider's suffocating flag sits among its data (the boost time, an
/// int, is before it).
pub const STRIDER_SUFFOCATING_DATA_ID: u8 = 19;
/// Where a slime's or a magma cube's size (`AbstractCubeMob.ID_SIZE`, an int) sits among its data.
pub const CUBE_MOB_SIZE_DATA_ID: u8 = 17;
/// Where a sulfur cube's fuse (`SulfurCube.MAX_FUSE`, an int) sits among its data.
pub const SULFUR_CUBE_FUSE_DATA_ID: u8 = 18;
/// Where a shulker's attach face (a direction), peek (hundredths, a byte) and colour (a byte, 16 for none) sit among its data.
pub const SHULKER_ATTACH_FACE_DATA_ID: u8 = 16;
pub const SHULKER_PEEK_DATA_ID: u8 = 17;
pub const SHULKER_COLOR_DATA_ID: u8 = 18;
/// The direction serializer (`EntityDataSerializers.DIRECTION`).
pub(crate) const DIRECTION_SERIALIZER_ID: i32 = 12;
/// Where a copper golem's weathering and its action sit among its data, and
/// the serializers that carry them (`WEATHERING_COPPER_STATE`, `COPPER_GOLEM_STATE`).
pub const COPPER_GOLEM_WEATHER_DATA_ID: u8 = 16;
pub const COPPER_GOLEM_STATE_DATA_ID: u8 = 17;
pub(crate) const COPPER_GOLEM_WEATHER_SERIALIZER_ID: i32 = 38;
pub(crate) const COPPER_GOLEM_STATE_SERIALIZER_ID: i32 = 37;
/// Where a glow squid's dark ticks (an int) sit among its data.
pub const GLOW_SQUID_DARK_TICKS_DATA_ID: u8 = 18;
/// Where a guardian's moving flag (a boolean) and the id of what its beam aims at (an int) sit among its data.
pub const GUARDIAN_MOVING_DATA_ID: u8 = 16;
/// Where a ghast's charging flag (a boolean) and a phantom's size (an int) sit among their data.
pub const GHAST_CHARGING_DATA_ID: u8 = 16;
/// `WitherSkull.DATA_DANGEROUS`, the first of the skull's own data after `Entity`'s eight.
pub const WITHER_SKULL_DANGEROUS_DATA_ID: u8 = 8;
/// Where a bat's flags, a vex's flags (each a byte) and whether an allay dances (a boolean) sit among their data.
pub const BAT_FLAGS_DATA_ID: u8 = 16;
/// Where a parrot's variant (an int) sits among its data.
pub const PARROT_VARIANT_DATA_ID: u8 = 20;
/// Where a boat's hurt time, hurt direction and damage (`VehicleEntity`), its paddles (each a
/// boolean) and its bubble time (`AbstractBoat`) sit among its data.
pub const BOAT_HURT_DATA_ID: u8 = 8;
pub const BOAT_HURT_DIR_DATA_ID: u8 = 9;
pub const BOAT_DAMAGE_DATA_ID: u8 = 10;
pub const BOAT_PADDLE_LEFT_DATA_ID: u8 = 11;
pub const BOAT_PADDLE_RIGHT_DATA_ID: u8 = 12;
pub const BOAT_BUBBLE_TIME_DATA_ID: u8 = 13;
pub const VEX_FLAGS_DATA_ID: u8 = 16;
pub const ALLAY_DANCING_DATA_ID: u8 = 16;
pub const PHANTOM_SIZE_DATA_ID: u8 = 16;
pub const GUARDIAN_ATTACK_TARGET_DATA_ID: u8 = 17;
/// Where a salmon's size, a tropical fish's packed variant and a pufferfish's
/// puff state (each an int) sit among their data.
pub const SALMON_TYPE_DATA_ID: u8 = 17;
pub const TROPICAL_FISH_VARIANT_DATA_ID: u8 = 17;
pub const PUFFERFISH_PUFF_STATE_DATA_ID: u8 = 17;
/// Where an axolotl's variant (an int) and whether it plays dead (a boolean) sit among its data.
pub const AXOLOTL_VARIANT_DATA_ID: u8 = 18;
pub const AXOLOTL_PLAYING_DEAD_DATA_ID: u8 = 19;
/// Where a turtle's egg and its laying of one (each a boolean) sit among its data.
pub const TURTLE_HAS_EGG_DATA_ID: u8 = 18;
pub const TURTLE_LAYING_EGG_DATA_ID: u8 = 19;
/// Where a hoglin's immunity to zombification sits among its data.
pub const HOGLIN_IMMUNE_DATA_ID: u8 = 18;
pub const PANDA_UNHAPPY_DATA_ID: u8 = 18;
pub const PANDA_SNEEZE_DATA_ID: u8 = 19;
pub const PANDA_EAT_DATA_ID: u8 = 20;
pub const PANDA_MAIN_GENE_DATA_ID: u8 = 21;
pub const PANDA_HIDDEN_GENE_DATA_ID: u8 = 22;
pub const PANDA_FLAGS_DATA_ID: u8 = 23;
pub const FOX_FLAGS_DATA_ID: u8 = 19;
pub const CAT_VARIANT_SERIALIZER_ID: i32 = 21;
pub const CAT_LYING_DATA_ID: u8 = 21;
pub const CAT_RELAX_DATA_ID: u8 = 22;
pub const CAT_COLLAR_DATA_ID: u8 = 23;
pub const LONG_SERIALIZER_ID: i32 = 2;
/// Where a rabbit's variant (an int: brown is zero, the killer 99) sits among
/// its data.
pub const RABBIT_TYPE_DATA_ID: u8 = 18;
/// Where a goat's two horn flags and a mooshroom's variant (an int: red is
/// zero, brown one) sit among their data.
pub const GOAT_LEFT_HORN_DATA_ID: u8 = 19;
pub const GOAT_RIGHT_HORN_DATA_ID: u8 = 20;
pub const MOOSHROOM_TYPE_DATA_ID: u8 = 18;
/// Where a snow golem's pumpkin flag sits among its data (a byte: sixteen is
/// the pumpkin).
pub const SNOW_GOLEM_PUMPKIN_DATA_ID: u8 = 16;
/// Where a raider's celebrating flag (a boolean) and a spellcaster's spell
/// (a byte, above zero while casting) sit among their data.
pub const RAIDER_CELEBRATING_DATA_ID: u8 = 16;
pub const SPELLCASTER_CASTING_DATA_ID: u8 = 17;
/// Where a creeper's fuse direction (an int) and ignited flag (a boolean)
/// sit among its data.
pub const CREEPER_SWELL_DIR_DATA_ID: u8 = 16;
/// Where a creeper's charged flag (a boolean) sits among its data.
pub const CREEPER_POWERED_DATA_ID: u8 = 17;
pub const CREEPER_IGNITED_DATA_ID: u8 = 18;
/// Where an experience orb's value sits among its data (an int).
pub const EXPERIENCE_ORB_VALUE_DATA_ID: u8 = 8;
pub(crate) const ENTITY_DATA_SERIALIZER_BYTE_ID: i32 = 0;
pub(crate) const ENTITY_DATA_SERIALIZER_INT_ID: i32 = 1;
pub(crate) const ENTITY_DATA_SERIALIZER_FLOAT_ID: i32 = 3;
pub(crate) const ENTITY_DATA_SERIALIZER_OPTIONAL_COMPONENT_ID: i32 = 6;
pub(crate) const ENTITY_DATA_SERIALIZER_BOOLEAN_ID: i32 = 8;
pub(crate) const ENTITY_DATA_SERIALIZER_OPTIONAL_BLOCK_POS_ID: i32 = 11;
pub(crate) const ENTITY_DATA_SERIALIZER_PARTICLES_ID: i32 = 17;
pub(crate) const ENTITY_DATA_SERIALIZER_OPTIONAL_UNSIGNED_INT_ID: i32 = 19;
pub(crate) const ENTITY_DATA_SERIALIZER_POSE_ID: i32 = 20;
pub(crate) const ENTITY_EFFECT_PARTICLE_TYPE_ID: i32 = 28;
pub(crate) const DEFAULT_AIR_SUPPLY: i32 = 300;
pub(crate) const DEFAULT_TICKS_REQUIRED_TO_FREEZE: i32 = 140;
pub(crate) const LIVING_ENTITY_HEAD_LERP_STEPS: u8 = 3;
pub(crate) const LIVING_ENTITY_POSITION_LERP_STEPS: u8 = 3;
pub(crate) const REMOTE_PLAYER_MOTION_LERP_STEPS: u8 = 3;
pub(crate) const ENTITY_INTERPOLATION_DISTANCE_SQR: f64 = 4096.0;
pub(crate) const ENTITY_RELATIVE_X_FLAG: i32 = 1 << 0;
pub(crate) const ENTITY_RELATIVE_Y_FLAG: i32 = 1 << 1;
pub(crate) const ENTITY_RELATIVE_Z_FLAG: i32 = 1 << 2;
pub(crate) const ENTITY_RELATIVE_Y_ROT_FLAG: i32 = 1 << 3;
pub(crate) const ENTITY_RELATIVE_X_ROT_FLAG: i32 = 1 << 4;
pub(crate) const ENTITY_RELATIVE_DELTA_X_FLAG: i32 = 1 << 5;
pub(crate) const ENTITY_RELATIVE_DELTA_Y_FLAG: i32 = 1 << 6;
pub(crate) const ENTITY_RELATIVE_DELTA_Z_FLAG: i32 = 1 << 7;
pub(crate) const ENTITY_RELATIVE_ROTATE_DELTA_FLAG: i32 = 1 << 8;
pub(crate) const SNOWBALL_GRAVITY: f64 = 0.03;
/// `ThrownExperienceBottle.getDefaultGravity`.
pub(crate) const VANILLA_EXPERIENCE_BOTTLE_GRAVITY: f64 = 0.07;
pub(crate) const LIVING_FLAG_USING_ITEM: u8 = 0;
pub(crate) const LIVING_FLAG_OFF_HAND: u8 = 1;
pub(crate) const LIVING_FLAG_SPIN_ATTACK: u8 = 2;
pub(crate) const MOB_FLAG_NO_AI: u8 = 0;
pub(crate) const MOB_FLAG_LEFT_HANDED: u8 = 1;
pub(crate) const MOB_FLAG_AGGRESSIVE: u8 = 2;
pub(crate) const SHARED_FLAG_ON_FIRE: u8 = 0;
pub(crate) const SHARED_FLAG_SHIFT_KEY_DOWN: u8 = 1;
pub(crate) const SHARED_FLAG_SPRINTING: u8 = 3;
pub(crate) const SHARED_FLAG_SWIMMING: u8 = 4;
pub(crate) const SHARED_FLAG_INVISIBLE: u8 = 5;
pub(crate) const SHARED_FLAG_GLOWING: u8 = 6;
pub(crate) const SHARED_FLAG_FALL_FLYING: u8 = 7;

/// Where a painting's variant (a holder) sits among its data (`HangingEntity` has the
/// direction before it), and the serializer that carries it.
pub const PAINTING_VARIANT_DATA_ID: u8 = 9;
pub const PAINTING_VARIANT_SERIALIZER_ID: i32 = 34;

/// Where an item frame's item and its turn sit among its data (`HangingEntity` has the direction
/// before them).
pub const ITEM_FRAME_ITEM_DATA_ID: u8 = 9;
pub const ITEM_FRAME_ROTATION_DATA_ID: u8 = 10;
