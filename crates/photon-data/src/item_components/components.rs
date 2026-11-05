use super::*;

impl Cursor<'_> {
    /// One component's value, by the type's registry id.
    pub fn component(&mut self, id: i32) -> R {
        match id {
            // Holder registries, ids, enumerations and plain numbers.
            ids::MAX_STACK_SIZE
            | ids::MAX_DAMAGE
            | ids::DAMAGE
            | ids::DAMAGE_TYPE
            | ids::RARITY
            | ids::REPAIR_COST
            | ids::ENCHANTABLE
            | ids::ADDITIONAL_TRADE_COST
            | ids::DYE
            | ids::MAP_ID
            | ids::MAP_POST_PROCESSING
            | ids::OMINOUS_BOTTLE_AMPLIFIER
            | ids::BASE_COLOR
            | ids::VILLAGER_VARIANT
            | ids::WOLF_VARIANT
            | ids::WOLF_SOUND_VARIANT
            | ids::WOLF_COLLAR
            | ids::FOX_VARIANT
            | ids::SALMON_SIZE
            | ids::PARROT_VARIANT
            | ids::TROPICAL_FISH_PATTERN
            | ids::TROPICAL_FISH_BASE_COLOR
            | ids::TROPICAL_FISH_PATTERN_COLOR
            | ids::MOOSHROOM_VARIANT
            | ids::RABBIT_VARIANT
            | ids::PIG_VARIANT
            | ids::PIG_SOUND_VARIANT
            | ids::COW_VARIANT
            | ids::COW_SOUND_VARIANT
            | ids::CHICKEN_VARIANT
            | ids::CHICKEN_SOUND_VARIANT
            | ids::ZOMBIE_NAUTILUS_VARIANT
            | ids::FROG_VARIANT
            | ids::HORSE_VARIANT
            | ids::LLAMA_VARIANT
            | ids::AXOLOTL_VARIANT
            | ids::CAT_VARIANT
            | ids::CAT_SOUND_VARIANT
            | ids::CAT_COLLAR
            | ids::SHEEP_COLOR
            | ids::SHULKER_COLOR => self.varint().map(|_| ()),
            // Unit components.
            ids::UNBREAKABLE | ids::CREATIVE_SLOT_LOCK | ids::GLIDER => Ok(()),
            // Components with no network codec of their own are sent as the
            // tag their persistent codec makes: `custom_data`,
            // `intangible_projectile`, `map_decorations`, `debug_stick_state`,
            // `recipes`, `lock` and `container_loot`.
            ids::CUSTOM_DATA
            | ids::INTANGIBLE_PROJECTILE
            | ids::MAP_DECORATIONS
            | ids::DEBUG_STICK_STATE
            | ids::RECIPES
            | ids::LOCK
            | ids::CONTAINER_LOOT => self.tag(),
            // `use_effects`: two flags and a speed.
            ids::USE_EFFECTS => self.skip(6),
            // Text components.
            ids::CUSTOM_NAME | ids::ITEM_NAME => self.tag(),
            ids::MINIMUM_ATTACK_CHARGE | ids::POTION_DURATION_SCALE => self.float(),
            ids::ITEM_MODEL | ids::TOOLTIP_STYLE | ids::NOTE_BLOCK_SOUND => self.identifier(),
            ids::LORE => self.list(Cursor::tag),
            // Enchantments, the stored ones alike.
            ids::ENCHANTMENTS | ids::STORED_ENCHANTMENTS => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            // What an item can be placed on or break.
            ids::CAN_PLACE_ON | ids::CAN_BREAK => self.list(Cursor::block_predicate),
            ids::ATTRIBUTE_MODIFIERS => self.list(|cursor| {
                cursor.id(Site::Attribute, 0)?;
                cursor.identifier()?;
                cursor.double()?;
                cursor.varint()?;
                cursor.varint()?;
                match cursor.varint()? {
                    ids::CUSTOM_DATA | ids::MAX_STACK_SIZE => Ok(()),
                    ids::MAX_DAMAGE => cursor.tag(),
                    _ => Err(Malformed),
                }
            }),
            // `custom_model_data`.
            ids::CUSTOM_MODEL_DATA => {
                self.list(Cursor::float)?;
                self.list(|cursor| cursor.byte().map(|_| ()))?;
                self.list(Cursor::string)?;
                self.list(Cursor::int)
            }
            ids::TOOLTIP_DISPLAY => {
                self.boolean()?;
                self.list(|cursor| cursor.varint().map(|_| ()))
            }
            ids::ENCHANTMENT_GLINT_OVERRIDE => self.boolean().map(|_| ()),
            ids::FOOD => {
                self.varint()?;
                self.float()?;
                self.boolean().map(|_| ())
            }
            ids::CONSUMABLE => {
                self.float()?;
                self.varint()?;
                self.sound()?;
                self.boolean()?;
                self.list(Cursor::consume_effect)
            }
            ids::USE_REMAINDER | ids::SULFUR_CUBE_CONTENT => self.template(),
            ids::USE_COOLDOWN => {
                self.float()?;
                self.optional(Cursor::identifier)
            }
            ids::DAMAGE_RESISTANT | ids::REPAIRABLE | ids::PROVIDES_BANNER_PATTERNS => {
                self.component_set(id)
            }
            ids::TOOL => self.tool(),
            ids::WEAPON => {
                self.varint()?;
                self.float()
            }
            ids::ATTACK_RANGE => self.skip(24),
            ids::EQUIPPABLE => self.equippable(),
            ids::DEATH_PROTECTION => self.list(Cursor::consume_effect),
            ids::BLOCKS_ATTACKS => {
                self.float()?;
                self.float()?;
                self.list(|cursor| {
                    cursor.float()?;
                    cursor.optional(Cursor::holder_set)?;
                    cursor.float()?;
                    cursor.float()
                })?;
                self.skip(12)?;
                self.optional(Cursor::holder_set)?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            ids::PIERCING_WEAPON => {
                self.skip(2)?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            ids::KINETIC_WEAPON => {
                self.varint()?;
                self.varint()?;
                for _ in 0..3 {
                    self.optional(|cursor| {
                        cursor.varint()?;
                        cursor.skip(8)
                    })?;
                }
                self.float()?;
                self.float()?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            ids::SWING_ANIMATION | ids::ATTACK_ANIMATION | ids::INTERACT_ANIMATION => {
                self.varint()?;
                self.varint().map(|_| ())
            }
            _ => self.component_more(id),
        }
    }

    /// A number that is a constant (an int) or a reference (a key).
    pub(super) fn resolvable_int(&mut self) -> R {
        if self.boolean()? {
            self.int()
        } else {
            self.identifier()
        }
    }

    /// A number that is a constant (a float) or a reference (a key).
    pub(super) fn resolvable_float(&mut self) -> R {
        if self.boolean()? {
            self.float()
        } else {
            self.identifier()
        }
    }

    /// A holder set a component holds: items for `repairable`, a dynamic registry's otherwise.
    pub(super) fn component_set(&mut self, id: i32) -> R {
        if id == ids::REPAIRABLE {
            self.skip_item_set()
        } else {
            self.holder_set()
        }
    }

    /// `tool`: the rules (blocks, speed, drops), the default speed, the damage and whether it
    /// can destroy blocks in creative.
    pub(super) fn tool(&mut self) -> R {
        self.list(|cursor| {
            cursor.skip_block_set()?;
            cursor.optional(Cursor::float)?;
            cursor.optional(|cursor| cursor.boolean().map(|_| ()))
        })?;
        self.float()?;
        self.varint()?;
        self.boolean().map(|_| ())
    }

    /// `equippable`: the slot, sounds, models, the entities it may be put on, and flags.
    pub(super) fn equippable(&mut self) -> R {
        self.varint()?;
        self.sound()?;
        self.optional(Cursor::identifier)?;
        self.optional(Cursor::identifier)?;
        self.optional(Cursor::skip_entity_type_set)?;
        self.skip(5)?;
        self.sound()
    }

    /// `entity_data` and `block_entity_data`: a type and a compound.
    pub(super) fn typed_data(&mut self, id: i32) -> R {
        let site = if id == ids::ENTITY_DATA {
            Site::EntityType
        } else {
            Site::BlockEntityType
        };
        self.id(site, 0)?;
        self.compound_tag()
    }

    pub fn firework_explosion(&mut self) -> R {
        self.varint()?;
        self.list(Cursor::int)?;
        self.list(Cursor::int)?;
        self.skip(2)
    }

    pub fn patch(&mut self) -> R<Patch> {
        let added_count = self.count()?;
        let removed_count = self.count()?;
        let mut patch = Patch {
            added: Vec::new(),
            removed: Vec::new(),
        };
        for _ in 0..added_count {
            let id = self.id(Site::ComponentType, 0)?;
            let value = self.captured(|cursor| cursor.component(id))?;
            patch.added.push((id, value));
        }
        for _ in 0..removed_count {
            patch.removed.push(self.id(Site::ComponentType, 0)?);
        }
        Ok(patch)
    }

    /// A patch inside a value, passed over (and renumbered, with a rewrite).
    pub fn skip_patch(&mut self) -> R {
        let added_count = self.count()?;
        let removed_count = self.count()?;
        for _ in 0..added_count {
            let id = self.id(Site::ComponentType, 0)?;
            self.component(id)?;
        }
        for _ in 0..removed_count {
            self.id(Site::ComponentType, 0)?;
        }
        Ok(())
    }

    pub fn mob_effect_instance(&mut self) -> R {
        self.varint()?;
        self.mob_effect_details()
    }

    pub fn mob_effect_details(&mut self) -> R {
        self.varint()?;
        self.varint()?;
        self.skip(3)?;
        self.optional(Cursor::mob_effect_details)
    }

    pub fn consume_effect(&mut self) -> R {
        match self.varint()? {
            0 => {
                self.list(Cursor::mob_effect_instance)?;
                self.float()
            }
            1 => self.holder_set(),
            2 => Ok(()),
            3 => {
                self.float()?;
                self.directional_particles()
            }
            4 => self.sound(),
            _ => Err(Malformed),
        }
    }

    /// `Filterable.streamCodec`: the raw value and an optional filtered one.
    pub fn filterable(&mut self, mut value: impl FnMut(&mut Self) -> R) -> R {
        value(self)?;
        self.optional(&mut value)
    }

    pub fn game_profile_properties(&mut self) -> R {
        self.list(|cursor| {
            cursor.string()?;
            cursor.string()?;
            cursor.optional(Cursor::string)
        })
    }

    /// `BlockPredicate.STREAM_CODEC` of the advancement predicates.
    pub fn block_predicate(&mut self) -> R {
        self.optional(Cursor::skip_block_set)?;
        self.optional(|cursor| {
            cursor.list(|cursor| {
                cursor.string()?;
                // Exact, or a range of two optional bounds.
                if cursor.boolean()? {
                    cursor.string()
                } else {
                    cursor.optional(Cursor::string)?;
                    cursor.optional(Cursor::string)
                }
            })
        })?;
        self.optional(Cursor::compound_tag)?;
        // `DataComponentMatchers`: the exact components, then the partial
        // predicates, which this does not read.
        self.list(|cursor| {
            let id = cursor.varint()?;
            cursor.component(id)
        })?;
        if self.count()? != 0 {
            return Err(Malformed);
        }
        Ok(())
    }
}

/// A patch, split into its parts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// The added components: the type's registry id and the value's bytes.
    pub added: Vec<(i32, Vec<u8>)>,
    /// The removed components' type ids.
    pub removed: Vec<i32>,
}

/// `DataComponentPatch.STREAM_CODEC`, read from the start of a patch.
pub fn read_patch(input: &mut &[u8]) -> R<Patch> {
    let mut cursor = Cursor::new(input);
    let patch = cursor.patch()?;
    *input = cursor.data;
    Ok(patch)
}

/// `PotionContents`, as far as the colour and the name read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionContents {
    /// The potion's registry id.
    pub potion: Option<i32>,
    pub custom_color: Option<i32>,
    /// The custom effects: the mob effect's registry id, its amplifier, its
    /// duration in ticks (-1 for no end) and whether it shows.
    pub effects: Vec<(i32, i32, i32, bool)>,
    pub custom_name: Option<String>,
}

