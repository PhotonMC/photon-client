//! Where a frame is seen from (`Camera`): the camera's place, its field of view and what it is in.

use super::*;

/// How far the player may be from the meshes' origin before it moves, in blocks.
const ORIGIN_RANGE: f64 = 4096.0;

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

    /// The eyes of the camera's entity: its feet and the smoothed eye height above them.
    fn eye_at(&self, feet: Vec3, partial: f64) -> Vec3 {
        Vec3::new(
            feet.x,
            feet.y + f64::from(self.camera.eye_height(partial as f32)),
            feet.z,
        )
    }

    /// Where the camera entity stands and which way it looks (`Camera.alignWithEntity`): the
    /// player, or the entity a spectator was handed to (`getViewYRot`: a living one looks where
    /// its head turns).
    fn camera_entity_pose(
        &self,
        player: &photon_world::local_player::LocalPlayer,
        partial: f64,
    ) -> (Vec3, (f32, f32)) {
        let world = self.session.world();
        if let Some(id) = self.session.camera_entity()
            && let Some(entity) = world.entities().at(id)
            && let Some(snapshot) = world.entity_snapshot(id, partial as f32)
        {
            let y_rot = if entity.is_living_entity() {
                snapshot.head_yaw
            } else {
                snapshot.yaw
            };
            let (x, y, z) = snapshot.position;
            return (Vec3::new(x, y, z), (y_rot, snapshot.pitch));
        }
        (player.interpolated_position(partial), player.rotation())
    }

    /// `Camera.alignWithEntity` for a sleeper in first person: the view turns to the foot of the
    /// bed and the eyes lift; the rotation it looks along, with `eye` moved.
    fn sleeping_view(
        &self,
        camera_type: CameraType,
        eye: &mut Vec3,
        rotation: (f32, f32),
    ) -> (f32, f32) {
        let world = self.session.world();
        let bed = (camera_type == CameraType::FirstPerson
            && self.session.camera_entity().is_none())
        .then(|| world.local_player().and_then(|state| state.sleeping_pos()))
        .flatten();
        let Some(bed) = bed else {
            return rotation;
        };
        eye.y += f64::from(0.15_f32);
        let facing = photon_world::block::bed::bed_orientation(world, (bed.x, bed.y, bed.z));
        (facing.map_or(0.0, |facing| facing.to_y_rot() - 180.0), 0.0)
    }

    /// The vertical field of view: the option's, the modifier, and the narrowing in a fluid
    /// (`Camera.getFluidInCamera`), with that fluid.
    fn view_fov(
        &self,
        camera: Vec3,
        rotation: (f32, f32),
        partial: f32,
    ) -> (f32, photon_render::sky::FogType) {
        let option_fov = self.controls.options.fov;
        let fluid = crate::view::camera_fluid(
            self.session.world(),
            camera,
            rotation,
            (option_fov, f64::from(self.aspect)),
        );
        let scale = fluid
            .narrows_view()
            .then_some(self.controls.options.fov_effect_scale as f32);
        (self.camera.fov(option_fov, partial, scale), fluid)
    }

    /// Meshes are measured from a fixed block so their f32 coordinates stay small; once the
    /// player is far from it (a teleport, another dimension, a long walk) the point moves to
    /// the player and the sections are built again around it.
    pub(super) fn recentre_origin(&mut self) -> Result<()> {
        let Some(player) = self.session.player() else {
            return Ok(());
        };
        let at = player.position();
        let far = [at.x, at.y, at.z]
            .iter()
            .zip(self.origin)
            .any(|(at, origin)| (at - f64::from(origin)).abs() > ORIGIN_RANGE);
        if far {
            self.origin = [
                at.x.floor() as i32,
                at.y.floor() as i32,
                at.z.floor() as i32,
            ];
            self.sections.rebuild_all(&mut self.backend)?;
        }
        Ok(())
    }

    /// `Camera.alignWithEntity` and the projection: the front view turns
    /// round, and a detached camera backs away from the eyes until a block
    /// is in the way.
    pub(super) fn frame_camera(
        &self,
        player: &photon_world::local_player::LocalPlayer,
    ) -> FrameCamera {
        let partial = self.accumulated.as_secs_f64() / TICK_LENGTH.as_secs_f64();
        let (feet, (player_y_rot, player_x_rot)) = self.camera_entity_pose(player, partial);
        let camera_type = self.controls.camera_type;
        let (y_rot, x_rot) = if camera_type == CameraType::ThirdPersonFront {
            (player_y_rot + 180.0, -player_x_rot)
        } else {
            (player_y_rot, player_x_rot)
        };
        let mut eye = self.eye_at(feet, partial);
        let (y_rot, x_rot) = self.sleeping_view(camera_type, &mut eye, (y_rot, x_rot));
        let camera = camera_position(self.session.world(), camera_type, eye, (y_rot, x_rot));
        let relative = [
            (camera.x - f64::from(self.origin[0])) as f32,
            (camera.y - f64::from(self.origin[1])) as f32,
            (camera.z - f64::from(self.origin[2])) as f32,
        ];
        let option_fov = self.controls.options.fov;
        let (fov, fluid) = self.view_fov(camera, (y_rot, x_rot), partial as f32);
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
                (fluid, camera.y, self.boss_fog(partial as f32)),
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

    /// Whether the local player is invisible: its model and shadow are then not drawn in third
    /// person (what it wears and holds goes with them here).
    pub(super) fn local_invisible(&self) -> bool {
        self.session
            .world()
            .local_player()
            .is_some_and(|state| state.is_invisible())
    }
}

