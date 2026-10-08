use super::*;

impl MobModel {
    /// `SilverfishModel.setupAnim` and `EndermiteModel.setupAnim`: each
    /// segment sways about y and slides sideways in a wave that travels down
    /// the body (the silverfish's three layers follow the segments under them).
    pub fn setup_segmented(&mut self, age_in_ticks: f32, silverfish: bool) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let (segments, turn, slide) = if silverfish {
            (7_i32, 0.05_f32, 0.2_f32)
        } else {
            (4, 0.01, 0.1)
        };
        for index in 0..segments {
            let phase = age_in_ticks * 0.9 + index as f32 * 0.15 * pi;
            let distance = (index - 2).abs();
            let y_rot = mth_cos_f64(f64::from(phase)) * pi * turn * (1 + distance) as f32;
            let x = mth_sin_f64(f64::from(phase)) * pi * slide * distance as f32;
            if let Some(segment) = self.root.child_mut(&format!("segment{index}")) {
                segment.y_rot = y_rot;
                segment.x = x;
            }
        }
        if !silverfish {
            return;
        }
        let pose = |root: &mut Part, index: usize| {
            root.child_mut(&format!("segment{index}"))
                .map(|segment| (segment.y_rot, segment.x))
        };
        for (layer, segment, slides) in [(0, 2, false), (1, 4, true), (2, 1, true)] {
            let Some((y_rot, x)) = pose(&mut self.root, segment) else {
                continue;
            };
            if let Some(layer) = self.root.child_mut(&format!("layer{layer}")) {
                layer.y_rot = y_rot;
                if slides {
                    layer.x = x;
                }
            }
        }
    }

    /// `SpiderModel.setupAnim`: the head follows the look; each leg is
    /// swung about y and lifted about z in turn, the pairs a quarter of a
    /// stride apart.
    pub fn setup_spider(&mut self, state: &QuadrupedState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let sin = |value: f32| mth_sin_f64(f64::from(value));
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = state.head_y_rot * degrees;
        }
        let position = state.walk_position * 0.6662;
        let speed = state.walk_speed;
        let quarter = (std::f64::consts::PI / 2.0) as f32;
        let three_quarters = (std::f64::consts::PI * 3.0 / 2.0) as f32;
        let swing = |phase: f32| -(cos(position * 2.0 + phase) * 0.4) * speed;
        let step = |phase: f32| (sin(position + phase) * 0.4).abs() * speed;
        // The pairs, back to front: each leg on the right moves one way and
        // its opposite the other.
        for (right, left, phase) in [
            ("right_hind_leg", "left_hind_leg", 0.0),
            ("right_middle_hind_leg", "left_middle_hind_leg", pi),
            ("right_middle_front_leg", "left_middle_front_leg", quarter),
            ("right_front_leg", "left_front_leg", three_quarters),
        ] {
            let (turn, lift) = (swing(phase), step(phase));
            if let Some(leg) = self.root.child_mut(right) {
                leg.y_rot += turn;
                leg.z_rot += lift;
            }
            if let Some(leg) = self.root.child_mut(left) {
                leg.y_rot -= turn;
                leg.z_rot -= lift;
            }
        }
    }

    /// `CreeperModel.setupAnim`: the head follows the look, and the legs
    /// swing in diagonal pairs (the model names its sides the other way
    /// about, which the swing follows).
    pub fn setup_creeper(&mut self, state: &QuadrupedState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let pi = std::f64::consts::PI as f32;
        let cos = |value: f32| mth_cos_f64(f64::from(value));
        if let Some(head) = self.root.child_mut("head") {
            head.y_rot = state.head_y_rot * degrees;
            head.x_rot = state.x_rot * degrees;
        }
        let (position, speed) = (state.walk_position, state.walk_speed);
        let swing = [
            ("left_hind_leg", cos(position * 0.6662) * 1.4 * speed),
            ("right_hind_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("left_front_leg", cos(position * 0.6662 + pi) * 1.4 * speed),
            ("right_front_leg", cos(position * 0.6662) * 1.4 * speed),
        ];
        for (name, x_rot) in swing {
            if let Some(leg) = self.root.child_mut(name) {
                leg.x_rot = x_rot;
            }
        }
    }

    /// `EndermanModel.setupAnim`: the humanoid pose with the limbs' swing
    /// halved and held within 0.4; a carried block holds the arms forward and
    /// a creepy one lifts its head off its hat.
    pub fn setup_enderman(&mut self, state: &EndermanState) {
        humanoid_anim(&mut self.root, &state.humanoid);
        if let Some(head) = self.root.child_mut("head") {
            head.visible = true;
        }
        for name in ["right_arm", "left_arm", "right_leg", "left_leg"] {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = (part.x_rot * 0.5).clamp(-0.4, 0.4);
            }
        }
        if state.carrying {
            for (name, x_rot, z_rot) in [("right_arm", -0.5, 0.05), ("left_arm", -0.5, -0.05)] {
                if let Some(arm) = self.root.child_mut(name) {
                    arm.x_rot = x_rot;
                    arm.z_rot = z_rot;
                }
            }
        }
        if state.creepy
            && let Some(head) = self.root.child_mut("head")
        {
            head.y -= 5.0;
            if let Some(hat) = head.child_mut("hat") {
                hat.y += 5.0;
            }
        }
    }

    /// `CreakingModel.setupAnim`: the head takes the look, the walk runs if the
    /// creaking can move, then the attack, the sway and the death animations
    /// that have started. The eyes' layer shares the parts' names.
    pub fn setup_creaking(&mut self, state: &CreakingRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self
            .root
            .child_mut("root")
            .and_then(|root| root.child_mut("upper_body"))
            .and_then(|upper| upper.child_mut("head"))
        {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        if state.can_move
            && let Some(definition) = photon_data::animation::definition("Creaking.CREAKING_WALK")
        {
            // `applyWalk(pos, speed, 1, 1)`.
            let millis = (state.quadruped.walk_position * 50.0 * 1.0) as i64;
            let scale = (state.quadruped.walk_speed * 1.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("Creaking.CREAKING_ATTACK", state.attack),
            ("Creaking.CREAKING_INVULNERABLE", state.invulnerable),
            ("Creaking.CREAKING_DEATH", state.death),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `ArmadilloModel.setupAnim` (the adult's and the baby's alike, with the
    /// baby's own animations): hiding in the shell draws the ball in place of
    /// the body's cubes, legs and tail, and the walk stops; otherwise the head
    /// takes the look within its limits and the walk runs; then the animations
    /// that have started.
    pub fn setup_armadillo(&mut self, state: &ArmadilloRenderState, baby: bool) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        let hiding = state.is_hiding_in_shell;
        if let Some(cube) = self.root.child_mut("cube") {
            cube.visible = hiding;
        }
        for name in ["left_hind_leg", "right_hind_leg"] {
            if let Some(leg) = self.root.child_mut(name) {
                leg.visible = !hiding;
            }
        }
        if let Some(body) = self.root.child_mut("body") {
            body.skip_draw = hiding;
            if let Some(tail) = body.child_mut("tail") {
                tail.visible = !hiding;
            }
            if !hiding && let Some(head) = body.child_mut("head") {
                head.x_rot = state.quadruped.x_rot.clamp(-22.5, 25.0) * degrees;
                head.y_rot = state.quadruped.head_y_rot.clamp(-32.5, 32.5) * degrees;
            }
        }
        let (walk, roll_out, roll_up, peek) = if baby {
            (
                "BabyArmadillo.ARMADILLO_BABY_WALK",
                "BabyArmadillo.ARMADILLO_BABY_ROLL_OUT",
                "BabyArmadillo.ARMADILLO_BABY_ROLL_UP",
                "BabyArmadillo.ARMADILLO_BABY_PEEK",
            )
        } else {
            (
                "Armadillo.ARMADILLO_WALK",
                "Armadillo.ARMADILLO_ROLL_OUT",
                "Armadillo.ARMADILLO_ROLL_UP",
                "Armadillo.ARMADILLO_PEEK",
            )
        };
        if !hiding && let Some(definition) = photon_data::animation::definition(walk) {
            // `applyWalk(pos, speed, 16.5, 2.5)`.
            let millis = (state.quadruped.walk_position * 50.0 * 16.5) as i64;
            let scale = (state.quadruped.walk_speed * 2.5).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            (roll_out, state.roll_out),
            (roll_up, state.roll_up),
            (peek, state.peek),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `SnifferModel.setupAnim` (the baby's layer has the same parts and uses
    /// the same model): the head takes the look, the walk (the searching one
    /// while it searches) runs, then the animations that have started.
    pub fn setup_sniffer(&mut self, state: &SnifferRenderState) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self
            .root
            .child_mut("bone")
            .and_then(|bone| bone.child_mut("body"))
            .and_then(|body| body.child_mut("head"))
        {
            head.x_rot = state.quadruped.x_rot * degrees;
            head.y_rot = state.quadruped.head_y_rot * degrees;
        }
        let walk = if state.is_searching {
            "Sniffer.SNIFFER_SNIFF_SEARCH"
        } else {
            "Sniffer.SNIFFER_WALK"
        };
        if let Some(definition) = photon_data::animation::definition(walk) {
            // `applyWalk(pos, speed, 9, 100)`.
            let millis = (state.quadruped.walk_position * 50.0 * 9.0) as i64;
            let scale = (state.quadruped.walk_speed * 100.0).min(1.0);
            self.animate(definition, millis, scale);
        }
        for (name, running) in [
            ("Sniffer.SNIFFER_DIG", state.digging),
            ("Sniffer.SNIFFER_LONGSNIFF", state.sniffing),
            ("Sniffer.SNIFFER_STAND_UP", state.rising),
            ("Sniffer.SNIFFER_HAPPY", state.feeling_happy),
            ("Sniffer.SNIFFER_SNIFFSNIFF", state.scenting),
        ] {
            self.animate_state(name, running, state.age_in_ticks);
        }
    }

    /// `ShulkerModel.setupAnim`: the lid rises and, past a third open, twists;
    /// with the lid more than fully open it also bobs; the head turns from the
    /// body to where the entity looks.
    pub fn setup_shulker(&mut self, state: &ShulkerRenderState) {
        self.root.reset();
        let pi = std::f32::consts::PI;
        let bs = (0.5 + state.peek_amount) * pi;
        let sine = mth_sin_f64(f64::from(bs));
        let q = -1.0 + sine;
        let mut extra = 0.0;
        if bs > pi {
            extra = mth_sin_f64(f64::from(state.age_in_ticks * 0.1)) * 0.7;
        }
        if let Some(lid) = self.root.child_mut("lid") {
            lid.x = 0.0;
            lid.y = 16.0 + sine * 8.0 + extra;
            lid.z = 0.0;
            lid.y_rot = if state.peek_amount > 0.3 {
                q * q * q * q * pi * 0.125
            } else {
                0.0
            };
        }
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(head) = self.root.child_mut("head") {
            head.x_rot = state.x_rot * degrees;
            head.y_rot = (state.y_head_rot - 180.0 - state.y_body_rot) * degrees;
        }
    }

    /// `MagmaCubeModel.setupAnim`: the eight slices lift apart by the squish
    /// (none while it is squashed), the lowest staying where it is.
    pub fn setup_magma_cube(&mut self, squish: f32) {
        self.root.reset();
        let squish = squish.max(0.0);
        for index in 0..8_i32 {
            if let Some(slice) = self.root.child_mut(&format!("cube{index}")) {
                slice.y = -((4 - index) as f32) * squish * 1.7;
            }
        }
    }
}

/// What a shulker's model is posed from (`ShulkerRenderState`): the lid's
/// peek, the age, the pitch, and the head's and the body's own turns.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ShulkerRenderState {
    pub peek_amount: f32,
    pub age_in_ticks: f32,
    pub x_rot: f32,
    pub y_head_rot: f32,
    pub y_body_rot: f32,
}

/// What an enderman's model is posed from: the humanoid state, whether it
/// carries a block (`carriedBlock` is not empty) and `isCreepy`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EndermanState {
    pub humanoid: PlayerRenderState,
    pub carrying: bool,
    pub creepy: bool,
}

/// What a creaking's models (the body and its eyes) are posed from
/// (`CreakingRenderState`): the look, the walk (which only a creaking that can
/// move shows), and the animation states its ticks run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CreakingRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub can_move: bool,
    pub attack: photon_world::animation_state::AnimationState,
    pub invulnerable: photon_world::animation_state::AnimationState,
    pub death: photon_world::animation_state::AnimationState,
}

/// What an armadillo's model is posed from (`ArmadilloRenderState`): the look,
/// the walk, whether it hides in its shell, and its animation states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArmadilloRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_hiding_in_shell: bool,
    pub roll_out: photon_world::animation_state::AnimationState,
    pub roll_up: photon_world::animation_state::AnimationState,
    pub peek: photon_world::animation_state::AnimationState,
}

/// What a sniffer's model is posed from (`SnifferRenderState`): the look, the
/// walk, whether it is searching, and its animation states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnifferRenderState {
    pub quadruped: QuadrupedState,
    pub age_in_ticks: f32,
    pub is_searching: bool,
    pub digging: photon_world::animation_state::AnimationState,
    pub sniffing: photon_world::animation_state::AnimationState,
    pub rising: photon_world::animation_state::AnimationState,
    pub feeling_happy: photon_world::animation_state::AnimationState,
    pub scenting: photon_world::animation_state::AnimationState,
}

