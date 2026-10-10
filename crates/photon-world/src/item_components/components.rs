use super::*;

impl Cursor<'_> {
    /// One component's value, by the type's registry id.
    #[expect(
        clippy::too_many_lines,
        reason = "one arm per component type's network codec"
    )]
    pub fn component(&mut self, id: i32) -> R {
        match id {
            // Holder registries, ids, enumerations and plain numbers.
            1 | 2 | 3 | 8 | 12 | 19 | 31 | 41 | 43 | 46 | 48 | 63 | 73 | 82..=102 | 104..=110 => {
                self.varint().map(|_| ())
            }
            // Unit components.
            4 | 20 | 34 => Ok(()),
            // Components with no network codec of their own are sent as the
            // tag their persistent codec makes: `custom_data`,
            // `intangible_projectile`, `map_decorations`, `debug_stick_state`,
            // `recipes`, `lock` and `container_loot`.
            0 | 22 | 47 | 57 | 66 | 79 | 80 => self.tag(),
            // `use_effects`: two flags and a speed.
            5 => self.skip(6),
            // Text components.
            6 | 9 => self.tag(),
            7 | 52 => self.float(),
            10 | 35 | 71 => self.identifier(),
            11 => self.list(Cursor::tag),
            // Enchantments, the stored ones alike.
            13 | 42 => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            // What an item can be placed on or break.
            14 | 15 => self.list(Cursor::block_predicate),
            16 => self.list(|cursor| {
                cursor.varint()?;
                cursor.identifier()?;
                cursor.double()?;
                cursor.varint()?;
                cursor.varint()?;
                match cursor.varint()? {
                    0 | 1 => Ok(()),
                    2 => cursor.tag(),
                    _ => Err(Malformed),
                }
            }),
            // `custom_model_data`.
            17 => {
                self.list(Cursor::float)?;
                self.list(|cursor| cursor.byte().map(|_| ()))?;
                self.list(Cursor::string)?;
                self.list(Cursor::int)
            }
            18 => {
                self.boolean()?;
                self.list(|cursor| cursor.varint().map(|_| ()))
            }
            21 => self.boolean().map(|_| ()),
            23 => {
                self.varint()?;
                self.float()?;
                self.boolean().map(|_| ())
            }
            24 => {
                self.float()?;
                self.varint()?;
                self.sound()?;
                self.boolean()?;
                self.list(Cursor::consume_effect)
            }
            25 | 78 => self.template(),
            26 => {
                self.float()?;
                self.optional(Cursor::identifier)
            }
            27 | 33 | 65 => self.holder_set(),
            28 => {
                self.list(|cursor| {
                    cursor.holder_set()?;
                    cursor.optional(Cursor::float)?;
                    cursor.optional(|cursor| cursor.boolean().map(|_| ()))
                })?;
                self.float()?;
                self.varint()?;
                self.boolean().map(|_| ())
            }
            29 => {
                self.varint()?;
                self.float()
            }
            30 => self.skip(24),
            32 => {
                self.varint()?;
                self.sound()?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::holder_set)?;
                self.skip(5)?;
                self.sound()
            }
            36 => self.list(Cursor::consume_effect),
            37 => {
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
            38 => {
                self.skip(2)?;
                self.optional(Cursor::sound)?;
                self.optional(Cursor::sound)
            }
            39 => {
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
            40 => {
                self.varint()?;
                self.varint().map(|_| ())
            }
            44 | 45 => self.int(),
            49 | 50 => self.list(Cursor::template),
            // `potion_contents`.
            51 => {
                self.optional(|cursor| cursor.varint().map(|_| ()))?;
                self.optional(Cursor::int)?;
                self.list(Cursor::mob_effect_instance)?;
                self.optional(Cursor::string)
            }
            53 => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            54 => self.list(|cursor| cursor.filterable(Cursor::string)),
            55 => {
                self.filterable(Cursor::string)?;
                self.string()?;
                self.varint()?;
                self.list(|cursor| cursor.filterable(Cursor::tag))?;
                self.boolean().map(|_| ())
            }
            // `trim`: the material, then the pattern.
            56 => {
                self.holder(|cursor| {
                    cursor.string()?;
                    cursor.list(|cursor| {
                        cursor.identifier()?;
                        cursor.string()
                    })?;
                    cursor.tag()
                })?;
                self.holder(|cursor| {
                    cursor.identifier()?;
                    cursor.tag()?;
                    cursor.boolean().map(|_| ())
                })
            }
            // Entity and block entity data: a type and a compound.
            58 | 60 => {
                self.varint()?;
                self.compound_tag()
            }
            59 => self.compound_tag(),
            61 => self.holder(|cursor| {
                cursor.sound()?;
                cursor.float()?;
                cursor.float()?;
                cursor.tag()
            }),
            62 => self.holder(|cursor| {
                cursor.string()?;
                cursor.list(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.tag()
            }),
            64 => self.holder(|cursor| {
                cursor.sound()?;
                cursor.tag()?;
                cursor.float()?;
                cursor.varint().map(|_| ())
            }),
            // `lodestone_tracker`: a dimension and a position, and a flag.
            67 => {
                self.optional(|cursor| {
                    cursor.identifier()?;
                    cursor.long()
                })?;
                self.boolean().map(|_| ())
            }
            68 => self.firework_explosion(),
            69 => {
                self.varint()?;
                self.list(Cursor::firework_explosion)
            }
            70 => {
                if self.boolean()? {
                    self.uuid()?;
                    self.string()?;
                    self.game_profile_properties()?;
                } else {
                    self.optional(Cursor::string)?;
                    self.optional(Cursor::uuid)?;
                    self.game_profile_properties()?;
                }
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(Cursor::identifier)?;
                self.optional(|cursor| cursor.boolean().map(|_| ()))
            }
            72 => self.list(|cursor| {
                cursor.holder(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.varint().map(|_| ())
            }),
            74 => self.list(|cursor| cursor.varint().map(|_| ())),
            75 => self.list(|cursor| cursor.optional(Cursor::template)),
            76 => self.list(|cursor| {
                cursor.string()?;
                cursor.string()
            }),
            77 => self.list(|cursor| {
                cursor.varint()?;
                cursor.compound_tag()?;
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            81 => self.sound(),
            // The painting: a registry id plus one, or the variant itself.
            103 => self.holder(|cursor| {
                cursor.varint()?;
                cursor.varint()?;
                cursor.identifier()?;
                cursor.optional(Cursor::tag)?;
                cursor.optional(Cursor::tag)
            }),
            _ => Err(Malformed),
        }
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
            let id = self.varint()?;
            let start = self.data;
            self.component(id)?;
            let length = start.len() - self.data.len();
            patch.added.push((id, start[..length].to_vec()));
        }
        for _ in 0..removed_count {
            patch.removed.push(self.varint()?);
        }
        Ok(patch)
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
            3 => self.float(),
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
        self.optional(Cursor::holder_set)?;
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
    let mut cursor = Cursor { data: input };
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
    let mut cursor = Cursor { data: value };
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
    let mut cursor = Cursor { data: value };
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
    let mut cursor = Cursor { data: value };
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
pub fn fireworks(value: &[u8]) -> Option<Vec<crate::particles::FireworkExplosion>> {
    use crate::particles::{FireworkExplosion, FireworkShape};
    let mut cursor = Cursor { data: value };
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

    #[test]
    fn a_rockets_explosions_read_in_order() {
        use crate::particles::FireworkShape;
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
        let mut cursor = Cursor { data: &tag };
        cursor.tag().expect("a tag");
        assert!(cursor.data.is_empty());
    }

    #[test]
    fn the_empty_patch_reads_as_two_zeros() {
        let mut input = &[0_u8, 0][..];
        let patch = read_patch(&mut input).expect("a patch");
        assert!(patch.added.is_empty() && patch.removed.is_empty());
        assert_eq!(delimit(&[0, 0]), Some(vec![0, 0]));
    }
}
