use super::*;

impl Gui {
    /// The default font, the GUI sprites (sent to the GPU) and the language
    /// from the asset folder.
    pub fn load(
        root: &Path,
        registries: Registries,
        backend: &mut VulkanBackend,
        language: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let font = Arc::new(Font::load(root).ok_or("the default font could not be read")?);
        let alt_font = Arc::new(Font::load_alt(root).ok_or("the alt font could not be read")?);
        let (atlas, texture) =
            GuiAtlas::load(root, &[&font, &alt_font]).ok_or("the HUD sprites could not be read")?;
        backend.load_gui_texture(&texture)?;
        let mut gui = Self {
            font,
            alt_font,
            atlas,
            lang: Arc::new(Lang::load(root)),
            languages: voxel_game::language::LanguageSource::open(root).map(Arc::new),
            registries: Arc::new(registries),
            hud: Hud::default(),
            player_skin: [0.0, 0.0, 1.0, 1.0],
            titles: Titles::default(),
            boss_bars: BossBars::default(),
            tab_health: HealthStates::default(),
            gui_ticks: 0,
            tab_header: Vec::new(),
            tab_footer: Vec::new(),
            last_tick: std::time::Instant::now(),
            items: ItemLayerCache::new(voxel_render::model::DisplayContext::Gui),
            started_at: std::time::Instant::now(),
        };
        gui.hud
            .set_waypoint_styles(voxel_render::gui::locator::WaypointStyles::load(root));
        gui.set_language(language);
        Ok(gui)
    }

    /// Where the skin the inventory's player wears is in the entity textures.
    pub fn set_player_skin(&mut self, skin: voxel_render::entity_textures::TextureRect) {
        self.player_skin = skin;
    }

    /// The default font and the sprites it is drawn from, for text in the world.
    /// The language the texts are worded in.
    pub fn lang(&self) -> &Lang {
        &self.lang
    }

    pub fn text_assets(&self) -> (&Font, &GuiAtlas) {
        (&self.font, &self.atlas)
    }

    /// What the inventory screens open with: the search trees, the font and
    /// the titles in the client's language.
    pub fn screen_assets(&self) -> ScreenAssets {
        let lang = &self.lang;
        ScreenAssets {
            creative: voxel_render::gui::screen::creative::CreativeAssets {
                trees: Arc::new(voxel_game::creative_menu::SearchTrees::new(
                    lang.clone(),
                    self.registries.clone(),
                )),
                font: self.font.clone(),
                titles: voxel_data::item::creative_tabs::CREATIVE_TABS
                    .iter()
                    .map(|tab| lang.text(tab.title))
                    .collect(),
            },
            inventory_title: lang.text("container.crafting"),
            registries: self.registries.clone(),
            alt_font: self.alt_font.clone(),
            lang: lang.clone(),
            languages: self.languages.clone(),
        }
    }

    /// The chat's messages are worded in the language now loaded.
    pub fn reword_chat(&self, log: &mut ChatLog) {
        log.set_lang(&self.lang, &self.font);
    }

    /// The client's language is another: texts are read again.
    pub fn set_language(&mut self, code: &str) {
        if let Some(source) = &self.languages {
            self.lang = Arc::new(source.lang(code));
        }
    }

    pub fn tick(&mut self) {
        self.hud.tick();
        self.titles.tick();
        self.gui_ticks += 1;
        self.last_tick = std::time::Instant::now();
    }

