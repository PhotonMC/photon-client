//! The HUD and the open screen over it, drawn in GUI units.

use std::path::Path;
use std::sync::Arc;

use voxel_game::registries::Registries;
use voxel_game::text::{Lang, Style};
use voxel_net::connection::{BossOperation, ChatEvent, ChatHolder, HudEvent, PlaySession};
use voxel_render::VulkanBackend;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::gui::boss_bar::BossBars;
use voxel_render::gui::chat::ChatLog;
use voxel_render::gui::font::Font;
use voxel_render::gui::items::GuiItemVertex;
use voxel_render::gui::locator::LocatorCamera;
use voxel_render::gui::pause::PauseScreen;
use voxel_render::gui::screen::ScreenDraw;
use voxel_render::gui::sidebar::{self, SidebarLine};
use voxel_render::gui::tab_list::{
    self, HealthStates, TabEntry, TabObjective, TabPlayer, TabScores,
};
use voxel_render::gui::titles::Titles;
use voxel_render::gui::{GuiAtlas, Hud};
use voxel_render::player_model::{PlayerModel, PlayerRenderState};
use voxel_render::window::WindowSurface;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::ClientItemStackState;

use crate::controls::Controls;
use crate::hud_input::hud_input;
use crate::item_layers::ItemLayerCache;
use crate::screens::ScreenAssets;
use crate::view::CameraType;

/// The HUD's sprites and font, its own state, the client's language, and
/// each item's GUI layers, built the first time a slot shows it.
pub struct Gui {
    font: Arc<Font>,
    alt_font: Arc<Font>,
    atlas: GuiAtlas,
    lang: Arc<Lang>,
    /// Where the other languages are, if the asset folder has them.
    languages: Option<Arc<voxel_game::language::LanguageSource>>,
    /// The data-driven registries tooltips read.
    registries: Arc<Registries>,
    hud: Hud,
    /// Where the skin the local player's model wears in the inventory is.
    player_skin: voxel_render::entity_textures::TextureRect,
    titles: Titles,
    boss_bars: BossBars,
    tab_health: HealthStates,
    gui_ticks: i64,
    /// The tab list's header and footer, and when the HUD last ticked (for
    /// the partial tick the fades are drawn at).
    tab_header: Vec<voxel_game::text::Run>,
    tab_footer: Vec<voxel_game::text::Run>,
    last_tick: std::time::Instant,
    items: ItemLayerCache,
    started_at: std::time::Instant,
}

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
    fn tab_entries(&self, session: &PlaySession) -> (Vec<TabEntry>, Option<TabObjective>) {
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
    fn decorate(
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
    fn deleted_marker(&self) -> Vec<voxel_game::text::Run> {
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
    fn show_player_chat(
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
    fn draw_menu(
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
        (player_model, locator_camera): (&mut PlayerModel, LocatorCamera),
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
                let runs = |component: &voxel_game::text::Component| {
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
                voxel_render::gui::items::item_vertices(layers, x, y, out);
            };
        let mut frame = self.hud.build(
            &self.atlas,
            &self.font,
            size.gui_scaled_width(),
            size.gui_scaled_height(),
            &hud_input(
                session,
                controls.camera_type == CameraType::FirstPerson,
                locator_camera,
                controls.options.attack_indicator,
            ),
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
        } else if let Some(death) = controls.death.as_mut() {
            let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
            death.draw(&mut frame, &self.atlas, &self.font, mouse);
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
                    voxel_render::player_renderer::gui_player_vertices(
                        player_model,
                        &state,
                        area,
                        size,
                        offset_y,
                        mouse,
                        self.player_skin,
                    )
                };
            // A stack's tooltip in words.
            let lang = &self.lang;
            let registries = &self.registries;
            let mut tooltip_lines = |stack: &ClientItemStackState, creative: bool| {
                voxel_game::tooltip::lines(
                    stack,
                    true,
                    voxel_game::tooltip::Flags { creative },
                    registries,
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
                tooltip: &mut tooltip_lines,
                control_held: controls.control_held,
                partial: (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0),
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

/// `FilterMask.applyWithFormatting`: the text with the hidden characters
/// shown as dark grey hashes, each run in one piece.
fn filtered_content(
    text: &str,
    mask: &voxel_net::connection::FilterMask,
) -> voxel_game::text::Component {
    use voxel_game::text::Component;
    let chars: Vec<char> = text.chars().collect();
    let mut result = Component::empty();
    let mut start = 0;
    while start < chars.len() {
        let hidden = mask.hides(start);
        let end = (start..chars.len())
            .find(|&index| mask.hides(index) != hidden)
            .unwrap_or(chars.len());
        if hidden {
            result =
                result.append(Component::literal("#".repeat(end - start)).with_colour("dark_gray"));
        } else {
            result = result.append(Component::literal(
                chars[start..end].iter().collect::<String>(),
            ));
        }
        start = end;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_net::connection::FilterMask;

    #[test]
    fn hidden_characters_become_dark_grey_hashes_in_runs() {
        // Characters 1 and 2 of "hello" hidden: bits 0b110.
        let mask = FilterMask::Partial(vec![0b110]);
        let component = filtered_content("hello", &mask);
        let lang = Lang::default();
        let runs = lang.runs(&component, &Style::default());
        let shown: Vec<(&str, Option<u32>)> = runs
            .iter()
            .map(|run| (run.text.as_str(), run.style.colour))
            .collect();
        assert_eq!(
            shown,
            [
                ("h", None),
                ("##", voxel_game::text::named_colour("dark_gray")),
                ("lo", None)
            ]
        );
    }
}
