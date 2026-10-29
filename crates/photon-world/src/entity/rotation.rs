use super::*;

/// Where an entity faces now and one tick ago: the body, the pitch and the head, in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, bevy_ecs::component::Component)]
pub struct Rotation {
    pub(crate) x_rot: JavaF32,
    pub(crate) y_rot: JavaF32,
    pub(crate) y_head_rot: JavaF32,
    pub(crate) y_body_rot: JavaF32,
    pub(crate) old_x_rot: JavaF32,
    pub(crate) old_y_rot: JavaF32,
    pub(crate) old_y_head_rot: JavaF32,
    pub(crate) old_y_body_rot: JavaF32,
}

impl Rotation {
    /// An entity that was just added: facing `(yaw, pitch, head)`, the body along the yaw, and
    /// nothing turned since the tick before.
    pub(crate) fn spawned(yaw: f32, pitch: f32, head: f32) -> Self {
        Self {
            x_rot: JavaF32::new(pitch),
            y_rot: JavaF32::new(yaw),
            y_head_rot: JavaF32::new(head),
            y_body_rot: JavaF32::new(yaw),
            old_x_rot: JavaF32::new(pitch),
            old_y_rot: JavaF32::new(yaw),
            old_y_head_rot: JavaF32::new(head),
            old_y_body_rot: JavaF32::new(yaw),
        }
    }
}

impl EntityViewMut<'_> {
    pub(crate) fn turn(&mut self, mouse_x: f64, mouse_y: f64) {
        let x_delta = mouse_y as f32 * 0.15_f32;
        let y_delta = mouse_x as f32 * 0.15_f32;
        let rotation = &mut *self.rotation;
        rotation.x_rot = JavaF32::new(rotation.x_rot.get() + x_delta);
        rotation.y_rot = JavaF32::new(rotation.y_rot.get() + y_delta);
        rotation.x_rot = JavaF32::new(clamp_f32(rotation.x_rot.get(), -90.0, 90.0));
        rotation.old_x_rot = JavaF32::new(rotation.old_x_rot.get() + x_delta);
        rotation.old_y_rot = JavaF32::new(rotation.old_y_rot.get() + y_delta);
        rotation.old_x_rot = JavaF32::new(clamp_f32(rotation.old_x_rot.get(), -90.0, 90.0));
    }

    pub(crate) fn tick_head_rotation(&mut self) {
        advance_head_rotation(
            &mut self.rotation.y_head_rot,
            self.entity.head_rotation_target,
            &mut self.entity.head_lerp_steps,
        );
    }
}

impl ClientEntity {
    /// `ElytraAnimationState.getRotX/Y/Z`: how far the wings are spread.
    pub fn elytra_rotations(&self, partial_tick: f32) -> [f32; 3] {
        self.living.elytra.rotations(partial_tick)
    }

    pub fn boat_delta_rotation(&self) -> f32 {
        self.delta_rotation.get()
    }

    pub fn head_rotation_target(&self) -> f32 {
        self.head_rotation_target.get()
    }

    pub fn head_rotation_lerp_steps(&self) -> u8 {
        self.head_lerp_steps
    }
}

impl Rotation {
    pub fn rotation(&self) -> (f32, f32, f32) {
        (self.y_rot.get(), self.x_rot.get(), self.y_head_rot.get())
    }

    /// `xRotO`.
    pub fn previous_x_rotation(&self) -> f32 {
        self.old_x_rot.get()
    }

    /// `yBodyRot`.
    pub fn y_body_rot(&self) -> f32 {
        self.y_body_rot.get()
    }

    /// `getYHeadRot`.
    pub fn y_head_rot(&self) -> f32 {
        self.y_head_rot.get()
    }

    pub fn body_y_rotation(&self) -> f32 {
        self.y_body_rot.get()
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        self.old_y_body_rot.get()
    }

    pub(crate) fn body_rotation(&self) -> (f32, f32) {
        (self.y_rot.get(), self.x_rot.get())
    }

    pub fn previous_rotation(&self) -> (f32, f32, f32) {
        (
            self.old_y_rot.get(),
            self.old_x_rot.get(),
            self.old_y_head_rot.get(),
        )
    }

    /// `Entity.getYRot(partialTick)`: the turn between two ticks, by the short way round
    /// (`Mth.rotLerp`), so a turn through the half circle does not swing back.
    pub fn y_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().0;
        if partial_tick == 1.0 {
            return current;
        }
        mth_rot_lerp_f32(partial_tick, self.previous_rotation().0, current)
    }

    /// `Entity.getXRot(partialTick)`: the pitch between two ticks, a plain lerp.
    pub fn x_rot_at(&self, partial_tick: f32) -> f32 {
        let current = self.rotation().1;
        if partial_tick == 1.0 {
            return current;
        }
        mth_lerp_f32(partial_tick, self.previous_rotation().1, current)
    }
}

/// An entity's rotation as the entity itself answers for it.
impl EntityView<'_> {
    pub fn rotation(&self) -> (f32, f32, f32) {
        self.rotation.rotation()
    }

    pub fn previous_x_rotation(&self) -> f32 {
        self.rotation.previous_x_rotation()
    }

    pub fn y_body_rot(&self) -> f32 {
        self.rotation.y_body_rot()
    }

    pub fn y_head_rot(&self) -> f32 {
        self.rotation.y_head_rot()
    }

    pub fn body_y_rotation(&self) -> f32 {
        self.rotation.body_y_rotation()
    }

    pub fn previous_body_y_rotation(&self) -> f32 {
        self.rotation.previous_body_y_rotation()
    }

    pub fn previous_rotation(&self) -> (f32, f32, f32) {
        self.rotation.previous_rotation()
    }

    pub fn y_rot_at(&self, partial_tick: f32) -> f32 {
        self.rotation.y_rot_at(partial_tick)
    }

    pub fn x_rot_at(&self, partial_tick: f32) -> f32 {
        self.rotation.x_rot_at(partial_tick)
    }
}

pub(crate) fn advance_head_rotation(current: &mut JavaF32, target: JavaF32, steps: &mut u8) {
    if *steps == 0 {
        return;
    }
    let step_count = f64::from(*steps);
    let from = f64::from(current.get());
    let to = f64::from(target.get());
    *current = JavaF32::new(mth_rot_lerp_f64(1.0 / step_count, from, to) as f32);
    *steps -= 1;
}
