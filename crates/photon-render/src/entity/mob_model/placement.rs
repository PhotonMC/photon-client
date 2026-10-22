use super::*;

impl MobPlacement {
    /// `CatRenderer.setupRotations`: a cat that lies down tips onto its side
    /// (and further over a sleeping player).
    pub fn cat_lying(&mut self, amount: f32, on_sleeping_player: bool) {
        if amount > 0.0 {
            self.shift = Some([0.4 * amount, 0.15 * amount, 0.1 * amount]);
            // `Mth.rotLerp(amount, 0, 90)`.
            self.roll = Some(amount * 90.0);
            if on_sleeping_player {
                self.roll_shift = Some([0.15 * amount, 0.0, 0.0]);
            }
        }
    }

    /// `FoxRenderer.setupRotations`: a fox that pounces or has its face
    /// planted tips with its pitch.
    pub fn fox_tipping(&mut self, pitch: f32) {
        self.pitch = Some(-pitch);
    }
}

/// Where a mob is and how it is lit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MobPlacement {
    /// The feet, from the mesh origin.
    pub feet: [f32; 3],
    /// The body's facing, in degrees.
    pub body_rot: f32,
    /// The entity's scale (the attribute; one unless the server says other).
    pub scale: f32,
    /// Packed block and sky light at the mob.
    pub light: i32,
    /// `LivingEntityRenderState.deathTime`: zero while it lives.
    pub death_time: f32,
    /// `LivingEntityRenderer.getFlipDegrees`: how far a dead mob falls over.
    pub flip_degrees: f32,
    /// Drawn with the hurt overlay (`hasRedOverlay`).
    pub red_overlay: bool,
    /// `getWhiteOverlayProgress`: how far the mob is flashed white.
    pub white_overlay: f32,
    /// What the renderer's own `scale` hook scales the pose by, after the
    /// flip (a creeper's swelling), or none.
    pub hook_scale: Option<[f32; 3]>,
    /// What the renderer's `scale` hook then moves the pose by (a phantom's lift), or none.
    pub hook_translate: Option<[f32; 3]>,
    /// A roll about z the renderer's own `setupRotations` adds after the
    /// fall of a dead mob, in degrees (the walking iron golem's sway).
    pub roll: Option<f32>,
    /// A shift the renderer's `setupRotations` makes before that roll, and one
    /// after it, along the rolled axes (a cat lying down), in blocks.
    pub shift: Option<[f32; 3]>,
    pub roll_shift: Option<[f32; 3]>,
    /// A turn about x the renderer's `setupRotations` adds last, in degrees
    /// (a fox that pounces or has its face planted tips with its pitch).
    pub pitch: Option<f32>,
    /// The panda renderer's turns (the somersault, sitting, lying on its back).
    pub panda: Option<PandaRotation>,
    /// `LivingEntityRenderer.isShaking`: the age in ticks of a mob whose body
    /// shudders, or none.
    pub shaking: Option<f32>,
    /// `SquidRenderer.setupRotations`, which replaces the renderer's own turns.
    pub squid: Option<SquidTurns>,
    /// What the fish renderers' `setupRotations` add to the pose.
    pub fish: Option<FishTurns>,
    /// `AbstractCubeMobRenderer.scale`, which comes before the hooks above.
    pub cube_mob: Option<CubeMobScale>,
    /// `ShulkerRenderer.setupRotations`: the data value of the face the shulker
    /// is attached to, which turns the pose about the middle of its box.
    pub attach_face: Option<i32>,
    /// The camera, from the mesh origin, when the viewer sees an invisible mob as a ghost
    /// (`isInvisibleTo` is false for a spectator); none when it sees nothing of the body.
    pub ghost: Option<[f32; 3]>,
    /// A mob a spawner shows (`SpawnerRenderer.submitEntityInSpawner`): the block's corner from
    /// the mesh origin, the turn in degrees and the scale that fits the mob in the cage.
    pub spawner: Option<SpawnerCage>,
    /// `LivingEntityRenderState.isUpsideDown` (a mob named Dinnerbone or Grumm): the height of its
    /// bounding box, which the flip lifts the mob by, or none for one that stands as usual.
    pub upside_down: Option<f32>,
    /// `Pose.SLEEPING`: how a mob lies in its bed.
    pub sleeping: Option<crate::entity::sleep::Sleep>,
}

/// Where a spawner puts the mob it shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnerCage {
    pub origin: [f32; 3],
    pub spin: f32,
    pub scale: f32,
}

impl SpawnerCage {
    /// The pose the mob is drawn in: up in the block, turned, tipped back and scaled.
    pub fn pose(self) -> Pose {
        let mut pose = Pose::default();
        pose.translate(
            self.origin[0] + 0.5,
            self.origin[1] + 0.4,
            self.origin[2] + 0.5,
        );
        pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
            self.spin,
        )));
        pose.translate(0.0, -0.2, 0.0);
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            -30.0,
        )));
        pose.scale(self.scale, self.scale, self.scale);
        pose
    }
}

