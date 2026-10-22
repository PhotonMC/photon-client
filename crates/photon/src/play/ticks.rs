use super::*;
use photon_game::menus::container::EnchantmentTraits;

impl Play {
    /// Join, open the window, and load everything the frames draw.
    pub(super) fn start(client: Client, mut session: PlaySession) -> Result<Self> {
        let Client {
            mut surface,
            mut backend,
            size,
            block_renderer,
            animated_atlas,
            tutorial,
            audio,
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
        let chunk_fade = pacing::chunk_fade(script.is_some())?;
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
        load_overlay_textures(&mut backend, &root)?;
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
                .chain(overlays::SHARED_TEXTURES)
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
        session.set_enchantment_traits(EnchantmentTraits::of(&registries));
        particles.set_jukebox_songs(&registries);
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
        let (sound, music, ambient) = super::audio::load(
            audio,
            &session,
            (&data_root, std::path::Path::new(&root)),
            &controls.options,
        );
        let mut gui = Gui::load(
            std::path::Path::new(&root),
            registries,
            &mut backend,
            &controls.options.language_code,
        )?;
        gui.set_player_skin(skin_rect);
        session.set_lang(gui.lang_shared());
        let capture_frames = env::var("PHOTON_CAPTURE_FRAMES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        surface.set_gui_options(controls.options.gui_scaling());
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
            tutorial,
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
            music,
            ambient,
            particles,
            frames: 0,
            profile: FrameProfile::from_env(),
            bench: Bench::from_env(),
            limiter: FrameLimiter::new(Instant::now()),
        })
    }

    /// Frames until the window closes, the player leaves the server or a script ends; the window
    /// and what was loaded for it are handed back, with whether to quit.
    pub(super) fn run(self) -> Result<(Client, Exit)> {
        let mut world = schedule::world(self);
        while schedule::continues(&world) {
            world.run_schedule(schedule::Frame);
        }
        if let Some(error) = schedule::take_failure(&mut world) {
            return Err(error);
        }
        let play = schedule::take_play(&mut world);
        play.report();
        play.into_client()
    }

    /// What the game counted, for the line a run ends with.
    fn report(&self) {
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
    }

    /// The window's events, as `Controls` takes them.
    pub(super) fn poll_events(&mut self, frame_start: Instant) -> Result<()> {
        for event in self.surface.poll() {
            if is_input(&event) {
                self.limiter.input_received(frame_start);
            }
            if self.switch_event(&event) {
                continue;
            }
            self.controls.handle(
                event,
                &mut self.session,
                &mut self.surface,
                &self.screen_assets,
            )?;
        }
        self.accept_chat_queue()?;
        self.clear_chat()
    }

    /// Saved options tell the server what the client is now.
    pub(super) fn send_client_information(&mut self) -> Result<()> {
        if std::mem::take(&mut self.controls.options_saved) {
            let information = crate::controls::client_information(&self.controls.options);
            self.session.send_client_information(information)?;
        }
        Ok(())
    }

    /// `LanguageSelectScreen.onDone`: the texts are read again, and the screens that are open are
    /// made again in them.
    pub(super) fn reload_language(&mut self) {
        if let Some(code) = self.controls.take_language_request() {
            self.gui.set_language(&code);
            self.session.set_lang(self.gui.lang_shared());
            self.screen_assets = self.gui.screen_assets();
            self.gui.reword_chat(&mut self.controls.chat_log);
            self.controls
                .rebuild_menus(&self.surface, &self.screen_assets);
        }
    }

    /// The time since the last frame, which the ticks and the partial tick are made of; the frame's
    /// length in ticks.
    pub(super) fn advance_clock(&mut self) -> f32 {
        let now = Instant::now();
        let frame_ticks = (now - self.last_frame).as_secs_f32() / TICK_LENGTH.as_secs_f32();
        self.accumulated += now - self.last_frame;
        self.last_frame = now;
        frame_ticks
    }