impl Play {
    /// The boss bars' darkening of the world between two ticks, and whether one makes world fog.
    fn boss_fog(&self, partial: f32) -> (f32, bool) {
        (
            self.camera.boss.at(partial),
            self.gui.boss_bars.creates_fog(),
        )
    }

    /// What the view is moved by, then turned by nausea's spin.
    fn view_motion(
        &self,
        player: &photon_world::local_player::LocalPlayer,
        partial: f32,
    ) -> [[f32; 4]; 4] {
        let options = &self.controls.options;
        // Only the player's own avatar sways the view and flinches it here.
        if self.session.camera_entity().is_some() {
            return photon_render::scene::multiply(
                photon_render::camera::view_hurt(&photon_render::camera::HurtView::none()),
                photon_render::camera::view_bob(0.0, 0.0),
            );
        }
        let motion = player_motion(
            player,
            partial,
            (options.view_bobbing, options.damage_tilt_strength),
        );
        let nausea = crate::world::fog::nausea_blend(&self.session, partial);
        let portal = player.portal_effect_intensity(partial);
        let scale = self.controls.options.screen_effect_scale as f32;
        match self.camera.spin((portal, nausea), (scale, partial)) {
            Some(spin) => photon_render::scene::multiply(motion, spin),
            None => motion,
        }
    }
}

/// What the view is moved by: `GameRenderer.bobHurt` first, then `bobView`, the view tilting away
/// from a blow and swaying as the player walks.
fn player_motion(
    player: &photon_world::local_player::LocalPlayer,
    partial: f32,
    (view_bobbing, damage_tilt): (bool, f64),
) -> [[f32; 4]; 4] {
    let (walked, bob) = player.view_bob(partial);
    // `bobView` off leaves the view without the sway.
    let bob = if view_bobbing { bob } else { 0.0 };
    let bob = photon_render::camera::view_bob(walked, bob);
    let (hurt_time, hurt_duration, hurt_dir, dying, death_time) = player.hurt_view(partial);
    let hurt = photon_render::camera::view_hurt(&photon_render::camera::HurtView {
        hurt_time,
        hurt_duration,
        hurt_dir,
        dying,
        death_time,
        tilt_strength: damage_tilt,
    });
    photon_render::scene::multiply(hurt, bob)
}

/// `Entity.getEyeHeight` of the camera entity: the player's, or the standing height of the entity
/// a spectator was handed to.
fn camera_eye_height(
    session: &photon_net::connection::PlaySession,
    player: &photon_world::local_player::LocalPlayer,
) -> f32 {
    session
        .camera_entity()
        .and_then(|id| session.world().entities().at(id))
        .and_then(photon_render::entity::tag_rules::eye_height)
        .map_or_else(|| player.eye_height(), |height| height as f32)
}

/// One tick of what follows the player's eyes: the camera, and the HUD with the brightness of the
/// light there (`getMaxLocalRawBrightness` with the sky's dimming, through the dimension's ambient
/// light).
pub(super) fn tick_view(
    (camera, gui): (&mut photon_render::camera::Camera, &mut crate::gui::Gui),
    session: &photon_net::connection::PlaySession,
    (sky, controls): (&crate::world::sky::Sky, &crate::controls::Controls),
) {
    let fov_effect_scale = controls.options.fov_effect_scale as f32;
    let first_person = controls.camera_type == CameraType::FirstPerson;
    // `GameRenderer.tick`: the boss darkening and the totem run only while the level does.
    if session.runs_normally() {
        camera.boss.tick(gui.boss_bars.darkens_screen());
        camera.activation.tick();
    }
    let eyes = session.player().map(|player| {
        camera.tick(
            player,
            (
                crate::world::fog::nausea_blend(session, 1.0),
                fov_effect_scale,
            ),
            (first_person, camera_eye_height(session, player)),
        );
        let at = player.position();
        [at.x, at.y + f64::from(player.eye_height()), at.z]
    });
    let darken = photon_world::particles::Ambience::sky_darken(&sky.ambience(session));
    gui.tick(eyes.map(|point| session.world().eye_brightness(point, darken)));
}
