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

mod gui_chat;
mod gui_draw;
