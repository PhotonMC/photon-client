//! Animated GUI sprites: a sprite whose `.mcmeta` has an `animation` section is stitched as
//! its first frame, and the frame that plays is written into the atlas as the clock moves on
//! (`SpriteContents.AnimationState`, ticked every 50 ms).

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::texture::metadata::AnimationFrames;
use crate::texture::png::DecodedImage;
use crate::vulkan::AtlasRegion;

/// One client tick.
const TICK: Duration = Duration::from_millis(50);
/// The most ticks caught up at once, after a long stall (`Minecraft.MAX_TICKS_PER_UPDATE`).
const MAX_CATCH_UP: u32 = 10;

/// One animated sprite: every frame of its strip and where it plays.
#[derive(Debug)]
struct Animation {
    key: String,
    /// The strip's frames, by strip index.
    frames: Vec<DecodedImage>,
    /// The frames that play: a strip index and a time in ticks.
    playback: Vec<(i32, i32)>,
    interpolate: bool,
    frame: usize,
    sub_frame: i32,
    dirty: bool,
    /// The sprite's corner in the atlas, once it is placed.
    origin: Option<(u32, u32)>,
}

impl Animation {
    /// `AnimationState.tick`.
    fn tick(&mut self) {
        self.sub_frame += 1;
        self.dirty = false;
        let (old_index, time) = self.playback[self.frame];
        if self.sub_frame >= time {
            self.frame = (self.frame + 1) % self.playback.len();
            self.sub_frame = 0;
            self.dirty = self.playback[self.frame].0 != old_index;
        }
    }

