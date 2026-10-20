//! The HUD and the open screen over it, drawn in GUI units.

use std::path::Path;
use std::sync::Arc;

use photon_game::registries::Registries;
use photon_game::text::{Lang, Style};
use photon_gui::Hud;
use photon_gui::chat::ChatLog;
use photon_gui::hud::boss_bar::BossBars;
use photon_gui::hud::locator::LocatorCamera;
use photon_gui::hud::sidebar::{self, SidebarLine};
use photon_gui::hud::tab_list::{self, HealthStates, TabEntry, TabObjective, TabPlayer, TabScores};
use photon_gui::hud::titles::Titles;
use photon_gui::screen::ScreenDraw;
use photon_gui::screen::pause::PauseScreen;
use photon_net::connection::{BossOperation, ChatEvent, ChatHolder, HudEvent, PlaySession};
use photon_render::VulkanBackend;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::player_model::{PlayerModel, PlayerRenderState};
use photon_render::gui::GuiAtlas;
use photon_render::gui::font::Font;
use photon_render::gui::items::GuiItemVertex;
use photon_render::window::WindowSurface;
use photon_world::ClientItemStackState;

use crate::controls::Controls;
use crate::equipment::item_layers::ItemLayerCache;
use crate::screens::ScreenAssets;

/// The HUD's sprites and font, its own state, the client's language, and
/// each item's GUI layers, built the first time a slot shows it.
pub struct Gui {
    pub(super) font: Arc<Font>,
    pub(super) alt_font: Arc<Font>,
    pub(super) atlas: GuiAtlas,
    /// The atlas as the GPU has it, which server icons are put into.
    pub(super) atlas_image: photon_render::texture::png::DecodedImage,
    pub(super) lang: Arc<Lang>,
    /// Where the other languages are, if the asset folder has them.
    pub(super) languages: Option<Arc<photon_game::language::LanguageSource>>,
    /// The data-driven registries tooltips read.
    pub(super) registries: Arc<Registries>,
    pub(super) hud: Hud,
    /// Where the skin the local player's model wears in the inventory is.
    pub(super) player_skin: photon_render::entity::textures::TextureRect,
    pub(super) titles: Titles,
    pub(super) boss_bars: BossBars,
    pub(super) tab_health: HealthStates,
    pub(super) gui_ticks: i64,
    /// The tab list's header and footer, and when the HUD last ticked (for
    /// the partial tick the fades are drawn at).
    pub(super) tab_header: Vec<photon_game::text::Run>,
    pub(super) tab_footer: Vec<photon_game::text::Run>,
    pub(super) last_tick: std::time::Instant,
    pub(super) items: ItemLayerCache,
    pub(super) started_at: std::time::Instant,
    /// `ChatListener`'s delayed messages.
    pub(super) chat_queue: photon_gui::chat::queue::ChatQueue<photon_net::connection::ChatEvent>,
    /// Where the assets are read from, and when (in `started_at` milliseconds) the last frame was
    /// drawn: the credits roll by the time between frames.
    pub(super) assets_root: std::path::PathBuf,
    pub(super) last_frame_ms: u64,
    /// The subtitles, the toasts and what they keep between frames.
    pub(super) layers: layers::Layers,
}

/// `FilterMask.applyWithFormatting`: the text with the hidden characters
/// shown as dark grey hashes, each run in one piece.
pub(super) fn filtered_content(
    text: &str,
    mask: &photon_net::connection::FilterMask,
) -> photon_game::text::Component {
    use photon_game::text::Component;
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

mod assets;
mod chat;
pub use chat::ChatView;
mod debug;
mod draw;
mod hud;
mod layers;
mod light_screens;
mod title;

#[cfg(test)]
mod tests {
    use super::*;
    use photon_net::connection::FilterMask;

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
                ("##", photon_game::text::named_colour("dark_gray")),
                ("lo", None)
            ]
        );
    }
}
