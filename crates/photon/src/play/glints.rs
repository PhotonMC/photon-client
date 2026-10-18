//! The item glints of a frame: how they move, and which are drawn with the armor's.

use super::*;
use crate::equipment::item_layers::GlintFrame;
use photon_render::render_types::{ITEM_GLINT_SCALE, ITEM_GLINT_TEXTURE};

impl Play {
    /// How this frame's item glints move, where their texture is and the server's item tags.
    pub(super) fn glint_frame(&self) -> GlintFrame {
        GlintFrame {
            matrix: self.armor.glint_matrix_scaled(ITEM_GLINT_SCALE),
            texture: self.entity_textures.rect(ITEM_GLINT_TEXTURE),
            tags: self
                .session
                .world()
                .block_state_registry()
                .item_tags()
                .clone(),
        }
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
