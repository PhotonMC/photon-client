use super::glints::set_item_glints;
use super::*;
use crate::objects::leashes;
use crate::world::sections;

impl Play {
    /// The listener of the sounds goes where the camera is, and what the sounds told since the
    /// last frame goes to the subtitles.
    fn place_listener(&mut self, frame: &FrameCamera) {
        self.sound.reload_for_device(&self.controls.options);
        self.sound.track_device(&self.controls.options);
        self.sound.play_ui_queue();
        if let Some(sound) = self.sound.as_ref() {
            let (forward, _, up) = crate::view::camera_vectors(frame.rotation);
            sound.set_listener(frame.camera, forward, up);
        }
        let song = self
            .sound
            .as_ref()
            .and_then(|sound| self.music.current_song(sound));
        let pending = self.session.take_pending_toasts();
        self.gui
            .add_pending_toasts(pending, self.controls.options.notification_display_time);
        self.social_interactions_toast();
        self.gui
            .feed_sounds(self.sound.as_mut(), &self.controls.options, song);
    }

    /// `EntityRenderer.getPackedLightCoords` of the local player: a burning one is lit by full
    /// block light.
    fn player_light(&self, frame: &FrameCamera) -> i32 {
        let light = packed_light(self.session.world(), frame.eye);
        let burning = self
            .session
            .world()
            .local_player()
            .is_some_and(photon_world::ClientPlayerState::is_on_fire);
        if burning {
            crate::overlays::full_block_light(light)
        } else {
            light
        }
    }

    /// What the frame's overlays are drawn from.
    fn frame_view(&self, frame: &FrameCamera) -> FrameView {
        let options = &self.controls.options;
        let spectating = self.session.game_mode().is_spectator();
        crate::mobs::outline::set_spectator_outlines(
            spectating && self.controls.spectator_outlines_held,
        );
        FrameView {
            render_distance: frame.render_distance,
            partial: frame.partial,
            light: self.player_light(frame),
            ambient_occlusion: options.ambient_occlusion,
            origin: self.origin,
            camera: frame.relative,
            rotation: frame.rotation,
            skin: self.skin_rect,
            cape: self.skins.own_cape,
            elytra: self.skins.own_elytra,
            view_bobbing: options.view_bobbing,
            damage_tilt: options.damage_tilt_strength,
            spectating,
        }
    }

    /// What is drawn over the blocks from the player's side: the hands in
    /// first person unless the player only watches, the aimed block's
    /// outline, the cracks, and the player's own model once the camera
    /// leaves its eyes.
    pub(super) fn draw_overlays(&mut self, frame: &FrameCamera) -> Result<()> {
        self.apply_glint_options();
        self.begin_skins();
        let view = self.frame_view(frame);
        self.place_listener(frame);
        let first_person = self.controls.camera_type == CameraType::FirstPerson;
        self.set_hands(view, first_person)?;
        self.set_debug_gizmo(frame, first_person)?;
        let player = self.session.player().ok_or("no local player")?;
        let line_width =
            photon_render::block::outline::appropriate_line_width(self.backend.extent().0);
        let (hook_sprites, hook_lines) = self.fishing_hooks(player, view, line_width);
        let (lines, backing) =
            self.world_lines((player, view, frame.relative), (line_width, hook_lines));
        self.backend.set_outline(&lines, backing)?;
        self.backend.set_breaking(&cracks(
            &self.session,
            &self.block_renderer,
            frame.camera,
            view,
        ))?;
        let spectating = player.spectating();
        // `LevelExtractor` draws the local player only while it is the camera entity.
        let local = (!first_person
            && !spectating
            && !self.local_invisible()
            && self.session.camera_entity().is_none())
        .then(|| player.interpolated_position(frame.partial));
        let held = self.draw_entities(view, first_person, hook_sprites)?;
        self.set_billboards(frame, (spectating, local), view)?;
        let tints = (&*self.block_renderer, &*self.biome_tints);
        let with_textures = (tints.0, tints.1, &self.entity_textures);
        set_item_glints(
            &self.glint_frame(),
            (&mut self.item_layers, &mut self.worn_layers),
            &mut self.gui.items,
        );
        let items = &mut self.item_vertices;
        items.clear();
        let pickups: Vec<_> = self.particles.pickups().collect();
        world_items(
            (&self.session, &pickups),
            &mut self.item_layers,
            with_textures,
            view,
            items,
        );
        if !first_person {
            crate::objects::flames::local_flames(
                &self.session,
                (&self.block_renderer, view),
                items,
            );
        }
        items.extend(held_item_vertices(&held, &mut self.worn_layers, tints));
        self.add_world_heads()?;
        self.hand_over_layers(view)
    }

    /// The hands in first person unless the player only watches, the screen effects, and the
    /// totem animation in any view, all drawn in the hand pass.
    /// `CameraRenderState.isSleeping`: whether the camera entity sleeps.
    fn camera_sleeping(&self) -> bool {
        let world = self.session.world();
        match self.session.camera_entity() {
            Some(id) => world
                .entities()
                .at(id)
                .is_some_and(|entity| entity.is_sleeping()),
            None => world
                .local_player()
                .is_some_and(|state| state.is_sleeping()),
        }
    }

