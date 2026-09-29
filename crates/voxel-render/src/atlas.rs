//! Texture atlas packing: where each sprite lands and how big the atlas gets.
//!
//! This is the reference `Stitcher`. Every sprite UV in the finished client
//! derives from these integers, so the layout is pinned before any texture is
//! uploaded. No GPU is involved.

use voxel_core::identifier::Identifier;
use voxel_core::math::{clamp_i32, smallest_encompassing_power_of_two};

/// One sprite offered to the packer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteEntry {
    /// Sprite width in texels, before padding.
    pub width: i32,
    /// Sprite height in texels, before padding.
    pub height: i32,
    /// Sprite name, which breaks ties in the packing order.
    pub name: Identifier,
}

/// Where one sprite ended up on the atlas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpritePlacement {
    /// The sprite's name.
    pub name: Identifier,
    /// Left edge of the padded slot in atlas texels; the sprite's own texels
    /// start one padding further in.
    pub x: i32,
    /// Top edge of the padded slot in atlas texels, likewise.
    pub y: i32,
    /// Padding the packer reserved around every sprite.
    pub padding: i32,
}

/// The packer refused to place a sprite within the atlas limits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StitchError {
    /// Name of the sprite that would not fit.
    pub name: Identifier,
}

/// Java `Identifier.compareTo`: path first, then namespace.
///
/// Comparing namespace first is the obvious reading and the wrong one; it would
/// reorder equally sized sprites and move every UV that follows them.
fn compare_identifiers(left: &Identifier, right: &Identifier) -> std::cmp::Ordering {
    java_string_cmp(left.path(), right.path())
        .then_with(|| java_string_cmp(left.namespace(), right.namespace()))
}

/// Java `String.compareTo`, which orders by UTF-16 code unit.
fn java_string_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

#[derive(Debug, Clone)]
struct Holder {
    entry: SpriteEntry,
    width: i32,
    height: i32,
}

#[derive(Debug)]
struct Region {
    origin_x: i32,
    origin_y: i32,
    width: i32,
    height: i32,
    sub_slots: Option<Vec<Region>>,
    holder: Option<Holder>,
}

impl Region {
    fn new(origin_x: i32, origin_y: i32, width: i32, height: i32) -> Self {
        Self {
            origin_x,
            origin_y,
            width,
            height,
            sub_slots: None,
            holder: None,
        }
    }

    /// Place a sprite here or in one of the slots this region splits into.
    ///
    /// The split happens once, on first use, and the reference chooses which
    /// way to cut by comparing the longer leftover edge against the longer
    /// original edge.
    fn add(&mut self, holder: &Holder) -> bool {
        if self.holder.is_some() {
            return false;
        }
        let texture_width = holder.width;
        let texture_height = holder.height;
        if texture_width > self.width || texture_height > self.height {
            return false;
        }
        if texture_width == self.width && texture_height == self.height {
            self.holder = Some(holder.clone());
            return true;
        }
        if self.sub_slots.is_none() {
            let mut slots = Vec::with_capacity(1);
            slots.push(Region::new(
                self.origin_x,
                self.origin_y,
                texture_width,
                texture_height,
            ));
            let spare_width = self.width - texture_width;
            let spare_height = self.height - texture_height;
            if spare_height > 0 && spare_width > 0 {
                let right = self.height.max(spare_width);
                let bottom = self.width.max(spare_height);
                if right >= bottom {
                    slots.push(Region::new(
                        self.origin_x,
                        self.origin_y + texture_height,
                        texture_width,
                        spare_height,
                    ));
                    slots.push(Region::new(
                        self.origin_x + texture_width,
                        self.origin_y,
                        spare_width,
                        self.height,
                    ));
                } else {
                    slots.push(Region::new(
                        self.origin_x + texture_width,
                        self.origin_y,
                        spare_width,
                        texture_height,
                    ));
                    slots.push(Region::new(
                        self.origin_x,
                        self.origin_y + texture_height,
                        self.width,
                        spare_height,
                    ));
                }
            } else if spare_width == 0 {
                slots.push(Region::new(
                    self.origin_x,
                    self.origin_y + texture_height,
                    texture_width,
                    spare_height,
                ));
            } else if spare_height == 0 {
                slots.push(Region::new(
                    self.origin_x + texture_width,
                    self.origin_y,
                    spare_width,
                    texture_height,
                ));
            }
            self.sub_slots = Some(slots);
        }
        let Some(slots) = self.sub_slots.as_mut() else {
            return false;
        };
        slots.iter_mut().any(|slot| slot.add(holder))
    }

