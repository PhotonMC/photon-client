//! The player model, its pose and walk animation, the elytra and the
//! camera's hurt tilt.

use serde_json::{Value, json};

use super::pose::{Fields, entity, fields, row, table};
use crate::bits::{f32_bits, f64_bits};
use crate::line::float_text;

/// What the view is turned by when the camera's entity flinches or is
/// dying: before and after the flinch, across it, from several directions,
/// with the tilt option changed, and while dying.
pub fn camera_hurt() -> Vec<Value> {
    let defaults = fields!(hurt_duration = 10, hurt_dir = 0.0, dying = false, death_time = 0.0);
    let hurt = |label: &str, hurt_time: f64, tilt: f64, set: Fields| {
        let mut row = row(label, defaults, set);
        row["hurt_time"] = json!(f32_bits(hurt_time));
        row["tilt"] = json!(f64_bits(tilt));
        row
    };
    let mut rows = vec![hurt("no flinch", -0.5, 1.0, fields!()), hurt("flinch ending", 0.0, 1.0, fields!())];
    for (index, hurt_time) in [0.5, 2.25, 4.0, 6.5, 9.0, 9.9].into_iter().enumerate() {
        let direction = (index * 57) as f64 - 90.0;
        rows.push(hurt(&format!("flinch {index}"), hurt_time, 1.0, fields!(hurt_dir = direction)));
    }
    rows.extend([
        hurt("weaker tilt", 3.0, 0.4, fields!(hurt_dir = 33.0)),
        hurt("no tilt", 3.0, 0.0, fields!(hurt_dir = 33.0)),
        hurt("longer flinch", 3.0, 1.0, fields!(hurt_duration = 20, hurt_dir = -120.0)),
        hurt("dying", -1.0, 1.0, fields!(dying = true, death_time = 0.5)),
        hurt("dying later", -1.0, 1.0, fields!(dying = true, death_time = 11.25)),
        hurt("dead", -1.0, 1.0, fields!(dying = true, death_time = 33.0)),
        hurt("dying and flinching", 4.0, 1.0, fields!(dying = true, death_time = 2.5, hurt_dir = 75.0)),
    ]);
    vec![entity("camera-hurt", "camera_hurt", rows)]
}

/// The elytra's models (grown and baby) posed by the real
/// `ElytraModel.setupAnim` from the wings' spread, standing and sneaking.
pub fn elytra() -> Vec<Value> {
    let spreads = [
        ("rest", [0.2617994, 0.0, -0.2617994], false),
        ("sneak", [0.6981317, 0.08726646, -0.7853982], true),
        ("glide", [0.34906584, 0.0, -1.5707964], false),
        ("odd", [0.4, -0.1, -0.9], true),
    ];
    let mut rows = Vec::new();
    for layer in ["minecraft:elytra#main", "minecraft:elytra_baby#main"] {
        for (label, spread, crouching) in spreads {
            rows.push(json!({
                "label": format!("{layer}-{label}"), "layer": layer, "crouching": crouching,
                "rot_x": f32_bits(spread[0]), "rot_y": f32_bits(spread[1]), "rot_z": f32_bits(spread[2]),
            }));
        }
    }
    vec![entity("elytra-model", "elytra_model", rows)]
}

fn avatar(label: &str, set: Fields, heading: Option<f64>) -> Value {
    let defaults = fields!(
        body_rot = 0.0,
        x_rot = 0.0,
        swim_amount = 0.0,
        in_water = false,
        visually_swimming = false,
        crouching = false,
        death_time = 0.0,
        fall_flying = false,
        spin_attack = false,
        fall_flying_ticks = 0.0,
        age_in_ticks = 0.0,
        feet_x = 10.5,
        feet_y = 64.0,
        feet_z = -3.5
    );
    let mut row = row(label, defaults, set);
    row["apply_flying_y_rot"] = json!(heading.is_some());
    row["flying_y_rot"] = json!(f32_bits(heading.unwrap_or(0.0)));
    row
}