impl Default for MobPlacement {
    /// A mob of the ordinary size, upright where the mesh starts, unlit.
    fn default() -> Self {
        Self {
            feet: [0.0; 3],
            body_rot: 0.0,
            scale: 1.0,
            light: 0,
            death_time: 0.0,
            flip_degrees: FLIP_DEGREES,
            red_overlay: false,
            white_overlay: 0.0,
            hook_scale: None,
            hook_translate: None,
            roll: None,
            shift: None,
            roll_shift: None,
            pitch: None,
            panda: None,
            shaking: None,
            squid: None,
            fish: None,
            cube_mob: None,
            attach_face: None,
            ghost: None,
            spawner: None,
            upside_down: None,
            sleeping: None,
        }
    }
}

/// `LivingEntityRenderer.getFlipDegrees` of most mobs.
pub const FLIP_DEGREES: f32 = 90.0;

/// What the entity shader reads as the alpha of a vertex lit by the world and blended
/// (`RenderTypes.entityTranslucent` with a colour whose alpha is not full): the shade, past
/// every code the overlay uses, plus the alpha in 255ths.
pub const BLENDED_SHADE: f32 = 64.0;

/// `Direction.getRotation` of the direction with a data value (down 0, up 1,
/// north 2, south 3, west 4, east 5).
pub fn direction_rotation(data_value: i32) -> Quaternion {
    let half = std::f32::consts::PI / 2.0;
    let pi = std::f32::consts::PI;
    match data_value {
        0 => Quaternion::rotation_x(pi),
        2 => Quaternion::rotation_xyz(half, 0.0, pi),
        3 => Quaternion::rotation_x(half),
        4 => Quaternion::rotation_xyz(half, 0.0, half),
        5 => Quaternion::rotation_xyz(half, 0.0, -half),
        _ => Quaternion::identity(),
    }
}

/// `Direction.getOpposite`, on data values.
pub fn opposite_direction(data_value: i32) -> i32 {
    data_value ^ 1
}

/// `TntRenderer.getSwellAmount`.
pub fn tnt_swell_amount(fuse: f32) -> f32 {
    let mut swell = (1.0 - fuse / 10.0).clamp(0.0, 1.0);
    swell *= swell;
    swell *= swell;
    swell * 0.3
}

/// `TntRenderer.isLit`: the flash of a fuse, every fifth tick.
pub fn tnt_is_lit(fuse: f32) -> bool {
    fuse >= 0.0 && (fuse / 5.0) as i32 % 2 == 0
}

/// `OverlayTexture.pack`: the overlay coordinates of a white flash by
/// progress and of the hurt overlay.
pub fn overlay_coords(red: bool, white_progress: f32) -> i32 {
    (white_progress * 15.0) as i32 | (if red { 3 } else { 10 }) << 16
}

/// `OverlayTexture`'s row for what a vertex is drawn with over its texel,
/// as the entity shader reads it: the hurt overlay is one, a white flash is
/// twice its column (`OverlayTexture.u`).
pub fn overlay_code(red: bool, white_progress: f32) -> f32 {
    if red {
        1.0
    } else {
        // `OverlayTexture.u`: truncated to a whole column.
        (2 * (white_progress * 15.0) as i32) as f32
    }
}

/// `LivingEntityRenderer.setupRotations`'s fall of a dying entity: the
/// turn about z, in degrees, once its death has begun. It eases out as the
/// square root of the time since the first tick, and stops at the full flip.
pub fn death_roll(death_time: f32, flip_degrees: f32) -> Option<f32> {
    (death_time > 0.0).then(|| {
        let fall = ((death_time - 1.0) / 20.0 * 1.6).sqrt();
        fall.min(1.0) * flip_degrees
    })
}

/// `LivingEntityRenderer.submit` for a mob standing on its feet: scaled,
/// turned to face away from the body's facing, flipped upright and lowered so
/// the feet meet the ground.
pub fn placement_pose(placement: &MobPlacement) -> Pose {
    let mut pose = placement_rotations(placement);
    pose.scale(-1.0, -1.0, 1.0);
    if let Some(cube) = placement.cube_mob {
        cube.apply(&mut pose);
    }
    if let Some([x, y, z]) = placement.hook_scale {
        pose.scale(x, y, z);
    }
    if let Some([x, y, z]) = placement.hook_translate {
        pose.translate(x, y, z);
    }
    pose.translate(0.0, -1.501, 0.0);
    pose
}

/// What `LivingEntityRenderer.setupRotations` adds to the body's facing while
/// it shakes: a cosine of the whole tick, in degrees.
pub fn shake_degrees(age_in_ticks: f32) -> f32 {
    (f64::from(age_in_ticks.floor() * 3.25_f32).cos() * std::f64::consts::PI * f64::from(0.4_f32))
        as f32
}

