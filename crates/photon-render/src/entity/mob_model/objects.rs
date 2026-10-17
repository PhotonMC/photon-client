use super::*;

/// What an arrow's model is posed from (`ArrowRenderState`): how long it has shaken.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrowRenderState {
    pub shake: f32,
}

impl MobModel {
    /// `SkullModel.setupAnim` and its piglin's and dragon's: the head faces straight ahead (the
    /// layer gives the skull no turn), a piglin's ears swing and a dragon's jaw works with the
    /// walk (`animation_pos`).
    pub fn setup_skull(&mut self, kind: crate::block_entity::skull::SkullKind, animation_pos: f32) {
        use crate::block_entity::skull::SkullKind;
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        match kind {
            SkullKind::Piglin => {
                if let Some(head) = self.root.child_mut("head") {
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                    // `Math.cos` of a float's double, not the table `Mth.cos` reads.
                    let ear = |value: f32| f64::from(value).cos();
                    if let Some(left) = head.child_mut("left_ear") {
                        left.z_rot = (-(ear(animation_pos * pi * 0.2 * 1.2) + 2.5)) as f32 * 0.2;
                    }
                    if let Some(right) = head.child_mut("right_ear") {
                        right.z_rot = (ear(animation_pos * pi * 0.2) + 2.5) as f32 * 0.2;
                    }
                }
            }
            SkullKind::Dragon => {
                if let Some(head) = self.root.child_mut("head") {
                    if let Some(jaw) = head.child_mut("jaw") {
                        jaw.x_rot = (f64::from(animation_pos * pi * 0.2).sin() + 1.0) as f32 * 0.2;
                    }
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                }
            }
            _ => {
                if let Some(head) = self.root.child_mut("head") {
                    head.y_rot = 0.0;
                    head.x_rot = 0.0;
                }
            }
        }
    }

    /// Move the whole model (`PoseStack.translate` before its own pose), in model units.
    pub fn shift_root(&mut self, [x, y, z]: [f32; 3]) {
        self.root.x += x;
        self.root.y += y;
        self.root.z += z;
    }

    /// `ElytraModel.setupAnim`: the wings spread by the animation state's rotations, lowered when
    /// the wearer sneaks.
    pub fn setup_elytra(&mut self, [rot_x, rot_y, rot_z]: [f32; 3], crouching: bool) {
        self.root.reset();
        let y = if crouching { 3.0 } else { 0.0 };
        if let Some(left) = self.root.child_mut("left_wing") {
            left.y = y;
            left.x_rot = rot_x;
            left.z_rot = rot_z;
            left.y_rot = rot_y;
        }
        if let Some(right) = self.root.child_mut("right_wing") {
            right.y = y;
            right.x_rot = rot_x;
            right.z_rot = -rot_z;
            right.y_rot = -rot_y;
        }
    }

    /// `WindChargeModel.setupAnim`: the charge and the wind about it spin opposite ways.
    pub fn setup_wind_charge(&mut self, age_in_ticks: f32) {
        self.root.reset();
        let spin = age_in_ticks * 16.0 * (std::f64::consts::PI / 180.0) as f32;
        let Some(bone) = self.root.child_mut("bone") else {
            return;
        };
        if let Some(charge) = bone.child_mut("wind_charge") {
            charge.y_rot = -spin;
        }
        if let Some(wind) = bone.child_mut("wind") {
            wind.y_rot = spin;
        }
    }

    /// `LlamaSpitModel` has no animation: its part stays where the mesh puts it.
    pub fn setup_llama_spit(&mut self) {
        self.root.reset();
    }

    /// `LeashKnotModel` has no animation: its part stays where the mesh puts it.
    pub fn setup_leash_knot(&mut self) {
        self.root.reset();
    }

