//! The next frame's content handed over by the caller, one buffer per kind of draw.

use super::*;

/// A kind of draw whose triangles the caller hands over each frame as [`Vertex`] values, and
/// [`VulkanBackend::set_draw`] takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawKind {
    /// The next frame's entity triangles, placed like the
    /// terrain's, with each vertex's normal in its tint.
    Entities,
    /// The next frame's glowing layers, as triangles that sample the
    /// entity textures, each vertex's tint its colour.
    EntityGlow,
    /// The next frame's energy swirls: each vertex carries its
    /// texture's place, the offset and the colour (see [`SWIRL_VERTEX_SHADER`]).
    EntitySwirl,
    /// The next frame's enchantment glints: each vertex's texture coordinate is moved by
    /// the glint's texture matrix and its tint carries the glint texture's place in the packed image
    /// (see [`GLINT_VERTEX_SHADER`]).
    EntityGlint,
    /// The next frame's dragon rays: each vertex's tint its colour.
    DragonRays,
    /// The next frame's end portal and end gateway surfaces: each vertex's first tint component is
    /// how many layers of the surface its block shows (fifteen for a portal, sixteen for a gateway).
    EndPortal,
    /// The next frame's dropped items: triangles placed like the
    /// terrain's, each with its shade and colour, sampling the block atlas.
    GroundItems,
    /// The next frame's entity shadows, as triangles that sample the
    /// entity textures.
    Shadows,
    /// The next frame's experience orbs, as triangles that sample the
    /// entity textures.
    ExperienceOrbs,
    /// The next frame's particles, as triangles that sample the entity textures: the quads
    /// the opaque layer draws, each vertex lit by the lightmap.
    Particles,
    /// The next frame's particles that show the block atlas (flakes of blocks), as triangles
    /// drawn like the dropped items': the tint is each vertex's colour.
    BlockParticles,
}

impl VulkanBackend {
    /// How far through the day the game is (`GlobalSettingsUniform`'s game time), which the end
    /// portal's layers slide by.
    pub fn set_portal_time(&mut self, game_time: u64, partial: f32) {
        self.portal_time = ((game_time % 24_000) as f32 + partial) / 24_000.0;
    }

    /// Set the lightmap's inputs for the frames that follow: the attributes' own, the block
    /// light's factor, and what the player's effects make of the ambient colour, the brightness
    /// and the darkness.
    pub fn set_lightmap(
        &mut self,
        inputs: &crate::sky::LightmapInputs,
        block_factor: f32,
        effects: &crate::sky::LightmapEffects,
    ) {
        let [sky_r, sky_g, sky_b] = inputs.sky_colour;
        let [tint_r, tint_g, tint_b] = inputs.block_tint;
        let [base_r, base_g, base_b] = effects.base;
        self.lightmap[..14].copy_from_slice(&[
            sky_r,
            sky_g,
            sky_b,
            effects.sky_factor,
            tint_r,
            tint_g,
            tint_b,
            block_factor,
            base_r,
            base_g,
            base_b,
            effects.brightness,
            effects.darkness,
            effects.boss_darkening,
        ]);
    }

    /// Choose the level's diffuse lights for the frames that follow
    /// (`Lighting.updateLevel`).
    pub fn set_level_lighting(&mut self, kind: photon_world::CardinalLightType) {
        crate::diffuse::set_level(kind);
        self.lightmap[14] = f32::from(kind == photon_world::CardinalLightType::Nether);
    }

    /// Set the terrain fog for the frames that follow: its colour, and the
    /// environmental start and end and the render distance start and end,
    /// in blocks.
    pub fn set_fog(&mut self, colour: [f32; 4], distances: [f32; 4]) {
        self.fog[..4].copy_from_slice(&colour);
        self.fog[4..].copy_from_slice(&distances);
    }

