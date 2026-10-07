//! The open inventory screen and what the client does with its clicks.

use voxel_net::connection::PlaySession;
use voxel_render::gui::screen::creative::{CreativeAssets, CreativeScreen};
use voxel_render::gui::screen::generic::{EnchantText, GenericScreen, Titles};
use voxel_render::gui::screen::inventory::InventoryScreen;
use voxel_render::gui::screen::recipe_book::RecipeAssets;
use voxel_render::gui::screen::{Screen, ScreenAction};
use voxel_render::window::WindowSurface;

/// The open container screen: the survival inventory or the creative one.
pub type OpenScreen = Option<Box<dyn Screen>>;

/// What the inventory screens are opened with.
pub struct ScreenAssets {
    pub creative: CreativeAssets,
    /// The font the enchanting table writes its names in.
    pub alt_font: std::sync::Arc<voxel_render::gui::font::Font>,
    /// The data-driven registries, for the loom's pattern names.
    pub registries: std::sync::Arc<voxel_game::registries::Registries>,
    pub inventory_title: String,
    /// The client's language, for the titles the server names.
    pub lang: std::sync::Arc<voxel_game::text::Lang>,
    /// The languages the client can switch to.
    pub languages: Option<std::sync::Arc<voxel_game::language::LanguageSource>>,
}

impl ScreenAssets {
    /// What the recipe book writes.
    fn recipe_assets(&self) -> std::sync::Arc<RecipeAssets> {
        let filtered = |key: &str| {
            self.lang
                .text(&format!("gui.recipebook.toggleRecipes.{key}"))
        };
        std::sync::Arc::new(RecipeAssets {
            font: self.creative.font.clone(),
            search_hint: self.lang.text("gui.recipebook.search_hint"),
            filtered: [
                filtered("craftable"),
                filtered("smeltable"),
                filtered("blastable"),
                filtered("smokable"),
            ],
            all: filtered("all"),
            lang: self.lang.clone(),
            registries: self.registries.clone(),
            more_recipes: self.lang.text("gui.recipebook.moreRecipes"),
            next_page: self.lang.text("gui.recipebook.next_page"),
            previous_page: self.lang.text("gui.recipebook.previous_page"),
        })
    }

    /// The screen of a container the server opened: `title_nbt` is the
    /// menu's title as the network wrote it.
    pub fn container_screen(
        &self,
        kind: voxel_game::container_menu::MenuKind,
        title_nbt: &[u8],
        width: i32,
        height: i32,
        may_build: bool,
    ) -> Box<dyn Screen> {
        if kind == voxel_game::container_menu::MenuKind::Lectern {
            return Box::new(voxel_render::gui::screen::book::BookScreen::new(
                width,
                height,
                self.lang.clone(),
                self.creative.font.clone(),
                may_build,
            ));
        }
        let mut data = title_nbt;
        let title = voxel_core::nbt::read_any_nbt(&mut data)
            .ok()
            .flatten()
            .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
            .map(|component| self.lang.plain(&component))
            .unwrap_or_default();
        Box::new(GenericScreen::new(
            kind,
            width,
            height,
            Titles {
                title,
                inventory: self.lang.text("container.inventory"),
                font: self.creative.font.clone(),
                repair_cost: self.lang.text("container.repair.cost"),
                repair_expensive: self.lang.text("container.repair.expensive"),
                patterns: std::sync::Arc::new(
                    self.registries
                        .banner_patterns
                        .iter()
                        .map(|pattern| (pattern.asset_id.clone(), pattern.translation_key.clone()))
                        .collect(),
                ),
                lang: self.lang.clone(),
                alt_font: self.alt_font.clone(),
                enchantments: std::sync::Arc::new(
                    self.registries
                        .enchantments
                        .iter()
                        .map(|enchantment| EnchantText {
                            name: enchantment
                                .description
                                .as_ref()
                                .and_then(voxel_game::text::Component::from_json)
                                .map(|component| self.lang.plain(&component))
                                .unwrap_or_default(),
                            curse: enchantment.curse,
                            max_level: enchantment.max_level,
                        })
                        .collect(),
                ),
                recipe: self.recipe_assets(),
            },
        ))
    }

