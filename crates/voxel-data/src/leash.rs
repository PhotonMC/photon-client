//! Where a leash is tied to an entity: the point on a leashed entity the line starts at
//! (`Leashable.getLeashOffset`), the four points a horse, camel, boat or sniffer is tied at
//! (`getQuadLeashOffsets`), and the four a ghast holds (`getQuadLeashHolderOffsets`). Each type
//! takes its size's share by its own factors, which the reference writes in each class.

use crate::entity_type;

/// `Entity.getBbWidth`, `getBbHeight` and `getEyeHeight` of an entity as it stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
    pub eye_height: f32,
}

/// What a camel's leash offset follows (`Camel.getLeashOffset`): whether it sits, and the ticks
/// since it last changed pose, as the level's game time says; and the partial tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CamelPose {
    pub sitting: bool,
    /// `Camel.getPoseTime`.
    pub pose_time: i64,
    pub partial: f32,
}

/// `Mth.lerp` of floats.
fn lerp(delta: f32, from: f32, to: f32) -> f32 {
    from + delta * (to - from)
}

/// `Camel.getBodyAnchorAnimationYOffset`: where the body is, at the front (the driver's seat, the
/// leash) or the back, as the camel sits down or rises. `height` is the height of the pose it is in.
pub fn camel_anchor_offset(height: f32, baby: bool, pose: CamelPose, front: bool) -> f64 {
    let scale: f32 = if baby { 0.6 } else { 1.0 };
    let age_sit_y: f64 = if baby { 0.09375 } else { 0.375 };
    let mut base = f64::from(height) - age_sit_y;
    let sitting_height_difference = scale * 1.43;
    let vertical_drop = sitting_height_difference - scale * 0.2;
    let bottom_point = sitting_height_difference - vertical_drop;
    let in_transition = pose.pose_time < if pose.sitting { 40 } else { 52 };
    if in_transition {
        let (duration, half_point, flex_offset): (i32, i32, f32) = match (pose.sitting, front) {
            (true, true) => (40, 28, 0.5),
            (true, false) => (40, 28, 0.1),
            (false, true) => (52, 24, 0.6),
            (false, false) => (52, 32, 0.35),
        };
        let pose_time = (pose.pose_time as f32 + pose.partial).clamp(0.0, duration as f32);
        let first_part = pose_time < half_point as f32;
        let part = if first_part {
            pose_time / half_point as f32
        } else {
            (pose_time - half_point as f32) / (duration - half_point) as f32
        };
        let flex_point = sitting_height_difference - flex_offset * vertical_drop;
        base += f64::from(if pose.sitting {
            lerp(
                part,
                if first_part {
                    sitting_height_difference
                } else {
                    flex_point
                },
                if first_part { flex_point } else { bottom_point },
            )
        } else {
            lerp(
                part,
                if first_part {
                    bottom_point - sitting_height_difference
                } else {
                    bottom_point - flex_point
                },
                if first_part {
                    bottom_point - flex_point
                } else {
                    0.0
                },
            )
        });
    }
    if pose.sitting && !in_transition {
        base += f64::from(bottom_point);
    }
    base
}

/// `Leashable.getLeashOffset` of the type with this registry name; `camel` is what a camel's
/// follows (a camel just made has stood since tick zero).
pub fn leash_offset(name: &str, size: Size, baby: bool, camel: CamelPose) -> [f64; 3] {
    let Size {
        width,
        height,
        eye_height: eye,
    } = size;
    // The factors the reference writes as `float` literals multiply as floats.
    let float_factors = |y: f32, z: f32| [0.0, f64::from(y * eye), f64::from(width * z)];
    match name {
        "minecraft:pig" | "minecraft:wolf" | "minecraft:rabbit" | "minecraft:strider" => {
            float_factors(0.6, 0.4)
        }
        "minecraft:fox" => float_factors(0.55, 0.4),
        "minecraft:ocelot" | "minecraft:parrot" => float_factors(0.5, 0.4),
        "minecraft:bee" => float_factors(0.5, 0.2),
        "minecraft:iron_golem" => float_factors(0.875, 0.4),
        "minecraft:snow_golem" => float_factors(0.75, 0.4),
        "minecraft:copper_golem" => [0.0, f64::from(0.75_f32 * eye), 0.0],
        // Written with `double` factors.
        "minecraft:allay" => [0.0, f64::from(eye) * 0.6, f64::from(width) * 0.1],
        "minecraft:llama" | "minecraft:trader_llama" => {
            [0.0, 0.75 * f64::from(eye), f64::from(width) * 0.5]
        }
        "minecraft:happy_ghast" => [0.0; 3],
        "minecraft:sulfur_cube" => [0.0, f64::from(height / 2.0), 0.0],
        "minecraft:camel" | "minecraft:camel_husk" => {
            let scale: f32 = if baby { 0.6 } else { 1.0 };
            [
                0.0,
                camel_anchor_offset(height, baby, camel, true) - f64::from(0.2_f32 * scale),
                f64::from(width * 0.56),
            ]
        }
        _ if is_boat(name) => [
            0.0,
            f64::from(0.88_f32 * height),
            f64::from(0.64_f32 * width),
        ],
        _ => [0.0, f64::from(eye), f64::from(width * 0.4)],
    }
}

fn is_boat(name: &str) -> bool {
    name.ends_with("boat") || name.ends_with("raft")
}

/// `Leashable.createQuadLeashOffsets`.
fn quad(
    size: Size,
    front_offset: f64,
    front_back: f64,
    left_right: f64,
    height: f64,
) -> [[f64; 3]; 4] {
    let width = f64::from(size.width);
    let front = front_offset * width;
    let front_back = front_back * width;
    let left_right = left_right * width;
    let height = height * f64::from(size.height);
    [
        [-left_right, height, front_back + front],
        [-left_right, height, -front_back + front],
        [left_right, height, -front_back + front],
        [left_right, height, front_back + front],
    ]
}

/// `getQuadLeashOffsets` of a type that is tied by four lines.
pub fn quad_leash_offsets(name: &str, size: Size) -> [[f64; 3]; 4] {
    match name {
        "minecraft:horse" | "minecraft:skeleton_horse" => quad(size, 0.04, 0.52, 0.23, 0.87),
        "minecraft:donkey" | "minecraft:mule" | "minecraft:zombie_horse" => {
            quad(size, 0.04, 0.41, 0.18, 0.73)
        }
        "minecraft:camel" | "minecraft:camel_husk" => quad(size, 0.02, 0.48, 0.25, 0.82),
        "minecraft:sniffer" => quad(size, -0.01, 0.63, 0.38, 1.15),
        _ if is_boat(name) => quad(size, 0.0, 0.64, 0.382, 0.88),
        _ => quad(size, 0.0, 0.5, 0.5, 0.5),
    }
}

/// `getQuadLeashHolderOffsets` of a type that holds four lines.
pub fn quad_holder_offsets(name: &str, size: Size) -> [[f64; 3]; 4] {
    match name {
        "minecraft:happy_ghast" => quad(size, -0.03125, 0.4375, 0.46875, 0.03125),
        _ => quad(size, 0.0, 0.5, 0.5, 0.0),
    }
}

/// The leash `entity_type::pick` says the type with this id takes, if any.
pub fn leash_of(type_id: i32) -> Option<entity_type::Leash> {
    entity_type::pick(type_id).and_then(|pick| pick.leash)
}
