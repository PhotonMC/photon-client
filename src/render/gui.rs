//! The in-game HUD, laid out the way the reference's HUD lays it out.
//!
//! Everything is in GUI units: the window's pixels divided by the GUI scale,
//! which by default is the largest whole number that keeps the GUI at least
//! 320 by 240. Sprites come from the GUI atlas, the reference's
//! `gui/sprites` folder, drawn at the size the HUD asks for.

use std::collections::BTreeMap;
use std::path::Path;

use crate::identifier::Identifier;
use crate::random::RandomSource;
use crate::render::atlas::{AtlasSprite, SpriteEntry, Stitcher};
use crate::render::png::DecodedImage;

/// The HUD sprites this module draws.
const SPRITES: [&str; 19] = [
    "hud/crosshair",
    "hud/hotbar",
    "hud/hotbar_selection",
    "hud/experience_bar_background",
    "hud/experience_bar_progress",
    "hud/armor_empty",
    "hud/armor_half",
    "hud/armor_full",
    "hud/food_empty",
    "hud/food_half",
    "hud/food_full",
    "hud/air",
    "hud/air_bursting",
    "hud/air_empty",
    "hud/heart/container",
    "hud/heart/full",
    "hud/heart/half",
    "hud/heart/hardcore_full",
    "hud/heart/hardcore_half",
];

/// The GUI sprites packed into one texture.
#[derive(Debug, Clone)]
pub struct GuiAtlas {
    pub image: DecodedImage,
    sprites: BTreeMap<String, [f32; 4]>,
}

impl GuiAtlas {
    /// Read the HUD's sprites from an asset folder and pack them.
    pub fn load(assets_root: &Path) -> Option<Self> {
        let mut images = BTreeMap::new();
        let mut stitcher = Stitcher::new(4096, 4096, 0, 0);
        for name in SPRITES {
            let path = assets_root.join(format!("minecraft/textures/gui/sprites/{name}.png"));
            let image = crate::render::png::decode_rgba(&std::fs::read(path).ok()?).ok()?;
            stitcher.register_sprite(SpriteEntry {
                width: image.width as i32,
                height: image.height as i32,
                name: Identifier::parse(&format!("minecraft:{name}")).ok()?,
            });
            images.insert(format!("minecraft:{name}"), image);
        }
        stitcher.stitch().ok()?;
        let width = stitcher.width().max(1) as u32;
        let height = stitcher.height().max(1) as u32;
        let mut atlas = DecodedImage {
            width,
            height,
            rgba: vec![0; (width * height * 4) as usize],
        };
        let mut sprites = BTreeMap::new();
        for placement in stitcher.placements() {
            let key = placement.name.as_string();
            let image = &images[&key];
            let left = (placement.x + placement.padding) as u32;
            let top = (placement.y + placement.padding) as u32;
            for row in 0..image.height {
                let from = (row * image.width * 4) as usize;
                let to = (((top + row) * width + left) * 4) as usize;
                atlas.rgba[to..to + image.width as usize * 4]
                    .copy_from_slice(&image.rgba[from..from + image.width as usize * 4]);
            }
            let sprite = AtlasSprite::new(
                &placement,
                image.width as i32,
                image.height as i32,
                width as i32,
                height as i32,
            );
            sprites.insert(key, [sprite.u0, sprite.v0, sprite.u1, sprite.v1]);
        }
        Some(Self {
            image: atlas,
            sprites,
        })
    }

    fn sprite(&self, name: &str) -> [f32; 4] {
        self.sprites
            .get(&format!("minecraft:{name}"))
            .copied()
            .unwrap_or([0.0; 4])
    }
}

/// One GUI vertex, in GUI units.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GuiVertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub colour: [f32; 4],
}

/// How a run of GUI triangles blends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiBlend {
    /// Source alpha over the destination.
    Translucent,
    /// The crosshair's inversion of what is behind it.
    Invert,
}

/// One frame of GUI: triangles and the runs they blend in.
#[derive(Debug, Default)]
pub struct GuiFrame {
    pub vertices: Vec<GuiVertex>,
    pub batches: Vec<(GuiBlend, u32, u32)>,
}

impl GuiFrame {
    /// `GuiGraphics.blitSprite` of a whole sprite at a size.
    #[allow(clippy::too_many_arguments)]
    fn blit(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        self.blit_part(atlas, blend, name, (w, h), (0, 0), x, y, w, h);
    }

    /// `GuiGraphics.blitSprite` of part of a sprite: the sprite is taken as
    /// `size` units, and the part from `(u, v)` as big as the drawn area.
    #[allow(clippy::too_many_arguments)]
    fn blit_part(
        &mut self,
        atlas: &GuiAtlas,
        blend: GuiBlend,
        name: &str,
        size: (i32, i32),
        (u, v): (i32, i32),
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) {
        if w == 0 || h == 0 {
            return;
        }
        let [u0, v0, u1, v1] = atlas.sprite(name);
        let su = |offset: i32| u0 + (u1 - u0) * offset as f32 / size.0 as f32;
        let sv = |offset: i32| v0 + (v1 - v0) * offset as f32 / size.1 as f32;
        let (left, right) = (su(u), su(u + w));
        let (top, bottom) = (sv(v), sv(v + h));
        let corner = |px: i32, py: i32, uv: [f32; 2]| GuiVertex {
            position: [px as f32, py as f32],
            uv,
            colour: [1.0; 4],
        };
        let corners = [
            corner(x, y, [left, top]),
            corner(x, y + h, [left, bottom]),
            corner(x + w, y + h, [right, bottom]),
            corner(x + w, y, [right, top]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        match self.batches.last_mut() {
            Some((last, _, count)) if *last == blend => *count += 6,
            _ => self.batches.push((blend, first, 6)),
        }
    }
}

/// What the HUD shows, as the client knows it this frame.
#[derive(Debug, Clone, Copy)]
pub struct HudInput {
    pub health: f32,
    pub max_health: f32,
    pub food: i32,
    pub saturation: f32,
    pub experience_progress: f32,
    pub armor: i32,
    pub selected_slot: i32,
    /// Survival or adventure: hearts, food and the experience bar show.
    pub survival: bool,
    pub spectator: bool,
    pub hardcore: bool,
    /// The camera is in the player's eyes: the crosshair shows.
    pub first_person: bool,
}

/// The HUD's own state across frames: its tick count and the health it
/// last showed.
#[derive(Debug)]
pub struct Hud {
    tick_count: i64,
    last_health: i32,
    display_health: i32,
    last_health_time: u128,
    random: RandomSource,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_health: 0,
            display_health: 0,
            last_health_time: 0,
            random: RandomSource::create(0),
        }
    }
}