    /// `ShulkerBulletModel.setupAnim`: the bullet is turned the way it flies.
    pub fn setup_shulker_bullet(&mut self, y_rot: f32, x_rot: f32) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        if let Some(main) = self.root.child_mut("main") {
            main.y_rot = y_rot * degrees;
            main.x_rot = x_rot * degrees;
        }
    }

    /// `EvokerFangsModel.setupAnim`: the jaws close as the bite goes on, the fangs sink and
    /// then shrink away at its end.
    pub fn setup_evoker_fangs(&mut self, bite_progress: f32) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let mut bite_amount = (bite_progress * 2.0).min(1.0);
        bite_amount = 1.0 - bite_amount * bite_amount * bite_amount;
        if let Some(base) = self.root.child_mut("base") {
            if let Some(jaw) = base.child_mut("upper_jaw") {
                jaw.z_rot = pi - bite_amount * 0.35 * pi;
            }
            if let Some(jaw) = base.child_mut("lower_jaw") {
                jaw.z_rot = pi + bite_amount * 0.35 * pi;
            }
            base.y -= (bite_progress + mth_sin_f64(f64::from(bite_progress * 2.7))) * 7.2;
        }
        let mut pre_scale = 1.0_f32;
        if bite_progress > 0.9 {
            pre_scale *= (1.0 - bite_progress) / 0.1;
        }
        self.root.y = 24.0 - 20.0 * pre_scale;
        self.root.scale = [pre_scale; 3];
    }

    /// `TridentModel` has no animation: its parts stay where the mesh puts them.
    pub fn setup_trident(&mut self) {
        self.root.reset();
    }

    /// `ArrowModel.setupAnim`: a stuck arrow shakes about its length for a few ticks.
    pub fn setup_arrow(&mut self, state: &ArrowRenderState) {
        self.root.reset();
        if state.shake > 0.0 {
            let pow = -mth_sin_f64(f64::from(state.shake * 3.0)) * state.shake;
            self.root.z_rot += pow * (std::f64::consts::PI / 180.0) as f32;
        }
    }

    /// `ChestModel.setupAnim`: the lid, and the lock on it, tip back by how open the (eased) lid is.
    pub fn setup_chest(&mut self, open: f32) {
        self.root.reset();
        let angle = -(open * (std::f64::consts::PI / 2.0) as f32);
        for name in ["lid", "lock"] {
            if let Some(part) = self.root.child_mut(name) {
                part.x_rot = angle;
            }
        }
    }

    /// `CopperGolemStatueModel.setupAnim`: the root is set down and turned over, which the entity
    /// models' own upside-down layers need.
    pub fn setup_copper_statue(&mut self) {
        self.root.reset();
        self.root.y = 0.0;
        self.root.z_rot = std::f32::consts::PI;
    }

    /// Only one of the named parts of the root is drawn (`ModelPart.visible`), as a renderer that
    /// draws each part in its own texture has it.
    pub fn show_only(&mut self, parts: &[&str], shown: &str) {
        for name in parts {
            if let Some(part) = self.root.child_mut(name) {
                part.visible = *name == shown;
            }
        }
    }

    /// `BookModel.setupAnim`: the lids open to `openness`, the pages follow and the two loose
    /// pages turn between them by how far each has been flipped.
    pub fn setup_book(&mut self, openness: f32, (flip1, flip2): (f32, f32)) {
        self.root.reset();
        let pi = std::f64::consts::PI as f32;
        let turns = [
            ("left_lid", pi + openness),
            ("right_lid", -openness),
            ("left_pages", openness),
            ("right_pages", -openness),
            ("flip_page1", openness - openness * 2.0 * flip1),
            ("flip_page2", openness - openness * 2.0 * flip2),
        ];
        for (name, y_rot) in turns {
            if let Some(part) = self.root.child_mut(name) {
                part.y_rot = y_rot;
            }
        }
        let x = mth_sin_f64(f64::from(openness));
        for name in ["left_pages", "right_pages", "flip_page1", "flip_page2"] {
            if let Some(part) = self.root.child_mut(name) {
                part.x = x;
            }
        }
    }

    /// `BannerFlagModel.setupAnim`: the flag tips by the phase of its wave.
    pub fn setup_banner_flag(&mut self, phase: f32) {
        self.root.reset();
        if let Some(flag) = self.root.child_mut("flag") {
            let pi = std::f64::consts::PI as f32;
            let wave = mth_cos_f64(f64::from(2.0 * std::f64::consts::PI as f32 * phase));
            flag.x_rot = (-0.0125 + 0.01 * wave) * pi;
        }
    }

    /// `BellModel.setupAnim`: the bell swings away from the side it was struck from, less each
    /// tick.
    pub fn setup_bell(&mut self, ticks: f32, shake: Option<photon_core::direction::Direction>) {
        use photon_core::direction::Direction;
        self.root.reset();
        let (mut x_rot, mut z_rot) = (0.0_f32, 0.0_f32);
        if let Some(direction) = shake {
            let base = photon_core::math::mth_sin_f64(f64::from(ticks / std::f32::consts::PI))
                / (4.0 + ticks / 3.0);
            match direction {
                Direction::North => x_rot = -base,
                Direction::South => x_rot = base,
                Direction::East => z_rot = -base,
                Direction::West => z_rot = base,
                Direction::Down | Direction::Up => {}
            }
        }
        if let Some(body) = self.root.child_mut("bell_body") {
            body.x_rot = x_rot;
            body.z_rot = z_rot;
        }
    }

    /// `ShulkerBoxModel.setupAnim`: the lid rises half a block and turns three quarters of a
    /// circle as the box opens.
    pub fn setup_shulker_box(&mut self, progress: f32) {
        self.root.reset();
        if let Some(lid) = self.root.child_mut("lid") {
            lid.x = 0.0;
            lid.y = 24.0 - progress * 0.5 * 16.0;
            lid.z = 0.0;
            lid.y_rot = 270.0 * progress * (std::f64::consts::PI / 180.0) as f32;
        }
    }

    /// `AbstractBoatModel.setupAnim`: each paddle swings by the time it has
    /// rowed (the right one mirrored).
    pub fn setup_boat(&mut self, state: &crate::objects::boat::BoatRenderState) {
        self.root.reset();
        use std::f64::consts::PI;
        for (name, time, side) in [
            ("left_paddle", state.rowing_time_left, 0),
            ("right_paddle", state.rowing_time_right, 1),
        ] {
            let Some(paddle) = self.root.child_mut(name) else {
                continue;
            };
            paddle.x_rot = photon_core::math::clamped_lerp_f32(
                (mth_sin_f64(f64::from(-time)) + 1.0) / 2.0,
                (-PI / 3.0) as f32,
                (-PI / 12.0) as f32,
            );
            paddle.y_rot = photon_core::math::clamped_lerp_f32(
                (mth_sin_f64(f64::from(-time + 1.0)) + 1.0) / 2.0,
                (-PI / 4.0) as f32,
                (PI / 4.0) as f32,
            );
            if side == 1 {
                paddle.y_rot = PI as f32 - paddle.y_rot;
            }
        }
    }
}
