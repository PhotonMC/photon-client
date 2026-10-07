//! Screens over the game: the survival inventory, the creative inventory and
//! the containers the server opens, and what their handling shares.
//!
//! A screen is a [`Screen`]: it takes the mouse and the keys while it is open,
//! answers with the clicks it wants run ([`ScreenAction`]), and draws itself
//! into the GUI frame through a [`ScreenDraw`].

pub mod book;
pub mod container;
pub mod creative;
pub mod generic;
pub mod inventory;
pub mod recipe_book;

use voxel_game::container_menu::{ClickType, ContainerMenu};
use voxel_game::creative_menu::Click;

use crate::gui::edit_box::Clipboard;
use crate::gui::font::Font;
use crate::gui::{GuiAtlas, GuiFrame, GuiItemVertex, ItemDrawer};
pub use container::SlotClick;

/// Draws the player's model into a box (`x0, y0, x1, y1`) at a size and an
/// offset (`InventoryScreen.extractEntityInInventoryFollowsMouse`), turned
/// towards the mouse, as triangles for [`GuiFrame::entity`].
pub type PlayerDrawer<'a> =
    dyn FnMut((i32, i32, i32, i32), f32, f32, (f32, f32)) -> Vec<GuiItemVertex> + 'a;

/// An item's tooltip as lines of styled text: for a stack, and whether it is
/// on one of the creative inventory's own slots.
pub type TooltipLines<'a> =
    dyn FnMut(&voxel_world::ClientItemStackState, bool) -> Vec<Vec<voxel_game::text::Run>> + 'a;

/// What a screen draws with: the frame and its atlas and font, and how the
/// client draws items, the player and tooltips.
pub struct ScreenDraw<'a, 'b> {
    pub frame: &'a mut GuiFrame,
    pub atlas: &'a GuiAtlas,
    pub font: &'a Font,
    pub item: &'a mut ItemDrawer<'b>,
    pub player: &'a mut PlayerDrawer<'b>,
    pub tooltip: &'a mut TooltipLines<'b>,
    /// The recipe book and the item tags its recipes are read against.
    pub book: &'a voxel_world::RecipeBook,
    pub tags: &'a voxel_world::block::tags::ItemTags,
    /// `Minecraft.hasControlDown`: the recipe book's clock stops while it is.
    pub control_held: bool,
    /// The fraction of the tick the frame is drawn at.
    pub partial: f32,
    /// Whether the tooltips the screen's own widgets ask for are drawn (the reference draws them
    /// last, after everything a draw log compares).
    pub widget_tooltips: bool,
}

/// A click a screen asks the session to run.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenAction {
    /// A click in the player's own menu, sent as a container click.
    Container(SlotClick),
    /// A click in the creative inventory, run by its menu.
    Creative(Click),
    /// The anvil's name box changed: what the server is to name the result.
    Rename(String),
    /// A button of the open menu pressed (`ContainerButtonClick`): a
    /// stonecutter recipe, a loom pattern or an enchantment.
    Button(i32),
    /// The beacon's screen confirmed its choice (`SetBeacon`), by effect registry id.
    SetBeacon {
        primary: Option<i32>,
        secondary: Option<i32>,
    },
    /// The merchant's screen chose an offer (`SelectTrade`).
    SelectTrade(i32),
    /// The crafter's screen switched a slot of its grid on or off (`ContainerSlotStateChanged`).
    SlotState { slot: i32, enabled: bool },
    /// The screen closes the container (`LocalPlayer.closeContainer`).
    Close,
    /// The recipe book asks the server to place a recipe (`PlaceRecipe`).
    PlaceRecipe { recipe: i32, shift: bool },
    /// The recipe book was opened or closed, or its filter flipped.
    RecipeBookSettings {
        book_type: i32,
        open: bool,
        filtering: bool,
    },
    /// The ghost recipe goes.
    ClearGhost,
    /// A recipe that was new has shown on a page: it is seen
    /// (`RecipeBookSeenRecipe`).
    RecipeSeen(i32),
}

/// What a key did to a screen.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyResult {
    Close,
    Actions(Vec<ScreenAction>),
}

/// The modifiers a key came with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KeyMods {
    pub control: bool,
    pub shift: bool,
    /// The command key, which text boxes read as control.
    pub command: bool,
}

/// A screen that takes the input while it is open (`Screen` and
/// `AbstractContainerScreen`). Mouse positions are in GUI units.
pub trait Screen {
    /// The GUI was resized: `Screen.resize` lays the screen out again.
    fn resize(&mut self, width: i32, height: i32);

    /// `mouseClicked`, with `MouseHandler`'s double click check first.
    fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        now: std::time::Instant,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction>;

    /// `mouseReleased`.
    fn mouse_released(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        shift: bool,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction>;

    /// `mouseDragged`, while a button is held.
    fn mouse_dragged(&mut self, mouse: (f64, f64), menu: &ContainerMenu);

    /// `mouseScrolled`.
    fn mouse_scrolled(&mut self, _y: f64) {}

    /// `keyPressed`.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        mods: KeyMods,
        menu: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> KeyResult;

    /// `keyReleased`.
    fn key_released(&mut self) {}

    /// `charTyped`.
    fn char_typed(&mut self, _character: char) {}

    /// The clicks a typed key or character asked for, which the events that
    /// carry no result could not return.
    fn drain_actions(&mut self) -> Vec<ScreenAction> {
        Vec::new()
    }

    /// A click a script asks for on a slot of the screen's layout, or -999
    /// outside it.
    fn scripted_click(&mut self, slot: i32, button: i32, kind: ClickType) -> ScreenAction;

    /// `AbstractContainerScreen.containerTick`, once a client tick.
    fn tick(&mut self, _menu: &ContainerMenu) {}

    /// `extractRenderState`.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64));

    /// The creative screen, for what only it has (its tab and menu).
    fn as_creative_mut(&mut self) -> Option<&mut creative::CreativeScreen> {
        None
    }
}
