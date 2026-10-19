//! The debug overlay F3 shows (`DebugScreenOverlay`): lines from the entries that are on, in two
//! columns over a dark background.

mod entries;
mod format;
mod layout;
mod list;

pub use entries::{CameraEntity, Input};
pub use format::fixed;
pub use layout::{Line, columns};
pub use list::{EntryList, Status};

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

/// `DebugScreenOverlay.extractLines`: every background first, then the text, each line nine
/// units under the one above from two units down; the right column ends two units from the edge.
pub fn draw(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    width: i32,
    (left, right): (&[String], &[String]),
) {
    for (lines, align_left) in [(left, true), (right, false)] {
        for (index, line) in lines
            .iter()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
        {
            let text_width = font.width(line);
            let x = if align_left {
                2
            } else {
                width - 2 - text_width
            };
            let y = 2 + 9 * index as i32;
            frame.fill(atlas, (x - 1, y - 1, x + text_width + 1, y + 8), BACKGROUND);
        }
    }
    for (lines, align_left) in [(left, true), (right, false)] {
        for (index, line) in lines
            .iter()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
        {
            let x = if align_left {
                2
            } else {
                width - 2 - font.width(line)
            };
            frame.text(atlas, font, line, x, 2 + 9 * index as i32, TEXT);
        }
    }
}
