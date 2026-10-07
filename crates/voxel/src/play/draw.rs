use super::*;
use crate::objects::displays;
use crate::objects::leashes;
use crate::world::sections;

impl Play {
    /// `Camera.alignWithEntity` and the projection: the front view turns
    /// round, and a detached camera backs away from the eyes until a block
    /// is in the way.
    pub(super) fn frame_camera(
        &self,
        player: &voxel_world::local_player::LocalPlayer,
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
        let eye_in_water = player.is_eye_in_water();
        let option_fov = self.controls.options.fov;
        let fov = self.camera.fov(option_fov, partial as f32, eye_in_water);
        // The camera's far plane: four times the render distance, or the
        // cloud range if that is farther.
        let render_distance = self.session.view_distance().clamp(2, 12);
        let far = ((render_distance * 16 * 4) as f32).max((CLOUD_RANGE * 16) as f32);
        // The view bob option's default is on.
        let (walked, bob) = player.view_bob(partial as f32);
        let bob = voxel_render::camera::view_bob(walked, bob);
        // `GameRenderer.bobHurt` first, then `bobView`: the view tilts away from a blow.
        let (hurt_time, hurt_duration, hurt_dir, dying, death_time) =
            player.hurt_view(partial as f32);
        let hurt = voxel_render::camera::view_hurt(&voxel_render::camera::HurtView {
            hurt_time,
            hurt_duration,
            hurt_dir,
            dying,
            death_time,
            // `damageTiltStrength`'s default: the option is not kept.
            tilt_strength: 1.0,
        });
        let bob = voxel_render::scene::multiply(hurt, bob);
        let locator = voxel_render::gui::hud::locator::LocatorCamera {
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
            eye_in_water,
            // The field of view never culls narrower than the option.
            frustum: voxel_render::visibility::Frustum::new(
                &crate::view::view_rotation(y_rot, x_rot),
                &voxel_render::visibility::culling_projection(
                    fov.max(option_fov as f32),
                    self.aspect,
                    far,
                ),
                [camera.x, camera.y, camera.z],
            ),
        }
    }

    /// What is drawn over the blocks from the player's side: the hands in
    /// first person unless the player only watches, the aimed block's
    /// outline, the cracks, and the player's own model once the camera
    /// leaves its eyes.
    pub(super) fn draw_overlays(&mut self, frame: &FrameCamera) -> Result<()> {
        let player = self.session.player().ok_or("no local player")?;
        let view = FrameView {
            partial: frame.partial,
            light: packed_light(self.session.world(), frame.eye),
            origin: self.origin,
            camera: frame.relative,
            rotation: frame.rotation,
            skin: self.skin_rect,
        };
        if let Some(sound) = &self.sound {
            let (forward, _, up) = crate::view::camera_vectors(frame.rotation);
            sound.set_listener(frame.camera, forward, up);
        }
        let first_person = self.controls.camera_type == CameraType::FirstPerson;
        if first_person && !player.spectating() {
            let (hand, held) = hands(
                player,
                &self.item_in_hand,
                &mut self.held_item_layers,
                (&self.block_renderer, &self.biome_tints),
                &mut self.player_model,
                view,
            );
            let hand_projection = voxel_render::scene::perspective(
                hud_fov(player).to_radians(),
                self.aspect,
                0.05,
                100.0,
            );
            self.backend.set_hand(&hand, &held, hand_projection)?;
        } else {
            self.backend.set_hand(&[], &[], [[0.0; 4]; 4])?;
        }
        let line_width =
            voxel_render::block::outline::appropriate_line_width(self.backend.extent().0);
        let (hook_sprites, hook_lines) = self.fishing_hooks(player, view, line_width);
        let mut lines = outline(&self.session, player, view, frame.relative, line_width);
        lines.extend(hook_lines);
        self.backend.set_outline(&lines)?;
        self.backend.set_breaking(&cracks(
            &self.session,
            &self.block_renderer,
            frame.camera,
            view,
        ))?;
        // The other players, and the local player's own once the camera
        // leaves its eyes.
        let mut held = Vec::new();
        let mut models = overlays::remote_players(
            &self.session,
            (&mut self.remote_models, &mut self.armor),
            &self.entity_textures,
            (view, &mut held),
        );
        models.extend(mobs::remote_mobs(
            &self.session,
            (&mut self.mob_models, &mut self.armor),
            &self.entity_textures,
            (view, &mut held),
        ));
        models.extend(hook_sprites);
        if !first_person {
            // What the local player holds: the selected hotbar slot and the
            // off hand, the main arm being the right.
            let stack = |slot: i16| {
                self.session
                    .world()
                    .local_player()
                    .and_then(|state| state.inventory_slot(slot))
                    .filter(|stack| stack.item_count() > 0)
            };
            let selected = self.session.status().selected_slot as i16;
            models.extend(overlays::player_model(
                (&self.session, player),
                &mut self.player_model,
                (view, &mut held),
                (stack(36 + selected), stack(45)),
                (
                    &mut self.armor,
                    Worn {
                        head: stack(5),
                        chest: stack(6),
                        legs: stack(7),
                        feet: stack(8),
                    },
                    &self.entity_textures,
                ),
            ));
        }
        self.backend
            .set_draw(voxel_render::vulkan::DrawKind::Entities, &models)?;
        mobs::hand_layers(&mut self.mob_models, &mut self.backend, &mut self.armor)?;
        let local = (!first_person && !player.spectating())
            .then(|| player.interpolated_position(frame.partial));
        self.set_billboards(frame, (player.spectating(), local), view)?;
        let tints = (&*self.block_renderer, &*self.biome_tints);
        let with_textures = (tints.0, tints.1, &self.entity_textures);
        let mut items = world_items(&self.session, &mut self.item_layers, with_textures, view);
        items.extend(held_item_vertices(&held, &mut self.worn_layers, tints));
        self.item_layers.hand_over(&mut self.backend, &items)?;
        set_moving_blocks(&mut self.backend, &self.session, tints, view)
    }

