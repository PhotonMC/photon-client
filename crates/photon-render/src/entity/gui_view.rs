//! A model in the box of a screen (`InventoryScreen.extractEntityInInventoryFollowsMouse`): the
//! view that places it, lights it by the two inventory lights and turns it to the mouse, and the
//! triangles of the player, its armor and the mobs in it.

use super::player_model::PlayerModel;
use super::player_renderer::{Placement, placement_pose};

/// `InventoryScreen.extractEntityInInventoryFollowsMouse` for any entity: the box `area`
/// (`x0, y0, x1, y1` in GUI units), `size` GUI units to a block, the entity turned towards the
/// mouse, the model lifted so that its middle (`height / 2` plus `offset_y`) sits in the box, lit
/// by the two inventory lights. The entity's own head and body turns come from
/// [`GuiEntityView::head_yaw`], [`GuiEntityView::pitch_degrees`] and
/// [`GuiEntityView::body_rot`].
#[derive(Debug, Clone, Copy)]
pub struct GuiEntityView {
    centre: (f32, f32),
    size: f32,
    lift: f32,
    x_angle: f32,
    y_angle: f32,
}

impl GuiEntityView {
    pub fn new(
        area: (i32, i32, i32, i32),
        size: f32,
        offset_y: f32,
        mouse: (f32, f32),
        height: f32,
    ) -> Self {
        let centre_x = (area.0 + area.2) as f32 / 2.0;
        let centre_y = (area.1 + area.3) as f32 / 2.0;
        Self {
            centre: (centre_x, centre_y),
            size,
            lift: height / 2.0 + offset_y,
            x_angle: ((centre_x - mouse.0) / 40.0).atan(),
            y_angle: ((centre_y - mouse.1) / 40.0).atan(),
        }
    }

    /// `LivingEntityRenderState.yRot`: the head's turn, in degrees.
    pub fn head_yaw(&self) -> f32 {
        self.x_angle * 20.0
    }

    /// `LivingEntityRenderState.xRot`: the head's pitch, in degrees.
    pub fn pitch_degrees(&self) -> f32 {
        -self.y_angle * 20.0
    }

    /// `LivingEntityRenderState.bodyRot`, in degrees.
    pub fn body_rot(&self) -> f32 {
        180.0 + self.x_angle * 20.0
    }

    /// The lift of the model's middle in blocks (`translation.y`).
    pub fn translation_y(&self) -> f32 {
        self.lift
    }

    /// The `rotation` the reference hands the entity renderer, `(x, y, z, w)`: a half turn about
    /// z times the pitch about x.
    pub fn rotation_quaternion(&self) -> [f32; 4] {
        let half = f64::from(self.y_angle * 20.0 * (std::f32::consts::PI / 180.0)) / 2.0;
        let (sin_x, cos_x) = half.sin_cos();
        let (sin_z, cos_z) = (std::f64::consts::FRAC_PI_2).sin_cos();
        [
            (cos_z * sin_x) as f32,
            (sin_z * sin_x) as f32,
            (sin_z * cos_x) as f32,
            (cos_z * cos_x) as f32,
        ]
    }

    /// `rotation`: a half turn about z, then the pitch about x.
    fn turn(&self, [x, y, z]: [f32; 3]) -> [f32; 3] {
        let pitch = self.y_angle * 20.0 * (std::f32::consts::PI / 180.0);
        let (sin, cos) = pitch.sin_cos();
        let (y, z) = (y * cos - z * sin, y * sin + z * cos);
        [-x, -y, z]
    }

    /// A triangle corner in GUI units with depth towards the viewer.
    fn point(&self, position: [f32; 3]) -> [f32; 3] {
        let [x, y, z] = self.turn(position);
        [
            self.centre.0 + self.size * x,
            self.centre.1 + self.size * (y + self.lift),
            -self.size * z,
        ]
    }

    /// The two inventory lights on a face's normal (the view space flips z).
    fn brightness(&self, normal: [f32; 3]) -> f32 {
        let normal = self.turn(normal);
        let view = [normal[0], normal[1], -normal[2]];
        let dot = |light: [f32; 3]| {
            (light[0] * view[0] + light[1] * view[1] + light[2] * view[2]).max(0.0)
        };
        let lights = inventory_lights();
        ((dot(lights[0]) + dot(lights[1])) * 0.6 + 0.4).min(1.0)
    }
}

