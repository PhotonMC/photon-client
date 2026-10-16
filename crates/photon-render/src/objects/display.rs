//! A display entity (`DisplayRenderer`): the matrix of its transformation, and where its block or
//! item is drawn.

use crate::gui::draw_runs;
use photon_world::objects::display::{Billboard, Transformation};

use crate::joml::{Matrix4, Pose, Quaternion};

/// `Transformation`'s matrix (`compose`): the translation, the left turn, the scale and the right
/// turn, in that order.
pub fn transformation_matrix_of(transformation: &Transformation) -> Matrix4 {
    let quaternion = |[x, y, z, w]: [f32; 4]| Quaternion { x, y, z, w };
    let [tx, ty, tz] = transformation.translation;
    let [sx, sy, sz] = transformation.scale;
    let mut matrix = Matrix4::translation(tx, ty, tz);
    matrix.rotate(&quaternion(transformation.left_rotation));
    matrix.scale(sx, sy, sz);
    matrix.rotate(&quaternion(transformation.right_rotation));
    matrix
}

/// The matrix as the 16 numbers `Matrix4f.get(float[])` gives (column by column).
pub fn transformation_matrix(transformation: &Transformation) -> [f32; 16] {
    let matrix = transformation_matrix_of(transformation);
    let mut out = [0.0; 16];
    for (index, value) in matrix.m.iter().flatten().enumerate() {
        out[index] = *value;
    }
    out
}

/// The turns the camera and the entity have, in degrees: yaw and pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayTurns {
    pub entity: (f32, f32),
    pub camera: (f32, f32),
}

/// `DisplayRenderer.calculateOrientation`: how the display faces, from the entity's turn or the
/// camera's as its billboard constraint says.
pub fn display_orientation(billboard: Billboard, turns: &DisplayTurns) -> Quaternion {
    let to_radians = (std::f64::consts::PI / 180.0) as f32;
    let negative = (-std::f64::consts::PI / 180.0) as f32;
    let (entity_y, entity_x) = turns.entity;
    let (camera_y, camera_x) = turns.camera;
    // `transformYRot` and `transformXRot`.
    let camera_y = camera_y - 180.0;
    let camera_x = -camera_x;
    let (yaw, pitch) = match billboard {
        Billboard::Fixed => (entity_y, entity_x),
        Billboard::Horizontal => (entity_y, camera_x),
        Billboard::Vertical => (camera_y, entity_x),
        Billboard::Center => (camera_y, camera_x),
    };
    Quaternion::rotation_yxz(negative * yaw, to_radians * pitch, 0.0)
}

/// What `DisplayRenderer.submit` leaves the pose at for `submitInner`: oriented, then moved by
/// the transformation at this progress. `base` is where the dispatcher puts the entity.
pub fn display_pose(
    base: &Pose,
    (billboard, transformation): (Billboard, &Transformation),
    turns: &DisplayTurns,
) -> Pose {
    let mut pose = *base;
    pose.rotate(&display_orientation(billboard, turns));
    pose.mul_matrix(&transformation_matrix_of(transformation));
    pose
}

/// `ItemDisplayRenderer.submitInner`: the item is turned to face the other way.
pub fn item_display_pose(pose: &Pose) -> Pose {
    let mut pose = *pose;
    pose.rotate(&Quaternion::rotation_y(std::f32::consts::PI));
    pose
}

/// `DisplayRenderer.getBlockLightLevel` and `getSkyLightLevel` together: the light the server set
/// (its block and sky levels, as `LightCoordsUtil` reads them) in place of the world's, unless it
/// set none (`-1`).
pub fn displayed_light(world_light: i32, brightness_override: i32) -> i32 {
    if brightness_override == photon_world::objects::display::NO_BRIGHTNESS_OVERRIDE {
        return world_light;
    }
    // `LightCoordsUtil.pack(block(packed), sky(packed))`.
    (((brightness_override >> 4) & 15) << 4) | (((brightness_override >> 20) & 15) << 20)
}

/// How `Display.TextDisplay.getAlign` reads the flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Center,
    Left,
    Right,
}

/// `TextDisplay.getAlign`.
pub fn text_align(flags: i8) -> TextAlign {
    if flags & 8 != 0 {
        TextAlign::Left
    } else if flags & 16 != 0 {
        TextAlign::Right
    } else {
        TextAlign::Center
    }
}

/// What a text display's `submitInner` submits: the pose its background and lines are drawn in,
/// the background box (its colour; none when clear), and each line's place and how it is drawn.
#[derive(Debug, Clone, PartialEq)]
pub struct TextSubmit {
    pub pose: Matrix4,
    /// The background's four corners in the pose's units and its colour, if it has one.
    pub background: Option<([[f32; 3]; 4], i32)>,
    pub see_through: bool,
    pub shadow: bool,
    /// The lines' left edges and tops.
    pub lines: Vec<(f32, f32)>,
    /// `textOpacity << 24 | 0xFFFFFF`.
    pub color: i32,
}

