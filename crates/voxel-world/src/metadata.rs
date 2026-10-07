//! `ClientWorld` metadata layer split out of the former single-file module.

use super::*;

/// What a wolf's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Wolf.DATA_INTERESTED_ID`,
/// `DATA_COLLAR_COLOR`, `DATA_ANGER_END_TIME`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WolfData {
    pub(crate) flags: Option<u8>,
    pub(crate) interested: Option<bool>,
    pub(crate) collar: Option<i32>,
    pub(crate) anger_end: Option<i64>,
}

/// What a cat's data holds (`TamableAnimal.DATA_FLAGS_ID`, `Cat.IS_LYING`,
/// `RELAX_STATE_ONE`, `DATA_COLLAR_COLOR`), each when it is there.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CatData {
    pub(crate) flags: Option<u8>,
    pub(crate) lying: Option<bool>,
    pub(crate) relaxed: Option<bool>,
    pub(crate) collar: Option<i32>,
}

/// What a boat's data holds of its hit, paddles and bubbles.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct BoatData {
    pub(crate) hurt_time: Option<i32>,
    pub(crate) hurt_dir: Option<i32>,
    pub(crate) damage: Option<f32>,
    pub(crate) paddle_left: Option<bool>,
    pub(crate) paddle_right: Option<bool>,
    pub(crate) bubble_time: Option<i32>,
}

/// A minecart's own synced data (`AbstractMinecart.DATA_ID_CUSTOM_DISPLAY_BLOCK`,
/// `DATA_ID_DISPLAY_OFFSET`, `MinecartFurnace.DATA_ID_FUEL`), each when the data holds it.
/// The custom block is a block state number, none inside when the server names none.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MinecartData {
    custom_display: Option<Option<u32>>,
    display_offset: Option<i32>,
    fuel: Option<bool>,
}

impl MinecartData {
    pub(crate) fn apply(self, state: &mut crate::minecart::MinecartState) {
        state.custom_display = self.custom_display.unwrap_or(state.custom_display);
        state.display_offset = self.display_offset.or(state.display_offset);
        state.fuel = self.fuel.unwrap_or(state.fuel);
    }
}

/// A wither's synced data, each field when the data holds it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct WitherData {
    targets: [Option<i32>; 2],
    invulnerable_ticks: Option<i32>,
}

impl WitherData {
    pub(crate) fn apply(self, state: &mut crate::wither::WitherState) {
        for (target, new) in state.targets.iter_mut().zip(self.targets) {
            *target = new.unwrap_or(*target);
        }
        state.invulnerable_ticks = self.invulnerable_ticks.unwrap_or(state.invulnerable_ticks);
    }
}

/// A creaking's synced data, each field when the data holds it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CreakingData {
    can_move: Option<bool>,
    active: Option<bool>,
    tearing_down: Option<bool>,
    heart_bound: Option<bool>,
}

impl CreakingData {
    pub(crate) fn apply(self, state: &mut crate::creaking::CreakingState) {
        state.can_move = self.can_move.unwrap_or(state.can_move);
        state.active = self.active.unwrap_or(state.active);
        state.tearing_down = self.tearing_down.unwrap_or(state.tearing_down);
        state.heart_bound = self.heart_bound.unwrap_or(state.heart_bound);
    }
}

/// `Avatar.DATA_PLAYER_MODE_CUSTOMISATION`: which skin layers show.
pub(crate) const PLAYER_MODE_CUSTOMISATION_DATA_ID: u8 = 16;
/// `Player.DATA_PLAYER_ABSORPTION_ID`.
pub(crate) const PLAYER_ABSORPTION_DATA_ID: u8 = 17;
/// `Player.DATA_SCORE_ID`: what the death screen shows.
pub(crate) const PLAYER_SCORE_DATA_ID: u8 = 18;

#[cfg(test)]
mod tests {
    use super::*;

