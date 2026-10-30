use super::*;
use photon_game::menus::container::EnchantmentTraits;

/// `PHOTON_SCRIPT`: the steps a scripted run takes.
fn script_from_env() -> Result<Option<Script>> {
    Ok(env::var("PHOTON_SCRIPT")
        .ok()
        .map(|text| Script::parse(&text))
        .transpose()?)
}

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
            gui: _,
        } = client;
        let root = photon_core::content_root::assets()
            .map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        let script = script_from_env()?;

        let mut particles = crate::particles::Particles::load(&root);
        let position = report_spawn(&session)?;
        // The vanilla data pack the server runs, which sits next to the
        // assets: the biomes' colours and the sky's data come from it.
        let data_root = super::reload::data_root(&root);
        let biome_tints = Arc::new(
            BiomeTints::load(
                &data_root,
                &*crate::resources::shared(std::path::Path::new(&root)),
                session.registry_names("minecraft:worldgen/biome"),
                session.hashed_seed(),
            )
            .ok_or("the biome colours could not be read")?,
        );
        particles.set_terrain(Arc::clone(&block_renderer), Arc::clone(&biome_tints));
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
        let sky_sprites = super::reload::sky_sprites(&block_renderer);
        let sky = Sky::load(&data_root, &root, &session, sky_sprites)?;
        super::reload::Images::read(&root)?.upload(&mut backend)?;
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
        let entity_textures = super::reload::entity_textures(&root, &armor, &particles)?;
        backend.load_entity_texture(&entity_textures.image)?;
        particles.bind_textures(&entity_textures);
        let skin_rect = entity_textures.rect(skin.texture).ok_or("player skin")?;
        // The registries a tooltip reads, from the data pack, named as the
        // server synchronised them.
        let registries = photon_game::registries::Registries::load(&data_root, |registry| {
            session.registry_names(registry).to_vec()
        });
        share_registries(&mut session, &mut particles, &registries);
        // `PHOTON_CAMERA=back` or `front` starts detached.
        let camera_type = match env::var("PHOTON_CAMERA").as_deref() {
            Ok("back") => CameraType::ThirdPersonBack,
            Ok("front") => CameraType::ThirdPersonFront,
            _ => CameraType::FirstPerson,
        };
        let mut controls = Controls::new(camera_type, script.is_none());
        apply_env_options(&mut controls.options);
        let (sound, music, ambient) = super::audio::load(
            audio,
            &session,
            (&data_root, std::path::Path::new(&root)),
            &controls.options,
        );
        let mut gui = Gui::load(
            std::path::Path::new(&root),
            (registries, session.versions().clone()),
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
            loading: None,
            reload: Default::default(),
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
            skins: crate::skins::Skins::for_game(&entity_textures),
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
            if self.switch_event(&event) || self.loading_event(&event) {
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

    /// The whole ticks the frame did not run are not owed any more (`DeltaTracker` keeps only the
    /// fraction of a tick).
    pub(super) fn drop_owed_ticks(&mut self) {
        self.accumulated = tick_fraction(self.accumulated);
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
            gui,
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
                gui: Some(gui),
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
        self.serve_pack_reload();
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

    /// A scripted turn of the view; a turn or `keep_awake` is input, so a long run is not an idle
    /// client.
    fn apply_script_turn(&mut self, held: HeldKeys, turn: f64, pitch: f64) {
        if turn != 0.0 || pitch != 0.0 || held.keep_awake {
            self.limiter.input_received(Instant::now());
        }
        if (turn != 0.0 || pitch != 0.0)
            && let Some(player) = self.session.player_mut()
        {
            let (y_rot, x_rot) = player.rotation();
            player.set_rotation(y_rot + turn as f32, x_rot + pitch as f32);
        }
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
                    self.apply_script_turn(held, turn, pitch);
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
    pub(super) fn tick_screens(&mut self) -> Result<()> {
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
    pub(super) fn tick_sound(&mut self) {
        let Some(sound) = self.sound.as_mut() else {
            return;
        };
        sound.apply_volumes(&self.controls.options);
        sound.tick(self.session.world(), false);
        self.music.tick(
            (&self.session, sound),
            &self.controls.options,
            (&mut self.gui, self.controls.credits.is_some()),
        );
    }

    /// What the player's own tick sounded.
    pub(super) fn tick_player_sounds(&mut self) {
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
}

/// A measuring run is not held to the player's limit, and a test can set either option without
/// touching the options file.
pub(super) fn apply_env_options(options: &mut photon_game::options::Options) {
    if env::var_os("PHOTON_BENCH").is_some() {
        options.framerate_limit = UNLIMITED_FRAMERATE_CUTOFF;
    }
    if let Some(distance) = env::var("PHOTON_RENDER_DISTANCE")
        .ok()
        .and_then(|text| text.parse().ok())
    {
        options.render_distance = distance;
    }
    if let Ok(value) = env::var("PHOTON_VSYNC") {
        options.enable_vsync = value != "0";
    }
    if let Some(limit) = env::var("PHOTON_MAX_FPS")
        .ok()
        .and_then(|text| text.parse().ok())
    {
        options.framerate_limit = limit;
    }
}

/// What the data pack says of the registries, handed to the parts that read it.
pub(super) fn share_registries(
    session: &mut PlaySession,
    particles: &mut crate::particles::Particles,
    registries: &photon_game::registries::Registries,
) {
    session.set_enchantment_traits(EnchantmentTraits::of(registries));
    session.set_spin_attack(
        registries
            .enchantments
            .iter()
            .map(|entry| entry.trident_spin_attack)
            .collect(),
    );
    session.set_prevents_armor_change(
        registries
            .enchantments
            .iter()
            .map(|entry| entry.prevents_armor_change)
            .collect(),
    );
    session.set_instruments(photon_game::item::instrument::instruments_of(registries));
    particles.set_jukebox_songs(registries);
}

/// What is left of `owed` once its whole ticks are taken away.
fn tick_fraction(owed: Duration) -> Duration {
    Duration::from_nanos((owed.as_nanos() % TICK_LENGTH.as_nanos()) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_ticks_leave_only_the_fraction() {
        let owed = TICK_LENGTH * 25 + TICK_LENGTH / 4;
        assert_eq!(tick_fraction(owed), TICK_LENGTH / 4);
        assert_eq!(tick_fraction(TICK_LENGTH / 2), TICK_LENGTH / 2);
    }
}
