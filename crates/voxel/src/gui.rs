//! The HUD and the open screen over it, drawn in GUI units.

use std::path::Path;
use std::sync::Arc;

use voxel_game::registries::Registries;
use voxel_game::text::{Lang, Style};
use voxel_net::connection::{ChatEvent, ChatHolder, HudEvent, PlaySession};
use voxel_render::VulkanBackend;
use voxel_render::biome_tint::BiomeTints;
use voxel_render::gui::chat::ChatLog;
use voxel_render::gui::font::Font;
use voxel_render::gui::items::GuiItemVertex;
use voxel_render::gui::screen::ScreenDraw;
use voxel_render::gui::tab_list::{self, TabEntry};
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
    /// The data-driven registries tooltips read.
    registries: Arc<Registries>,
    hud: Hud,
    titles: Titles,
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
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let font = Arc::new(Font::load(root).ok_or("the default font could not be read")?);
        let alt_font = Arc::new(Font::load_alt(root).ok_or("the alt font could not be read")?);
        let (atlas, texture) =
            GuiAtlas::load(root, &[&font, &alt_font]).ok_or("the HUD sprites could not be read")?;
        backend.load_gui_texture(&texture)?;
        Ok(Self {
            font,
            alt_font,
            atlas,
            lang: Arc::new(Lang::load(root)),
            registries: Arc::new(registries),
            hud: Hud::default(),
            titles: Titles::default(),
            tab_header: Vec::new(),
            tab_footer: Vec::new(),
            last_tick: std::time::Instant::now(),
            items: ItemLayerCache::new(voxel_render::model::DisplayContext::Gui),
            started_at: std::time::Instant::now(),
        })
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
        }
    }

    pub fn tick(&mut self) {
        self.hud.tick();
        self.titles.tick();
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
            }
        }
    }

    /// The players of the tab list, in the order it lists them.
    fn tab_entries(&self, session: &PlaySession) -> Vec<TabEntry> {
        let mut entries: Vec<TabEntry> = session
            .listed_players()
            .into_iter()
            .map(|info| {
                let display = info
                    .display_name
                    .as_ref()
                    .map(|component| self.lang.runs(component, &Style::default()));
                TabEntry::new(
                    &info.name,
                    display,
                    info.game_mode,
                    info.latency,
                    info.list_order,
                )
            })
            .collect();
        tab_list::sort(&mut entries);
        entries
    }

    /// The chat the server sent, worded and added to the log.
    pub fn receive_chat(&mut self, events: Vec<ChatEvent>, log: &mut ChatLog, focused: bool) {
        for event in events {
            let component = match event {
                ChatEvent::System {
                    content,
                    overlay: true,
                } => {
                    let runs = self.lang.runs(&content, &Style::default());
                    self.titles.set_overlay(runs);
                    continue;
                }
                ChatEvent::System { content, .. } => content,
                ChatEvent::Disguised { content, bound }
                | ChatEvent::Player { content, bound, .. } => {
                    let decoration = match &bound.holder {
                        ChatHolder::Direct(decoration) => Some(decoration),
                        ChatHolder::Registry(id) => usize::try_from(*id)
                            .ok()
                            .and_then(|id| self.registries.chat_types.get(id)),
                    };
                    match decoration {
                        Some(decoration) => {
                            decoration.decorate(content, &bound.name, bound.target.as_ref())
                        }
                        None => content,
                    }
                }
            };
            log.add(
                &self.lang.runs(&component, &Style::default()),
                &self.font,
                focused,
            );
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
        player_model: &mut PlayerModel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let size = surface.scale();
        let screen_menu = controls
            .screen
            .is_some()
            .then(|| session.active_menu().clone());
        let session = &*session;
        let hud_player = session.player();
        let tab_entries = controls.tab_held.then(|| self.tab_entries(session));
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
            &hud_input(session, controls.camera_type == CameraType::FirstPerson),
            self.started_at.elapsed().as_millis(),
            &mut draw_item,
        );
        // The chat, over the hotbar; open, it is the box's and brighter.
        controls.chat_log.draw(
            &mut frame,
            &self.atlas,
            &self.font,
            size.gui_scaled_height(),
            controls.chat.is_some(),
        );
        if let Some(chat) = controls.chat.as_mut() {
            chat.draw(
                &mut frame,
                &self.atlas,
                (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32),
            );
        }
        let gui_size = (size.gui_scaled_width(), size.gui_scaled_height());
        let partial = (self.last_tick.elapsed().as_secs_f32() / 0.05).min(1.0);
        self.titles
            .draw(&mut frame, &self.atlas, &self.font, gui_size, partial);
        if let Some(entries) = tab_entries {
            tab_list::draw(
                &mut frame,
                &self.atlas,
                &self.font,
                gui_size.0,
                &entries,
                &self.tab_header,
                &self.tab_footer,
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
