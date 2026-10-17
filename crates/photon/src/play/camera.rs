//! Where a frame is seen from (`Camera`): the camera's place, its field of view and what it is in.

use super::*;

impl Play {
    /// `Options.getEffectiveRenderDistance`: the option, no more than the server's.
    fn effective_render_distance(&self) -> i32 {
        let option = self.controls.options.render_distance;
        let server = self.session.view_distance();
        if server > 0 {
            option.min(server)
        } else {
            option
        }
    }

    /// `Camera.alignWithEntity` and the projection: the front view turns
    /// round, and a detached camera backs away from the eyes until a block
    /// is in the way.
    pub(super) fn frame_camera(
        &self,
        player: &photon_world::local_player::LocalPlayer,
    ) -> FrameCamera {
        let partial = self.accumulated.as_secs_f64() / TICK_LENGTH.as_secs_f64();
        let feet = player.interpolated_position(partial);
        let (player_y_rot, player_x_rot) = player.rotation();
        let camera_type = self.controls.camera_type;
        let (y_rot, x_rot) = if camera_type == CameraType::ThirdPersonFront {
            (player_y_rot + 180.0, -player_x_rot)
        } else {
            (player_y_rot, player_x_rot)
        };
        let eye = Vec3::new(
            feet.x,
            feet.y + f64::from(self.camera.eye_height(partial as f32)),
            feet.z,
        );
        let camera = camera_position(self.session.world(), camera_type, eye, (y_rot, x_rot));
        let relative = [
            (camera.x - f64::from(self.origin[0])) as f32,
            (camera.y - f64::from(self.origin[1])) as f32,
            (camera.z - f64::from(self.origin[2])) as f32,
        ];
        let option_fov = self.controls.options.fov;
        // `Camera.getFluidInCamera`, which narrows the field of view in water and lava.
        let fluid = crate::view::camera_fluid(
            self.session.world(),
            camera,
            (y_rot, x_rot),
            (option_fov, f64::from(self.aspect)),
        );
        let fov = self
            .camera
            .fov(option_fov, partial as f32, fluid.narrows_view());
        // The camera's far plane: four times the render distance, or the
        // cloud range if that is farther.
        let render_distance = self.effective_render_distance();
        let far = ((render_distance * 16 * 4) as f32).max((CLOUD_RANGE * 16) as f32);
        let bob = self.view_motion(player, partial as f32);
        let locator = photon_gui::hud::locator::LocatorCamera {
            y_rot,
            x_rot,
            position: [camera.x, camera.y, camera.z],
            fov,
            size: (self.aspect, 1.0),
            z_near: 0.05,
            z_far: far,
        };
        FrameCamera {
            locator,
            partial,
            eye,
            camera,
            relative,
            rotation: (y_rot, x_rot),
            view_projection: player_camera(self.aspect, fov, far, relative, y_rot, x_rot, bob),
            sky_view_projection: player_camera(self.aspect, fov, far, [0.0; 3], y_rot, x_rot, bob),
            render_distance,
            fog: crate::world::fog::fog_view(
                &self.session,
                (fluid, camera.y),
                render_distance,
                partial as f32,
            ),
            // The field of view never culls narrower than the option.
            frustum: photon_render::visibility::Frustum::new(
                &crate::view::view_rotation(y_rot, x_rot),
                &photon_render::visibility::culling_projection(
                    fov.max(option_fov as f32),
                    self.aspect,
                    far,
                ),
                [camera.x, camera.y, camera.z],
            ),
        }
    }
}

impl Play {
    /// What the view is moved by, then turned by nausea's spin.
    fn view_motion(
        &self,
        player: &photon_world::local_player::LocalPlayer,
        partial: f32,
    ) -> [[f32; 4]; 4] {
        let motion = player_motion(player, partial);
        let nausea = crate::world::fog::nausea_blend(&self.session, partial);
        let portal = player.portal_effect_intensity(partial);
        match self.camera.spin((portal, nausea), partial) {
            Some(spin) => photon_render::scene::multiply(motion, spin),
            None => motion,
        }
    }
}

/// What the view is moved by: `GameRenderer.bobHurt` first, then `bobView`, the view tilting away
/// from a blow and swaying as the player walks.
fn player_motion(player: &photon_world::local_player::LocalPlayer, partial: f32) -> [[f32; 4]; 4] {
    // The view bob option's default is on.
    let (walked, bob) = player.view_bob(partial);
    let bob = photon_render::camera::view_bob(walked, bob);
    let (hurt_time, hurt_duration, hurt_dir, dying, death_time) = player.hurt_view(partial);
    let hurt = photon_render::camera::view_hurt(&photon_render::camera::HurtView {
        hurt_time,
        hurt_duration,
        hurt_dir,
        dying,
        death_time,
        // `damageTiltStrength`'s default: the option is not kept.
        tilt_strength: 1.0,
    });
    photon_render::scene::multiply(hurt, bob)
}
