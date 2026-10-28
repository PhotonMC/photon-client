use super::*;
use photon_core::ids::EntityId;
use photon_gui::FrameRuns;
use photon_gui::screen::restrictions::RestrictionsScreen;

/// What the figures of the screens are drawn with: the mob models, the armor and the layers of
/// what is held.
type Dressing<'a> = (
    &'a mut crate::mobs::MobModels,
    &'a mut crate::equipment::armor::Armor,
    &'a mut [ItemLayerCache; 4],
);

impl Gui {
    /// The chat's messages, the box's and brighter when it is open; the lines of the tooltip
    /// the mouse is over, if any.
    fn draw_chat_log(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        frame: &mut photon_render::gui::GuiFrame,
        controls: &Controls,
        size: &photon_render::window::WindowScale,
    ) -> Option<Vec<Vec<photon_game::text::Run>>> {
        let focused = controls.chat.is_some();
        controls.chat_log.draw_with(
            frame,
            (atlas, font, lang),
            (size.gui_scaled_width(), size.gui_scaled_height()),
            focused,
            focused.then_some((controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32)),
        )
    }

    /// The link question, the dialog a server put over the game (or the question about it) or a
    /// confirmation: whether one is up.
    fn draw_server_dialog(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        (gui_size, item): ((i32, i32), &mut photon_render::gui::ItemDrawer<'_>),
    ) -> bool {
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        let blurriness = controls.options.menu_background_blurriness;
        if let Some((link, _)) = controls.link_confirm.as_mut() {
            if link.size() != gui_size {
                *link = link.resized(font, lang, gui_size);
            }
            link.draw(frame, (atlas, font), mouse, blurriness);
        } else if let Some(restrictions) = controls.restrictions.as_mut() {
            if restrictions.size() != gui_size {
                *restrictions = RestrictionsScreen::new(
                    font,
                    lang,
                    gui_size,
                    controls.chat_log.abilities(),
                    true,
                );
            }
            restrictions.draw(frame, (atlas, font), (mouse, blurriness));
        } else if let Some(warning) = controls.dialog_warning.as_mut() {
            if warning.size() != gui_size {
                *warning = warning.resized(font, lang, gui_size);
            }
            warning.draw(frame, (atlas, font), mouse, blurriness);
        } else if let Some(layer) = controls.dialog.as_mut() {
            if layer.size() != gui_size {
                *layer = layer.resized(font, lang, gui_size);
            }
            layer.draw(frame, (atlas, font), item, (mouse, blurriness));
        } else if controls.confirm.is_some() {
            Self::draw_confirm((atlas, font, lang), controls, frame, gui_size);
        } else {
            return false;
        }
        true
    }

    /// The titles, which the hidden interface does not show.
    fn draw_announcements(
        titles: &mut Titles,
        (atlas, font): (&GuiAtlas, &Font),
        frame: &mut photon_render::gui::GuiFrame,
        (hidden, gui_size, partial): (bool, (i32, i32), f32),
    ) {
        if !hidden {
            titles.draw(frame, atlas, font, gui_size, partial);
        }
    }

    /// The frame's clock, the boss bars (unless the interface is hidden) and the sleep veil over
    /// the HUD: `Hud.extractRenderState` draws the bars, then the veil, which the sidebar, the
    /// overlay message, the titles and the chat come over.
    fn finish_hud_frame(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, boss_bars): (&GuiAtlas, &Font, &mut BossBars),
        (started_at, hidden, hud): (std::time::Instant, bool, &Hud),
        (session, size, input, options): (
            &PlaySession,
            &photon_render::window::scale::WindowScale,
            &photon_gui::HudInput,
            &photon_game::options::Options,
        ),
    ) {
        let gui_size = (size.gui_scaled_width(), size.gui_scaled_height());
        frame.millis = started_at.elapsed().as_millis() as u64;
        if !hidden {
            boss_bars.draw(frame, atlas, font, gui_size, frame.millis);
            // The held item's name comes before the veil, with the contextual bar's decorations.
            let backdrop = if options.background_for_chat_only {
                0.0
            } else {
                options.text_background_opacity as f32
            };
            hud.draw_selected_name(frame, (atlas, font), gui_size, (input, backdrop));
        }
        let timer = session
            .world()
            .local_player()
            .map_or(0, |player| player.sleep_timer());
        photon_gui::hud::sleep::draw(frame, atlas, gui_size, timer);
    }

