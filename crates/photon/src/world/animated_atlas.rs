//! Animated sprites: they tick with the client and redraw their slots of the
//! block atlas, which then goes to the GPU. The atlas itself lives only on
//! the GPU; a redrawn slot is built on its own.

use photon_render::resolve::{BuiltAtlas, SpriteAnimation};
use photon_render::vulkan::AtlasRegion;

pub struct AnimatedAtlas {
    /// The size in texels of every mip level.
    level_sizes: Vec<(u32, u32)>,
    animations: Vec<SpriteAnimation>,
    /// Whether each animation changed since its slot was last copied.
    redraw: Vec<bool>,
}

impl AnimatedAtlas {
    /// Take the animations out of a built atlas, leaving it without them.
    pub fn new(atlas: &mut BuiltAtlas) -> Self {
        let animations = std::mem::take(&mut atlas.animations);
        Self {
            level_sizes: atlas
                .levels
                .iter()
                .map(|level| (level.width, level.height))
                .collect(),
            redraw: vec![false; animations.len()],
            animations,
        }
    }

    /// One client tick of every animation.
    pub fn tick(&mut self) {
        for (animation, redraw) in self.animations.iter_mut().zip(&mut self.redraw) {
            animation.tick();
            *redraw |= animation.needs_to_draw();
        }
    }

    /// The slots of every mip level that changed since the last call.
    pub fn changed_regions(&mut self) -> Vec<AtlasRegion> {
        let mut regions = Vec::new();
        for (animation, redraw) in self.animations.iter().zip(&mut self.redraw) {
            if !std::mem::take(redraw) {
                continue;
            }
            for (level, size) in self.level_sizes.iter().enumerate() {
                let (x, y, image) = animation.draw(level, *size);
                regions.push(AtlasRegion {
                    level: level as u32,
                    x,
                    y,
                    width: image.width,
                    height: image.height,
                    rgba: image.rgba,
                });
            }
        }
        regions
    }
}
