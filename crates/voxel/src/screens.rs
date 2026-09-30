//! The open inventory screen and what the client does with its clicks.

use voxel_net::connection::PlaySession;
use voxel_render::gui::screen::creative::{CreativeAssets, CreativeScreen};
use voxel_render::gui::screen::generic::{GenericScreen, Titles};
use voxel_render::gui::screen::inventory::InventoryScreen;
use voxel_render::gui::screen::{Screen, ScreenAction};
use voxel_render::window::WindowSurface;

/// The open container screen: the survival inventory or the creative one.
pub type OpenScreen = Option<Box<dyn Screen>>;

/// What the inventory screens are opened with.
pub struct ScreenAssets {
    pub creative: CreativeAssets,
    pub inventory_title: String,
    /// The client's language, for the titles the server names.
    pub lang: std::sync::Arc<voxel_game::text::Lang>,
}

impl ScreenAssets {
    /// The screen of a container the server opened: `title_nbt` is the
    /// menu's title as the network wrote it.
    pub fn container_screen(
        &self,
        kind: voxel_game::container_menu::MenuKind,
        title_nbt: &[u8],
        width: i32,
        height: i32,
    ) -> Box<dyn Screen> {
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
            ScreenAction::Creative(click) => {
                if let Some(open) = screen.as_mut().and_then(|open| open.as_creative_mut()) {
                    session.creative_click(open.menu_mut(), click)?;
                }
            }
        }
    }
    Ok(())
}