    /// Entity data as the wire writes it: fields of `(data id, serializer id,
    /// value bytes)`, ended by 255.
    fn data(fields: &[(u8, i32, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (id, serializer, value) in fields {
            out.push(*id);
            voxel_core::wire::write_varint(*serializer, &mut out);
            out.extend_from_slice(value);
        }
        out.push(255);
        out
    }

    #[test]
    fn a_wolfs_flags_collar_and_anger_are_read_and_its_variant_is_a_registry_id() {
        let wolf = voxel_data::entity_type::id("minecraft:wolf").unwrap();
        let pig = voxel_data::entity_type::id("minecraft:pig").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (WOLF_INTERESTED_DATA_ID, 8, &[1]),
            (WOLF_COLLAR_DATA_ID, 1, &[3]),
            (WOLF_ANGER_END_DATA_ID, 2, &[0x90, 0x4e]),
            (WOLF_VARIANT_DATA_ID, WOLF_VARIANT_SERIALIZER_ID, &[4]),
        ]);
        let read = wolf_data_from_metadata(wolf, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.interested, Some(true));
        assert_eq!(read.collar, Some(3));
        assert_eq!(read.anger_end, Some(10_000));
        assert_eq!(variant_from_metadata(wolf, &metadata), Some(4));
        assert_eq!(wolf_data_from_metadata(pig, &metadata), WolfData::default());
    }

    #[test]
    fn a_cats_flags_lying_relax_and_collar_are_read_and_its_variant_is_a_registry_id() {
        let cat = voxel_data::entity_type::id("minecraft:cat").unwrap();
        let wolf = voxel_data::entity_type::id("minecraft:wolf").unwrap();
        let metadata = data(&[
            (TAMABLE_FLAGS_DATA_ID, 0, &[5]),
            (CAT_VARIANT_DATA_ID, CAT_VARIANT_SERIALIZER_ID, &[7]),
            (CAT_LYING_DATA_ID, 8, &[1]),
            (CAT_RELAX_DATA_ID, 8, &[0]),
            (CAT_COLLAR_DATA_ID, 1, &[3]),
        ]);
        let read = cat_data_from_metadata(cat, &metadata);
        assert_eq!(read.flags, Some(5));
        assert_eq!(read.lying, Some(true));
        assert_eq!(read.relaxed, Some(false));
        assert_eq!(read.collar, Some(3));
        assert_eq!(variant_from_metadata(cat, &metadata), Some(7));
        assert_eq!(cat_data_from_metadata(wolf, &metadata), CatData::default());
    }

    #[test]
    fn a_villagers_data_and_unhappy_counter_are_read_from_their_fields() {
        let villager = voxel_data::entity_type::id("minecraft:villager").expect("a villager");
        let cow = voxel_data::entity_type::id("minecraft:cow").expect("a cow");
        let metadata = data(&[
            (VILLAGER_UNHAPPY_DATA_ID, 1, &[40]),
            (
                VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[4, 5, 3],
            ),
        ]);
        assert_eq!(
            villager_data_from_metadata(villager, &metadata),
            Some([4, 5, 3])
        );
        assert_eq!(unhappy_counter_from_metadata(villager, &metadata), Some(40));
        assert_eq!(villager_data_from_metadata(cow, &metadata), None);
        assert_eq!(unhappy_counter_from_metadata(cow, &metadata), None);
    }

