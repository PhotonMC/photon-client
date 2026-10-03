//! `KeyframeAnimation`: a model's parts posed from an animation definition at a
//! time, each channel adding its interpolated vector to the part's position,
//! rotation or scale (`ModelPart.offsetPos`, `offsetRotation`, `offsetScale`).

use voxel_data::animation::{Definition, Interpolation, Keyframe, Target};

use crate::player_model::Part;

/// `Mth.binarySearch`: the first index in `from..to` for which the condition holds.
fn binary_search(mut from: usize, to: usize, condition: impl Fn(usize) -> bool) -> usize {
    let mut len = to - from;
    while len > 0 {
        let half = len / 2;
        let middle = from + half;
        if condition(middle) {
            len = half;
        } else {
            from = middle + 1;
            len -= half + 1;
        }
    }
    from
}

/// `Mth.catmullrom`.
fn catmull_rom(alpha: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
    0.5 * (2.0 * p1
        + (p2 - p0) * alpha
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * alpha * alpha
        + (3.0 * p1 - p0 - 3.0 * p2 + p3) * alpha * alpha * alpha)
}

/// `Vector3f.lerp` of two points, then `mul` by the target scale.
fn lerp(from: [f32; 3], to: [f32; 3], alpha: f32, scale: f32) -> [f32; 3] {
    std::array::from_fn(|axis| (from[axis] + (to[axis] - from[axis]) * alpha) * scale)
}

/// The vector a channel gives at `seconds` (`KeyframeAnimation.Entry.apply`).
fn sample(keyframes: &[Keyframe], seconds: f32, scale: f32) -> [f32; 3] {
    let previous = binary_search(0, keyframes.len(), |index| seconds <= keyframes[index].time)
        .saturating_sub(1);
    let next = (keyframes.len() - 1).min(previous + 1);
    let (before, after) = (&keyframes[previous], &keyframes[next]);
    let alpha = if next == previous {
        0.0
    } else {
        ((seconds - before.time) / (after.time - before.time)).clamp(0.0, 1.0)
    };
    match after.interpolation {
        Interpolation::Linear => lerp(before.post, after.pre, alpha, scale),
        Interpolation::CatmullRom => {
            let points = [
                keyframes[previous.saturating_sub(1)].post,
                before.post,
                after.post,
                keyframes[(keyframes.len() - 1).min(next + 1)].post,
            ];
            std::array::from_fn(|axis| {
                catmull_rom(
                    alpha,
                    points[0][axis],
                    points[1][axis],
                    points[2][axis],
                    points[3][axis],
                ) * scale
            })
        }
    }
}

/// Where `ModelPart.createPartLookup` finds a name: the root, else the first
/// of the children, then of each child's own descendants in turn.
fn path_to(part: &Part, name: &str) -> Option<Vec<usize>> {
    if let Some(index) = part
        .children()
        .iter()
        .position(|child| child.name() == name)
    {
        return Some(vec![index]);
    }
    part.children()
        .iter()
        .enumerate()
        .find_map(|(index, child)| {
            let mut path = path_to(child, name)?;
            path.insert(0, index);
            Some(path)
        })
}

fn part_at<'a>(root: &'a mut Part, path: &[usize]) -> &'a mut Part {
    let mut part = root;
    for index in path {
        part = part.child_at_mut(*index);
    }
    part
}

/// `KeyframeAnimation.apply(millisSinceStart, targetScale)`: every channel of
/// the definition applied to the model whose root is `root`. A bone the model
/// does not have is skipped (the reference refuses to bake it).
pub fn apply(root: &mut Part, definition: &Definition, millis: i64, target_scale: f32) {
    let seconds = millis as f32 / 1000.0;
    let seconds = if definition.looping {
        seconds % definition.length
    } else {
        seconds
    };
    for bone in definition.bones {
        let path = if bone.name == "root" {
            Vec::new()
        } else if let Some(path) = path_to(root, bone.name) {
            path
        } else {
            continue;
        };
        let part = part_at(root, &path);
        for channel in bone.channels {
            let vector = sample(channel.keyframes, seconds, target_scale);
            match channel.target {
                Target::Position => {
                    part.x += vector[0];
                    part.y += vector[1];
                    part.z += vector[2];
                }
                Target::Rotation => {
                    part.x_rot += vector[0];
                    part.y_rot += vector[1];
                    part.z_rot += vector[2];
                }
                Target::Scale => {
                    part.scale[0] += vector[0];
                    part.scale[1] += vector[1];
                    part.scale[2] += vector[2];
                }
            }
        }
    }
}