    /// The fishing bobbers' sprites and lines for the frame.
    pub(super) fn fishing_hooks(
        &self,
        player: &voxel_world::local_player::LocalPlayer,
        view: overlays::FrameView,
        line_width: f32,
    ) -> (
        Vec<voxel_render::scene::Vertex>,
        Vec<voxel_render::block::outline::LineVertex>,
    ) {
        let selected = self.session.status().selected_slot as i16;
        let main_hand = self
            .session
            .world()
            .local_player()
            .and_then(|state| state.inventory_slot(36 + selected))
            .filter(|stack| stack.item_count() > 0);
        fishing_hooks(
            &self.session,
            (player, &self.entity_textures),
            &HookView {
                view,
                camera_type: self.controls.camera_type,
                option_fov: self.controls.options.fov,
                aspect: self.aspect,
                line_width,
                main_hand,
            },
        )
    }

    /// What the world draws as flat shapes over the entities: the shadows
    /// under them, the name tags above players and the experience orbs.
    pub(super) fn set_billboards(
        &mut self,
        frame: &FrameCamera,
        (spectating, local): (bool, Option<voxel_core::math::Vec3>),
        view: overlays::FrameView,
    ) -> Result<()> {
        let camera = overlays::TagCamera {
            position: frame.camera,
            relative: frame.relative,
            rotation: frame.rotation,
            spectating,
        };
        let (mut see_through, mut normal) =
            overlays::name_tags(&self.session, self.gui.text_assets(), &camera, view);
        let (font, atlas) = self.gui.text_assets();
        let (shown_through, shown) =
            displays::display_texts(&self.session, (self.gui.lang(), font, atlas), view);
        see_through.extend(shown_through);
        normal.extend(shown);
        // The leads are a flat colour lit by the lightmap, as the world text is.
        normal.extend(leashes::leashes(&self.session, view));
        self.backend.set_name_tags(&see_through, &normal)?;
        self.backend.set_draw(
            voxel_render::vulkan::DrawKind::Shadows,
            &overlays::entity_shadows(&self.session, &self.entity_textures, &camera, local, view),
        )?;
        self.backend.set_draw(
            voxel_render::vulkan::DrawKind::ExperienceOrbs,
            &overlays::experience_orbs(&self.session, &self.entity_textures, &camera, view),
        )?;
        Ok(())
    }

