//! The recipe book beside a crafting screen (`RecipeBookComponent`): the
//! toggle button, the panel with its tabs, search box, filter button and
//! pages of recipe buttons, and the ghost recipe over the menu's slots. The
//! recipes and what can be crafted are [`voxel_game::recipe_book`]'s.
//!
//! Not drawn: the overlay of a collection's other recipes (right click),
//! the squeeze animation of new recipes' buttons and tabs, and the toasts.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Instant;

use super::{ScreenAction, ScreenDraw};
use crate::gui::edit_box::{Clipboard, EditBox, edit_key};
use crate::gui::font::Font;
use voxel_game::container_menu::{ContainerMenu, MenuKind};
use voxel_game::recipe_book::{
    BookKind, BookModel, Collection, OverlaySlot, RECIPES_PER_PAGE, button_stack, display_stacks,
    ghost_slots, overlay_origin, overlay_slots, same_display,
};
use voxel_game::text::{Lang, Run, Style};
use voxel_world::{ClientItemStackState as Stack, RecipeBook};

/// The panel's width and height, and where the button, tabs and pages sit
/// in it.
const PANEL: (i32, i32) = (147, 166);
/// A GUI narrower than this has no room beside the screen.
const TOO_NARROW: i32 = 379;

/// What the panel writes, in the client's language, and the font its search
/// box measures with.
#[derive(Debug, Clone, Default)]
pub struct RecipeAssets {
    pub font: Arc<Font>,
    pub lang: Arc<Lang>,
    pub registries: Arc<voxel_game::registries::Registries>,
    pub search_hint: String,
    /// The filter's tooltip while it shows what can be crafted, by book type.
    pub filtered: [String; 4],
    pub all: String,
    pub more_recipes: String,
    pub next_page: String,
    pub previous_page: String,
}

/// What a collection's search reads: its recipes' results' tooltip lines
/// (lowercased) and ids.
#[derive(Debug, Clone, Default)]
struct SearchText {
    lines: Vec<String>,
    ids: Vec<(String, String)>,
}

/// `OverlayRecipeComponent`: a collection's recipes in a box over its button.
#[derive(Debug, Clone)]
struct Overlay {
    x: i32,
    y: i32,
    buttons: Vec<OverlayButton>,
}

/// One recipe of the overlay.
#[derive(Debug, Clone)]
struct OverlayButton {
    x: i32,
    y: i32,
    recipe: i32,
    craftable: bool,
    slots: Vec<OverlaySlot>,
}

/// The recipe book of one screen.
#[derive(Debug, Clone)]
pub struct RecipeBookPanel {
    overlay: Option<Overlay>,
    kind: BookKind,
    image_width: i32,
    /// The toggle button's corner from the image's left and the GUI's middle.
    button: (i32, i32),
    assets: Arc<RecipeAssets>,
    model: BookModel,
    search: EditBox,
    focused: bool,
    focused_at: Instant,
    /// `RecipeBookComponent.time`: ticks the open book has shown, which the
    /// buttons and the ghost cycle with.
    time: f32,
    last_frame: Option<Instant>,
    size: (i32, i32),
    visible: bool,
    filtering: bool,
    listed: Vec<usize>,
    /// `lastPlacedRecipe`, and the recipe Enter places again.
    last_placed: Option<i32>,
    last_recipe: Option<(i32, i32)>,
    queued: Vec<ScreenAction>,
    /// A press a widget took, so the release is not the menu's.
    swallow_release: bool,
    /// `ignoreTextInput`: the key that focused the search is not typed.
    ignore_text: bool,
    /// The squeeze a tab (by place) and a button (by its collection's first
    /// recipe) is in, in ticks left; 15 when it starts (`animationTime`).
    tab_animation: HashMap<usize, f32>,
    button_animation: HashMap<i32, f32>,
    /// The squeezes as this frame draws them.
    squeezing: (HashMap<usize, f32>, HashMap<i32, f32>),
    /// The book revision the tabs were last checked for new recipes at.
    tabs_checked: Option<u64>,
    /// The overlay was just opened: its slots are filled at the next draw,
    /// when the tags are at hand.
    overlay_pending: bool,
    /// What the search reads of each collection, by its first recipe.
    texts: HashMap<i32, SearchText>,
    texts_for: Option<u64>,
    /// Highlighted recipes already reported seen.
    seen: BTreeSet<i32>,
}

/// A line of plain text.
fn plain(text: &str) -> Vec<Run> {
    vec![Run {
        text: text.to_owned(),
        style: Style::default(),
    }]
}

mod recipe_book_panel_clicked;
mod recipe_book_panel_draw;
mod recipe_book_panel_ghost;