/// `TextDisplayRenderer.submitInner`: `pose` is what `submit` leaves (oriented and moved by the
/// transformation), `line_widths` the width of each line the text split into, `opacity` the
/// text opacity at this progress (taken as a byte), `background` the colour the background
/// interpolator gives and `default_background_opacity` what the default background is drawn with.
pub fn text_submit(
    pose: &Pose,
    flags: i8,
    (opacity, background): (i32, i32),
    (line_widths, default_background_opacity): (&[i32], f32),
) -> TextSubmit {
    let see_through = flags & 2 != 0;
    let use_default_background = flags & 4 != 0;
    let shadow = flags & 1 != 0;
    let align = text_align(flags);
    let text_opacity = opacity as i8;
    let background = if use_default_background {
        ((default_background_opacity * 255.0) as i32) << 24
    } else {
        background
    };
    let mut matrix = pose.pose;
    matrix.rotate_y(std::f32::consts::PI);
    matrix.scale(-0.025, -0.025, -0.025);
    let width = line_widths.iter().copied().max().unwrap_or(0);
    let line_height = 9 + 1;
    let height = line_widths.len() as i32 * line_height - 1;
    matrix.translate(1.0 - width as f32 / 2.0, -(height as f32), 0.0);
    let corners = |z: f32| {
        [
            [-1.0, -1.0, z],
            [-1.0, height as f32, z],
            [width as f32, height as f32, z],
            [width as f32, -1.0, z],
        ]
    };
    let mut y = 0.0_f32;
    let lines = line_widths
        .iter()
        .map(|line| {
            let offset = match align {
                TextAlign::Left => 0.0,
                TextAlign::Right => (width - line) as f32,
                TextAlign::Center => width as f32 / 2.0 - *line as f32 / 2.0,
            };
            let place = (offset, y);
            y += line_height as f32;
            place
        })
        .collect();
    TextSubmit {
        pose: matrix,
        background: (background != 0).then(|| (corners(-0.01), background)),
        see_through,
        shadow,
        lines,
        color: (i32::from(text_opacity) << 24) | 0x00FF_FFFF,
    }
}

/// A text display's triangles in the pose's world: the background box a hundredth behind the
/// lines, then each line's glyphs (`Font.prepareText`), shadowed when the flag says, lit by `light`.
pub fn text_vertices(
    submit: &TextSubmit,
    lines: &[Vec<photon_game::text::Run>],
    (font, atlas): (&crate::gui::font::Font, &crate::gui::GuiAtlas),
    light: i32,
) -> Vec<crate::scene::Vertex> {
    use crate::scene::Vertex;
    let light = crate::block::light::light_uv(light);
    let place = |x: f32, y: f32, z: f32, uv: [f32; 2], tint: [f32; 4]| {
        let position = submit.pose.transform_position(x, y, z);
        Vertex {
            position,
            uv,
            shade: 1.0,
            tint,
            light,
        }
    };
    let mut vertices = Vec::new();
    let mut push = |corners: [Vertex; 4]| {
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(corners[index]);
        }
    };
    if let Some((corners, color)) = submit.background {
        let tint = [16, 8, 0, 24].map(|shift| ((color as u32 >> shift) & 0xFF) as f32 / 255.0);
        // No texel: a negative coordinate tells the shader.
        let none = [-1.0, -1.0];
        push(corners.map(|[x, y, z]| place(x, y, z, none, tint)));
    }
    for (runs, (x, y)) in lines.iter().zip(&submit.lines) {
        let quads = crate::gui::frame::glyph_quads(
            atlas,
            font,
            &draw_runs(runs),
            (*x, *y),
            submit.color as u32,
            submit.shadow,
        );
        world_glyphs(&submit.pose, quads, light, &mut vertices);
    }
    vertices
}

/// Glyph quads laid out in a pose's plane (`z` 0) as world triangles, lit by `light` (the
/// lightmap's coordinates).
pub(crate) fn world_glyphs(
    pose: &Matrix4,
    quads: Vec<[crate::gui::frame::GuiVertex; 4]>,
    light: [f32; 2],
    vertices: &mut Vec<crate::scene::Vertex>,
) {
    for corners in quads {
        let corners = corners.map(|corner| crate::scene::Vertex {
            position: pose.transform_position(corner.position[0], corner.position[1], 0.0),
            uv: corner.uv,
            shade: 1.0,
            tint: corner.colour,
            light,
        });
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(corners[index]);
        }
    }
}
