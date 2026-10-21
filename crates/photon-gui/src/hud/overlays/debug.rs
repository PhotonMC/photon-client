//! The debug overlay F3 shows (`DebugScreenOverlay`): lines from the entries that are on, in two
//! columns over a dark background.

mod entries;
mod format;
mod layout;
mod list;
mod profile_file;

pub use entries::{
    CameraEntity, Clouds, DeviceType, Filtering, FrameStats, Impactors, Input, MemoryUsage,
    PresentMode, ServerInfo, SystemSpecs, vendor_name,
};
pub use format::fixed;
pub use layout::{Line, columns};
pub use list::{Category, ENTRIES, Entry, EntryList, Profile, Status, entry, listed_entries};
pub use profile_file::{parse_profile, profile_text};

use crate::{FontRuns, FrameRuns};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// `Options.getBackgroundColor`'s fixed colour for the overlay and the text's.
const BACKGROUND: u32 = 0x9050_5050;
const TEXT: u32 = 0xFFE0_E0E0;

/// The lines of every entry that is on, in the order the list gives, laid into the two columns.
pub fn lines(
    list: &EntryList,
    reduced_debug_info: bool,
    input: &Input,
    footer: Option<[String; 3]>,
) -> (Vec<String>, Vec<String>) {
    let all = list
        .enabled(reduced_debug_info)
        .into_iter()
        .flat_map(|id| entries::display(id, input))
        .collect();
    columns(all, footer)
}

/// The footer: which keys toggle the charts and the options screen. Each key is written the way
/// `formatKeybind` does, such as `[F3+1]`.
pub fn footer(
    keys: &FooterKeys,
    (profiler, fps, network, lightmap): (bool, bool, bool, bool),
    (has_server, local_server): (bool, bool),
) -> [String; 3] {
    let state = |shown: bool| if shown { "visible" } else { "hidden" };
    [
        format!(
            "Debug charts: {} Profiler {}; {} {} {};",
            keys.profiler,
            state(profiler),
            keys.fps,
            if has_server { "FPS + TPS" } else { "FPS" },
            state(fps)
        ),
        format!(
            "{} {} {}; {} Lightmap {}",
            keys.network,
            if local_server {
                "Ping"
            } else {
                "Bandwidth + Ping"
            },
            state(network),
            keys.lightmap,
            state(lightmap)
        ),
        format!("To edit: press {}", keys.options),
    ]
}

/// The keys as `formatKeybind` writes them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterKeys {
    pub profiler: String,
    pub fps: String,
    pub network: String,
    pub lightmap: String,
    pub options: String,
}

impl FooterKeys {
    /// The default bindings with F3 as the modifier.
    pub fn defaults() -> Self {
        let key = |name: &str| format!("[F3+{name}]");
        Self {
            profiler: key("1"),
            fps: key("2"),
            network: key("3"),
            lightmap: key("4"),
            options: key("F6"),
        }
    }
}

/// `DebugEntryVersion`: the game's version, the one launched and the client's brand.
pub fn version_line(version: &str, launched: &str) -> String {
    format!("Minecraft {version} ({launched}/vanilla)")
}

/// Whether the overlay draws: an entry is on and the interface is shown or a screen is up.
pub fn shown(
    list: &EntryList,
    reduced_debug_info: bool,
    (hud_hidden, screen_open): (bool, bool),
) -> bool {
    !list.enabled(reduced_debug_info).is_empty() && (!hud_hidden || screen_open)
}

/// `DebugScreenOverlay.extractLines`, a column at a time: every background of the column, then its
/// text, each line nine units under the one above from two units down; the right column ends two
/// units from the edge.
pub fn draw(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    width: i32,
    (left, right): (&[String], &[String]),
) {
    for (lines, align_left) in [(left, true), (right, false)] {
        let placed = || {
            lines
                .iter()
                .enumerate()
                .filter(|(_, line)| !line.is_empty())
                .map(|(index, line)| {
                    let text_width = width_of(font, line);
                    let x = if align_left {
                        2
                    } else {
                        width - 2 - text_width
                    };
                    (line, text_width, (x, 2 + 9 * index as i32))
                })
        };
        for (_, text_width, (x, y)) in placed() {
            frame.fill(atlas, (x - 1, y - 1, x + text_width + 1, y + 8), BACKGROUND);
        }
        for (line, _, (x, y)) in placed() {
            draw_line(frame, (atlas, font), line, (x, y));
        }
    }
}

/// `Font.width` of a line, whose `\u{a7}` codes style it and take no room.
fn width_of(font: &Font, line: &str) -> i32 {
    if line.contains('\u{a7}') {
        font.width_of_runs(&crate::legacy_runs(line))
    } else {
        font.width(line)
    }
}

/// One line of the overlay, in the colours its `\u{a7}` codes set.
fn draw_line(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    line: &str,
    (x, y): (i32, i32),
) {
    if line.contains('\u{a7}') {
        frame.styled_text(atlas, font, &crate::legacy_runs(line), x, y, TEXT, false);
    } else {
        frame.text(atlas, font, line, x, y, TEXT);
    }
}