    /// The mouse's movement of the frame turns the player.
    pub(super) fn turn_player(&mut self) {
        self.controls.turn_frame(&mut self.session, Instant::now());
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
            tutorial,
            sound,
            music,
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
                tutorial,
                audio: Some(crate::music::Carried::new(sound, music)),
            },
            exit,
        ))
    }

    /// What the server sent since the last frame: suggestions, a disconnection, a death, the HUD's
    /// packets and chat.
    pub(super) fn receive_session(&mut self) -> Result<()> {
        profile::timed(Span::Drain, || self.session.drain())?;
        self.controls.receive_suggestions(&mut self.session);
        self.controls
            .receive_disconnect(&self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .receive_death(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls.receive_pack_requests(
            &mut self.session,
            &mut self.surface,
            &self.screen_assets,
        );
        self.controls
            .receive_dialogs(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .receive_win_game(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_sign_editor(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_block_screens(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_book_editor(&mut self.session, &mut self.surface, &self.screen_assets);
        self.controls
            .open_held_book(&mut self.session, &mut self.surface, &self.screen_assets);
        let hud = self.session.drain_hud();
        if !hud.is_empty() {
            self.gui.receive_hud(hud);
        }
        self.receive_chat()?;
        self.tutorial_items();
        Ok(())
    }

    /// The next tick the time since the last frame owes, with the keys a person or the script
    /// holds; none once the time is spent. `script_done` is set once the script has ended.
    pub(super) fn begin_tick(&mut self, script_done: &mut bool) -> Result<Option<HeldKeys>> {
        // A reconfiguration or a transfer has no level to tick.
        if self.tick_switch() {
            return Ok(None);
        }
        if self.accumulated < TICK_LENGTH {
            return Ok(None);
        }
        self.accumulated -= TICK_LENGTH;
        let held = match self.script.as_mut() {
            Some(script) => {
                if let Some((held, turn, pitch)) = script.next() {
                    if (turn != 0.0 || pitch != 0.0)
                        && let Some(player) = self.session.player_mut()
                    {
                        let (y_rot, x_rot) = player.rotation();
                        player.set_rotation(y_rot + turn as f32, x_rot + pitch as f32);
                    }
                    held
                } else {
                    *script_done = true;
                    HeldKeys::default()
                }
            }
            None => self.controls.keys,
        };
        // Once the script ends the keys stay as its last step left them,
        // so a capture can show a block mid-break.
        if self.script.is_some() && !*script_done {
            self.controls.script_step(
                held,
                &mut self.session,
                &mut self.surface,
                &self.screen_assets,
            )?;
        }
        Ok(Some(held))
    }

    /// The screens that tick with the game: the death screen, the credits, the confirmations,
    /// the waiting dialog, the sign, the bed's chat box and the chat's lines.
    fn tick_screens(&mut self) -> Result<()> {
        let session = &mut self.session;
        if let Some(death) = self.controls.death.as_mut() {
            death.tick();
        }
        self.controls.credits_tick(session, &mut self.surface)?;
        if let Some(confirm) = self.controls.confirm.as_mut() {
            confirm.tick();
        }
        if let Some(crate::controls::DialogLayer::Waiting(waiting)) = self.controls.dialog.as_mut()
        {
            waiting.tick(&self.screen_assets.lang);
        }
        self.controls.sign_tick(session, &mut self.surface)?;
        self.controls
            .command_block_tick(session, &mut self.surface, &self.screen_assets)?;
        self.controls
            .screens_tick(session, &mut self.surface, &self.screen_assets);
        self.tick_chat()?;
        Ok(())
    }

    /// The sound engine and the music manager for one tick.
    fn tick_sound(&mut self) {
        let Some(sound) = self.sound.as_mut() else {
            return;
        };
        sound.apply_volumes(&self.controls.options);
        sound.tick(self.session.world(), false);
        self.music.tick(
            (&self.session, sound),
            &self.controls.options,
            &mut self.gui,
        );
    }

    /// What the player's own tick sounded.
    fn tick_player_sounds(&mut self) {
        let session = &mut self.session;
        let mut own_sounds = session
            .player_mut()
            .map(LocalPlayer::take_sounds)
            .unwrap_or_default();
        own_sounds.extend(session.take_world_sounds());
        let (underwater, under_water) =
            session.player_mut().map_or((Vec::new(), false), |player| {
                (player.take_underwater_sounds(), player.is_under_water())
            });
        if let Some(sound) = self.sound.as_mut() {
            sound.handle_local(own_sounds);
            sound.handle_underwater(underwater, under_water);
            if let Some(player) = session.player() {
                let (motion, position) = (player.delta_movement(), player.position());
                sound.set_glide(
                    position,
                    [motion.x, motion.y, motion.z],
                    player.is_fall_flying(),
                );
            }
            self.ambient.tick(session, sound);
        }
    }

    /// One client tick.
    pub(super) fn tick(&mut self, held: HeldKeys) -> Result<()> {
        self.tick_sound();
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
                control_down: self.controls.mods.ctrl,
            },
            aim,
        )?;
        for hand in session.take_items_used() {
            self.item_in_hand.item_used(hand);
        }
        self.tutorial_before_tick(&aim, &held);
        let session = &mut self.session;
        // After the keys and before the player ticks.
        tick_hands(&mut self.item_in_hand, session, &self.block_renderer);
        session.tick(held.input())?;
        self.tutorial_tick();
        self.tick_player_sounds();
        let session = &mut self.session;
        self.animated_atlas.tick();
        // What the level's own tick queued (a sulfur block's gas) joins the particles in this tick.
        super::effects::route_level_events(
            session,
            (&mut self.particles, &mut self.gui, self.sound.as_mut()),
        );
        let breaking = session.take_breaking_effects();
        self.particles.breaking(session.world(), breaking);
        super::effects::tick_blocks(
            session,
            (&mut self.particles, &self.sky),
            self.sound.as_mut(),
        );
        super::camera::tick_view(
            (&mut self.camera, &mut self.gui),
            session,
            (&self.sky, &self.controls),
        );
        self.tick_screens()?;
        self.sky.tick(&self.session, &self.controls.options);
        self.ticks += 1;
        Ok(())
    }
}

/// The textures drawn over the world rather than in it: the world border's forcefield, the water
/// over the view, and the end portal's sky and picture.
fn load_overlay_textures(backend: &mut photon_render::VulkanBackend, root: &str) -> Result<()> {
    backend.load_world_border_texture(&texture(root, "misc/forcefield", "forcefield")?)?;
    backend.load_underwater_texture(&texture(root, "misc/underwater", "underwater")?)?;
    backend.load_end_portal_textures((
        &texture(root, "environment/end_sky", "end_sky")?,
        &texture(root, "entity/end_portal/end_portal", "end_portal")?,
    ))?;
    Ok(())
}
