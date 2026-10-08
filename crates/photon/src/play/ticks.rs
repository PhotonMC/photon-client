use super::*;

impl Play {
    /// Join, open the window, and load everything the frames draw.
    #[expect(
        clippy::too_many_lines,
        reason = "start-up builds each part of the session in turn"
    )]
    pub(super) fn start(client: Client, mut session: PlaySession) -> Result<Self> {
        let Client {
            mut surface,
            mut backend,
            size,
            block_renderer,
            animated_atlas,
        } = client;
        let root =
            env::var("PHOTON_ASSETS").map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        let script = env::var("PHOTON_SCRIPT")
            .ok()
            .map(|text| Script::parse(&text))
            .transpose()?;

        let mut particles = crate::particles::Particles::load(&root);
        let position = report_spawn(&session)?;
        // The vanilla data pack the server runs, which sits next to the
        // assets: the biomes' colours and the sky's data come from it.
        let data_root = env::var("PHOTON_DATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::path::Path::new(&root).with_file_name("data"));
        let biome_tints = Arc::new(
            BiomeTints::load(
                &data_root,
                std::path::Path::new(&root),
                session.registry_names("minecraft:worldgen/biome"),
                session.hashed_seed(),
            )
            .ok_or("the biome colours could not be read")?,
        );
        particles.set_terrain(Arc::clone(&block_renderer), Arc::clone(&biome_tints));
        let sky_sprites = sky_sprites(&block_renderer);
        let origin = [
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        ];
        // The chunk fade option, 0.75 seconds by default and
        // `PHOTON_CHUNK_FADE` seconds if given; a scripted run shows every
        // section at once unless asked, so its captures do not catch one
        // mid-fade.
        let chunk_fade = match env::var("PHOTON_CHUNK_FADE") {
            Ok(seconds) => Duration::from_secs_f64(seconds.parse::<f64>()?.clamp(0.0, 2.0)),
            Err(_) if script.is_some() => Duration::ZERO,
            Err(_) => Duration::from_millis(750),
        };
        let sections = SectionMeshes::new(
            &mut session,
            Arc::clone(&block_renderer),
            Arc::clone(&biome_tints),
            chunk_fade,
        );
        let sky = Sky::load(&data_root, &root, &session, sky_sprites)?;
        backend.load_weather_textures(
            &texture(&root, "environment/rain", "rain")?,
            &texture(&root, "environment/snow", "snow")?,
        )?;
        backend.load_crack_texture(&destroy_stages(&root)?)?;
        backend.load_world_border_texture(&texture(&root, "misc/forcefield", "forcefield")?)?;
        // The player's default skin, picked by its profile id as the
        // reference picks one when the profile has no skin of its own; the
        // others the server adds draw in theirs, and the mobs in their own
        // textures, so the entity texture holds them all.
        let skin = photon_render::entity::player_renderer::default_skin(
            session
                .profile_id()
                .unwrap_or(photon_core::binary::Uuid { most: 0, least: 0 }),
        );
        let mut armor = Armor::load(&root);
        armor.set_trim_registries(
            session.registry_names("minecraft:trim_pattern").to_vec(),
            session.registry_names("minecraft:trim_material").to_vec(),
        );
        let entity_textures = photon_render::entity::textures::EntityTextures::build(
            photon_render::entity::player_renderer::skin_textures()
                .chain(mobs::textures())
                .chain([overlays::ORB_TEXTURE, overlays::SHADOW_TEXTURE])
                .map(str::to_owned)
                .chain(villagers::textures())
                .chain(armor.textures())
                .chain(particles.textures().iter().cloned())
                // A texture two layers share is packed once.
                .scan(std::collections::HashSet::new(), |seen, name| {
                    Some(seen.insert(name.clone()).then_some(name))
                })
                .flatten()
                .map(|name| {
                    let image = texture(&root, &name, "entity texture")?;
                    Ok((name, image))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .chain(Armor::trim_images(&root))
                .collect(),
        )
        .ok_or("the entity textures did not fit")?;
        backend.load_entity_texture(&entity_textures.image)?;
        particles.bind_textures(&entity_textures);
        let skin_rect = entity_textures
            .rect(skin.texture)
            .ok_or("the player's skin")?;
        // The registries a tooltip reads, from the data pack, named as the
        // server synchronised them.
        let registries = photon_game::registries::Registries::load(&data_root, |registry| {
            session.registry_names(registry).to_vec()
        });
        session.set_enchantment_traits(photon_game::menus::container::EnchantmentTraits::of(
            &registries,
        ));
        // `PHOTON_CAMERA=back` or `front` starts detached.
        let camera_type = match env::var("PHOTON_CAMERA").as_deref() {
            Ok("back") => CameraType::ThirdPersonBack,
            Ok("front") => CameraType::ThirdPersonFront,
            _ => CameraType::FirstPerson,
        };
        let mut controls = Controls::new(camera_type, script.is_none());
        // A measuring run is not held to the player's limit, and a test can set either option
        // without touching the options file.
        if env::var_os("PHOTON_BENCH").is_some() {
            controls.options.framerate_limit = UNLIMITED_FRAMERATE_CUTOFF;
        }
        if let Ok(value) = env::var("PHOTON_VSYNC") {
            controls.options.enable_vsync = value != "0";
        }
        if let Some(limit) = env::var("PHOTON_MAX_FPS")
            .ok()
            .and_then(|text| text.parse().ok())
        {
            controls.options.framerate_limit = limit;
        }
        let sound = SoundSystem::open(std::path::Path::new(&root), &controls.options);
        let language = &controls.options.language_code;
        let mut gui = Gui::load(
            std::path::Path::new(&root),
            registries,
            &mut backend,
            language,
        )?;
        gui.set_player_skin(skin_rect);
        let capture_frames = env::var("PHOTON_CAPTURE_FRAMES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        surface.set_gui_scale_option(controls.options.gui_scale);
        surface.set_cursor_captured(controls.captured);
        Ok(Self {
            screen_assets: gui.screen_assets(),
            session,
            surface,
            backend,
            script,
            block_renderer,
            biome_tints,
            animated_atlas,
            origin,
            sections,
            sky,
            player_model: photon_render::entity::player_model::PlayerModel::new(skin.slim),
            remote_models: overlays::RemoteModels::new(),
            mob_models: mobs::MobModels::new(),
            item_layers: WorldItemLayers::new(),
            entity_vertices: Vec::new(),
            item_vertices: Vec::new(),
            armor,
            worn_layers: [
                ItemLayerCache::new(photon_render::model::DisplayContext::ThirdPersonRightHand),
                ItemLayerCache::new(photon_render::model::DisplayContext::ThirdPersonLeftHand),
                ItemLayerCache::new(photon_render::model::DisplayContext::Head),
                ItemLayerCache::new(photon_render::model::DisplayContext::Ground),
            ],
            entity_textures,
            skin_rect,
            gui,
            item_in_hand: photon_game::item::in_hand::ItemInHand::default(),
            held_item_layers: [
                ItemLayerCache::new(photon_render::model::DisplayContext::FirstPersonRightHand),
                ItemLayerCache::new(photon_render::model::DisplayContext::FirstPersonLeftHand),
            ],
            camera: photon_render::camera::Camera::new(),
            controls,
            capture_frames,
            captured_frames: 0,
            aspect: size.0 as f32 / size.1 as f32,
            sized_for: size,
            accumulated: Duration::ZERO,
            last_frame: Instant::now(),
            ticks: 0,
            sounds_heard: 0,
            sound,
            particles,
            frames: 0,
            profile: FrameProfile::from_env(),
            bench: Bench::from_env(),
            limiter: FrameLimiter::new(Instant::now()),
        })
    }

    /// Frames until the window closes, the player leaves the server or a script ends; the window
    /// and what was loaded for it are handed back, with whether to quit.
    pub(super) fn run(mut self) -> Result<(Client, Exit)> {
        while !self.surface.should_close() && !self.controls.leave {
            let frame_start = Instant::now();
            // A frame that ended early (nothing to draw yet) leaves its parts behind.
            profile::take_parts();
            for event in self.surface.poll() {
                if is_input(&event) {
                    self.limiter.input_received(frame_start);
                }
                self.controls.handle(
                    event,
                    &mut self.session,
                    &mut self.surface,
                    &self.screen_assets,
                )?;
            }
            if std::mem::take(&mut self.controls.options_saved) {
                let information = crate::controls::client_information(&self.controls.options);
                self.session.send_client_information(information)?;
            }
            // `LanguageSelectScreen.onDone`: the texts are read again, and the
            // screens that are open are made again in them.
            if let Some(code) = self.controls.take_language_request() {
                self.gui.set_language(&code);
                self.screen_assets = self.gui.screen_assets();
                self.gui.reword_chat(&mut self.controls.chat_log);
                self.controls
                    .rebuild_menus(&self.surface, &self.screen_assets);
            }
            self.backend.set_vsync(self.controls.options.enable_vsync);
            self.fit_swapchain()?;
            let now = Instant::now();
            let frame_ticks = (now - self.last_frame).as_secs_f32() / TICK_LENGTH.as_secs_f32();
            self.accumulated += now - self.last_frame;
            self.last_frame = now;
            self.receive_session()?;
            self.controls.container_tick(
                &mut self.session,
                &mut self.surface,
                &self.screen_assets,
            )?;
            // A respawn or a change of dimension: nothing of the old level stays.
            if self.session.take_level_reset() {
                self.backend.clear_sections()?;
                self.sections.clear();
                self.sky.reset();
                self.particles.clear();
            }
            // The particles option is not kept yet: all of them.
            let level_particles = self.session.take_level_particles();
            let pickups = self.session.take_item_pickups();
            self.particles.add_pickups(pickups);
            let events = self.session.take_level_events();
            let event_sounds = self.particles.level_events(self.session.world(), events);
            if let Some(sound) = self.sound.as_mut() {
                sound.handle_local(event_sounds);
            }
            self.particles.receive(
                level_particles,
                self.session.world(),
                photon_world::particles::ParticleStatus::All,
            );
            let commands = self.session.take_sounds();
            self.sounds_heard += commands.len();
            if let Some(sound) = self.sound.as_mut() {
                sound.handle(commands, self.session.world());
            }
            let script_done = self.run_ticks()?;
            let ticked = Instant::now();
            if let Flow::Stop = self.draw([frame_start, ticked], frame_ticks, script_done)? {
                break;
            }
            let options = &self.controls.options;
            let limit = self.limiter.limit(
                options.framerate_limit,
                options.inactivity_fps_limit,
                self.surface.is_iconified(),
                Instant::now(),
            );
            self.limiter.wait(limit);
        }
        println!(
            "play_frames={} play_ticks={} entities={} ticking={} sounds={} own_sounds={} particle_packets={} most_particles={} chunks={}",
            self.frames,
            self.ticks,
            self.session.entity_count(),
            self.session.ticking_entity_count(),
            self.sounds_heard,
            self.sound
                .as_ref()
                .map_or(0, SoundSystem::local_sound_count),
            self.particles.report().0,
            self.particles.report().1,
            self.session.world().loaded_chunk_count()
        );
        self.into_client()
    }

    /// The window and what was loaded for it, back from an ended game, with whether to quit.
    fn into_client(self) -> Result<(Client, Exit)> {
        let exit = if self.surface.should_close() {
            Exit::Quit
        } else {
            Exit::Title
        };
        let Self {
            surface,
            mut backend,
            sized_for,
            block_renderer,
            animated_atlas,
            ..
        } = self;
        backend.clear_sections()?;
        Ok((
            Client {
                surface,
                backend,
                size: sized_for,
                block_renderer,
                animated_atlas,
            },
            exit,
        ))
    }

    /// What the server sent since the last frame: suggestions, a disconnection, a death, the HUD's
    /// packets and chat.
    fn receive_session(&mut self) -> Result<()> {
        profile::timed(Span::Drain, || self.session.drain())?;
        self.controls.receive_suggestions(&mut self.session);
        self.controls
            .receive_disconnect(&self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .receive_death(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_sign_editor(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_book_editor(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_held_book(&mut self.session, &mut self.surface, &self.screen_assets);
        let hud = self.session.drain_hud();
        if !hud.is_empty() {
            self.gui.receive_hud(hud);
        }
        let chat = self.session.drain_chat();
        if !chat.is_empty() {
            self.gui.receive_chat(
                chat,
                &mut self.controls.chat_log,
                self.controls.chat.is_some(),
            );
        }
        Ok(())
    }

    /// A resized or fullscreen window gets a swapchain of its new size.
    pub(super) fn fit_swapchain(&mut self) -> Result<()> {
        if resize_to_window(&self.surface, &mut self.backend, &mut self.sized_for)? {
            let (width, height) = self.backend.extent();
            self.aspect = width as f32 / height as f32;
        }
        Ok(())
    }

    /// The ticks the time since the last frame owes, each with the keys a
    /// person or the script holds. Whether the script has ended.
    pub(super) fn run_ticks(&mut self) -> Result<bool> {
        let mut script_done = false;
        while self.accumulated >= TICK_LENGTH {
            self.accumulated -= TICK_LENGTH;
            let held = match self.script.as_mut() {
                Some(script) => match script.next() {
                    Some((held, turn, pitch)) => {
                        if (turn != 0.0 || pitch != 0.0)
                            && let Some(player) = self.session.player_mut()
                        {
                            let (y_rot, x_rot) = player.rotation();
                            player.set_rotation(y_rot + turn as f32, x_rot + pitch as f32);
                        }
                        held
                    }
                    None => {
                        script_done = true;
                        HeldKeys::default()
                    }
                },
                None => self.controls.keys,
            };
            // Once the script ends the keys stay as its last step left them,
            // so a capture can show a block mid-break.
            if self.script.is_some() && !script_done {
                self.controls.script_step(
                    held,
                    &mut self.session,
                    &mut self.surface,
                    &self.screen_assets,
                )?;
            }
            self.tick(held)?;
        }
        Ok(script_done)
    }

    /// One client tick.
    pub(super) fn tick(&mut self, held: HeldKeys) -> Result<()> {
        if let Some(sound) = self.sound.as_mut() {
            sound.apply_volumes(&self.controls.options);
            sound.tick(self.session.world(), false);
        }
        if let Some(screen) = self.controls.screen.as_mut() {
            screen.tick(self.session.active_menu());
        }
        // What the screen asked for as it was drawn (a bundle the mouse left).
        self.controls.drain_screen_actions(&mut self.session)?;
        let session = &mut self.session;
        // `Minecraft.handleKeybinds`: the attack key is spent before the
        // player moves, aiming the way `GameRenderer.pick` aims.
        let aim = session
            .player()
            .map_or(photon_game::game_mode::Aim::Miss, |player| {
                pick(session.world(), player, eye_position(player, 1.0))
            });
        let grabbed = self.controls.captured || self.script.is_some();
        session.keys(
            photon_game::game_mode::Keys {
                attack_clicks: self.controls.attack_key.take_clicks(),
                attack_held: self.controls.attack_key.is_down() && grabbed,
                use_clicks: self.controls.use_key.take_clicks(),
                use_held: self.controls.use_key.is_down(),
                pick_clicks: self.controls.pick_key.take_clicks(),
                control_down: self.controls.ctrl_down,
            },
            aim,
        )?;
        for hand in session.take_items_used() {
            self.item_in_hand.item_used(hand);
        }
        // After the keys and before the player ticks.
        tick_hands(&mut self.item_in_hand, session, &self.block_renderer);
        session.tick(held.input())?;
        let mut own_sounds = session
            .player_mut()
            .map(LocalPlayer::take_sounds)
            .unwrap_or_default();
        own_sounds.extend(session.take_world_sounds());
        if let Some(sound) = self.sound.as_mut() {
            sound.handle_local(own_sounds);
        }
        self.animated_atlas.tick();
        let breaking = session.take_breaking_effects();
        self.particles.breaking(session.world(), breaking);
        let local = session.player().map(|player| {
            let at = player.position();
            photon_world::particles::pickup_target((at.x, at.y, at.z), player.eye_height())
        });
        let block = session.player().map(|player| {
            let at = player.position();
            (
                at.x.floor() as i32,
                at.y.floor() as i32,
                at.z.floor() as i32,
            )
        });
        let block_sounds = self.particles.tick(session.world(), (block, local));
        if let Some(sound) = self.sound.as_mut() {
            sound.handle_local(block_sounds);
        }
        if let Some(player) = session.player() {
            self.camera.tick(player);
        }
        self.gui.tick();
        if let Some(death) = self.controls.death.as_mut() {
            death.tick();
        }
        if let Some(confirm) = self.controls.confirm.as_mut() {
            confirm.tick();
        }
        self.controls.sign_tick(session, &mut self.surface)?;
        self.gui.tick_chat(&mut self.controls.chat_log);
        self.sky
            .tick(session, self.controls.options.hide_lightning_flashes);
        self.ticks += 1;
        Ok(())
    }
}
