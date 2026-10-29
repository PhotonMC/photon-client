use super::super::*;

impl ClientEntity {
    pub(crate) fn is_shulker(&self) -> bool {
        photon_data::entity_type::name(self.type_id) == Some("minecraft:shulker")
    }

    /// `Shulker.sanitizeScale` of `getScale`.
    pub(in super::super) fn shulker_scale(&self) -> f32 {
        self.scale().min(3.0)
    }

    /// Whether this is a slime or a magma cube (`AbstractCubeMob`).
    pub fn is_cube_mob(&self) -> bool {
        crate::metadata::is_cube_mob_type(self.type_id)
    }
}

impl EntityViewMut<'_> {
    /// `Shulker.stopRiding` (client side): it slides from where it stands, and
    /// its body faces south again.
    pub(crate) fn shulker_stops_riding(&mut self) {
        let block = block_of(self.position());
        self.living.shulker.set_old_attach(block);
        self.rotation.y_body_rot = JavaF32::new(0.0_f32);
        self.rotation.old_y_body_rot = JavaF32::new(0.0_f32);
    }

    /// `AbstractCubeMob.onSyncedDataUpdated` for the size: each time the data
    /// says it (the same size too), the body and the entity turn to where the
    /// head looks.
    pub(crate) fn set_cube_size(&mut self, size: i32) {
        self.rotation.y_rot = self.rotation.y_head_rot;
        self.rotation.y_body_rot = self.rotation.y_head_rot;
        self.living.cube_size = Some(size);
    }
}

impl EntityView<'_> {
    /// `Shulker.makeBoundingBox`: the body and the lid as far open as it is,
    /// on the side opposite the one it clings to.
    pub fn shulker_bounding_box(&self) -> Aabb {
        let physical = crate::mobs::shulker::physical_peek(self.living.shulker.current_peek());
        crate::mobs::shulker::progress_delta_aabb(
            self.shulker_scale(),
            self.living.shulker_face ^ 1,
            (-1.0, physical),
            self.position(),
        )
    }

    /// `Shulker.getRenderPosition`: where the slide to its block draws it,
    /// relative to the block.
    pub fn shulker_render_offset(&self, partial_tick: f32) -> Option<[f64; 3]> {
        self.living.shulker.render_offset(
            block_of(self.position()),
            self.shulker_scale(),
            partial_tick,
        )
    }
}

impl EntityViewMut<'_> {
    /// `Shulker.setPos`: unless it rides, it is held to the middle of its
    /// block (`floor(y + 0.5)` vertically); having come to another block after
    /// its first tick it closes its lid, and a shulker that is not riding
    /// slides there from the old one.
    pub(in super::super) fn shulker_set_pos(
        &mut self,
        position: (f64, f64, f64),
        old_block: [i32; 3],
    ) {
        let riding = self.links.vehicle_id.is_some();
        let (x, y, z) = if riding {
            position
        } else {
            (
                position.0.floor() + 0.5,
                (position.1 + 0.5).floor(),
                position.2.floor() + 0.5,
            )
        };
        self.pos.position.x = JavaF64::new(x);
        self.pos.position.y = JavaF64::new(y);
        self.pos.position.z = JavaF64::new(z);
        let block = block_of((x, y, z));
        if self.tick_count != 0 && block != old_block {
            self.living.shulker_raw_peek = 0;
            if !riding && self.living.shulker.start_teleport(old_block, block) {
                self.pos.old_position.x = JavaF64::new(x);
                self.pos.old_position.y = JavaF64::new(y);
                self.pos.old_position.z = JavaF64::new(z);
            }
        }
    }
}

impl EntityView<'_> {
    /// `Shulker.getAttachFace`'s data value (down 0, up 1, north 2, south 3, west 4, east 5).
    pub fn shulker_attach_face(&self) -> i32 {
        self.living.shulker_face
    }

    /// `Shulker.getColor`'s dye id, none for the default.
    pub fn shulker_color(&self) -> Option<u8> {
        self.living.shulker_color
    }

    /// A shulker's lid between two ticks.
    pub fn shulker(&self) -> &crate::mobs::shulker::ShulkerState {
        &self.living.shulker
    }

    /// `SulfurCube.getFuse`.
    pub fn sulfur_fuse(&self) -> i32 {
        self.living.cube.fuse()
    }

    /// `AbstractCubeMob.getSize`.
    pub fn cube_size(&self) -> i32 {
        self.living.cube_size.unwrap_or(1)
    }

    /// A slime's or a magma cube's squish and what it eases to.
    pub fn cube_mob(&self) -> &crate::mobs::cube_mob::CubeMobState {
        &self.living.cube
    }

    /// `GlowSquid.getDarkTicksRemaining`.
    pub fn glow_dark_ticks(&self) -> i32 {
        self.living.glow_dark_ticks
    }
}

impl EntityViewMut<'_> {
    /// `Shulker.tick`'s lid.
    pub(crate) fn tick_shulker(&mut self) {
        if self.is_shulker() {
            let raw = self.living.shulker_raw_peek;
            self.living.shulker.tick(raw);
            self.living.shulker.tick_teleport();
        }
    }

    /// `Shulker.startRiding` (client side): it stops sliding and clings to
    /// the floor of its vehicle.
    pub(crate) fn shulker_starts_riding(&mut self) {
        self.living.shulker.cancel_teleport();
        self.living.shulker_face = 0;
    }

    /// `AbstractCubeMob.tick`'s squish (`decreaseSquish`: a magma cube's target
    /// fades slower).
    pub(crate) fn tick_cube_mob(&mut self) {
        if !self.is_cube_mob() {
            return;
        }
        let magma = photon_data::entity_type::name(self.type_id) == Some("minecraft:magma_cube");
        let on_ground = self.on_ground;
        // `SulfurCube.tick` runs its fuse before `AbstractCubeMob.tick`.
        self.living.cube.tick_fuse();
        self.living
            .cube
            .tick(on_ground, if magma { 0.9 } else { 0.6 });
    }
}
