//! The game's GUI above the world: the screens, the HUD, chat, options and widgets, laid out and
//! turned into draw calls on `photon-render`'s `GuiFrame`.

pub mod chat;
pub mod entity_name;
pub mod favicon;
pub mod hud;
pub mod layout;
pub mod loading_overlay;
pub mod options;
pub mod runs;
pub mod screen;
pub mod sign_text;
mod splitter;
mod stack_tints;
pub use stack_tints::{StackTints, stack_tints};
mod stack_numbers;
pub use stack_numbers::{stack_custom_model_data, stack_numbers};
pub mod text_hit;
pub mod toasts;
pub mod tooltip;
pub mod tutorial;
pub mod ui_sound;
pub mod widgets;

pub use hud::{HeartType, Hud, HudInput, JumpInput, VehicleInput};
pub use runs::{
    FontRuns, FrameRuns, ItemSlots, draw_lines, draw_runs, legacy_runs, legacy_styled_runs,
};