/// `block_state` (`BlockItemStateProperties`): the property names and the
/// values the stack gives them.
pub fn block_state_properties(value: &[u8]) -> Option<Vec<(String, String)>> {
    let mut cursor = Cursor::new(value);
    let string = |cursor: &mut Cursor| -> Option<String> {
        let length = cursor.count().ok()?;
        let bytes = cursor.data.get(..length)?;
        let text = String::from_utf8_lossy(bytes).into_owned();
        cursor.skip(length).ok()?;
        Some(text)
    };
    let mut properties = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let name = string(&mut cursor)?;
        properties.push((name, string(&mut cursor)?));
    }
    Some(properties)
}

/// `potion_contents`.
pub fn potion_contents(value: &[u8]) -> Option<PotionContents> {
    let mut cursor = Cursor::new(value);
    let potion = if cursor.boolean().ok()? {
        Some(cursor.varint().ok()?)
    } else {
        None
    };
    let custom_color = if cursor.boolean().ok()? {
        let bytes = cursor.data.get(..4)?;
        let color = i32::from_be_bytes(bytes.try_into().ok()?);
        cursor.skip(4).ok()?;
        Some(color)
    } else {
        None
    };
    let mut effects = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let effect = cursor.varint().ok()?;
        let amplifier = cursor.varint().ok()?;
        let duration = cursor.varint().ok()?;
        cursor.boolean().ok()?;
        let visible = cursor.boolean().ok()?;
        cursor.boolean().ok()?;
        if cursor.boolean().ok()? {
            cursor.mob_effect_details().ok()?;
        }
        effects.push((effect, amplifier, duration, visible));
    }
    let custom_name = if cursor.boolean().ok()? {
        let length = cursor.count().ok()?;
        let bytes = cursor.data.get(..length)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    } else {
        None
    };
    Some(PotionContents {
        potion,
        custom_color,
        effects,
        custom_name,
    })
}