    /// Emit placements in the order the reference walks its regions.
    fn walk(&self, padding: i32, out: &mut Vec<SpritePlacement>) {
        if let Some(holder) = &self.holder {
            out.push(SpritePlacement {
                name: holder.entry.name.clone(),
                x: self.origin_x,
                y: self.origin_y,
                padding,
            });
        } else if let Some(slots) = &self.sub_slots {
            for slot in slots {
                slot.walk(padding, out);
            }
        }
    }
}

/// The reference atlas packer.
#[derive(Debug)]
pub struct Stitcher {
    mip_level: i32,
    textures: Vec<Holder>,
    storage: Vec<Region>,
    storage_x: i32,
    storage_y: i32,
    max_width: i32,
    max_height: i32,
    padding: i32,
}

impl Stitcher {
    /// Java `Stitcher(maxWidth, maxHeight, mipLevel, anisotropyBit)`.
    pub fn new(max_width: i32, max_height: i32, mip_level: i32, anisotropy_bit: i32) -> Self {
        Self {
            mip_level,
            textures: Vec::new(),
            storage: Vec::new(),
            storage_x: 0,
            storage_y: 0,
            max_width,
            max_height,
            padding: 1 << mip_level << clamp_i32(anisotropy_bit - 1, 0, 4),
        }
    }

    /// Atlas width after stitching.
    pub fn width(&self) -> i32 {
        self.storage_x
    }

    /// Atlas height after stitching.
    pub fn height(&self) -> i32 {
        self.storage_y
    }

    /// Padding reserved around every sprite.
    pub fn padding(&self) -> i32 {
        self.padding
    }

    /// Java `registerSprite`: record the sprite at its padded, mip-aligned size.
    pub fn register_sprite(&mut self, entry: SpriteEntry) {
        let width = smallest_fitting_min_texel(entry.width + self.padding * 2, self.mip_level);
        let height = smallest_fitting_min_texel(entry.height + self.padding * 2, self.mip_level);
        self.textures.push(Holder {
            entry,
            width,
            height,
        });
    }

