//! The value layouts of the components the first table leaves out.

use super::*;

impl Cursor<'_> {
    /// Component values, group 1.
    pub(super) fn component_more(&mut self, id: i32) -> R {
        match id {
            // Registry holders and plain numbers 26.3 added.
            ids::BLOCK_TRANSFORMER
            | ids::VILLAGER_FOOD
            | ids::PROVIDES_POTTERY_PATTERN
            | ids::CUSHION_COLOR => self.varint().map(|_| ()),
            ids::WAXED => Ok(()),
            // `compostable`: a number that is a constant or a reference.
            ids::COMPOSTABLE => self.resolvable_int(),
            // The fuels: a time or uses, and a speed multiplier.
            ids::COOKING_FUEL | ids::BREWING_FUEL => {
                self.resolvable_int()?;
                self.resolvable_float()
            }
            // `mob_visibility`: the entity types it targets, and the visibility.
            ids::MOB_VISIBILITY => {
                self.holder_set()?;
                self.float()
            }
            // The text of a sign face: four lines, the filtered four if there are any, the dye
            // and the glow flag.
            ids::SIGN_TEXT_FRONT | ids::SIGN_TEXT_BACK => {
                for _ in 0..4 {
                    self.tag()?;
                }
                self.optional(|cursor| {
                    for _ in 0..4 {
                        cursor.tag()?;
                    }
                    Ok(())
                })?;
                self.varint()?;
                self.boolean().map(|_| ())
            }
            ids::DYED_COLOR | ids::MAP_COLOR => self.int(),
            _ => self.component_part1(id),
        }
    }

    /// Component values, group 2.
    fn component_part1(&mut self, id: i32) -> R {
        match id {
            ids::CHARGED_PROJECTILES | ids::BUNDLE_CONTENTS => self.list(Cursor::template),
            // `potion_contents`.
            ids::POTION_CONTENTS => {
                self.optional(|cursor| cursor.varint().map(|_| ()))?;
                self.optional(Cursor::int)?;
                self.list(Cursor::mob_effect_instance)?;
                self.optional(Cursor::string)
            }
            ids::SUSPICIOUS_STEW_EFFECTS => self.list(|cursor| {
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            ids::WRITABLE_BOOK_CONTENT => self.list(|cursor| cursor.filterable(Cursor::string)),
            ids::WRITTEN_BOOK_CONTENT => {
                self.filterable(Cursor::string)?;
                self.string()?;
                self.varint()?;
                self.list(|cursor| cursor.filterable(Cursor::tag))?;
                self.boolean().map(|_| ())
            }
            // `trim`: the material, then the pattern.
            ids::TRIM => {
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
            ids::ENTITY_DATA | ids::BLOCK_ENTITY_DATA => self.typed_data(id),
            _ => self.component_part2(id),
        }
    }

    /// Component values, group 3.
    fn component_part2(&mut self, id: i32) -> R {
        match id {
            ids::BUCKET_ENTITY_DATA => self.compound_tag(),
            ids::INSTRUMENT => self.holder(|cursor| {
                cursor.sound()?;
                cursor.float()?;
                cursor.float()?;
                cursor.tag()
            }),
            ids::PROVIDES_TRIM_MATERIAL => self.holder(|cursor| {
                cursor.string()?;
                cursor.list(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.tag()
            }),
            ids::JUKEBOX_PLAYABLE => self.holder(|cursor| {
                cursor.sound()?;
                cursor.tag()?;
                cursor.float()?;
                cursor.varint().map(|_| ())
            }),
            // `lodestone_tracker`: a dimension and a position, and a flag.
            ids::LODESTONE_TRACKER => {
                self.optional(|cursor| {
                    cursor.identifier()?;
                    cursor.long()
                })?;
                self.boolean().map(|_| ())
            }
            ids::FIREWORK_EXPLOSION => self.firework_explosion(),
            ids::FIREWORKS => {
                self.varint()?;
                self.list(Cursor::firework_explosion)
            }
            _ => self.component_part3(id),
        }
    }

    /// Component values, group 4.
    fn component_part3(&mut self, id: i32) -> R {
        match id {
            ids::PROFILE => {
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
            ids::BANNER_PATTERNS => self.list(|cursor| {
                cursor.holder(|cursor| {
                    cursor.identifier()?;
                    cursor.string()
                })?;
                cursor.varint().map(|_| ())
            }),
            ids::POT_DECORATIONS => self.list(|cursor| cursor.varint().map(|_| ())),
            ids::CONTAINER => self.list(|cursor| cursor.optional(Cursor::template)),
            ids::BLOCK_STATE => self.list(|cursor| {
                cursor.string()?;
                cursor.string()
            }),
            ids::BEES => self.list(|cursor| {
                cursor.varint()?;
                cursor.compound_tag()?;
                cursor.varint()?;
                cursor.varint().map(|_| ())
            }),
            ids::BREAK_SOUND => self.sound(),
            _ => self.component_part4(id),
        }
    }

    /// Component values, group 5.
    fn component_part4(&mut self, id: i32) -> R {
        match id {
            // The painting: a registry id plus one, or the variant itself.
            ids::PAINTING_VARIANT => self.holder(|cursor| {
                cursor.varint()?;
                cursor.varint()?;
                cursor.identifier()?;
                cursor.optional(Cursor::tag)?;
                cursor.optional(Cursor::tag)
            }),
            _ => Err(Malformed),
        }
    }
}
