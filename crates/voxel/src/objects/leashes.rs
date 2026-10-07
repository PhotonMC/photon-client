//! The leads (`EntityRenderer.extractRenderState`'s leash states and `LeashFeatureRenderer`): the
//! line from each leashed entity to whoever holds it. A lead whose holder this client does not
//! know is not drawn. Not carried: a holder that glides with an elytra or spins with a trident
//! (`Player.getRopeHoldPosition`'s other branch), the light a burning or glowing entity gives
//! (`getBlockLightLevel`), the camel's sitting size, the passenger offset of a ridden entity.

use voxel_core::math::Vec3;
use voxel_data::leash::{leash_of, leash_offset, quad_holder_offsets, quad_leash_offsets};
use voxel_net::connection::PlaySession;
use voxel_render::block::light::light_uv;
use voxel_render::joml::Pose;
use voxel_render::objects::fishing::y_rot;
use voxel_render::objects::leash::{
    LeashState, PlayerRope, entity_rope_hold_position, knot_rope_hold_position, leash_vertices,
    player_rope_hold_position, strip_triangles,
};
use voxel_render::scene::Vertex;
use voxel_world::ClientEntityPose;
use voxel_world::local_player::LocalPlayer;

use crate::overlays::FrameView;
use crate::view::eye_position;

/// Whoever holds a lead, as the line needs them.
struct Holder {
    /// `getEyePosition`, where the light is read.
    eye: [f64; 3],
    /// `getRopeHoldPosition`.
    rope: [f64; 3],
    /// `getPosition`, and `getPreciseBodyRotation` in radians, for four lines.
    position: [f64; 3],
    body_rot: f32,
    /// The type's name, for the four points a holder of four lines has.
    name: Option<&'static str>,
    size: voxel_data::leash::Size,
}

fn vec(vector: Vec3) -> [f64; 3] {
    [vector.x, vector.y, vector.z]
}

const DEGREES: f32 = (std::f64::consts::PI / 180.0) as f32;

/// The local player as a holder.
fn local_holder(player: &LocalPlayer, partial: f32) -> Holder {
    let position = vec(player.interpolated_position(f64::from(partial)));
    let pose = player.model_pose(partial);
    let (old_body, body) = player.body_rotations();
    let pitch = player.rotation().1;
    let eye = vec(eye_position(player, f64::from(partial)));
    let rope = player_rope_hold_position(
        &PlayerRope {
            position,
            box_height: {
                let bounds = player.bounding_box();
                bounds.max_y - bounds.min_y
            },
            x_rot: pitch,
            old_x_rot: pitch,
            body_rot: body,
            old_body_rot: old_body,
            // The local player's main arm is the right (`ClientInformation`).
            right_arm: true,
            crouching: pose.crouching,
            swimming: pose.pose == ClientEntityPose::Swimming,
        },
        partial,
    );
    Holder {
        eye,
        rope,
        position,
        body_rot: voxel_core::math::mth_lerp_f32(partial, old_body, body),
        name: None,
        size: voxel_data::leash::Size {
            width: 0.6,
            height: 1.8,
            eye_height: player.eye_height(),
        },
    }
}

/// An entity the world holds as a holder.
fn world_holder(entity: &voxel_world::ClientEntity, partial: f32, position: [f64; 3]) -> Holder {
    let name = voxel_data::entity_type::name(entity.type_id);
    let size = entity.leash_size();
    let eye = [
        position[0],
        position[1] + f64::from(size.eye_height),
        position[2],
    ];
    let rope = if entity.is_player() {
        let bounds = entity.bounding_box();
        player_rope_hold_position(
            &PlayerRope {
                position,
                box_height: bounds.max_y - bounds.min_y,
                x_rot: entity.rotation().1,
                old_x_rot: entity.previous_x_rotation(),
                body_rot: entity.body_y_rotation(),
                old_body_rot: entity.previous_body_y_rotation(),
                right_arm: !entity.main_arm_is_left(),
                crouching: entity.is_crouching(),
                swimming: entity.pose() == ClientEntityPose::Swimming,
            },
            partial,
        )
    } else if name == Some("minecraft:leash_knot") {
        knot_rope_hold_position(position)
    } else {
        entity_rope_hold_position(position, size.eye_height)
    };
    Holder {
        eye,
        rope,
        position,
        body_rot: entity.precise_body_rotation(partial),
        name,
        size,
    }
}