    /// One frame: the overlays, the sections, the atlas, the sky, then the
    /// GUI and the scene. A finished script captures its frames and stops.
    pub(super) fn draw(
        &mut self,
        [frame_start, ticked]: [Instant; 2],
        frame_ticks: f32,
        script_done: bool,
    ) -> Result<Flow> {
        // While a respawn waits for its position there is nothing to draw.
        let Some(player) = self.session.player().filter(|_| self.session.is_loaded()) else {
            std::thread::sleep(Duration::from_millis(5));
            return Ok(Flow::Next);
        };
        let frame = self.frame_camera(player);
        self.draw_overlays(&frame)?;
        // Drop the sections whose chunk went away, then mesh the dirty ones.
        self.sections
            .forget_chunks(self.session.take_forgotten_chunks(), &mut self.backend)?;
        self.sections.update(
            &mut self.session,
            &mut self.backend,
            &sections::SectionView {
                camera: [frame.camera.x, frame.camera.y, frame.camera.z],
                frustum: frame.frustum,
                view_distance: frame.render_distance,
                origin: self.origin,
            },
            script_done,
        )?;
        let meshed = Instant::now();
        self.backend.set_sort_origin(frame.relative);
        self.backend
            .update_atlas(&self.animated_atlas.changed_regions())?;
        let colour = self.sky.draw(
            &mut self.backend,
            &self.session,
            &self.biome_tints,
            &SkyCamera {
                eye: frame.eye,
                rotation: frame.rotation,
                view_projection: frame.sky_view_projection,
                render_distance: frame.render_distance,
                eye_in_water: frame.eye_in_water,
                partial: frame.partial,
                frame_ticks,
                origin: self.origin,
            },
        )?;
        if script_done {
            return self.finish_script(colour, &frame);
        }
        self.gui.draw(
            &mut self.backend,
            &self.surface,
            &mut self.session,
            &mut self.controls,
            (&self.block_renderer, &self.biome_tints),
            (&mut self.player_model, frame.locator),
        )?;
        let built = Instant::now();
        self.backend.draw_scene(colour, frame.view_projection)?;
        self.frames += 1;
        if self.profile.is_some() || self.bench.is_some() {
            let drawn = Instant::now();
            let spans = [frame_start, ticked, meshed, built, drawn];
            if let Some(profile) = self.profile.as_mut() {
                profile.record(spans, &mut self.sections);
            }
            if let Some(bench) = self.bench.as_mut() {
                bench.record(spans, &self.backend);
            }
        }
        Ok(Flow::Next)
    }

    /// With a frame count, the frames after the script ends are captured
    /// one per tick into numbered files; then the run reports where the
    /// player ended, captures `VOXEL_CAPTURE_PPM` and stops.
    pub(super) fn finish_script(&mut self, colour: [f32; 4], frame: &FrameCamera) -> Result<Flow> {
        let capture = env::var("VOXEL_CAPTURE_PPM");
        if self.captured_frames < self.capture_frames {
            if let Ok(path) = capture {
                let captured = self.backend.capture_scene(colour, frame.view_projection)?;
                let numbered = format!("{path}.{:03}.ppm", self.captured_frames);
                ppm::write(&numbered, captured.width, captured.height, &captured.rgba)?;
                println!("captured_frame={numbered}");
            }
            self.captured_frames += 1;
            return Ok(Flow::Next);
        }
        report_end(&self.session, self.ticks, frame.rotation)?;
        if let Some(bench) = &self.bench {
            let extent = self.backend.extent();
            bench.write(&RunInfo {
                device: self.backend.description().device_name.clone(),
                extent,
                render_distance: frame.render_distance,
                loaded_chunks: self.session.world().loaded_chunk_count(),
                entities: self.session.entity_count(),
                validation: voxel_render::validation_findings(),
                memory: self.backend.section_memory(),
                present_mode: format!("{:?}", self.backend.description().present_mode),
                framerate_limit: self.controls.options.framerate_limit,
            })?;
        }
        if let Ok(path) = capture {
            let captured = self.backend.capture_scene(colour, frame.view_projection)?;
            ppm::write(&path, captured.width, captured.height, &captured.rgba)?;
            println!("captured_frame={path}");
        }
        Ok(Flow::Stop)
    }
}