    /// The scoreboard sidebar's title and lines, in the language's runs.
    fn sidebar_lines(
        &self,
        session: &PlaySession,
    ) -> Option<(Vec<photon_game::text::Run>, Vec<SidebarLine>)> {
        let (name, objective) = session
            .scoreboard()
            .sidebar_objective(session.local_name())?;
        let runs =
            |component: &photon_game::text::Component| self.lang.runs(component, &Style::default());
        let lines: Vec<SidebarLine> = session
            .scoreboard()
            .sidebar_lines(name)
            .iter()
            .map(|(name, score)| (runs(name), runs(score)))
            .collect();
        Some((runs(&objective.display_name), lines))
    }

    /// The sign's or the command block's screen, at the size of the window.
    fn draw_text_edit(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        (session, controls, gui_size): (&PlaySession, &mut Controls, (i32, i32)),
    ) {
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        if let Some(sign) = controls.sign_edit.as_mut() {
            if sign.screen.size() != gui_size {
                sign.screen.resize(gui_size);
            }
            sign.screen.draw(frame, atlas, font, mouse);
        } else if let Some(edit) = controls.structure_block.as_mut() {
            edit.draw(frame, (atlas, font, lang), (mouse, gui_size));
        } else if let Some(edit) = controls.jigsaw_block.as_mut() {
            edit.draw(frame, (atlas, font, lang), (mouse, gui_size));
        } else if let Some(edit) = controls.test_block.as_mut() {
            edit.draw(frame, (atlas, font, lang), (mouse, gui_size));
        } else if let Some(edit) = controls.test_instance_block.as_mut() {
            edit.draw(frame, (atlas, font, lang), (mouse, gui_size));
        } else if let Some(edit) = controls.command_block.as_mut() {
            let commands = session.commands().map(|tree| &**tree);
            edit.draw(frame, (atlas, font, lang), (mouse, gui_size, commands));
        }
    }