    #[test]
    fn an_endermans_block_and_a_zombie_villagers_data_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                ENDERMAN_CARRY_DATA_ID,
                OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
                &[0x81, 0x01],
            ),
            (
                ENDERMAN_CREEPY_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                ZOMBIE_VILLAGER_DATA_DATA_ID,
                VILLAGER_DATA_SERIALIZER_ID,
                &[1, 2, 3],
            ),
        ]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &metadata),
            (Some(129), Some(true))
        );
        let empty = data(&[(
            ENDERMAN_CARRY_DATA_ID,
            OPTIONAL_BLOCK_STATE_SERIALIZER_ID,
            &[0],
        )]);
        assert_eq!(
            enderman_flags_from_metadata(id("minecraft:enderman"), &empty),
            (Some(0), None)
        );
        let pumpkin = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[16],
        )]);
        let bare = data(&[(
            SNOW_GOLEM_PUMPKIN_DATA_ID,
            ENTITY_DATA_SERIALIZER_BYTE_ID,
            &[0],
        )]);
        let golem = id("minecraft:snow_golem");
        assert_eq!(
            snow_golem_pumpkin_from_metadata(golem, &pumpkin),
            Some(true)
        );
        assert_eq!(snow_golem_pumpkin_from_metadata(golem, &bare), Some(false));
        assert_eq!(
            snow_golem_pumpkin_from_metadata(id("minecraft:cow"), &pumpkin),
            None
        );
        assert_eq!(
            villager_data_from_metadata(id("minecraft:zombie_villager"), &metadata),
            Some([1, 2, 3])
        );
    }

    #[test]
    fn a_bees_flags_and_anger_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let mut anger = Vec::new();
        voxel_core::wire::write_varlong(1_234_567_890_123, &mut anger);
        let metadata = data(&[
            (BEE_FLAGS_DATA_ID, ENTITY_DATA_SERIALIZER_BYTE_ID, &[12]),
            (BEE_ANGER_END_DATA_ID, LONG_SERIALIZER_ID, &anger),
        ]);
        assert_eq!(
            bee_data_from_metadata(id("minecraft:bee"), &metadata),
            (Some(12), Some(1_234_567_890_123))
        );
        assert_eq!(
            bee_data_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn a_goats_horns_and_a_mooshrooms_variant_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (MOOSHROOM_TYPE_DATA_ID, ENTITY_DATA_SERIALIZER_INT_ID, &[1]),
            (
                GOAT_LEFT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[0],
            ),
            (
                GOAT_RIGHT_HORN_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
        ]);
        assert_eq!(
            goat_horns_from_metadata(id("minecraft:goat"), &metadata),
            (Some(false), Some(true))
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:mooshroom"), &metadata),
            Some(1)
        );
        assert_eq!(
            mooshroom_type_from_metadata(id("minecraft:cow"), &metadata),
            None
        );
    }

    #[test]
    fn an_illagers_celebrating_and_casting_are_read_from_their_fields() {
        let id = |name| voxel_data::entity_type::id(name).expect("a type");
        let metadata = data(&[
            (
                RAIDER_CELEBRATING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BOOLEAN_ID,
                &[1],
            ),
            (
                SPELLCASTER_CASTING_DATA_ID,
                ENTITY_DATA_SERIALIZER_BYTE_ID,
                &[2],
            ),
        ]);
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:evoker"), &metadata),
            (Some(true), Some(true))
        );
        // A vindicator does not cast, and a cow is no raider.
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:vindicator"), &metadata),
            (Some(true), None)
        );
        assert_eq!(
            illager_flags_from_metadata(id("minecraft:cow"), &metadata),
            (None, None)
        );
    }

    #[test]
    fn the_serializer_ids_are_the_names_this_walk_assumes() {
        let name = |id| voxel_data::entity_type::data_serializer(id).expect("a serializer");
        assert_eq!(name(ENTITY_DATA_SERIALIZER_PARTICLES_ID), "PARTICLES");
        assert_eq!(name(PIG_VARIANT_SERIALIZER_ID), "PIG_VARIANT");
        assert_eq!(name(COW_VARIANT_SERIALIZER_ID), "COW_VARIANT");
        // Registry holders and enums are one var int each.
        for id in 21..=32 {
            assert!(name(id).ends_with("_VARIANT"), "{id}");
        }
        for id in 35..=38 {
            assert!(name(id).contains("STATE"), "{id}");
        }
        assert_eq!(name(42), "HUMANOID_ARM");
        assert_eq!(name(7), "ITEM_STACK");
        assert_eq!(name(41), "RESOLVABLE_PROFILE");
    }

    #[test]
    fn every_serializer_the_walk_knows_is_stepped_over() {
        // The value of each serializer, so the field after it is still found.
        let values: [(i32, &[u8]); 41] = [
            (0, &[7]),
            (1, &[0x81, 0x01]),
            (2, &[0xFF, 0xFF, 0x03]),
            (3, &[0, 0, 0x80, 0x3F]),
            (4, &[2, b'h', b'i']),
            (5, &[8, 0, 2, b'h', b'i']),
            (6, &[0]),
            (7, &[0]),
            (8, &[1]),
            (9, &[0; 12]),
            (10, &[0; 8]),
            (11, &[0]),
            (12, &[3]),
            (
                13,
                &[1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            ),
            (14, &[0x81, 0x01]),
            (15, &[0]),
            (17, &[0]),
            (18, &[1, 2, 3]),
            (19, &[0]),
            (20, &[5]),
            (21, &[1]),
            (22, &[1]),
            (23, &[1]),
            (24, &[1]),
            (25, &[1]),
            (26, &[1]),
            (27, &[1]),
            (28, &[1]),
            (29, &[1]),
            (30, &[1]),
            (31, &[1]),
            (32, &[1]),
            (33, &[1, 3, b'a', b'b', b'c', 0, 0, 0, 0, 0, 0, 0, 0]),
            (34, &[4]),
            (35, &[1]),
            (36, &[1]),
            (37, &[1]),
            (38, &[1]),
            (39, &[0; 12]),
            (40, &[0; 16]),
            (42, &[1]),
        ];
        for (serializer, value) in values {
            let metadata = data(&[(1, serializer, value), (2, 8, &[1])]);
            assert_eq!(
                metadata_boolean_value(&metadata, 2),
                Some(true),
                "serializer {serializer}"
            );
        }
        // A particle with data of its own, a profile and a painting sent
        // whole are not walked, and the fields after them are not read.
        for serializer in [16, 41] {
            let metadata = data(&[(1, serializer, &[1, 1]), (2, 8, &[1])]);
            assert_eq!(metadata_boolean_value(&metadata, 2), None, "{serializer}");
        }
        let direct_painting = data(&[(1, 34, &[0, 1, 1]), (2, 8, &[1])]);
        assert_eq!(metadata_boolean_value(&direct_painting, 2), None);
    }

    #[test]
    fn a_pigs_and_a_cows_variant_is_read_after_the_fields_before_it() {
        let pig = voxel_data::entity_type::id("minecraft:pig").expect("pig");
        let cow = voxel_data::entity_type::id("minecraft:cow").expect("cow");
        let zombie = voxel_data::entity_type::id("minecraft:zombie").expect("zombie");
        let metadata = data(&[
            (6, 20, &[0]),
            (PIG_VARIANT_DATA_ID, PIG_VARIANT_SERIALIZER_ID, &[2]),
        ]);
        assert_eq!(variant_from_metadata(pig, &metadata), Some(2));
        // A cow's data is told apart by the entity's type, not the id alone.
        assert_eq!(variant_from_metadata(cow, &metadata), None);
        assert_eq!(variant_from_metadata(zombie, &metadata), None);
        let metadata = data(&[(COW_VARIANT_DATA_ID, COW_VARIANT_SERIALIZER_ID, &[1])]);
        assert_eq!(variant_from_metadata(cow, &metadata), Some(1));
        // The variant not sent: the default.
        assert_eq!(variant_from_metadata(pig, &data(&[(6, 20, &[0])])), None);
    }
}

mod data;
pub(crate) use data::*;
mod data_2;
pub use data_2::*;
mod player;
pub(crate) use player::*;