    /// The picture now: the current frame, or the blend towards the next one.
    fn image(&self) -> DecodedImage {
        let (index, time) = self.playback[self.frame];
        let current = &self.frames[index as usize];
        if !self.interpolate {
            return current.clone();
        }
        let next = &self.frames[self.playback[(self.frame + 1) % self.playback.len()].0 as usize];
        // The progress goes through the reference's draw as thousandths.
        let progress = ((self.sub_frame as f32 / time as f32) * 1000.0) as i32 as f32 / 1000.0;
        DecodedImage {
            width: current.width,
            height: current.height,
            rgba: current
                .rgba
                .iter()
                .zip(&next.rgba)
                .map(|(&a, &b)| {
                    let (a, b) = (f32::from(a) / 255.0, f32::from(b) / 255.0);
                    ((a * (1.0 - progress) + b * progress) * 255.0).round() as u8
                })
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
struct State {
    animations: Vec<Animation>,
    /// The time the sprites were last brought up to date.
    last: Option<Instant>,
}

/// The animated sprites of an atlas, shared by its clones.
#[derive(Debug, Clone, Default)]
pub(super) struct Animations(Arc<Mutex<State>>);

/// A sprite image read from its file: the picture to stitch, and its animation if it has one.
pub(super) struct Split {
    pub(super) image: DecodedImage,
    animation: Option<Animation>,
}

/// Cut one frame out of a strip: frames run left to right, then down.
fn cut(strip: &DecodedImage, (width, height): (u32, u32), index: u32) -> DecodedImage {
    let per_row = strip.width / width;
    let (left, top) = (index % per_row * width, index / per_row * height);
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let start = (((top + row) * strip.width + left) * 4) as usize;
        rgba.extend_from_slice(&strip.rgba[start..start + width as usize * 4]);
    }
    DecodedImage {
        width,
        height,
        rgba,
    }
}

/// `strip` read with its animation section. A strip the frame size does not divide is left as
/// it is, whole.
pub(super) fn split(key: &str, strip: DecodedImage, section: &AnimationFrames) -> Split {
    let (width, height) = (strip.width as i32, strip.height as i32);
    let (frame_width, frame_height) = section.frame_size(width, height);
    let fits = frame_width > 0
        && frame_height > 0
        && width % frame_width == 0
        && height % frame_height == 0;
    if !fits {
        return Split {
            image: strip,
            animation: None,
        };
    }
    let size = (frame_width as u32, frame_height as u32);
    let count = (width / frame_width) * (height / frame_height);
    let Some(playback) = section.playback(count) else {
        // Still sprites show the list's first frame.
        let first = section.first_frame.clamp(0, count - 1) as u32;
        return Split {
            image: cut(&strip, size, first),
            animation: None,
        };
    };
    let frames: Vec<DecodedImage> = (0..count as u32)
        .map(|index| cut(&strip, size, index))
        .collect();
    Split {
        image: frames[playback[0].0 as usize].clone(),
        animation: Some(Animation {
            key: key.to_owned(),
            frames,
            playback,
            interpolate: section.interpolate,
            frame: 0,
            sub_frame: 0,
            dirty: false,
            origin: None,
        }),
    }
}

impl Animations {
    /// Keep the animation of a split sprite.
    pub(super) fn add(&self, split: &mut Split) {
        if let (Some(animation), Ok(mut state)) = (split.animation.take(), self.0.lock()) {
            state.animations.push(animation);
        }
    }

    /// Note where the sprite `key` was placed.
    pub(super) fn placed(&self, key: &str, origin: (u32, u32)) {
        if let Ok(mut state) = self.0.lock()
            && let Some(animation) = state.animations.iter_mut().find(|found| found.key == key)
        {
            animation.origin = Some(origin);
        }
    }

    /// Bring the sprites up to the clock: the rectangles whose picture changed, written into
    /// `atlas` and returned for the GPU.
    pub(super) fn apply(&self, atlas: &mut DecodedImage) -> Vec<AtlasRegion> {
        let Ok(mut state) = self.0.lock() else {
            return Vec::new();
        };
        if state.animations.is_empty() {
            return Vec::new();
        }
        let now = Instant::now();
        let last = state.last.replace(now).unwrap_or(now);
        let ticks = ((now - last).as_millis() / TICK.as_millis()) as u32;
        if ticks == 0 {
            // The time did not advance a tick: keep the old mark so the remainder adds up.
            state.last = Some(last);
            return Vec::new();
        }
        // Only whole ticks are consumed; the remainder stays on the clock.
        state.last = Some(last + TICK * ticks);
        let mut regions = Vec::new();
        for animation in &mut state.animations {
            let mut drawn = false;
            for _ in 0..ticks.min(MAX_CATCH_UP) {
                animation.tick();
                drawn |= animation.dirty;
            }
            let Some((x, y)) = animation.origin else {
                continue;
            };
            if !(drawn || animation.interpolate) {
                continue;
            }
            let image = animation.image();
            for row in 0..image.height {
                let from = (row * image.width * 4) as usize;
                let to = (((y + row) * atlas.width + x) * 4) as usize;
                let bytes = image.width as usize * 4;
                atlas.rgba[to..to + bytes].copy_from_slice(&image.rgba[from..from + bytes]);
            }
            regions.push(AtlasRegion {
                level: 0,
                x,
                y,
                width: image.width,
                height: image.height,
                rgba: image.rgba,
            });
        }
        regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(colours: &[[u8; 4]]) -> DecodedImage {
        DecodedImage {
            width: 1,
            height: colours.len() as u32,
            rgba: colours.concat(),
        }
    }

    fn section(frame_time: i32) -> AnimationFrames {
        AnimationFrames {
            frame_time,
            ..AnimationFrames::default()
        }
    }

    #[test]
    fn a_strip_stitches_as_its_first_frame() {
        let red = [255, 0, 0, 255];
        let split = split("k", strip(&[red, [0, 0, 255, 255]]), &section(20));
        assert_eq!((split.image.width, split.image.height), (1, 1));
        assert_eq!(split.image.rgba, red);
        assert!(split.animation.is_some());
    }

    #[test]
    fn the_frame_changes_after_its_time() {
        let (red, blue) = ([255, 0, 0, 255], [0, 0, 255, 255]);
        let mut split = split("k", strip(&[red, blue]), &section(2));
        let animation = split.animation.as_mut().unwrap();
        animation.tick();
        assert!(!animation.dirty);
        animation.tick();
        assert!(animation.dirty);
        assert_eq!(animation.image().rgba, blue);
    }

    #[test]
    fn a_single_frame_is_still() {
        let split = split("k", strip(&[[1, 2, 3, 4]]), &section(2));
        assert!(split.animation.is_none());
    }

    #[test]
    fn a_strip_the_frame_size_does_not_divide_stays_whole() {
        let wide = DecodedImage {
            width: 3,
            height: 2,
            rgba: vec![0; 24],
        };
        let section = AnimationFrames {
            frame_width: Some(2),
            frame_height: Some(2),
            ..AnimationFrames::default()
        };
        let split = split("k", wide, &section);
        assert_eq!((split.image.width, split.image.height), (3, 2));
    }
}