/// A placed mob's triangles in a screen's box ([`GuiEntityView`]): `vertices` are the six corners
/// of each quad [`crate::entity::mob_model::mob_vertices`] makes for a mob stood at the origin,
/// turned to [`GuiEntityView::body_rot`], each carrying its normal in its tint.
pub fn gui_mob_vertices(
    vertices: &[crate::scene::Vertex],
    view: &GuiEntityView,
) -> Vec<crate::gui::items::GuiItemVertex> {
    vertices
        .iter()
        .map(|vertex| {
            let normal = [vertex.tint[0], vertex.tint[1], vertex.tint[2]];
            let brightness = view.brightness(normal);
            crate::gui::items::GuiItemVertex {
                position: view.point(vertex.position),
                uv: vertex.uv,
                colour: [brightness, brightness, brightness, 1.0],
            }
        })
        .collect()
}

/// `InventoryScreen.extractEntityInInventoryFollowsMouse`: the player's model
/// in the box `area` (`x0, y0, x1, y1` in GUI units), `size` GUI units to a
/// block, turned towards the mouse, as triangles in GUI units with depth
/// towards the viewer, lit by the two inventory lights.
///
/// `state` is what the model is posed from; the head, the body and the pitch
/// come from the mouse.
pub fn gui_player_vertices(
    model: &mut PlayerModel,
    state: &crate::entity::player_model::PlayerRenderState,
    gui_box: GuiBox,
    skin: crate::entity::textures::TextureRect,
) -> Vec<crate::gui::items::GuiItemVertex> {
    let scene = gui_player(model, state, gui_box, skin);
    let pose = placement_pose(&scene.placement);
    let mut vertices = Vec::new();
    for corners in model.quads(&pose) {
        let brightness = scene.view.brightness(corners[0].normal);
        for index in [0, 1, 2, 0, 2, 3] {
            let corner = corners[index];
            vertices.push(crate::gui::items::GuiItemVertex {
                position: scene.view.point(corner.position),
                uv: crate::entity::textures::uv(skin, corner.uv),
                colour: [brightness, brightness, brightness, 1.0],
            });
        }
    }
    vertices
}

/// A screen's box for a model: its area, its size and offset, and the mouse it looks at.
pub type GuiBox = ((i32, i32, i32, i32), f32, f32, (f32, f32));

/// The player of a screen's box: how its model is posed and where it stands, which what it
/// wears is drawn from too.
pub struct GuiPlayer {
    pub view: GuiEntityView,
    pub state: crate::entity::player_model::PlayerRenderState,
    pub placement: Placement,
}

/// Pose `model` for a screen's box (head, body and pitch from the mouse) and place it.
pub fn gui_player(
    model: &mut PlayerModel,
    state: &crate::entity::player_model::PlayerRenderState,
    (area, size, offset_y, mouse): GuiBox,
    skin: crate::entity::textures::TextureRect,
) -> GuiPlayer {
    let height = if state.crouching { 1.5 } else { 1.8 };
    let view = GuiEntityView::new(area, size, offset_y, mouse, height);
    let state = crate::entity::player_model::PlayerRenderState {
        head_y_rot: view.head_yaw(),
        x_rot: view.pitch_degrees(),
        ..*state
    };
    model.setup_anim(&state);
    let placement = Placement {
        feet: [0.0; 3],
        body_rot: view.body_rot(),
        x_rot: state.x_rot,
        swim_amount: 0.0,
        in_water: false,
        visually_swimming: false,
        crouching: state.crouching,
        light: 0,
        skin,
        death_time: 0.0,
        red_overlay: false,
        fall_flying: None,
        auto_spin: None,
        upside_down: None,
        sleeping: None,
    };
    GuiPlayer {
        view,
        state,
        placement,
    }
}

/// Triangles placed at the origin (what a player wears, as [`layer_vertices`] makes them) in a
/// screen's box: each corner carries its normal in its tint and the colour (`0xRRGGBB`, zero for
/// none) after it, lit by the two inventory lights.
pub fn gui_layer_vertices(
    vertices: &[crate::scene::Vertex],
    view: &GuiEntityView,
) -> Vec<crate::gui::items::GuiItemVertex> {
    vertices
        .iter()
        .map(|vertex| {
            let normal = [vertex.tint[0], vertex.tint[1], vertex.tint[2]];
            let brightness = view.brightness(normal);
            let rgb = vertex.tint[3] as u32;
            let channel = |shift: u32| {
                if rgb == 0 {
                    1.0
                } else {
                    ((rgb >> shift) & 0xFF) as f32 / 255.0
                }
            };
            crate::gui::items::GuiItemVertex {
                position: view.point(vertex.position),
                uv: vertex.uv,
                colour: [
                    brightness * channel(16),
                    brightness * channel(8),
                    brightness * channel(0),
                    1.0,
                ],
            }
        })
        .collect()
}

/// `Lighting.Entry.ENTITY_IN_UI`'s two lights.
fn inventory_lights() -> [[f32; 3]; 2] {
    let normalise = |v: [f32; 3]| {
        let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        v.map(|axis| axis / length)
    };
    [normalise([0.2, -1.0, 1.0]), normalise([-0.2, -1.0, 0.0])]
}