/// What `LivingEntityRenderer.setupRotations` does after the turn to face away: the fall of a dead
/// mob, else the lying of a sleeper, or else the flip of one named Dinnerbone or Grumm, lifted by the height of its box and a
/// little more, then turned over.
fn fall_or_flip(pose: &mut Pose, placement: &MobPlacement) {
    let degrees = Quaternion::degrees_to_radians;
    if let Some(roll) = death_roll(placement.death_time, placement.flip_degrees) {
        pose.rotate(&Quaternion::rotation_z(degrees(roll)));
    } else if let Some(sleep) = placement.sleeping {
        pose.rotate(&Quaternion::rotation_y(degrees(sleep.angle)));
        pose.rotate(&Quaternion::rotation_z(degrees(placement.flip_degrees)));
        pose.rotate(&Quaternion::rotation_y(degrees(270.0)));
    } else if let Some(height) = placement.upside_down {
        pose.translate(0.0, (height + 0.1) / placement.scale, 0.0);
        pose.rotate(&Quaternion::rotation_z(degrees(180.0)));
    }
}

/// The feet, and for a sleeper the shift away from the head end of its bed, which
/// `LivingEntityRenderer.submit` makes before it scales.
fn lying_feet(placement: &MobPlacement) -> [f32; 3] {
    let [x, y, z] = placement.feet;
    let [dx, dz] = placement.sleeping.map_or([0.0; 2], |sleep| sleep.shift);
    [x + dx, y, z + dz]
}

/// The turn to face away from the body's facing, which a sleeper does not make.
fn face_away(pose: &mut Pose, placement: &MobPlacement) {
    if placement.sleeping.is_none() {
        pose.rotate(&Quaternion::rotation_y(Quaternion::degrees_to_radians(
            180.0 - (placement.body_rot + placement.shaking.map_or(0.0, shake_degrees)),
        )));
    }
}

/// The swing of a fish's body and its flop on land.
fn fish_rotate(pose: &mut Pose, fish: &FishTurns) {
    let degrees = Quaternion::degrees_to_radians;
    if let Some(swing) = fish.swing {
        pose.rotate(&Quaternion::rotation_y(degrees(swing)));
    }
    if let Some([x, y, z]) = fish.on_land {
        pose.translate(x, y, z);
        pose.rotate(&Quaternion::rotation_z(degrees(90.0)));
    }
}

/// What `LivingEntityRenderer.submit` has done by the end of the renderer's
/// `setupRotations`: the feet, the scale, the turn to face away from the
/// body's facing, the fall of a dead mob and the renderer's own turns.
pub fn placement_rotations(placement: &MobPlacement) -> Pose {
    let mut pose = placement
        .spawner
        .map_or_else(Pose::default, SpawnerCage::pose);
    let [x, y, z] = lying_feet(placement);
    pose.translate(x, y, z);
    pose.scale(placement.scale, placement.scale, placement.scale);
    if let Some(fish) = placement.fish {
        pose.translate(0.0, fish.bob, 0.0);
    }
    if let Some(squid) = placement.squid {
        // `SquidRenderer.setupRotations`: no fall of a dead one, no shudder.
        pose.translate(0.0, if squid.baby { 0.25 } else { 0.5 }, 0.0);
        let degrees = Quaternion::degrees_to_radians;
        pose.rotate(&Quaternion::rotation_y(degrees(180.0 - placement.body_rot)));
        pose.rotate(&Quaternion::rotation_x(degrees(squid.x_body_rot)));
        pose.rotate(&Quaternion::rotation_y(degrees(squid.z_body_rot)));
        pose.translate(0.0, if squid.baby { -0.6 } else { -1.2 }, 0.0);
        return pose;
    }
    face_away(&mut pose, placement);
    fall_or_flip(&mut pose, placement);
    if let Some(fish) = placement.fish {
        fish_rotate(&mut pose, &fish);
    }
    if let Some([x, y, z]) = placement.shift {
        pose.translate(x, y, z);
    }
    if let Some(roll) = placement.roll {
        pose.rotate(&Quaternion::rotation_z(Quaternion::degrees_to_radians(
            roll,
        )));
    }
    if let Some([x, y, z]) = placement.roll_shift {
        pose.translate(x, y, z);
    }
    if let Some(pitch) = placement.pitch {
        pose.rotate(&Quaternion::rotation_x(Quaternion::degrees_to_radians(
            pitch,
        )));
    }
    if let Some(panda) = placement.panda {
        panda_rotate(&mut pose, &panda);
    }
    if let Some(face) = placement.attach_face {
        pose.rotate_around(
            &direction_rotation(opposite_direction(face)),
            (0.0, 0.5, 0.0),
        );
    }
    pose
}