/// `BlockPos.containing` of a point and the block and sky light there.
fn light_at(session: &PlaySession, at: [f64; 3]) -> (i32, i32) {
    let [x, y, z] = at.map(|value| value.floor() as i32);
    let world = session.world();
    (
        i32::from(world.block_brightness(x, y, z)),
        i32::from(world.sky_brightness(x, y, z)),
    )
}

/// The leads of the frame, as triangles for the world text pipeline (a flat colour, lit by the
/// lightmap at each vertex).
pub fn leashes(session: &PlaySession, view: FrameView, vertices: &mut Vec<Vertex>) {
    let world = session.world();
    let partial = view.partial as f32;
    let player = session.player();
    let game_time = world.game_time() as i64;
    for entity in world.entities().all() {
        let Some(holder_id) = entity.leash_holder_id() else {
            continue;
        };
        let (Some(leash), Some(name)) = (
            leash_of(entity.type_id),
            voxel_data::entity_type::name(entity.type_id),
        ) else {
            continue;
        };
        let Some(snapshot) = world.entity_snapshot(entity.id(), partial) else {
            continue;
        };
        let holder = match player {
            Some(player) if player.entity_id() == holder_id => local_holder(player, partial),
            _ => {
                let Some(roper) = world.entities().all().find(|roper| roper.id() == holder_id)
                else {
                    continue;
                };
                let Some(roper_snapshot) = world.entity_snapshot(holder_id, partial) else {
                    continue;
                };
                let (x, y, z) = roper_snapshot.position;
                world_holder(roper, partial, [x, y, z])
            }
        };
        let (x, y, z) = snapshot.position;
        let position = [x, y, z];
        let size = entity.leash_size();
        let yaw = entity.precise_body_rotation(partial) * DEGREES;
        let eye = [
            position[0],
            position[1] + f64::from(size.eye_height),
            position[2],
        ];
        let (start_block, start_sky) = light_at(session, eye);
        let (end_block, end_sky) = light_at(session, holder.eye);
        let state = |offset: [f64; 3], end: [f64; 3], slack: bool| LeashState {
            offset,
            start: [
                position[0] + offset[0],
                position[1] + offset[1],
                position[2] + offset[2],
            ],
            end,
            start_block_light: start_block,
            end_block_light: end_block,
            start_sky_light: start_sky,
            end_sky_light: end_sky,
            slack,
        };
        let mut states = Vec::new();
        if leash.quad && holder.name.is_some_and(quad_holder_offsets_of) {
            // Four lines, each from a corner of the leashed entity to a corner of the holder.
            let holder_size = holder.size;
            let tied = quad_leash_offsets(name, size);
            let held = quad_holder_offsets(holder.name.unwrap_or_default(), holder_size);
            let roper_yaw = holder.body_rot * DEGREES;
            for (tied, held) in tied.into_iter().zip(held) {
                let offset = y_rot(tied, -yaw);
                let at = y_rot(held, -roper_yaw);
                let end = [
                    holder.position[0] + at[0],
                    holder.position[1] + at[1],
                    holder.position[2] + at[2],
                ];
                states.push(state(offset, end, false));
            }
        } else {
            let camel = entity.camel_leash_pose(game_time, partial);
            let offset = y_rot(leash_offset(name, size, entity.is_baby(), camel), -yaw);
            states.push(state(offset, holder.rope, true));
        }
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        for state in &states {
            for vertex in strip_triangles(&leash_vertices(&base, state)) {
                let [r, g, b] = vertex.colour.map(|channel| f32::from(channel) / 255.0);
                vertices.push(Vertex {
                    position: vertex.position,
                    uv: [-1.0, -1.0],
                    shade: 1.0,
                    tint: [r, g, b, 1.0],
                    light: light_uv(vertex.light),
                });
            }
        }
    }
}

/// Whether the holder's type holds four lines.
fn quad_holder_offsets_of(name: &str) -> bool {
    voxel_data::entity_type::id(name)
        .and_then(voxel_data::entity_type::pick)
        .is_some_and(|pick| pick.quad_holder)
}
