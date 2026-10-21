//! The item glints of a frame: how they move, and which are drawn with the armor's.

use super::*;
use crate::equipment::item_layers::GlintFrame;
use photon_render::render_types::{ITEM_GLINT_SCALE, ITEM_GLINT_TEXTURE};

impl Play {
    /// `GameRenderer`'s options state: the glint's speed and strength as the options hold them.
    pub(super) fn apply_glint_options(&mut self) {
        let options = &self.controls.options;
        self.armor.glint_speed = options.glint_speed;
        self.backend
            .set_glint_strength(options.glint_strength as f32);
    }

    /// The arms of the local player that hold a rod with its bobber out.
    pub(super) fn local_casts(&self) -> [bool; 2] {
        let main = self.item_in_hand.item(photon_game::game_mode::MAIN_HAND);
        self.session.player().map_or([false; 2], |player| {
            crate::objects::fishing_hooks::local_casts(&self.session, player, main)
        })
    }

    /// How this frame's item glints move, where their texture is and the server's item tags.
    pub(super) fn glint_frame(&self) -> GlintFrame {
        self.set_needle_frame();
        GlintFrame {
            matrix: self.armor.glint_matrix_scaled(ITEM_GLINT_SCALE),
            texture: self.entity_textures.rect(ITEM_GLINT_TEXTURE),
            tags: self
                .session
                .world()
                .block_state_registry()
                .item_tags()
                .clone(),
            quick_charge: crate::equipment::crossbow::quick_charge_id(&self.session),
            trim_materials: self.armor.trim_materials(),
            needles: self.armor.needles.clone(),
        }
    }

    /// What the compass needles of this frame read: the level, and the local player as the
    /// holder.
    fn set_needle_frame(&self) {
        let session = &self.session;
        let holder = session.player().map(|player| {
            let position = player.position();
            photon_render::item_model::Holder {
                position: [position.x, position.y, position.z],
                yaw: player.rotation().0,
            }
        });
        let (sun_angle, moon_phase) = self.sky.clock();
        self.armor
            .needles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_frame(crate::equipment::needle::NeedleFrame {
                game_time: session.game_time(),
                dimension: session.dimension().to_owned(),
                respawn: session.level_respawn().cloned(),
                last_death: session.last_death().cloned(),
                holder,
                runs_normally: session.runs_normally(),
                sun_angle,
                moon_phase,
            });
    }
}

/// Set how this frame's item glints move and where their texture is.
pub(super) fn set_item_glints(
    glint: &GlintFrame,
    (world, worn): (&mut WorldItemLayers, &mut [ItemLayerCache; 4]),
    gui: &mut ItemLayerCache,
) {
    world.set_glint(glint);
    gui.set_glint(glint);
    for cache in worn {
        cache.set_glint(glint);
    }
}

/// The items' glints are drawn with the armor's, over the depth the items wrote.
pub(super) fn take_item_glints(
    armor: &mut Armor,
    (world, worn): (&mut WorldItemLayers, &mut [ItemLayerCache; 4]),
) {
    armor.glint.extend(world.take_glint());
    for cache in worn {
        armor.glint.extend(cache.take_glint());
    }
}