    /// The creative screen on a tab, or the survival one, on a GUI of this
    /// size.
    pub fn screen(&self, creative: bool, width: i32, height: i32, tab: usize) -> Box<dyn Screen> {
        if creative {
            Box::new(CreativeScreen::new(
                width,
                height,
                tab,
                self.creative.clone(),
            ))
        } else {
            Box::new(InventoryScreen::new(
                width,
                height,
                self.inventory_title.clone(),
                self.recipe_assets(),
            ))
        }
    }
}

/// The window's clipboard, for an edit box.
pub struct WindowClipboard<'a>(pub &'a mut WindowSurface);

impl voxel_render::gui::edit_box::Clipboard for WindowClipboard<'_> {
    fn get(&mut self) -> String {
        self.0.clipboard()
    }

    fn set(&mut self, text: &str) {
        self.0.set_clipboard(text);
    }
}

/// `Minecraft.handleKeybinds` for the inventory key: the survival or the
/// creative inventory screen opens and the mouse is let go, at the window's
/// centre. The creative screen opens on the tab it was left on.
pub fn open_inventory(
    surface: &mut WindowSurface,
    screen: &mut OpenScreen,
    creative: bool,
    creative_tab: usize,
    assets: &ScreenAssets,
) {
    let size = surface.scale();
    let (width, height) = (size.gui_scaled_width(), size.gui_scaled_height());
    *screen = Some(assets.screen(creative, width, height, creative_tab));
    surface.set_cursor_captured(false);
    let (width, height) = surface.window_size();
    surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
}

/// `AbstractContainerScreen.onClose`: the container closes and the mouse is
/// grabbed again. The creative screen's tab is kept for the next time.
pub fn close_inventory(
    session: &mut PlaySession,
    surface: &mut WindowSurface,
    screen: &mut OpenScreen,
    creative_tab: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(open) = screen.as_mut().and_then(|open| open.as_creative_mut()) {
        *creative_tab = open.tab();
    }
    *screen = None;
    // The ghost recipe belongs to the screen that showed it.
    session.clear_recipe_ghost();
    session.close_inventory()?;
    surface.set_cursor_captured(true);
    Ok(())
}

/// The screen's clicks, one after another.
pub fn run_actions(
    session: &mut PlaySession,
    screen: &mut OpenScreen,
    actions: Vec<ScreenAction>,
) -> Result<(), Box<dyn std::error::Error>> {
    for action in actions {
        match action {
            ScreenAction::Container(click) => {
                session.container_click(click.slot, click.button, click.kind)?;
            }
            ScreenAction::Rename(name) => session.rename_item(&name)?,
            ScreenAction::Button(button) => session.container_button(button)?,
            ScreenAction::SetBeacon { primary, secondary } => {
                session.set_beacon(primary, secondary)?;
            }
            ScreenAction::SelectTrade(trade) => session.select_trade(trade)?,
            ScreenAction::SlotState { slot, enabled } => {
                session.set_crafter_slot_state(slot, enabled)?;
            }
            ScreenAction::Close => {
                *screen = None;
                session.clear_recipe_ghost();
                session.close_inventory()?;
            }
            ScreenAction::PlaceRecipe { recipe, shift } => session.place_recipe(recipe, shift)?,
            ScreenAction::RecipeBookSettings {
                book_type,
                open,
                filtering,
            } => session.set_recipe_book_settings(book_type, open, filtering)?,
            ScreenAction::ClearGhost => session.clear_recipe_ghost(),
            ScreenAction::RecipeSeen(recipe) => session.mark_recipe_seen(recipe)?,
            ScreenAction::Creative(click) => {
                if let Some(open) = screen.as_mut().and_then(|open| open.as_creative_mut()) {
                    session.creative_click(open.menu_mut(), click)?;
                }
            }
        }
    }
    Ok(())
}