/// What a slime's or a magma cube's renderer scales the pose by
/// (`AbstractCubeMobRenderer.applySizeAndSquish`, and for a slime
/// `downscaleSlightly` before it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubeMobScale {
    pub size: i32,
    pub squish: f32,
    pub downscale: bool,
    pub sulfur: Option<SulfurCubeScale>,
}

/// What a sulfur cube's renderer scales the pose by besides the cube mobs'
/// (`SulfurCubeRenderer.scale`): the swell of a fuse about to run out, and
/// the fit of the small or the full shell about the model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SulfurCubeScale {
    /// `SulfurCubeRenderState.fuseRemainingTicks`: zero while it is not primed.
    pub fuse_remaining: f32,
    pub baby: bool,
    /// Whether it holds a block that is drawn (`SulfurCubeRenderState.containedBlock`
    /// is not empty): it then keeps no squish.
    pub holds_block: bool,
}

/// What `SulfurCubeInnerLayer` draws with: a fuse that is lit flashes white,
/// else the entity's own overlay (`LivingEntityRenderer.getOverlayCoords`
/// with no white). Whether it is red and the white's progress.
pub fn sulfur_cube_overlay(fuse_remaining: f32, red: bool) -> (bool, f32) {
    if fuse_remaining > 0.0 && tnt_is_lit(fuse_remaining) {
        (false, 1.0)
    } else {
        (red, 0.0)
    }
}