    /// The title, action bar and tab list packets.
    pub fn receive_hud(&mut self, events: Vec<HudEvent>) {
        let runs = |lang: &Lang, component: &voxel_game::text::Component| {
            lang.runs(component, &Style::default())
        };
        for event in events {
            match event {
                HudEvent::Title(title) => self.titles.set_title(runs(&self.lang, &title)),
                HudEvent::Subtitle(subtitle) => {
                    self.titles.set_subtitle(runs(&self.lang, &subtitle));
                }
                HudEvent::ActionBar(message) => {
                    self.titles.set_overlay(runs(&self.lang, &message));
                }
                HudEvent::Times {
                    fade_in,
                    stay,
                    fade_out,
                } => self.titles.set_times(fade_in, stay, fade_out),
                HudEvent::Clear { reset_times } => self.titles.clear(reset_times),
                HudEvent::TabList { header, footer } => {
                    self.tab_header = runs(&self.lang, &header);
                    self.tab_footer = runs(&self.lang, &footer);
                }
                HudEvent::Boss { id, operation } => {
                    let now = self.started_at.elapsed().as_millis() as u64;
                    match operation {
                        BossOperation::Add {
                            name,
                            progress,
                            color,
                            overlay,
                            flags,
                        } => {
                            let name = runs(&self.lang, &name);
                            self.boss_bars
                                .add(id, name, progress, color, overlay, flags, now);
                        }
                        BossOperation::Remove => self.boss_bars.remove(id),
                        BossOperation::Progress(progress) => {
                            self.boss_bars.set_progress(id, progress, now);
                        }
                        BossOperation::Name(name) => {
                            let name = runs(&self.lang, &name);
                            self.boss_bars.set_name(id, name);
                        }
                        BossOperation::Style { color, overlay } => {
                            self.boss_bars.set_style(id, color, overlay);
                        }
                        BossOperation::Properties(flags) => self.boss_bars.set_flags(id, flags),
                    }
                }
            }
        }
    }

    /// The players of the tab list, in the order it lists them, and the
    /// objective whose scores it shows.
    pub(super) fn tab_entries(
        &self,
        session: &PlaySession,
    ) -> (Vec<TabEntry>, Option<TabObjective>) {
        tab_list::entries(
            session.listed_players().into_iter().map(|info| TabPlayer {
                name: &info.name,
                display_name: info.display_name.as_ref(),
                game_mode: info.game_mode,
                latency: info.latency,
                list_order: info.list_order,
            }),
            session.scoreboard(),
            &self.lang,
        )
    }

    /// The decoration of a bound chat type, if the registries know it.
    pub(super) fn decorate(
        &self,
        content: voxel_game::text::Component,
        bound: &voxel_net::connection::ChatBound,
    ) -> voxel_game::text::Component {
        let decoration = match &bound.holder {
            ChatHolder::Direct(decoration) => Some(decoration),
            ChatHolder::Registry(id) => usize::try_from(*id)
                .ok()
                .and_then(|id| self.registries.chat_types.get(id)),
        };
        match decoration {
            Some(decoration) => decoration.decorate(content, &bound.name, bound.target.as_ref()),
            None => content,
        }
    }

    /// `ChatComponent.DELETED_CHAT_MESSAGE`.
    pub(super) fn deleted_marker(&self) -> Vec<voxel_game::text::Run> {
        let style = Style {
            colour: voxel_game::text::named_colour("gray"),
            italic: Some(true),
            ..Style::default()
        };
        self.lang.runs(
            &voxel_game::text::Component::translatable("chat.deleted_marker"),
            &style,
        )
    }

    /// The chat's clock, and the deletions that have waited long enough.
    pub fn tick_chat(&self, log: &mut ChatLog) {
        log.tick(&self.font, &self.deleted_marker());
    }