    fn set_hands(&mut self, view: FrameView, first_person: bool) -> Result<()> {
        let glint = self.glint_frame();
        let (activation, activation_glint) = self.activation_vertices(&glint);
        for cache in &mut self.held_item_layers {
            cache.set_glint(&glint);
        }
        let sleeping = self.camera_sleeping();
        let watching = self
            .session
            .player()
            .is_some_and(|player| player.spectating());
        // `GameRenderer.renderItemInHand` and `ScreenEffectRenderer.submit`: nothing of the hands
        // or the view-blocking effects for a sleeper, and no hands with the HUD hidden.
        let hands = (first_person && !watching && !sleeping).then(|| self.first_person_hands(view));
        let player = self.session.player().ok_or("no local player")?;
        if let Some(overlays::HandVertices {
            hand,
            mut held,
            mut glint,
        }) = hands
        {
            let hand_projection = photon_render::scene::perspective(
                hud_fov(player, self.controls.options.fov_effect_scale as f32).to_radians(),
                self.aspect,
                0.05,
                100.0,
            );
            held.extend(activation);
            glint.extend(activation_glint);
            self.backend
                .set_hand(&hand, (&held, &glint), hand_projection)?;
            let darken =
                photon_world::particles::Ambience::sky_darken(&self.sky.ambience(&self.session));
            let effects = overlays::screen_effect::screen_effects(
                &self.session,
                player,
                (&self.block_renderer, darken),
            );
            self.backend.set_screen_effects(
                (&effects.block, &effects.water, &effects.fire),
                hand_projection,
            )?;
        } else {
            let hand_projection = photon_render::scene::perspective(
                hud_fov(player, self.controls.options.fov_effect_scale as f32).to_radians(),
                self.aspect,
                0.05,
                100.0,
            );
            self.backend
                .set_hand(&[], (&activation, &activation_glint), hand_projection)?;
            self.backend
                .set_screen_effects((&[], &[], &[]), [[0.0; 4]; 4])?;
        }
        Ok(())
    }

    /// The other players, the mobs and the objects the server added, the
    /// bobbers, and the local player's own model once the camera leaves its
    /// eyes (an invisible one keeps what it wears and holds); what they hold is given back for the item pass.
    fn draw_entities(
        &mut self,
        view: overlays::FrameView,
        first_person: bool,
        hook_sprites: Vec<photon_render::scene::Vertex>,
    ) -> Result<Vec<crate::equipment::held_items::HeldItem>> {
        self.session.player().ok_or("no local player")?;
        self.resolve_head_skins(view);
        overlays::forget_gone_mannequins(self.session.world());
        overlays::set_local_layers(self.session.world());
        overlays::set_local_main_arm(self.session.main_arm_is_left());
        let (mut held, invisible) = (Vec::new(), self.local_invisible());
        let models = &mut self.entity_vertices;
        models.clear();
        overlays::remote_players(
            &self.session,
            (&mut self.remote_models, &mut self.armor),
            (&mut self.entity_textures, &mut self.skins),
            (view, &mut held),
            models,
        );
        mobs::remote_mobs(
            &self.session,
            (&mut self.mob_models, &mut self.armor),
            &self.entity_textures,
            (view, &mut held),
            models,
        );
        models.extend(hook_sprites);
        models.extend(
            self.particles
                .elder_guardian_vertices(&self.entity_textures, (view.partial as f32, &view)),
        );
        if !first_person {
            let own = self.own_model(view, &mut held, invisible)?;
            self.entity_vertices.extend(own);
        }
        self.upload_skins(&mut held)?;
        self.backend.set_draw(
            photon_render::vulkan::DrawKind::Entities,
            &self.entity_vertices,
        )?;
        Ok(held)
    }

