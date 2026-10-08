//! The locator bar (`LocatorBar`): a bar where the experience bar is, with a
//! dot for each waypoint the server tracks, as far along it as the waypoint
//! lies off the way the camera looks, and an arrow when it is above or
//! below.

use std::collections::HashMap;
use std::path::Path;

use photon_core::json::{JsonValue, read_object};
use photon_core::math::{mth_atan2, mth_degrees_difference_f32, mth_lerp_int, wrap_degrees_f32};
use photon_game::waypoint::{TrackedWaypoint, WaypointId, WaypointKind, WaypointManager};

use crate::gui::{GuiAtlas, GuiFrame};

const DEGREES_PER_RADIAN: f32 = 180.0 / std::f32::consts::PI;
/// `VISIBLE_DEGREE_RANGE`.
const VISIBLE_DEGREES: f64 = 60.0;

/// `WaypointStyle`: how far a waypoint must be for each of its sprites.
#[derive(Debug, Clone)]
pub struct WaypointStyle {
    near: i32,
    far: i32,
    /// The sprites' names in the GUI atlas, nearest first.
    sprites: Vec<String>,
}

impl WaypointStyle {
    /// `sprite(distance)`.
    fn sprite(&self, distance: f32) -> &str {
        let last = self.sprites.len() - 1;
        if distance < self.near as f32 {
            &self.sprites[0]
        } else if distance >= self.far as f32 {
            &self.sprites[last]
        } else if self.sprites.len() == 1 {
            &self.sprites[0]
        } else if self.sprites.len() == 3 {
            &self.sprites[1]
        } else {
            let alpha = (distance - self.near as f32) / (self.far - self.near) as f32;
            &self.sprites[mth_lerp_int(alpha, 1, last as i32) as usize]
        }
    }
}

/// `WaypointStyleManager`: the styles of `waypoint_style/*.json`, by
/// identifier.
#[derive(Debug, Clone, Default)]
pub struct WaypointStyles {
    styles: HashMap<String, WaypointStyle>,
}

impl WaypointStyles {
    /// The styles of an asset folder.
    pub fn load(assets_root: &Path) -> Self {
        let mut styles = HashMap::new();
        let folder = assets_root.join("minecraft/waypoint_style");
        for entry in std::fs::read_dir(folder).into_iter().flatten().flatten() {
            let path = entry.path();
            let (Some(stem), Some(object)) = (
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_owned),
                read_object(&path),
            ) else {
                continue;
            };
            let Some(JsonValue::Array(sprites)) = object.get_str("sprites") else {
                continue;
            };
            let sprites: Vec<String> = sprites
                .iter()
                .filter_map(|sprite| match sprite {
                    JsonValue::String(units) => {
                        let name = String::from_utf16_lossy(units);
                        let name = name.strip_prefix("minecraft:").unwrap_or(&name);
                        Some(format!("hud/locator_bar_dot/{name}"))
                    }
                    _ => None,
                })
                .collect();
            let (near, far) = (
                object.i32("near_distance").unwrap_or(128),
                object.i32("far_distance").unwrap_or(332),
            );
            if sprites.is_empty() || near <= 0 || near >= far {
                continue;
            }
            styles.insert(
                format!("minecraft:{stem}"),
                WaypointStyle { near, far, sprites },
            );
        }
        Self { styles }
    }

    fn get(&self, id: &str) -> Option<&WaypointStyle> {
        self.styles.get(id)
    }
}

/// The camera the locator bar reads: the direction it looks, where it is
/// and the projection it draws with.
#[derive(Debug, Clone, Copy)]
pub struct LocatorCamera {
    /// `yRot` and `xRot`, in degrees.
    pub y_rot: f32,
    pub x_rot: f32,
    pub position: [f64; 3],
    /// The vertical field of view, in degrees, and the window's size.
    pub fov: f32,
    pub size: (f32, f32),
    pub z_near: f32,
    pub z_far: f32,
}