/// `CreeperRenderer.scale`: a creeper swells, wobbling as it nears its blast;
/// the factors along x and z, and along y.
pub fn creeper_scale(swelling: f32) -> [f32; 3] {
    let wobble = 1.0 + mth_sin_f64(f64::from(swelling * 100.0)) * swelling * 0.01;
    let mut swell = swelling.clamp(0.0, 1.0);
    swell *= swell;
    swell *= swell;
    let wide = (1.0 + swell * 0.4) * wobble;
    let tall = (1.0 + swell * 0.1) / wobble;
    [wide, tall, wide]
}

/// `CreeperRenderer.getWhiteOverlayProgress`: a swelling creeper flashes
/// white every other tenth of its swell.
pub fn creeper_white_overlay(swelling: f32) -> f32 {
    if (swelling * 10.0) as i32 % 2 == 0 {
        0.0
    } else {
        swelling.clamp(0.5, 1.0)
    }
}

impl CubeMobScale {
    pub(super) fn apply(&self, pose: &mut Pose) {
        if self.downscale {
            pose.scale(0.999, 0.999, 0.999);
            pose.translate(0.0, 0.001, 0.0);
        }
        let size = self.size as f32;
        // `SulfurCubeRenderer.applySizeAndSquish`: a held block does not squash.
        let squish = if self.sulfur.is_some_and(|sulfur| sulfur.holds_block) {
            0.0
        } else {
            self.squish
        };
        let squash = squish / (size * 0.5 + 1.0);
        let wide = 1.0 / (squash + 1.0);
        pose.scale(wide * size, 1.0 / wide * size, wide * size);
        if let Some(sulfur) = self.sulfur {
            let fuse = sulfur.fuse_remaining;
            if fuse < 10.0 && fuse > 0.0 {
                let swell = 1.0 + tnt_swell_amount(fuse);
                pose.scale(swell, swell, swell);
            }
            let (offset, fit) = if sulfur.baby {
                (1.24, 1.0)
            } else {
                (0.98, 0.5)
            };
            pose.scale(fit, fit, fit);
            // One pixel up while it is seen; `translate(-0.0F, ..., -0.0F)`.
            pose.translate(-0.0, offset - 1.0 / 16.0, -0.0);
        }
    }
}