    /// Hand over the next frame's breaking overlay, placed like the
    /// terrain's.
    pub fn set_breaking(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.breaking.buffer);
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.breaking.buffer = buffer;
        self.breaking.count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's maps: quads over the maps' image, drawn as text is, and the
    /// decorations' quads over the entity textures.
    pub fn set_map_quads(
        &mut self,
        (maps, decorations): (&[Vertex], &[Vertex]),
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.map.buffer);
        self.fill_vertex_buffer(&mut buffer, maps)?;
        self.map.buffer = buffer;
        self.map.count = maps.len() as u32;
        let mut buffer = std::mem::take(&mut self.map_decoration.buffer);
        self.fill_vertex_buffer(&mut buffer, decorations)?;
        self.map_decoration.buffer = buffer;
        self.map_decoration.count = decorations.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's dissolving entities (a dying dragon's body): the
    /// entity textures' triangles, each vertex's shade its alpha, and the offset
    /// from a texel of the texture to the same texel of its mask in the packed image.
    pub fn set_entity_dissolve(
        &mut self,
        vertices: &[Vertex],
        mask_offset: [f32; 2],
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.dissolve.buffer);
        self.fill_vertex_buffer(&mut buffer, vertices)?;
        self.dissolve.buffer = buffer;
        self.dissolve.count = vertices.len() as u32;
        self.dissolve_mask_offset = mask_offset;
        Ok(())
    }

    /// Hand over the next frame's triangles of one kind of draw.
    pub fn set_draw(&mut self, kind: DrawKind, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut list = std::mem::take(self.draw_list_mut(kind));
        let filled = self.fill_vertex_buffer(&mut list.buffer, vertices);
        list.count = vertices.len() as u32;
        *self.draw_list_mut(kind) = list;
        filled
    }

    fn draw_list_mut(&mut self, kind: DrawKind) -> &mut DrawList {
        match kind {
            DrawKind::Entities => &mut self.entity,
            DrawKind::EntityGlow => &mut self.glow,
            DrawKind::EntitySwirl => &mut self.swirl,
            DrawKind::EntityGlint => &mut self.glint,
            DrawKind::DragonRays => &mut self.rays,
            DrawKind::EndPortal => &mut self.portal,
            DrawKind::GroundItems => &mut self.ground,
            DrawKind::Shadows
            | DrawKind::ExperienceOrbs
            | DrawKind::Particles
            | DrawKind::BlockParticles => self.flat_list_mut(kind),
        }
    }

    /// The lists of the shapes drawn flat over the entity textures and the block atlas.
    fn flat_list_mut(&mut self, kind: DrawKind) -> &mut DrawList {
        match kind {
            DrawKind::Shadows => &mut self.shadow,
            DrawKind::ExperienceOrbs => &mut self.orb,
            DrawKind::Particles => &mut self.particle,
            _ => &mut self.block_particle,
        }
    }

    /// Hand over the next frame's name tags: the text seen through blocks,
    /// which is drawn first, and the text depth-tested. Both sample the GUI
    /// atlas the font's images are in.
    pub fn set_name_tags(
        &mut self,
        see_through: &[Vertex],
        normal: &[Vertex],
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.name_tag_see.buffer);
        self.fill_vertex_buffer(&mut buffer, see_through)?;
        self.name_tag_see.buffer = buffer;
        self.name_tag_see.count = see_through.len() as u32;
        let mut buffer = std::mem::take(&mut self.name_tag.buffer);
        self.fill_vertex_buffer(&mut buffer, normal)?;
        self.name_tag.buffer = buffer;
        self.name_tag.count = normal.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's GUI: triangles in GUI units, their runs by
    /// blend, and the window's size in GUI units.
    pub fn set_gui(
        &mut self,
        frame: &crate::gui::GuiFrame,
        size: [f32; 2],
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.gui_buffer);
        let bytes = as_bytes(&frame.vertices);
        self.fill_bytes(&mut buffer, bytes)?;
        self.gui_buffer = buffer;
        if !frame.items.is_empty() {
            let mut items = std::mem::take(&mut self.gui_item_buffer);
            let bytes = as_bytes(&frame.items);
            self.fill_bytes(&mut items, bytes)?;
            self.gui_item_buffer = items;
        }
        self.gui_batches = frame.batches.clone();
        self.gui_size = size;
        Ok(())
    }

    /// Hand over the next frame's block outline strips, placed like the
    /// terrain's.
    pub fn set_outline(
        &mut self,
        vertices: &[crate::block::outline::LineVertex],
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.outline.buffer);
        let bytes = as_bytes(vertices);
        self.fill_bytes(&mut buffer, bytes)?;
        self.outline.buffer = buffer;
        self.outline.count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's first-person hand: triangles in view
    /// space, and the projection they are drawn with.
    pub fn set_hand(
        &mut self,
        vertices: &[Vertex],
        held: &[Vertex],
        projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        let mut buffer = std::mem::take(&mut self.hand.buffer);
        let all: Vec<Vertex> = vertices.iter().chain(held).copied().collect();
        self.fill_vertex_buffer(&mut buffer, &all)?;
        self.hand.buffer = buffer;
        self.hand.count = vertices.len() as u32;
        self.held_count = held.len() as u32;
        self.hand_projection = projection;
        Ok(())
    }

    /// Write vertices into a host buffer, growing it when they do not fit.
    pub(super) fn fill_vertex_buffer(
        &mut self,
        slot: &mut DynamicBuffer,
        vertices: &[Vertex],
    ) -> Result<(), VulkanError> {
        let bytes = as_bytes(vertices);
        self.fill_bytes(slot, bytes)
    }

    /// Write bytes into a dynamic vertex buffer, growing it when they do not
    /// fit.
    pub(super) fn fill_bytes(
        &mut self,
        slot: &mut DynamicBuffer,
        bytes: &[u8],
    ) -> Result<(), VulkanError> {
        // SAFETY: live handles; `fill_dynamic` waits for the frames that may draw the buffer it
        // replaces.
        unsafe { self.fill_dynamic(slot, &[bytes], 1 << 16) }
    }

    /// Hand over the next frame's rain and snow triangles, placed like the
    /// terrain's.
    pub fn set_weather(&mut self, rain: &[Vertex], snow: &[Vertex]) -> Result<(), VulkanError> {
        let mut slot = std::mem::take(&mut self.weather_buffer);
        // SAFETY: live handles; `fill_dynamic` waits for the frames that may draw the buffer it
        // replaces.
        let written =
            unsafe { self.fill_dynamic(&mut slot, &[as_bytes(rain), as_bytes(snow)], 1 << 16) };
        self.weather_buffer = slot;
        written?;
        self.weather_counts = (rain.len() as u32, snow.len() as u32);
        Ok(())
    }

    /// Hand over the next frame's world border overlay triangles, placed
    /// like the terrain's.
    pub fn set_world_border(&mut self, vertices: &[Vertex]) -> Result<(), VulkanError> {
        let mut slot = std::mem::take(&mut self.world_border_buffer);
        // SAFETY: live handles; `fill_dynamic` waits for the frames that may draw the buffer it
        // replaces.
        let written = unsafe { self.fill_dynamic(&mut slot, &[as_bytes(vertices)], 1 << 16) };
        self.world_border_buffer = slot;
        written?;
        self.world_border_count = vertices.len() as u32;
        Ok(())
    }

    /// Hand over the next frame's screen effects, in the hand's view space and drawn with the
    /// hand's projection: the block the eyes are in and the fire sample the block atlas, the water
    /// its own texture.
    pub fn set_screen_effects(
        &mut self,
        (block, water, fire): (&[Vertex], &[Vertex], &[Vertex]),
        projection: [[f32; 4]; 4],
    ) -> Result<(), VulkanError> {
        let mut slot = std::mem::take(&mut self.screen_effect_buffer);
        // SAFETY: live handles; `fill_dynamic` waits for the frames that may draw the buffer it
        // replaces.
        let written = unsafe {
            self.fill_dynamic(
                &mut slot,
                &[as_bytes(block), as_bytes(water), as_bytes(fire)],
                1 << 12,
            )
        };
        self.screen_effect_buffer = slot;
        written?;
        self.screen_effect_counts = [block.len() as u32, water.len() as u32, fire.len() as u32];
        self.screen_effect_projection = projection;
        Ok(())
    }

    /// Replace the clouds' faces, placed from the camera's cell corner.
    pub fn set_cloud_faces(
        &mut self,
        vertices: &[crate::sky::clouds::CloudVertex],
    ) -> Result<(), VulkanError> {
        // SAFETY: the fence wait leaves the previous frame done with the buffer, so it may be
        // replaced and written.
        unsafe {
            self.wait_in_flight()?;
            let bytes = as_bytes(vertices);
            if self
                .cloud_buffer
                .as_ref()
                .is_none_or(|clouds| clouds.host.capacity < bytes.len())
            {
                self.cloud_buffer = None;
                self.cloud_buffer = Some(CloudBuffer {
                    host: HostBuffer::new(self, bytes.len().max(1 << 20))?,
                    vertex_count: 0,
                    push: [0.0; 24],
                    visible: false,
                });
            }
            let clouds = self.cloud_buffer.as_mut().expect("allocated above");
            clouds.host.write(&[bytes])?;
            clouds.vertex_count = vertices.len() as u32;
            Ok(())
        }
    }

    /// Place the clouds for the frames that follow: the camera-centred
    /// view-projection, the camera's cell corner from the camera, where the
    /// cloud fog ends and the cloud colour; or hide them.
    pub fn set_cloud_frame(&mut self, placement: Option<CloudFrame>) {
        let Some(clouds) = self.cloud_buffer.as_mut() else {
            return;
        };
        clouds.visible = placement.is_some();
        if let Some(CloudFrame {
            view_projection,
            offset,
            fog_end: end,
            colour,
        }) = placement
        {
            for (index, value) in clouds.push[..16].iter_mut().enumerate() {
                *value = view_projection[index / 4][index % 4];
            }
            clouds.push[16..19].copy_from_slice(&offset);
            clouds.push[19] = end;
            clouds.push[20..].copy_from_slice(&colour);
        }
    }

    /// Hand over the next frame's sky: triangles in view-rotated space, the
    /// runs they draw in, the rotation-only view-projection, and the fog.
    pub fn set_sky(
        &mut self,
        vertices: &[crate::sky::SkyVertex],
        batches: &[(crate::sky::SkyBlend, u32, u32)],
        view_projection: [[f32; 4]; 4],
        fog_colour: [f32; 4],
        sky_end: f32,
    ) -> Result<(), VulkanError> {
        let mut slot = std::mem::take(&mut self.sky_buffer);
        // SAFETY: live handles; `fill_dynamic` waits for the frames that may draw the buffer it
        // replaces.
        let written = unsafe { self.fill_dynamic(&mut slot, &[as_bytes(vertices)], 1 << 20) };
        self.sky_buffer = slot;
        written?;
        self.pending_sky = Some(PendingSky {
            vertex_count: vertices.len() as u32,
            batches: batches.to_vec(),
            view_projection,
            fog_colour,
            sky_end,
        });
        Ok(())
    }
}