/// `firework_explosion`'s colours.
pub fn firework_colors(value: &[u8]) -> Option<Vec<i32>> {
    let mut cursor = Cursor::new(value);
    cursor.varint().ok()?;
    let mut colors = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let bytes = cursor.data.get(..4)?;
        colors.push(i32::from_be_bytes(bytes.try_into().ok()?));
        cursor.skip(4).ok()?;
    }
    Some(colors)
}

/// `fireworks` (`Fireworks.STREAM_CODEC`): the explosions of a rocket, in order.
pub fn fireworks(value: &[u8]) -> Option<Vec<crate::particle_values::FireworkExplosion>> {
    use crate::particle_values::{FireworkExplosion, FireworkShape};
    let mut cursor = Cursor::new(value);
    // The flight duration.
    cursor.varint().ok()?;
    let colors = |cursor: &mut Cursor| -> Option<Vec<i32>> {
        let mut list = Vec::new();
        for _ in 0..cursor.count().ok()? {
            let bytes = cursor.data.get(..4)?;
            list.push(i32::from_be_bytes(bytes.try_into().ok()?));
            cursor.skip(4).ok()?;
        }
        Some(list)
    };
    let mut explosions = Vec::new();
    for _ in 0..cursor.count().ok()? {
        let shape = FireworkShape::from_id(cursor.varint().ok()?);
        let fills = colors(&mut cursor)?;
        let fades = colors(&mut cursor)?;
        let trail = cursor.boolean().ok()?;
        let twinkle = cursor.boolean().ok()?;
        explosions.push(FireworkExplosion {
            shape,
            colors: fills,
            fade_colors: fades,
            trail,
            twinkle,
        });
    }
    Some(explosions)
}