    /// `ChatListener.showMessageToPlayer`: a player's message, tagged by how
    /// far it can be trusted, and cut by what the server filtered.
    pub(super) fn show_player_chat(
        &self,
        packet: voxel_net::connection::PlayerChatPacket,
        sender_is_local: bool,
        log: &mut ChatLog,
        focused: bool,
    ) {
        use voxel_net::connection::FilterMask;
        use voxel_render::gui::chat::Tag;
        if packet.filter == FilterMask::FullyFiltered {
            return;
        }
        let content = packet
            .unsigned
            .clone()
            .unwrap_or_else(|| voxel_game::text::Component::literal(packet.signed_text.clone()));
        let decorated = self.decorate(content, &packet.bound);
        let decorated_text = self.lang.plain(&decorated);
        // `ChatTrustLevel.evaluate`: no signature or an old one is not
        // secure; text that is not what was signed is modified.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |time| time.as_millis() as i64);
        let expired = now > packet.timestamp_millis + 7 * 60 * 1000;
        let tag = if sender_is_local {
            None
        } else if packet.signature.is_none() || expired {
            Some(Tag::NotSecure)
        } else if !decorated_text.contains(&packet.signed_text) {
            Some(Tag::Modified(packet.signed_text.clone()))
        } else {
            None
        };
        let shown = if packet.filter == FilterMask::PassThrough {
            decorated
        } else {
            self.decorate(
                filtered_content(&packet.signed_text, &packet.filter),
                &packet.bound,
            )
        };
        log.add_component(
            &shown,
            &self.lang,
            (packet.signature.clone(), tag),
            &self.font,
            focused,
        );
    }

    /// The chat the server sent, worded and added to the log.
    pub fn receive_chat(&mut self, events: Vec<ChatEvent>, log: &mut ChatLog, focused: bool) {
        use voxel_render::gui::chat::Tag;
        for event in events {
            match event {
                ChatEvent::System {
                    content,
                    overlay: true,
                } => {
                    let runs = self.lang.runs(&content, &Style::default());
                    self.titles.set_overlay(runs);
                }
                ChatEvent::System { content, .. } => log.add_component(
                    &content,
                    &self.lang,
                    (None, Some(Tag::SystemSinglePlayer)),
                    &self.font,
                    focused,
                ),
                ChatEvent::Disguised { content, bound } => {
                    let decorated = self.decorate(content, &bound);
                    log.add_component(
                        &decorated,
                        &self.lang,
                        (None, Some(Tag::System)),
                        &self.font,
                        focused,
                    );
                }
                ChatEvent::Player {
                    packet,
                    sender_known,
                    sender_is_local,
                } => {
                    if sender_known {
                        self.show_player_chat(packet, sender_is_local, log, focused);
                    } else {
                        // `handleChatMessageError`: a sender the client has
                        // no player for.
                        let style = Style {
                            colour: voxel_game::text::named_colour("red"),
                            italic: Some(true),
                            ..Style::default()
                        };
                        let error =
                            voxel_game::text::Component::translatable("chat.validation_error")
                                .with_style(&style);
                        let decorated = self.decorate(error, &packet.bound);
                        log.add_component(
                            &decorated,
                            &self.lang,
                            (None, Some(Tag::Error)),
                            &self.font,
                            focused,
                        );
                    }
                }
                ChatEvent::Delete(signature) => {
                    log.delete(&signature, &self.font, &self.deleted_marker());
                }
            }
        }
    }

    /// The settings screen that is open, the topmost of those over the
    /// options, or the options screen, or the pause menu.
    pub(super) fn draw_menu(
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        controls: &mut Controls,
        frame: &mut voxel_render::gui::GuiFrame,
        gui_size: (i32, i32),
    ) {
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        macro_rules! sized {
            ($screen:expr) => {
                if let Some(screen) = $screen.as_mut() {
                    if screen.size() != gui_size {
                        screen.resize(gui_size);
                    }
                    screen.draw(frame, atlas, font, mouse);
                    return;
                }
            };
        }
        sized!(controls.language_screen);
        sized!(controls.mouse_screen);
        sized!(controls.controls_screen);
        sized!(controls.chat_options_screen);
        sized!(controls.sound_screen);
        sized!(controls.video_screen);
        if let Some(options_screen) = controls.options_screen.as_mut() {
            if options_screen.size() != gui_size {
                *options_screen =
                    options_screen.resized(font, lang, gui_size, controls.options.fov);
            }
            options_screen.draw(frame, atlas, font, mouse);
        } else if let Some(pause) = controls.pause.as_mut() {
            if pause.size() != gui_size {
                *pause = PauseScreen::new(font, lang, gui_size);
            }
            pause.draw(frame, atlas, font, mouse, gui_size, true);
        }
    }
}
