use super::*;

impl ClientWorld {
    /// The chest at a block, if it is one the client draws as a block entity.
    pub fn chest_view(&self, x: i32, y: i32, z: i32) -> Option<ChestView> {
        let state = self.block_state_at(x, y, z)?;
        let block = crate::block::state_or_air(state).block();
        let name = block.name;
        if !is_chest_state(state) {
            return None;
        }
        let property = |key| crate::local_player::property(Some(state), key);
        let facing = Direction::from_name(property("facing")?)?;
        let kind = match property("type") {
            Some("left") => 1,
            Some("right") => 2,
            _ => 0,
        };
        let partner = (kind != 0).then(|| {
            // `ChestBlock.getConnectedDirection`: the left half's partner is clockwise of the
            // facing, the right half's counter-clockwise.
            let toward = if kind == 1 {
                facing.clockwise_y()
            } else {
                facing.counter_clockwise_y()
            };
            let (dx, _, dz) = toward.expect("a chest faces sideways").step();
            (x + dx, y, z + dz)
        });
        // The partner has to be the same chest block, the other half, and face the same way.
        let partner = partner.filter(|&(px, py, pz)| {
            self.block_state_at(px, py, pz).is_some_and(|other| {
                let other_block = crate::block::state_or_air(other).block();
                let other_property = |key| crate::local_player::property(Some(other), key);
                other_block.name == name
                    && other_property("facing") == property("facing")
                    && matches!(
                        (kind, other_property("type")),
                        (1, Some("right")) | (2, Some("left"))
                    )
            })
        });
        Some(ChestView {
            name,
            facing,
            kind,
            partner,
        })
    }

    /// How far a chest's lid is open (`ChestBlock.opennessCombiner`): the more open of a double
    /// chest's halves.
    pub fn chest_openness(
        &self,
        pos: (i32, i32, i32),
        partner: Option<(i32, i32, i32)>,
        partial: f32,
    ) -> f32 {
        let own = |pos: (i32, i32, i32)| {
            self.block_animations
                .chests
                .get(&pos)
                .map_or(0.0, |lid| lid.openness(partial))
        };
        match partner {
            Some(other) => own(pos).max(own(other)),
            None => own(pos),
        }
    }

    /// The shulker box at a block.
    pub fn shulker_box_view(&self, x: i32, y: i32, z: i32) -> Option<ShulkerBoxView> {
        let state = self.block_state_at(x, y, z)?;
        if !is_shulker_state(state) {
            return None;
        }
        let name = crate::block::state_or_air(state).block().name;
        // `minecraft:shulker_box` is undyed, the rest are `minecraft:<dye>_shulker_box`.
        let color = name
            .strip_prefix("minecraft:")
            .and_then(|name| name.strip_suffix("_shulker_box"));
        // `ShulkerBoxRenderer.extractRenderState`: a box without a facing faces up.
        let facing = crate::local_player::property(Some(state), "facing")
            .and_then(Direction::from_name)
            .unwrap_or(Direction::Up);
        Some(ShulkerBoxView { color, facing })
    }

    /// How far a shulker box is open (`getProgress`).
    pub fn shulker_box_progress(&self, pos: (i32, i32, i32), partial: f32) -> f32 {
        self.block_animations
            .shulkers
            .get(&pos)
            .map_or(0.0, |lid| lid.progress(partial))
    }

