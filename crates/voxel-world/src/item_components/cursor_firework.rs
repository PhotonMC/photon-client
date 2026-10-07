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
}