/// Who looks: the player whose own waypoint is left out, and where.
#[derive(Debug, Clone, Copy)]
pub struct Viewer {
    pub uuid: u128,
    /// The player's feet, which distances are from.
    pub position: [f64; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pitch {
    None,
    Up,
    Down,
}

/// `Vec3.atCenterOf(BlockPos)`.
fn centre_of(block: [i32; 3]) -> [f64; 3] {
    block.map(|coordinate| f64::from(coordinate) + 0.5)
}

impl LocatorCamera {
    /// `Camera.yaw`.
    fn yaw(&self) -> f32 {
        wrap_degrees_f32(self.y_rot)
    }

    /// The angle a point is off the camera's heading, `degreesDifference`
    /// of the camera's yaw and the angle the point lies at.
    fn yaw_to_point(&self, point: [f64; 3]) -> f64 {
        // `subtract(...).rotateClockwise90()`: (x, y, z) becomes (-z, y, x).
        let direction = [
            -(self.position[2] - point[2]),
            self.position[1] - point[1],
            self.position[0] - point[0],
        ];
        let angle = mth_atan2(direction[2], direction[0]) as f32 * DEGREES_PER_RADIAN;
        f64::from(mth_degrees_difference_f32(self.yaw(), angle))
    }

    /// `projectHorizonToScreen`.
    fn horizon(&self) -> f64 {
        if self.x_rot <= -90.0 {
            f64::NEG_INFINITY
        } else if self.x_rot >= 90.0 {
            f64::INFINITY
        } else {
            let degrees = std::f32::consts::PI / 180.0;
            f64::from(self.x_rot * degrees).tan() / f64::from(self.fov / 2.0 * degrees).tan()
        }
    }

    /// `projectPointToScreen`: the point under the view rotation and the
    /// perspective (a reversed depth, as the renderer draws with).
    fn project(&self, point: [f64; 3]) -> [f32; 3] {
        let offset = [
            (point[0] - self.position[0]) as f32,
            (point[1] - self.position[1]) as f32,
            (point[2] - self.position[2]) as f32,
        ];
        // The view rotation is the inverse of the camera's rotation, which
        // is `rotationYXZ(PI - yRot, -xRot, 0)`.
        let degrees = std::f32::consts::PI / 180.0;
        let (half_y, half_x) = (
            (std::f32::consts::PI - self.y_rot * degrees) * 0.5,
            (-self.x_rot * degrees) * 0.5,
        );
        let (sx, cx) = (half_x.sin(), half_x.cos());
        let (sy, cy) = (half_y.sin(), half_y.cos());
        let q = [cy * sx, sy * cx, -(sy * sx), cy * cx];
        // v' = q^-1 v q.
        let (qx, qy, qz, qw) = (-q[0], -q[1], -q[2], q[3]);
        let [vx, vy, vz] = offset;
        let (tx, ty, tz) = (
            2.0 * (qy * vz - qz * vy),
            2.0 * (qz * vx - qx * vz),
            2.0 * (qx * vy - qy * vx),
        );
        let view = [
            vx + qw * tx + (qy * tz - qz * ty),
            vy + qw * ty + (qz * tx - qx * tz),
            vz + qw * tz + (qx * ty - qy * tx),
        ];
        // `Projection.getMatrix`: near and far swapped, depth zero to one.
        let (near, far) = (self.z_far, self.z_near);
        let h = (self.fov * degrees * 0.5).tan();
        let aspect = self.size.0 / self.size.1;
        let m22 = far / (near - far);
        let m32 = far * near / (near - far);
        let w = -view[2];
        let inverse_w = 1.0 / w;
        [
            view[0] / (h * aspect) * inverse_w,
            view[1] / h * inverse_w,
            (m22 * view[2] + m32) * inverse_w,
        ]
    }
}

/// What the bar asks of a waypoint, against the camera and the viewer.
trait Located {
    fn yaw_angle(&self, camera: &LocatorCamera) -> f64;
    fn pitch(&self, camera: &LocatorCamera) -> Pitch;
    fn distance_squared(&self, viewer: &Viewer) -> f64;
}

impl Located for TrackedWaypoint {
    /// `yawAngleToCamera`.
    fn yaw_angle(&self, camera: &LocatorCamera) -> f64 {
        match &self.kind {
            WaypointKind::Empty => f64::NAN,
            WaypointKind::Azimuth(angle) => f64::from(mth_degrees_difference_f32(
                camera.yaw(),
                angle * DEGREES_PER_RADIAN,
            )),
            WaypointKind::Position(block) => camera.yaw_to_point(centre_of(*block)),
            WaypointKind::Chunk([x, z]) => camera.yaw_to_point(centre_of([
                x * 16 + 8,
                camera.position[1] as i32,
                z * 16 + 8,
            ])),
        }
    }

    /// `pitchDirectionToCamera`.
    fn pitch(&self, camera: &LocatorCamera) -> Pitch {
        let from_horizon = |horizon: f64| {
            if horizon < -1.0 {
                Pitch::Down
            } else if horizon > 1.0 {
                Pitch::Up
            } else {
                Pitch::None
            }
        };
        match &self.kind {
            WaypointKind::Empty => Pitch::None,
            WaypointKind::Azimuth(_) | WaypointKind::Chunk(_) => from_horizon(camera.horizon()),
            WaypointKind::Position(block) => {
                let on_screen = camera.project(centre_of(*block));
                let behind = on_screen[2] > 1.0;
                let in_front = if behind { -on_screen[1] } else { on_screen[1] };
                if in_front < -1.0 {
                    Pitch::Down
                } else if in_front > 1.0 || (behind && on_screen[1] > 0.0) {
                    Pitch::Up
                } else if behind && on_screen[1] < 0.0 {
                    Pitch::Down
                } else {
                    Pitch::None
                }
            }
        }
    }

    /// `distanceSquared` from the viewer.
    fn distance_squared(&self, viewer: &Viewer) -> f64 {
        let from = |point: [f64; 3]| {
            (0..3)
                .map(|axis| (viewer.position[axis] - point[axis]).powi(2))
                .sum::<f64>()
        };
        match &self.kind {
            WaypointKind::Position(block) => from(centre_of(*block)),
            WaypointKind::Chunk([x, z]) => from(centre_of([
                x * 16 + 8,
                viewer.position[1].floor() as i32,
                z * 16 + 8,
            ])),
            WaypointKind::Empty | WaypointKind::Azimuth(_) => f64::INFINITY,
        }
    }
}

/// `ARGB.setBrightness`.
fn set_brightness(colour: i32, brightness: f32) -> i32 {
    let (mut red, mut green, mut blue) =
        ((colour >> 16) & 0xFF, (colour >> 8) & 0xFF, colour & 0xFF);
    let alpha = (colour >> 24) & 0xFF;
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let range = (max - min) as f32;
    let saturation = if max == 0 { 0.0 } else { range / max as f32 };
    let pack = |alpha: i32, red: i32, green: i32, blue: i32| {
        (alpha & 0xFF) << 24 | (red & 0xFF) << 16 | (green & 0xFF) << 8 | (blue & 0xFF)
    };
    let round = |value: f32| (value + 0.5).floor() as i32;
    if saturation == 0.0 {
        let grey = round(brightness * 255.0);
        return pack(alpha, grey, grey, grey);
    }
    let constant = |channel: i32| (max - channel) as f32 / range;
    let mut hue = if red == max {
        constant(blue) - constant(green)
    } else if green == max {
        2.0 + constant(red) - constant(blue)
    } else {
        4.0 + constant(green) - constant(red)
    };
    hue /= 6.0;
    if hue < 0.0 {
        hue += 1.0;
    }
    let segment = (hue - hue.floor()) * 6.0;
    let offset = segment - segment.floor();
    let primary = brightness * (1.0 - saturation);
    let secondary = brightness * (1.0 - saturation * offset);
    let tertiary = brightness * (1.0 - saturation * (1.0 - offset));
    (red, green, blue) = match segment as i32 {
        0 => (
            round(brightness * 255.0),
            round(tertiary * 255.0),
            round(primary * 255.0),
        ),
        1 => (
            round(secondary * 255.0),
            round(brightness * 255.0),
            round(primary * 255.0),
        ),
        2 => (
            round(primary * 255.0),
            round(brightness * 255.0),
            round(tertiary * 255.0),
        ),
        3 => (
            round(primary * 255.0),
            round(secondary * 255.0),
            round(brightness * 255.0),
        ),
        4 => (
            round(tertiary * 255.0),
            round(primary * 255.0),
            round(brightness * 255.0),
        ),
        _ => (
            round(brightness * 255.0),
            round(primary * 255.0),
            round(secondary * 255.0),
        ),
    };
    pack(alpha, red, green, blue)
}

/// What the bar draws from: the waypoints, the camera and the viewer.
#[derive(Debug, Clone)]
pub struct LocatorInput {
    pub waypoints: WaypointManager,
    pub camera: LocatorCamera,
    pub viewer: Viewer,
}

/// `LocatorBar.extractBackground`.
pub fn background(frame: &mut GuiFrame, atlas: &GuiAtlas, (width, height): (i32, i32)) {
    frame.sprite(
        atlas,
        "hud/locator_bar_background",
        width / 2 - 91,
        height - 24 - 5,
        182,
        5,
    );
}

/// `LocatorBar.extractRenderState`: a dot for each waypoint, the farthest
/// first, and an arrow under or over it when it is out of the view.
pub fn draw_dots(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    input: &LocatorInput,
    styles: &WaypointStyles,
    (gui_width, gui_height): (i32, i32),
) {
    let (camera, viewer) = (&input.camera, &input.viewer);
    let top = gui_height - 24 - 5;
    let waypoints = input
        .waypoints
        .farthest_first(|waypoint| waypoint.distance_squared(viewer));
    for waypoint in waypoints {
        if waypoint.id == WaypointId::Uuid(viewer.uuid) {
            continue;
        }
        let angle = waypoint.yaw_angle(camera);
        if angle <= -VISIBLE_DEGREES || angle > VISIBLE_DEGREES {
            continue;
        }
        let Some(style) = styles.get(&waypoint.icon.style) else {
            continue;
        };
        let middle = ((gui_width - 9) as f32 / 2.0).ceil() as i32;
        let distance = (waypoint.distance_squared(viewer) as f32).sqrt();
        let colour = match waypoint.icon.colour {
            Some(rgb) => (0xFF << 24) | rgb,
            None => set_brightness(
                0xFF00_0000_u32 as i32 | (waypoint.id.java_hash() & 0x00FF_FFFF),
                0.9,
            ),
        };
        let dot = (angle * 173.0 / 2.0 / 60.0).floor() as i32;
        frame.sprite_tinted(
            atlas,
            style.sprite(distance),
            (middle + dot, top - 2, 9, 9),
            colour as u32,
        );
        match waypoint.pitch(camera) {
            Pitch::None => {}
            Pitch::Down => {
                frame.sprite(
                    atlas,
                    "hud/locator_bar_arrow_down",
                    middle + dot + 1,
                    top + 6,
                    7,
                    5,
                );
            }
            Pitch::Up => {
                frame.sprite(
                    atlas,
                    "hud/locator_bar_arrow_up",
                    middle + dot + 1,
                    top - 6,
                    7,
                    5,
                );
            }
        }
    }
}