/// Where the real `AvatarRenderer` puts a player: standing, sneaking,
/// turned, dying, swimming (in water and crawling), gliding (new, long,
/// past the full tip, with and without a heading, spinning) and
/// combinations that prefer one over another.
pub fn avatar_pose() -> Vec<Value> {
    let rows = vec![
        avatar("standing", fields!(), None),
        avatar("turned", fields!(body_rot = 123.5), None),
        avatar("negative-turn", fields!(body_rot = -77.25), None),
        avatar("sneaking", fields!(crouching = true, body_rot = 40.0), None),
        avatar("far", fields!(feet_x = -100.25, feet_y = 70.5, feet_z = 33.75, body_rot = 10.0), None),
        avatar("dying-early", fields!(death_time = 2.0), None),
        avatar("dying-late", fields!(death_time = 18.0), None),
        avatar("dead", fields!(death_time = 40.0, body_rot = 33.0), None),
        avatar("swim-water", fields!(swim_amount = 0.6, in_water = true, x_rot = -20.0, body_rot = 15.0), None),
        avatar("swim-water-down", fields!(swim_amount = 1.0, in_water = true, x_rot = 45.0), None),
        avatar("swim-air", fields!(swim_amount = 0.4, x_rot = 30.0), None),
        avatar("crawl", fields!(swim_amount = 1.0, visually_swimming = true, x_rot = 10.0, body_rot = -60.0), None),
        avatar(
            "crawl-water",
            fields!(swim_amount = 1.0, visually_swimming = true, in_water = true, x_rot = -45.0),
            None,
        ),
    ];
    let mut rows = rows;
    rows.extend(gliding());
    vec![entity("avatar-pose", "avatar_pose", rows)]
}

fn gliding() -> Vec<Value> {
    vec![
        avatar("glide-new", fields!(fall_flying = true, fall_flying_ticks = 0.0, x_rot = -30.0), None),
        avatar("glide-early", fields!(fall_flying = true, fall_flying_ticks = 2.5, x_rot = -30.0), None),
        avatar("glide-mid", fields!(fall_flying = true, fall_flying_ticks = 6.0, x_rot = 15.0, body_rot = 75.0), None),
        avatar("glide-full", fields!(fall_flying = true, fall_flying_ticks = 10.0, x_rot = 40.0), None),
        avatar("glide-long", fields!(fall_flying = true, fall_flying_ticks = 300.0, x_rot = -70.0), None),
        avatar(
            "glide-heading-left",
            fields!(fall_flying = true, fall_flying_ticks = 12.0, x_rot = -10.0, body_rot = 20.0),
            Some(0.7),
        ),
        avatar("glide-heading-right", fields!(fall_flying = true, fall_flying_ticks = 12.0, x_rot = -10.0), Some(-1.2)),
        avatar(
            "glide-heading-partial",
            fields!(fall_flying = true, fall_flying_ticks = 4.0, x_rot = 25.0, body_rot = -30.0),
            Some(2.9),
        ),
        avatar(
            "glide-spinning",
            fields!(
                fall_flying = true,
                spin_attack = true,
                fall_flying_ticks = 12.0,
                x_rot = -10.0,
                age_in_ticks = 7.5
            ),
            None,
        ),
        avatar(
            "glide-and-swim",
            fields!(fall_flying = true, swim_amount = 0.8, in_water = true, fall_flying_ticks = 12.0, x_rot = -10.0),
            None,
        ),
        avatar(
            "glide-sneaking",
            fields!(fall_flying = true, crouching = true, fall_flying_ticks = 12.0, x_rot = -10.0),
            None,
        ),
        avatar(
            "glide-dying",
            fields!(fall_flying = true, death_time = 10.0, fall_flying_ticks = 12.0, x_rot = -10.0, body_rot = 5.0),
            None,
        ),
        avatar("spin-alone", fields!(spin_attack = true, x_rot = -20.0, age_in_ticks = 9.0), None),
        avatar("spin-dying", fields!(spin_attack = true, death_time = 5.0, age_in_ticks = 9.0), None),
    ]
}

/// A living entity's pose as `LivingEntityRenderer.submit` builds it: the
/// facing, the scale and the fall of a dying one (`deathTime` with its part
/// of a tick, never below one) which tips over by its sideways flip.
pub fn living_pose() -> Vec<Value> {
    let mut rows = Vec::new();
    for death_time in [0.0, 1.0, 1.25, 2.0, 5.5, 10.25, 15.0, 20.0, 20.9, 25.5] {
        for body_rot in [0.0, 135.5] {
            rows.push(json!({
                "label": format!("death-{}-facing-{}", float_text(death_time), float_text(body_rot)),
                "scale": f32_bits(1.0), "body_rot": f32_bits(body_rot), "death_time": f32_bits(death_time),
            }));
        }
    }
    // A spider falls over by half a turn (`getFlipDegrees`).
    for death_time in [1.0, 5.5, 15.0, 25.5] {
        rows.push(json!({
            "label": format!("spider-death-{}", float_text(death_time)), "scale": f32_bits(1.0),
            "body_rot": f32_bits(35.5), "death_time": f32_bits(death_time), "flip_degrees": f32_bits(180.0),
        }));
    }
    rows.push(json!({
        "label": "scaled", "scale": f32_bits(0.5), "body_rot": f32_bits(-70.0), "death_time": f32_bits(8.5),
    }));
    vec![entity("living-pose", "living_pose", rows)]
}