impl Hud {
    /// `Gui.tick`: the HUD's tick count moves on.
    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    /// `Hud.extractRenderState` for what this client has: the crosshair,
    /// the hotbar, hearts, armour, food and the experience bar.
    pub fn build(
        &mut self,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
    ) -> GuiFrame {
        let mut frame = GuiFrame::default();
        let t = GuiBlend::Translucent;
        // `Hud.extractCrosshair`: in first person, and not for a spectator
        // (who sees it only over something with a menu, not picked yet).
        if input.first_person && !input.spectator {
            frame.blit(
                atlas,
                GuiBlend::Invert,
                "hud/crosshair",
                (width - 15) / 2,
                (height - 15) / 2,
                15,
                15,
            );
        }
        let centre = width / 2;
        if !input.spectator {
            frame.blit(atlas, t, "hud/hotbar", centre - 91, height - 22, 182, 22);
            frame.blit(
                atlas,
                t,
                "hud/hotbar_selection",
                centre - 91 - 1 + input.selected_slot * 20,
                height - 22 - 1,
                24,
                23,
            );
        }
        if input.survival {
            self.player_health(&mut frame, atlas, width, height, input, now_millis);
            // The experience bar: its background, then as much of the
            // progress as the level has.
            let left = (width - 182) / 2;
            let top = height - 24 - 5;
            let progress = (input.experience_progress * 183.0) as i32;
            frame.blit(atlas, t, "hud/experience_bar_background", left, top, 182, 5);
            if progress > 0 {
                frame.blit_part(
                    atlas,
                    t,
                    "hud/experience_bar_progress",
                    (182, 5),
                    (0, 0),
                    left,
                    top,
                    progress,
                    5,
                );
            }
        }
        frame
    }

    fn player_health(
        &mut self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        width: i32,
        height: i32,
        input: &HudInput,
        now_millis: u128,
    ) {
        let t = GuiBlend::Translucent;
        let current = input.health.ceil() as i32;
        if now_millis.saturating_sub(self.last_health_time) > 1000 {
            self.display_health = current;
            self.last_health_time = now_millis;
        }
        self.last_health = current;
        let old_health = self.display_health;
        self.random.set_seed(self.tick_count.wrapping_mul(312_871));
        let x_left = width / 2 - 91;
        let x_right = width / 2 + 91;
        let y_base = height - 39;
        let max_health = input.max_health.max(old_health.max(current) as f32);
        let rows = ((max_health / 2.0 / 10.0).ceil()) as i32;
        let row_height = (10 - (rows - 2)).max(3);

        if input.armor > 0 {
            let y = y_base - (rows - 1) * row_height - 10;
            for i in 0..10 {
                let x = x_left + i * 8;
                let sprite = if i * 2 + 1 < input.armor {
                    "hud/armor_full"
                } else if i * 2 + 1 == input.armor {
                    "hud/armor_half"
                } else {
                    "hud/armor_empty"
                };
                frame.blit(atlas, t, sprite, x, y, 9, 9);
            }
        }

        let containers = (f64::from(max_health) / 2.0).ceil() as i32;
        for index in (0..containers).rev() {
            let row = index / 10;
            let column = index % 10;
            let x = x_left + column * 8;
            let mut y = y_base - row * row_height;
            if current <= 4 {
                y += self.random.next_int_bound(2).unwrap_or(0);
            }
            frame.blit(atlas, t, "hud/heart/container", x, y, 9, 9);
            let halves = index * 2;
            if halves < current {
                let half = halves + 1 == current;
                let sprite = match (input.hardcore, half) {
                    (false, false) => "hud/heart/full",
                    (false, true) => "hud/heart/half",
                    (true, false) => "hud/heart/hardcore_full",
                    (true, true) => "hud/heart/hardcore_half",
                };
                frame.blit(atlas, t, sprite, x, y, 9, 9);
            }
        }

        for i in 0..10 {
            let mut y = y_base;
            if input.saturation <= 0.0 && self.tick_count % i64::from(input.food * 3 + 1) == 0 {
                y += self.random.next_int_bound(3).unwrap_or(1) - 1;
            }
            let x = x_right - i * 8 - 9;
            frame.blit(atlas, t, "hud/food_empty", x, y, 9, 9);
            if i * 2 + 1 < input.food {
                frame.blit(atlas, t, "hud/food_full", x, y, 9, 9);
            }
            if i * 2 + 1 == input.food {
                frame.blit(atlas, t, "hud/food_half", x, y, 9, 9);
            }
        }
    }
}