impl Patch {
    /// The patch as the trusted codec writes it, the way a server sends a
    /// stack.
    pub fn trusted(&self) -> Vec<u8> {
        self.write(false)
    }

    /// The patch as the delimited codec writes it, the way a client sends a
    /// stack to the server: each value behind its length.
    pub fn delimited(&self) -> Vec<u8> {
        self.write(true)
    }

    pub(super) fn write(&self, delimited: bool) -> Vec<u8> {
        let mut out = Vec::new();
        photon_core::wire::write_varint(self.added.len() as i32, &mut out);
        photon_core::wire::write_varint(self.removed.len() as i32, &mut out);
        for (id, value) in &self.added {
            photon_core::wire::write_varint(*id, &mut out);
            if delimited {
                photon_core::wire::write_varint(value.len() as i32, &mut out);
            }
            out.extend_from_slice(value);
        }
        for id in &self.removed {
            photon_core::wire::write_varint(*id, &mut out);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::version_registries::{IdMap, VersionRegistries};

    #[test]
    fn a_rockets_explosions_read_in_order() {
        use crate::particle_values::FireworkShape;
        // Flight duration 2; two explosions: a star in red with a fade and a trail, and a burst
        // with no colours that twinkles.
        let mut bytes = vec![2, 2, 2, 1];
        bytes.extend_from_slice(&0x00FF_0000_i32.to_be_bytes());
        bytes.push(1);
        bytes.extend_from_slice(&0x0000_00FF_i32.to_be_bytes());
        bytes.extend_from_slice(&[1, 0, 4, 0, 0, 0, 1]);
        let explosions = fireworks(&bytes).unwrap();
        assert_eq!(explosions.len(), 2);
        assert_eq!(explosions[0].shape, FireworkShape::Star);
        assert_eq!(explosions[0].colors, vec![0x00FF_0000]);
        assert_eq!(explosions[0].fade_colors, vec![0xFF]);
        assert!(explosions[0].trail && !explosions[0].twinkle);
        assert_eq!(explosions[1].shape, FireworkShape::Burst);
        assert!(explosions[1].colors.is_empty() && !explosions[1].trail && explosions[1].twinkle);
    }

    #[test]
    fn a_patch_splits_into_its_values_and_writes_them_back() {
        // A painting variant (103) by registry id, and a removed component.
        let wire = [1, 1, 103, 5, 4];
        let mut input = &wire[..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(input.is_empty());
        assert_eq!(patch.added, vec![(103, vec![5])]);
        assert_eq!(patch.removed, vec![4]);
        assert_eq!(patch.trusted(), wire);
        assert_eq!(patch.delimited(), [1, 1, 103, 1, 5, 4]);
    }

    #[test]
    fn a_component_without_a_network_codec_is_a_tag() {
        // `custom_data` is sent as the compound its codec makes.
        let mut input = &[1_u8, 0, 0, 10, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(input.is_empty());
        assert_eq!(patch.added, vec![(0, vec![10, 0])]);
    }

    #[test]
    fn an_unknown_component_is_refused() {
        let mut input = &[1_u8, 0, 111, 0][..];
        assert_eq!(read_patch(&mut input), Err(Malformed));
    }

    #[test]
    fn nested_tags_are_skipped_whole() {
        // A compound holding a list of two ints, then a string.
        let tag = [
            10, 9, 0, 1, b'a', 3, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 8, 0, 1, b'b', 0, 1, b'x', 0,
        ];
        let mut cursor = Cursor::new(&tag);
        cursor.tag().expect("a tag");
        assert!(cursor.data.is_empty());
    }

    #[test]
    fn the_empty_patch_reads_as_two_zeros() {
        let mut input = &[0_u8, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(patch.added.is_empty() && patch.removed.is_empty());
        assert_eq!(
            delimit(
                &[0, 0],
                &Numbering::outgoing(&VersionRegistries::of(photon_core::version::Version::V26_2))
            ),
            Some(vec![0, 0])
        );
    }

    #[test]
    fn a_patch_of_a_version_with_a_gap_reads_in_canonical_ids_and_writes_back() {
        use photon_core::version::Version;
        let older = VersionRegistries::of(Version::V26_1);
        let newest = VersionRegistries::of(Version::V26_2);
        // max_stack_size 16, then removed: the id before the gap and the one after it.
        let before = ids::SULFUR_CUBE_CONTENT - 1;
        let after = ids::SULFUR_CUBE_CONTENT + 1;
        let old_wire = |id: i32| older.component_types.to_wire(id as u32).expect("in 26.1") as u8;
        let new_wire = |id: i32| newest.component_types.to_wire(id as u32).expect("in 26.2") as u8;
        let max_stack = old_wire(ids::MAX_STACK_SIZE);
        let wire = [1, 2, max_stack, 16, old_wire(before), old_wire(after)];
        let patch = read_wire_patch(&mut &wire[..], &older).expect("a patch");
        assert_eq!(patch.added, vec![(ids::MAX_STACK_SIZE, vec![16])]);
        assert_eq!(patch.removed, vec![before, after]);
        assert_eq!(
            patch.trusted(),
            [
                1,
                2,
                ids::MAX_STACK_SIZE as u8,
                16,
                before as u8,
                after as u8
            ]
        );
        let back = patch.delimited_for(&Numbering::outgoing(&older));
        assert_eq!(
            back.as_deref(),
            Some(&[1, 2, max_stack, 1, 16, old_wire(before), old_wire(after)][..])
        );
        let same = patch.delimited_for(&Numbering::outgoing(&newest));
        assert_eq!(
            same.as_deref(),
            Some(
                &[
                    1,
                    2,
                    new_wire(ids::MAX_STACK_SIZE),
                    1,
                    16,
                    new_wire(before),
                    new_wire(after)
                ][..]
            )
        );
    }

    #[test]
    fn attribute_ids_inside_values_follow_the_version() {
        use photon_core::version::Version;
        let older = VersionRegistries::of(Version::V26_1);
        let newer = VersionRegistries::of(Version::V26_2);
        let canonical = 10;
        let moved = older.attributes.to_wire(canonical).expect("both have it");
        assert_ne!(moved, canonical, "the test needs an attribute that moved");
        // attribute_modifiers: one entry (attribute, id "a", amount, operation, slot, display).
        let mut value = vec![1, moved as u8, 1, b'a'];
        value.extend([0; 8]);
        value.extend([0, 0, 0]);
        let mut wire = vec![1, 0, ids::ATTRIBUTE_MODIFIERS as u8];
        wire.extend(&value);
        let patch = read_wire_patch(&mut &wire[..], &older).expect("a patch");
        assert_eq!(
            patch.added[0].1[1],
            newer.attributes.to_wire(canonical).unwrap() as u8
        );
        let back = patch
            .delimited_for(&Numbering::outgoing(&older))
            .expect("writes");
        assert_eq!(back[3], value.len() as u8);
        assert_eq!(back[4..], value[..]);
    }

    #[test]
    fn sound_ids_inside_values_follow_the_version() {
        use photon_core::version::Version;
        let older = VersionRegistries::of(Version::V26_1);
        let canonical = 900;
        let moved = older.sound_events.to_wire(canonical).expect("both have it");
        assert_ne!(moved, canonical, "the test needs a sound that moved");
        // Component 38: two flags, then two optional sounds (a holder: its id plus one).
        let id = 38;
        let mut value = vec![0, 0, 1];
        photon_core::wire::write_varint(moved as i32 + 1, &mut value);
        value.push(0);
        let mut wire = vec![1, 0, id];
        wire.extend(&value);
        let patch = read_wire_patch(&mut &wire[..], &older).expect("a patch");
        let mut expected = vec![0, 0, 1];
        photon_core::wire::write_varint(canonical as i32 + 1, &mut expected);
        expected.push(0);
        assert_eq!(patch.added[0].1, expected);
        let back = patch
            .delimited_for(&Numbering::outgoing(&older))
            .expect("writes");
        assert_eq!(back[4..], value[..]);
    }

    #[test]
    fn item_and_block_ids_inside_values_follow_the_version() {
        use photon_core::version::Version;
        let older = VersionRegistries::of(Version::V26_1);
        let moved = |map: &IdMap| {
            (0..map.canonical_len() as u32)
                .find(|id| map.to_wire(*id).is_some_and(|wire| wire != *id))
                .expect("an id that moved")
        };
        let item = moved(&older.items);
        let wire_item = older.items.to_wire(item).unwrap();
        // repairable: a holder set of one item.
        let mut wire = vec![
            1,
            0,
            older
                .component_types
                .to_wire(ids::REPAIRABLE as u32)
                .unwrap() as u8,
        ];
        photon_core::wire::write_varint(2, &mut wire);
        photon_core::wire::write_varint(wire_item as i32, &mut wire);
        let patch = read_wire_patch(&mut &wire[..], &older).expect("a patch");
        let mut expected = Vec::new();
        photon_core::wire::write_varint(2, &mut expected);
        photon_core::wire::write_varint(item as i32, &mut expected);
        assert_eq!(patch.added[0].1, expected);
        let back = patch
            .delimited_for(&Numbering::outgoing(&older))
            .expect("writes");
        assert_eq!(back[4..], wire[3..]);
        // entity_data: the entity type, then an empty compound.
        let kind = moved(&older.entity_types);
        let mut wire = vec![
            1,
            0,
            older
                .component_types
                .to_wire(ids::ENTITY_DATA as u32)
                .unwrap() as u8,
        ];
        photon_core::wire::write_varint(
            older.entity_types.to_wire(kind).unwrap() as i32,
            &mut wire,
        );
        wire.push(0);
        let patch = read_wire_patch(&mut &wire[..], &older).expect("a patch");
        assert_eq!(patch.added[0].1[0], kind as u8);
    }
}