    /// The HUD, then the open screen over it, with the player in the screen
    /// posed as the entity stands now.
    pub fn draw(
        &mut self,
        backend: &mut VulkanBackend,
        surface: &WindowSurface,
        session: &mut PlaySession,
        controls: &mut Controls,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
        (player_model, locator_camera, (mob_models, armor, held_layers), entity_textures): (
            &mut PlayerModel,
            LocatorCamera,
            Dressing<'_>,
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
        let sidebar = self.sidebar_lines(session);
        let mut draw_item = self.items.drawer((renderer, tints));
        // F1 hides the interface (`Hud.isHidden`); the screens over it are not the hud.
        let hidden = controls.hud_hidden;
        let partial = (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0);
        let hud_input =
            crate::hud_input::frame_input((session, controls), (renderer, partial, locator_camera));
        let mut frame = if hidden {
            photon_render::gui::GuiFrame {
                cooldowns: hud_input.cooldowns.clone(),
                ..Default::default()
            }
        } else {
            self.hud.build(
                (&self.atlas, &self.font),
                (size.gui_scaled_width(), size.gui_scaled_height()),
                (&hud_input, self.started_at.elapsed().as_millis()),
                &mut draw_item,
            )
        };
        Self::finish_hud_frame(
            &mut frame,
            (&self.atlas, &self.font, &mut self.boss_bars),
            (self.started_at, hidden, &self.hud),
            (session, &size, &hud_input, &controls.options),
        );
        if let Some((title, lines)) = sidebar.as_ref().filter(|_| !hidden) {
            sidebar::draw(
                &mut frame,
                &self.atlas,
                &self.font,
                (size.gui_scaled_width(), size.gui_scaled_height()),
                title,
                lines,
            );
        }
        // `Hud.extractRenderState`: the overlay message and the titles come before the chat.
        let gui_size = (size.gui_scaled_width(), size.gui_scaled_height());
        Self::draw_announcements(
            &mut self.titles,
            (&self.atlas, &self.font),
            &mut frame,
            (hidden, gui_size, partial),
        );
        // The chat, over the hotbar; open, it is the box's and brighter.
        let chat_tooltip = (!hidden)
            .then(|| {
                Self::draw_chat_log(
                    (&self.atlas, &self.font, &self.lang),
                    &mut frame,
                    controls,
                    &size,
                )
            })
            .flatten();
        if let Some(chat) = controls.chat.as_mut() {
            chat.draw(
                &mut frame,
                &self.atlas,
                (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32),
                (session.commands().map(|tree| &**tree), &self.lang),
            );
        }
        if let Some(lines) = chat_tooltip {
            frame.tooltip(
                &self.atlas,
                &self.font,
                &lines,
                (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32),
                gui_size,
            );
        }
        // `Hud.extractRenderState`: the tab list and the subtitles come after the chat, before the
        // screens and dialogs over the HUD.
        Self::draw_tab_and_subtitles(
            &mut frame,
            (&self.atlas, &self.font, &self.lang),
            (tab_entries.filter(|_| !hidden), gui_size.0),
            (
                (&mut self.tab_health, &self.tab_header, &self.tab_footer),
                self.gui_ticks,
            ),
            (&mut self.layers, (controls, &size)),
        );
        let now = self.started_at.elapsed().as_millis() as u64;
        let last_frame = std::mem::replace(&mut self.last_frame_ms, now);
        if Self::draw_server_dialog(
            (&self.atlas, &self.font, &self.lang),
            controls,
            &mut frame,
            (gui_size, &mut draw_item),
        ) {
        } else if let Some(credits) = controls.credits.as_mut() {
            // `deltaTracker.getGameTimeDeltaPartialTick`: the time since the last frame in ticks.
            let delta = now.saturating_sub(last_frame) as f32 / 50.0;
            credits.draw(&mut frame, &self.atlas, &self.font, delta);
        } else if let Some(death) = controls.death.as_mut() {
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            death.draw(&mut frame, &self.atlas, &self.font, mouse);
        } else if controls.text_edit_open() {
            let assets = (&self.atlas, &self.font, &self.lang);
            Self::draw_text_edit(&mut frame, assets, (session, controls, gui_size));
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
        } else if controls.light_screen_open() {
            let assets = (&self.atlas, &self.font, &self.lang);
            Self::draw_light_screen(&mut frame, assets, (session, controls, gui_size));
        }
        Self::draw_switcher_and_disconnect(
            (&self.atlas, &self.font, &self.lang),
            controls,
            &mut frame,
            (gui_size, &mut draw_item),
        );
        // The menus drawn over the game have no map data.
        let mut menu_tooltip = Self::stack_tooltip_lines(
            (&self.lang, &self.registries),
            controls.options.advanced_item_tooltips,
            photon_game::tooltip::Context::EMPTY,
        );
        Self::draw_menu(
            (&self.atlas, &self.font, &self.lang),
            controls,
            &mut frame,
            (gui_size, &mut draw_item, &mut menu_tooltip),
        );
        if let (Some(screen), Some(menu)) = (controls.screen.as_mut(), screen_menu) {
            let skin = self.player_skin;
            // Both previews dress what they draw from the same cache, one after the other.
            let armor = std::cell::RefCell::new(armor);
            let mut draw_player = preview::drawer(
                session,
                (hud_player, player_model),
                (&armor, entity_textures),
                (skin, held_layers, (renderer, tints)),
            );
            // A mount in the box of its screen, between two ticks as it stands now.
            let world = session.world();
            let mut draw_entity = |id: EntityId,
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
                    (&mut *mob_models, &mut **armor.borrow_mut()),
                    entity_textures,
                    entity,
                    &view,
                )
                .or_else(|| {
                    crate::mobs::aquatic::nautilus::gui_preview(
                        world,
                        session.registry_names("minecraft:zombie_nautilus_variant"),
                        (&mut *mob_models, &mut **armor.borrow_mut()),
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
            let maps = |id: i32| map_info(world, id);
            let mut tooltip_lines = Self::stack_tooltip_lines(
                (&self.lang, &self.registries),
                controls.options.advanced_item_tooltips,
                photon_game::tooltip::Context {
                    maps: &maps,
                    tick_rate: session.tick_rate(),
                },
            );
            let mut draw = ScreenDraw {
                frame: &mut frame,
                atlas: &self.atlas,
                font: &self.font,
                item: &mut draw_item,
                player: &mut draw_player,
                entity: &mut draw_entity,
                tooltip: &mut tooltip_lines,
                lang: &self.lang,
                control_held: controls.mods.control,
                partial: (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0),
                widget_tooltips: true,
                viewer: crate::hud_input::screen_viewer(session),
                book: session.world().recipe_book(),
                tags: session.world().block_state_registry().item_tags(),
            };
            screen.build(&mut draw, &menu, controls.gui_mouse);
        }
        Self::draw_late_layers(
            &mut frame,
            (&self.atlas, &self.font, &self.lang),
            (&mut self.layers, &mut draw_item),
            (controls, &size, session),
        );
        let atlas = (&mut self.atlas, &mut self.atlas_image);
        Self::finish(backend, (&frame, &size), (atlas, controls))
    }
}

impl Gui {
    /// The frame sent to the GPU, after the icons the Resource Packs screen asked for were put
    /// into the atlas (they show from the next frame).
    fn finish(
        backend: &mut VulkanBackend,
        (frame, size): (
            &photon_render::gui::GuiFrame,
            &photon_render::window::WindowScale,
        ),
        ((atlas, image), controls): (
            (
                &mut GuiAtlas,
                &mut photon_render::texture::png::DecodedImage,
            ),
            &mut Controls,
        ),
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(screen) = controls.pack_screen.as_mut()
            && photon_gui::favicon::sync_pack_icons(screen, atlas, image)
        {
            backend.replace_gui_texture(image)?;
        }
        Self::present(backend, (frame, size), (atlas, image))
    }

    /// The game mode switcher and the disconnected screen, when one is open.
    fn draw_switcher_and_disconnect(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        (gui_size, item): ((i32, i32), &mut photon_render::gui::ItemDrawer<'_>),
    ) {
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        if let Some(switcher) = controls.game_mode_switcher.as_mut() {
            switcher.resize(gui_size);
            switcher.draw(frame, (atlas, font, lang), item, mouse, "F4");
        }
        if let Some(disconnected) = controls.disconnected.as_mut() {
            if disconnected.size() != gui_size {
                *disconnected = disconnected.resized(font, lang, gui_size);
            }
            disconnected.draw(frame, atlas, font, mouse);
        }
    }
}

impl Gui {
    /// A stack's tooltip in words, in the context of the level's maps and tick rate.
    fn stack_tooltip_lines<'a>(
        (lang, registries): (
            &'a photon_game::text::Lang,
            &'a photon_game::registries::Registries,
        ),
        advanced: bool,
        context: photon_game::tooltip::Context<'a>,
    ) -> impl FnMut(&ClientItemStackState, bool) -> Vec<Vec<photon_game::text::Run>> + 'a {
        move |stack, creative| {
            photon_game::tooltip::lines(
                stack,
                true,
                photon_game::tooltip::Flags { creative, advanced },
                registries,
                context,
            )
            .iter()
            .map(|line| lang.runs(line, &Style::default()))
            .collect()
        }
    }
}

/// What a map's tooltip reads of the map with this id.
fn map_info(world: &photon_world::ClientWorld, id: i32) -> Option<photon_game::tooltip::MapInfo> {
    world.map_data(id).map(|map| photon_game::tooltip::MapInfo {
        scale: map.scale,
        locked: map.locked,
    })
}
