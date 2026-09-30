//! Animated sprites: they tick with the client and redraw their slots of a
//! copy of the block atlas, which then goes to the GPU.

use voxel_render::resolve::{BuiltAtlas, SpriteAnimation};
use voxel_render::vulkan::AtlasRegion;

pub struct AnimatedAtlas {
    levels: Vec<voxel_render::DecodedImage>,
    animations: Vec<SpriteAnimation>,
    /// Whether each animation changed since its slot was last copied.
    redraw: Vec<bool>,
}

impl AnimatedAtlas {
    pub fn new(atlas: &BuiltAtlas) -> Self {
        Self {
            levels: atlas.levels.clone(),
            animations: atlas.animations.clone(),
            redraw: vec![false; atlas.animations.len()],
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
            for (level, target) in self.levels.iter_mut().enumerate() {
                let (x, y, width, height) = animation.draw(level, target);
                let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                for row in y..y + height {
                    let start = ((row * target.width + x) * 4) as usize;
                    rgba.extend_from_slice(&target.rgba[start..start + width as usize * 4]);
                }
                regions.push(AtlasRegion {
                    level: level as u32,
                    x,
                    y,
                    width,
                    height,
                    rgba,
                });
            }
        }
        regions
    }
}
