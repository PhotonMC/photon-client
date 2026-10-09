use super::*;

impl Gui {
    /// The settings screen that is open, the topmost of those over the
    /// options, or the options screen, or the pause menu.
    pub(super) fn draw_menu(
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
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
        sized!(controls.key_binds_screen);
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

    /// The HUD, then the open screen over it, with the player in the screen
    /// posed as the entity stands now.
    #[expect(
        clippy::too_many_lines,
        reason = "one frame's HUD, screen and overlays in drawing order"
    )]
    pub fn draw(
        &mut self,
        backend: &mut VulkanBackend,
        surface: &WindowSurface,
        session: &mut PlaySession,
        controls: &mut Controls,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
        (player_model, locator_camera, (mob_models, armor), entity_textures): (
            &mut PlayerModel,
            LocatorCamera,
            (
                &mut crate::mobs::MobModels,
                &mut crate::equipment::armor::Armor,
            ),
            &photon_render::entity::textures::EntityTextures,
        ),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = surface.scale();
        let screen_menu = controls
            .screen
            .is_some()
            .then(|| session.active_menu().clone());
        let session = &*session;
        let hud_player = session.player();
        let tab_entries = controls.tab_held.then(|| self.tab_entries(session));
        if !controls.tab_held {
            self.tab_health.clear();
        }
        let sidebar = session
            .scoreboard()
            .sidebar_objective(session.local_name())
            .map(|(name, objective)| {
                let runs = |component: &photon_game::text::Component| {
                    self.lang.runs(component, &Style::default())
                };
                let lines: Vec<SidebarLine> = session
                    .scoreboard()
                    .sidebar_lines(name)
                    .iter()
                    .map(|(name, score)| (runs(name), runs(score)))
                    .collect();
                (runs(&objective.display_name), lines)
            });
        let items = &mut self.items;
        let mut draw_item =
            |stack: &ClientItemStackState, x: f32, y: f32, out: &mut Vec<GuiItemVertex>| {
                let layers = items.get(stack, renderer, tints);
                photon_render::gui::items::item_vertices(layers, x, y, out);
            };
        let mut frame = self.hud.build(
            &self.atlas,
            &self.font,
            size.gui_scaled_width(),
            size.gui_scaled_height(),
            &photon_render::gui::HudInput {
                // `getGameTimeDeltaPartialTick(true)`.
                cooldowns: session
                    .item_cooldowns()
                    .percents((self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0)),
                ..hud_input(
                    session,
                    controls.camera_type == CameraType::FirstPerson,
                    locator_camera,
                    controls.options.attack_indicator,
                )
            },
            self.started_at.elapsed().as_millis(),
            &mut draw_item,
        );
        frame.millis = self.started_at.elapsed().as_millis() as u64;
        if let Some((title, lines)) = &sidebar {
            sidebar::draw(
                &mut frame,
                &self.atlas,
                &self.font,
                (size.gui_scaled_width(), size.gui_scaled_height()),
                title,
                lines,
            );
        }
        // The chat, over the hotbar; open, it is the box's and brighter.
        let focused = controls.chat.is_some();
        let chat_tooltip = controls.chat_log.draw_with(
            &mut frame,
            &self.atlas,
            &self.font,
            (size.gui_scaled_width(), size.gui_scaled_height()),
            focused,
            focused.then_some((controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32)),
            &self.lang,
        );
        if let Some(chat) = controls.chat.as_mut() {
            chat.draw(
                &mut frame,
                &self.atlas,
                (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32),
                session.commands().map(|tree| (&**tree, &*self.lang)),
            );
        }
        let gui_size = (size.gui_scaled_width(), size.gui_scaled_height());
        if let Some(lines) = chat_tooltip {
            frame.tooltip(
                &self.atlas,
                &self.font,
                &lines,
                (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32),
                gui_size,
            );
        }
        let partial = (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0);
        let now = self.started_at.elapsed().as_millis() as u64;
        let last_frame = std::mem::replace(&mut self.last_frame_ms, now);
        self.boss_bars
            .draw(&mut frame, &self.atlas, &self.font, gui_size, now);
        self.titles
            .draw(&mut frame, &self.atlas, &self.font, gui_size, partial);
        if let Some((link, _)) = controls.link_confirm.as_mut() {
            if link.size() != gui_size {
                *link = link.resized(&self.font, &self.lang, gui_size);
            }
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            link.draw(&mut frame, &self.atlas, &self.font, mouse);
        } else if let Some(confirm) = controls.confirm.as_mut() {
            if confirm.size() != gui_size {
                *confirm = confirm.resized(&self.font, &self.lang, gui_size);
            }
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            confirm.draw(&mut frame, &self.atlas, &self.font, mouse);
        } else if let Some(credits) = controls.credits.as_mut() {
            // `deltaTracker.getGameTimeDeltaPartialTick`: the time since the last frame in ticks.
            let delta = now.saturating_sub(last_frame) as f32 / 50.0;
            credits.draw(&mut frame, &self.atlas, &self.font, delta);
        } else if let Some(death) = controls.death.as_mut() {
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            death.draw(&mut frame, &self.atlas, &self.font, mouse);
        } else if let Some(sign) = controls.sign_edit.as_mut() {
            if sign.screen.size() != gui_size {
                sign.screen.resize(gui_size);
            }
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            sign.screen.draw(&mut frame, &self.atlas, &self.font, mouse);
        } else if let Some(book) = controls.book_edit.as_mut() {
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            if let Some(sign) = book.sign.as_mut() {
                if sign.size() != gui_size {
                    sign.resize(gui_size);
                }
                sign.draw(&mut frame, &self.atlas, &self.font, mouse);
            } else {
                if book.screen.size() != gui_size {
                    book.screen.resize(gui_size);
                }
                book.screen.draw(&mut frame, &self.atlas, &self.font, mouse);
            }
        }
        if let Some(switcher) = controls.game_mode_switcher.as_mut() {
            switcher.resize(gui_size);
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            switcher.draw(
                &mut frame,
                (&self.atlas, &self.font, &self.lang),
                &mut draw_item,
                mouse,
                "F4",
            );
        }
        if let Some(disconnected) = controls.disconnected.as_mut() {
            if disconnected.size() != gui_size {
                *disconnected = disconnected.resized(&self.font, &self.lang, gui_size);
            }
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            disconnected.draw(&mut frame, &self.atlas, &self.font, mouse);
        }
        Self::draw_menu(
            (&self.atlas, &self.font, &self.lang),
            controls,
            &mut frame,
            gui_size,
        );
        if let Some((entries, objective)) = tab_entries {
            tab_list::draw(
                &mut frame,
                &self.atlas,
                &self.font,
                gui_size.0,
                &entries,
                (&self.tab_header, &self.tab_footer),
                TabScores {
                    objective,
                    health: &mut self.tab_health,
                    gui_ticks: self.gui_ticks,
                    lang: &self.lang,
                },
            );
        }
        if let (Some(screen), Some(menu)) = (controls.screen.as_mut(), screen_menu) {
            let mut draw_player =
                |area: (i32, i32, i32, i32), size: f32, offset_y: f32, mouse: (f32, f32)| {
                    let Some(player) = hud_player else {
                        return Vec::new();
                    };
                    let pose = player.model_pose(1.0);
                    let state = PlayerRenderState {
                        walk_position: pose.walk_position,
                        walk_speed: pose.walk_speed,
                        age_in_ticks: player.tick_count() as f32 + 1.0,
                        swim_amount: pose.swim_amount,
                        crouching: pose.crouching,
                        spectator: player.spectating(),
                        attack_time: player.attack_anim(1.0),
                        ..Default::default()
                    };
                    photon_render::entity::player_renderer::gui_player_vertices(
                        player_model,
                        &state,
                        area,
                        size,
                        offset_y,
                        mouse,
                        self.player_skin,
                    )
                };
            // A mount in the box of its screen, between two ticks as it stands now.
            let world = session.world();
            let mut draw_entity = |id: i32,
                                   area: (i32, i32, i32, i32),
                                   size: f32,
                                   offset_y: f32,
                                   mouse: (f32, f32)| {
                let Some(entity) = world.entities().at(id) else {
                    return Vec::new();
                };
                let view = photon_render::entity::player_renderer::GuiEntityView::new(
                    area,
                    size,
                    offset_y,
                    mouse,
                    entity.bb_height(),
                );
                crate::mobs::animals::farm_animals::gui_preview(
                    world,
                    mob_models,
                    entity_textures,
                    entity,
                    &view,
                )
                .or_else(|| {
                    crate::mobs::aquatic::nautilus::gui_preview(
                        world,
                        session.registry_names("minecraft:zombie_nautilus_variant"),
                        (mob_models, armor),
                        entity_textures,
                        entity,
                        &view,
                    )
                })
                .map(|vertices| {
                    photon_render::entity::player_renderer::gui_mob_vertices(&vertices, &view)
                })
                .unwrap_or_default()
            };
            // A stack's tooltip in words.
            let lang = &self.lang;
            let registries = &self.registries;
            let maps = |id: i32| {
                world.map_data(id).map(|map| photon_game::tooltip::MapInfo {
                    scale: map.scale,
                    locked: map.locked,
                })
            };
            let mut tooltip_lines = |stack: &ClientItemStackState, creative: bool| {
                // `advancedItemTooltips` is off: its `F3+H` toggle is not ported.
                photon_game::tooltip::lines(
                    stack,
                    true,
                    photon_game::tooltip::Flags {
                        creative,
                        advanced: false,
                    },
                    registries,
                    &maps,
                )
                .iter()
                .map(|line| lang.runs(line, &Style::default()))
                .collect::<Vec<_>>()
            };
            let mut draw = ScreenDraw {
                frame: &mut frame,
                atlas: &self.atlas,
                font: &self.font,
                item: &mut draw_item,
                player: &mut draw_player,
                entity: &mut draw_entity,
                tooltip: &mut tooltip_lines,
                lang: &self.lang,
                control_held: controls.control_held,
                partial: (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0),
                widget_tooltips: true,
                book: session.world().recipe_book(),
                tags: session.world().block_state_registry().item_tags(),
            };
            screen.build(&mut draw, &menu, controls.gui_mouse);
        }
        let scale = size.gui_scale() as f32;
        backend.set_gui(
            &frame,
            [
                size.framebuffer_width() as f32 / scale,
                size.framebuffer_height() as f32 / scale,
            ],
        )?;
        Ok(())
    }
}