    /// The fishing bobbers' sprites and lines for the frame.
    pub(super) fn fishing_hooks(
        &self,
        player: &photon_world::local_player::LocalPlayer,
        view: overlays::FrameView,
        line_width: f32,
    ) -> (
        Vec<photon_render::scene::Vertex>,
        Vec<photon_render::block::outline::LineVertex>,
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
        (spectating, local): (bool, Option<photon_core::math::Vec3>),
        view: overlays::FrameView,
    ) -> Result<()> {
        let camera = overlays::TagCamera {
            position: frame.camera,
            relative: frame.relative,
            rotation: frame.rotation,
            spectating,
            hud_hidden: self.controls.hud_hidden,
        };
        let (see_through, normal) = self.world_texts(&camera, view);
        let mut normal = normal;
        // The leads are a flat colour lit by the lightmap, as the world text is.
        leashes::leashes(&self.session, view, &mut normal);
        self.backend.set_name_tags(&see_through, &normal)?;
        let mut shadows = Vec::new();
        overlays::entity_shadows(
            &self.session,
            &self.entity_textures,
            &camera,
            local,
            view,
            &mut shadows,
        );
        self.backend
            .set_draw(photon_render::vulkan::DrawKind::Shadows, &shadows)?;
        let particle_camera = photon_render::particles::ParticleCamera {
            orientation: photon_render::entity::name_tag::camera_orientation(
                frame.rotation.0,
                frame.rotation.1,
            ),
            position: frame.camera,
        };
        let (particles, flakes) = self.particles.vertices(
            self.session.world(),
            &particle_camera,
            (&frame.frustum, frame.relative, frame.partial as f32),
        );
        self.backend
            .set_draw(photon_render::vulkan::DrawKind::Particles, &particles)?;
        self.backend
            .set_draw(photon_render::vulkan::DrawKind::BlockParticles, &flakes)?;
        let mut orbs = Vec::new();
        let pickups: Vec<_> = self.particles.pickups().collect();
        overlays::experience_orbs(
            (&self.session, &pickups),
            &self.entity_textures,
            &camera,
            view,
            &mut orbs,
        );
        self.backend
            .set_draw(photon_render::vulkan::DrawKind::ExperienceOrbs, &orbs)?;
        Ok(())
    }

    /// Where the sky is seen from this frame, with the options its lightmap reads.
    fn sky_camera(&self, frame: &FrameCamera, frame_ticks: f32) -> SkyCamera {
        let options = &self.controls.options;
        SkyCamera {
            eye: frame.eye,
            rotation: frame.rotation,
            view_projection: frame.sky_view_projection,
            render_distance: frame.render_distance,
            fog: frame.fog,
            partial: frame.partial,
            frame_ticks,
            origin: self.origin,
            lightmap_options: (options.darkness_effect_scale as f32, options.gamma as f32),
            hide_flashes: options.hide_lightning_flashes,
        }
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
        self.particles.set_camera(frame.camera);
        self.leave_out_of_view(&frame);
        profile::timed(Span::Overlays, || self.draw_overlays(&frame))?;
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
        let sky_camera = self.sky_camera(&frame, frame_ticks);
        let colour = self.sky.draw(
            &mut self.backend,
            &self.session,
            &self.biome_tints,
            &sky_camera,
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
            (
                &mut self.player_model,
                frame.locator,
                (&mut self.mob_models, &mut self.armor, &mut self.worn_layers),
                &self.entity_textures,
            ),
        )?;
        let built = Instant::now();
        if std::mem::take(&mut self.controls.screenshot_pending) {
            let captured = self.backend.capture_scene(colour, frame.view_projection)?;
            self.save_screenshot(&captured);
        } else {
            self.backend.draw_scene(colour, frame.view_projection)?;
        }
        self.frames += 1;
        let parts = profile::take_parts();
        if self.profile.is_some() || self.bench.is_some() {
            let drawn = Instant::now();
            let spans = [frame_start, ticked, meshed, built, drawn];
            if let Some(profile) = self.profile.as_mut() {
                profile.record(spans, parts, &mut self.sections);
            }
            if let Some(bench) = self.bench.as_mut() {
                bench.record(spans, self.limiter.sampled_at(), parts, &self.backend);
            }
        }
        Ok(Flow::Next)
    }

    /// `Screenshot.grab`: the frame goes to `screenshots/`, and the chat says where.
    fn save_screenshot(&mut self, frame: &photon_render::CapturedFrame) {
        use photon_game::text::{ClickEvent, Component, Style};
        let saved = crate::screenshot::save(
            &crate::controls::game_directory(),
            (frame.width, frame.height, &frame.rgba),
            crate::screenshot::DateTime::now(),
        );
        let message = match saved {
            Ok(file) => {
                let name = file
                    .file_name()
                    .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
                let absolute = file.canonicalize().unwrap_or(file);
                let link = Component::literal(name).with_style(&Style {
                    underlined: Some(true),
                    click: Some(Box::new(ClickEvent::OpenFile(
                        absolute.to_string_lossy().into_owned(),
                    ))),
                    ..Style::default()
                });
                Component::translatable_with("screenshot.success", vec![link])
            }
            Err(error) => {
                log::warn!("Couldn't save screenshot: {error}");
                Component::translatable_with(
                    "screenshot.failure",
                    vec![Component::literal(error.to_string())],
                )
            }
        };
        let focused = self.controls.chat.is_some();
        self.gui
            .client_message(&mut self.controls.chat_log, &message, focused);
    }

    /// With a frame count, the frames after the script ends are captured
    /// one per tick into numbered files; then the run reports where the
    /// player ended, captures `PHOTON_CAPTURE_PPM` and stops.
    pub(super) fn finish_script(&mut self, colour: [f32; 4], frame: &FrameCamera) -> Result<Flow> {
        let capture = env::var("PHOTON_CAPTURE_PPM");
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
                validation: photon_render::validation_findings(),
                memory: self.backend.section_memory(),
                present_mode: format!("{:?}", self.backend.description().present_mode),
                render_mode: crate::settings::Settings::get().render.name(),
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