    /// Java `stitch`: sort, then place every sprite or fail naming the first
    /// one that would not fit.
    pub fn stitch(&mut self) -> Result<(), StitchError> {
        let mut holders = self.textures.clone();
        holders.sort_by(|left, right| {
            (-left.height)
                .cmp(&-right.height)
                .then_with(|| (-left.width).cmp(&-right.width))
                .then_with(|| compare_identifiers(&left.entry.name, &right.entry.name))
        });
        for holder in &holders {
            if !self.add_to_storage(holder) {
                return Err(StitchError {
                    name: holder.entry.name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Java `gatherSprites`: the placements, in region walk order.
    pub fn placements(&self) -> Vec<SpritePlacement> {
        let mut out = Vec::with_capacity(self.textures.len());
        for region in &self.storage {
            region.walk(self.padding, &mut out);
        }
        out
    }

    fn add_to_storage(&mut self, holder: &Holder) -> bool {
        if self.storage.iter_mut().any(|region| region.add(holder)) {
            return true;
        }
        self.expand(holder)
    }

    /// Grow the atlas by a power of two on whichever axis is behind.
    fn expand(&mut self, holder: &Holder) -> bool {
        let x_current = smallest_encompassing_power_of_two(self.storage_x);
        let y_current = smallest_encompassing_power_of_two(self.storage_y);
        let x_new = smallest_encompassing_power_of_two(self.storage_x + holder.width);
        let y_new = smallest_encompassing_power_of_two(self.storage_y + holder.height);
        let x_can_grow = x_new <= self.max_width;
        let y_can_grow = y_new <= self.max_height;
        if !x_can_grow && !y_can_grow {
            return false;
        }
        let x_will_grow = x_can_grow && x_current != x_new;
        let y_will_grow = y_can_grow && y_current != y_new;
        let grow_on_x = if x_will_grow ^ y_will_grow {
            x_will_grow
        } else {
            x_can_grow && x_current <= y_current
        };

        let mut slot = if grow_on_x {
            if self.storage_y == 0 {
                self.storage_y = y_new;
            }
            let region = Region::new(self.storage_x, 0, x_new - self.storage_x, self.storage_y);
            self.storage_x = x_new;
            region
        } else {
            let region = Region::new(0, self.storage_y, self.storage_x, y_new - self.storage_y);
            self.storage_y = y_new;
            region
        };
        slot.add(holder);
        self.storage.push(slot);
        true
    }
}

/// Java `smallestFittingMinTexel`: round up to a whole mip-level texel.
fn smallest_fitting_min_texel(input: i32, max_mip_level: i32) -> i32 {
    let rounded = if input & ((1 << max_mip_level) - 1) == 0 {
        0
    } else {
        1
    };
    ((input >> max_mip_level) + rounded) << max_mip_level
}

/// A stitched sprite's place in the finished atlas.
///
/// The packer hands back the padded slot's origin, not the sprite's own, so
/// the padding has to be added back before anything is written or any UV is
/// taken. Getting this wrong shifts every texture by one texel, which is easy
/// to miss because the shift is the same for the write and for the UV.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasSprite {
    /// The sprite's name.
    pub name: Identifier,
    /// Left edge of the padded slot in atlas texels.
    pub x: i32,
    /// Top edge of the padded slot in atlas texels.
    pub y: i32,
    /// Padding reserved on each side.
    pub padding: i32,
    /// Sprite width in texels, without the padding.
    pub width: i32,
    /// Sprite height in texels, without the padding.
    pub height: i32,
    /// Left edge in atlas coordinates.
    pub u0: f32,
    /// Right edge in atlas coordinates.
    pub u1: f32,
    /// Top edge in atlas coordinates.
    pub v0: f32,
    /// Bottom edge in atlas coordinates.
    pub v1: f32,
}

impl AtlasSprite {
    /// Place one stitched sprite in an atlas of the given size.
    pub fn new(
        placement: &SpritePlacement,
        width: i32,
        height: i32,
        atlas_width: i32,
        atlas_height: i32,
    ) -> Self {
        let padded_x = placement.x + placement.padding;
        let padded_y = placement.y + placement.padding;
        Self {
            name: placement.name.clone(),
            x: placement.x,
            y: placement.y,
            padding: placement.padding,
            width,
            height,
            u0: padded_x as f32 / atlas_width as f32,
            u1: (padded_x + width) as f32 / atlas_width as f32,
            v0: padded_y as f32 / atlas_height as f32,
            v1: (padded_y + height) as f32 / atlas_height as f32,
        }
    }

    /// Interpolate across the sprite, as a model's face UV does.
    ///
    /// The span is taken once and then scaled, so an offset outside zero to
    /// one runs off the sprite rather than being clamped.
    pub fn u(&self, offset: f32) -> f32 {
        let span = self.u1 - self.u0;
        self.u0 + span * offset
    }

    /// The same across the other axis.
    pub fn v(&self, offset: f32) -> f32 {
        let span = self.v1 - self.v0;
        self.v0 + span * offset
    }
}