    /// The banner at a block.
    /// `LecternRenderer.extractRenderState`: how far the book on a lectern is turned about the
    /// vertical, in degrees; none where it holds no book.
    pub fn lectern_book_turn(&self, pos: (i32, i32, i32)) -> Option<f32> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_lectern_state(state) {
            return None;
        }
        let property = |key| crate::local_player::property(Some(state), key);
        if property("has_book") != Some("true") {
            return None;
        }
        let facing = property("facing").and_then(Direction::from_name)?;
        Some(facing.clockwise_y().ok()?.to_y_rot())
    }

    /// `DecoratedPotRenderer.extractRenderState`: a pot's facing, its sherds and how far through a
    /// wobble it is at a partial tick.
    pub fn pot_view(&self, pos: (i32, i32, i32), partial: f32) -> Option<PotView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_decorated_pot_state(state) {
            return None;
        }
        let facing = Direction::from_name(crate::local_player::property(Some(state), "facing")?)?;
        let entity = self.block_entity_at(pos.0, pos.1, pos.2);
        let sherds = entity
            .and_then(|entity| entity.tag.as_deref())
            .map(pot_sherds)
            .unwrap_or_default();
        // `WobbleStyle.duration`: seven ticks for the first style, ten for the second.
        let wobble = match entity.and_then(|entity| entity.events().wobble) {
            Some((start, style)) => {
                let duration = if style == 0 { 7.0 } else { 10.0 };
                ((self.game_time() as i64 - start) as f32 + partial) / duration
            }
            None => 0.0,
        };
        Some(PotView {
            facing,
            sherds,
            wobble,
        })
    }

    /// `BrushableBlockRenderer.extractRenderState`: the item a suspicious block holds, and the face
    /// brushed, once it has been brushed.
    pub fn brushable_view(&self, pos: (i32, i32, i32)) -> Option<BrushableView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_brushable_state(state) {
            return None;
        }
        let dusted = crate::local_player::property(Some(state), "dusted")?
            .parse()
            .ok()?;
        let tag = self.block_entity_at(pos.0, pos.1, pos.2)?.tag.as_deref()?;
        let (hit_direction, item) = brushable_tag(tag)?;
        Some(BrushableView {
            hit_direction,
            dusted,
            item,
        })
    }

    /// `ShelfRenderer.extractRenderState`: the items on a shelf and the way it faces.
    pub fn shelf_view(&self, pos: (i32, i32, i32)) -> Option<ShelfView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_shelf_state(state) {
            return None;
        }
        let facing = Direction::from_name(crate::local_player::property(Some(state), "facing")?)?;
        let tag = self.block_entity_at(pos.0, pos.1, pos.2)?.tag.as_deref()?;
        let (align_to_bottom, items) = shelf_tag(tag);
        Some(ShelfView {
            facing,
            align_to_bottom,
            items,
        })
    }

    /// `CampfireRenderer.extractRenderState`: the way a campfire faces and what cooks on it.
    pub fn campfire_view(&self, pos: (i32, i32, i32)) -> Option<CampfireView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_campfire_state(state) {
            return None;
        }
        let facing = Direction::from_name(crate::local_player::property(Some(state), "facing")?)?;
        let tag = self.block_entity_at(pos.0, pos.1, pos.2)?.tag.as_deref()?;
        Some(CampfireView {
            facing,
            items: campfire_tag(tag),
        })
    }

    pub fn banner_view(&self, pos: (i32, i32, i32)) -> Option<BannerView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_banner_state(state) {
            return None;
        }
        let name = crate::block::state_or_air(state).block().name;
        let property = |key| crate::local_player::property(Some(state), key);
        let dye = name
            .strip_prefix("minecraft:")?
            .strip_suffix("_wall_banner")
            .or_else(|| name.strip_prefix("minecraft:")?.strip_suffix("_banner"))?;
        let color = DYES.iter().copied().find(|candidate| *candidate == dye)?;
        let wall = name
            .ends_with("_wall_banner")
            .then(|| property("facing").and_then(Direction::from_name))
            .flatten();
        let patterns = self
            .block_entity_at(pos.0, pos.1, pos.2)
            .and_then(|entity| entity.tag.as_deref())
            .map(banner_patterns)
            .unwrap_or_default();
        Some(BannerView {
            color,
            wall,
            rotation: property("rotation")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            patterns,
        })
    }

    /// The sign or hanging sign at a block.
    pub fn sign_view(&self, pos: (i32, i32, i32)) -> Option<SignView<'_>> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_sign_state(state) {
            return None;
        }
        let name = crate::block::state_or_air(state).block().name;
        let property = |key| crate::local_player::property(Some(state), key);
        Some(SignView {
            block: name,
            hanging: is_hanging_sign_state(state),
            wall: name
                .contains("_wall_")
                .then(|| property("facing").and_then(Direction::from_name))
                .flatten(),
            rotation: property("rotation")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            tag: self
                .block_entity_at(pos.0, pos.1, pos.2)
                .and_then(|entity| entity.tag.as_deref()),
        })
    }

    /// The skull or head at a block.
    pub fn skull_block_view(&self, pos: (i32, i32, i32), partial: f32) -> Option<SkullBlockView> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_skull_state(state) {
            return None;
        }
        let name = crate::block::state_or_air(state).block().name;
        let property = |key| crate::local_player::property(Some(state), key);
        let wall = name
            .contains("_wall_")
            .then(|| property("facing").and_then(Direction::from_name))
            .flatten();
        Some(SkullBlockView {
            name,
            wall,
            rotation: property("rotation")
                .and_then(|value| value.parse().ok())
                .unwrap_or(0),
            animation: self
                .block_animations
                .skulls
                .get(&pos)
                .map_or(0.0, |skull| skull.animation(partial)),
        })
    }

    /// `AbstractSkullBlock.getTicker`: the powered dragon and piglin heads count their ticks.
    pub(crate) fn tick_skulls(&mut self) {
        let mut animated = Vec::new();
        for chunk in self.chunks.values() {
            for &pos in &chunk.chests {
                let Some(state) = self.block_state_at(pos.0, pos.1, pos.2) else {
                    continue;
                };
                let name = crate::block::state_or_air(state).block().name;
                if is_skull_state(state)
                    && [
                        "dragon_head",
                        "dragon_wall_head",
                        "piglin_head",
                        "piglin_wall_head",
                    ]
                    .iter()
                    .any(|head| name.strip_prefix("minecraft:") == Some(head))
                {
                    let powered =
                        crate::local_player::property(Some(state), "powered") == Some("true");
                    animated.push((pos, powered));
                }
            }
        }
        for (pos, powered) in animated {
            self.block_animations
                .skulls
                .entry(pos)
                .or_default()
                .tick(powered);
        }
    }

    /// Whether a block is a bell, and how it is shaking: the ticks since the blow with the part
    /// of a tick that has passed, and the direction it was struck from while it still swings
    /// (`BellRenderer.extractRenderState`).
    pub fn bell_shake(
        &self,
        pos: (i32, i32, i32),
        partial: f32,
    ) -> Option<(f32, Option<Direction>)> {
        let state = self.block_state_at(pos.0, pos.1, pos.2)?;
        if !is_bell_state(state) {
            return None;
        }
        let bell = self
            .block_animations
            .bells
            .get(&pos)
            .copied()
            .unwrap_or_default();
        Some((
            bell.ticks as f32 + partial,
            bell.shaking.then_some(bell.direction).flatten(),
        ))
    }

    /// `ClientLevel.blockEvent` on a chest, a shulker box or a bell: event 1 opens the lid for a
    /// positive count, shuts it for none (a box opens for one only), or strikes the bell from the
    /// direction the data says.
    pub fn apply_chest_block_event(&mut self, pos: (i32, i32, i32), event: u8, count: u8) -> bool {
        if event != 1 {
            return false;
        }
        if self.chest_view(pos.0, pos.1, pos.2).is_some() {
            self.block_animations
                .chests
                .entry(pos)
                .or_default()
                .should_be_open = count > 0;
            return true;
        }
        if self.shulker_box_view(pos.0, pos.1, pos.2).is_some() {
            self.block_animations
                .shulkers
                .entry(pos)
                .or_default()
                .trigger(count);
            return true;
        }
        if self.bell_shake(pos, 0.0).is_some() {
            self.block_animations
                .bells
                .entry(pos)
                .or_default()
                .ring(count);
            return true;
        }
        false
    }

    /// The chests, double ones once each half, around a block column within a distance in chunks.
    pub fn chest_positions_near(&self, chunk: (i32, i32), radius: i32) -> Vec<(i32, i32, i32)> {
        let mut found = Vec::new();
        for chunk_x in chunk.0 - radius..=chunk.0 + radius {
            for chunk_z in chunk.1 - radius..=chunk.1 + radius {
                let Some(loaded) = self.chunk_for_query(chunk_x, chunk_z) else {
                    continue;
                };
                found.extend(loaded.chests.iter().copied());
            }
        }
        found
    }
}