/// Every model layer the game builds (`LayerDefinitions.createRoots`): its
/// texture size and the tree of its parts. The reference answers one row
/// for each layer, by name.
pub fn model_layer() -> Vec<Value> {
    vec![table("model-layers", "model_layers")]
}

/// The player model, posed by the reference's `setupAnim` and drawn by its
/// part renderer: both arm widths; standing, walking, crouching, looking
/// about, swinging either arm, swimming through each stroke phase, and a
/// spectator's head alone.
pub fn player_model() -> Vec<Value> {
    let defaults = fields!(
        slim = false,
        head_y_rot = 0.0,
        x_rot = 0.0,
        walk_position = 0.0,
        walk_speed = 0.0,
        age_in_ticks = 0.0,
        attack_time = 0.0,
        attack_arm = "right",
        swim_amount = 0.0,
        crouching = false,
        spectator = false
    );
    let poses: [(&str, Fields); 15] = [
        ("standing", fields!()),
        ("standing-slim", fields!(slim = true)),
        ("breathing", fields!(age_in_ticks = 37.25)),
        ("walking", fields!(walk_position = 3.7, walk_speed = 0.83, age_in_ticks = 12.5)),
        ("walking-slim", fields!(slim = true, walk_position = 11.2, walk_speed = 0.5, age_in_ticks = 4.0)),
        ("looking", fields!(head_y_rot = -35.5, x_rot = 48.0, age_in_ticks = 100.75)),
        ("crouch-walk", fields!(crouching = true, walk_position = 6.1, walk_speed = 0.3, x_rot = 20.0)),
        ("swing-early", fields!(attack_time = 0.1, x_rot = -10.0)),
        ("swing-mid", fields!(attack_time = 0.5, age_in_ticks = 3.3)),
        ("swing-late-left", fields!(attack_time = 0.85, attack_arm = "left", slim = true)),
        ("swim-reach", fields!(swim_amount = 1.0, walk_position = 7.0, walk_speed = 0.6, x_rot = 30.0)),
        ("swim-pull", fields!(swim_amount = 0.64, walk_position = 17.5, walk_speed = 0.9)),
        ("swim-return", fields!(swim_amount = 1.0, walk_position = 24.25, walk_speed = 0.4)),
        ("swim-swinging", fields!(swim_amount = 1.0, walk_position = 3.0, attack_time = 0.4)),
        ("spectator", fields!(spectator = true, head_y_rot = 12.0)),
    ];
    let rows = poses.iter().map(|(label, set)| row(label, defaults, set)).collect();
    vec![entity("player-model", "player_model", rows)]
}

fn step(target: f64, factor: f64, scale: f64, stop: bool) -> Value {
    json!({"target": f32_bits(target), "factor": f32_bits(factor), "scale": f32_bits(scale), "stop": stop})
}

fn walk(distance: f64) -> Value {
    step((distance * 4.0).min(1.0), 0.4, 1.0, false)
}

/// The limb animation state: a player walking up to speed and stopping,
/// standing, the position scale a baby has, a stop in the middle of a walk,
/// and reads between two ticks.
pub fn walk_animation() -> Vec<Value> {
    let partials: Vec<String> = [0.0, 0.25, 0.5, 0.75, 1.0].into_iter().map(f32_bits).collect();
    let row = |label: &str, steps: Vec<Value>| json!({"label": label, "steps": steps, "partials": partials});
    let slow = vec![walk(0.03), walk(0.06), walk(0.0), walk(0.09), walk(0.05)];
    let midway = [vec![walk(0.2); 4], vec![step(0.0, 0.4, 1.0, true)], vec![walk(0.2); 3]].concat();
    let rows = vec![
        row("standing", vec![step(0.0, 0.4, 1.0, false); 5]),
        row("walk-up-to-speed", vec![walk(0.2158); 12]),
        row("walk-then-stop", [vec![walk(0.2158); 6], vec![walk(0.0); 8]].concat()),
        row("sprint", vec![walk(0.28); 10]),
        row("slow-steps", slow),
        row("baby-scale", vec![step(1.0, 0.4, 3.0, false); 6]),
        row("stop-midway", midway),
        row("factor-one", vec![step(1.0, 1.0, 1.0, false); 3]),
        row("factor-zero", vec![step(1.0, 0.0, 1.0, false); 3]),
        row("long-run", vec![walk(0.2158); 200]),
    ];
    vec![entity("walk-animation", "walk_animation", rows)]
}
