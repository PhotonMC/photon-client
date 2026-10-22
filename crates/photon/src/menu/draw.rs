//! One frame of the open menu screen.

use std::time::Instant;

use photon_game::text::Lang;
use photon_gui::screen::connect::draw_level_loading;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use super::Screen;

/// The open screen drawn into `frame`.
pub(super) fn draw_screen(
    screen: &mut Screen,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, mouse): (&Lang, (i32, i32)),
) {
    match screen {
        Screen::Title(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Servers(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Form(screen, _) => screen.draw(frame, atlas, font, mouse),
        // Not over a level, so the panorama is what blurs, not this screen.
        Screen::Confirm(screen, _) => screen.draw(frame, (atlas, font), mouse, 0),
        // In the level: `LevelLoadingScreen` until it has loaded.
        Screen::Connecting(screen, task) if task.status.loading() => {
            draw_level_loading(frame, atlas, font, lang, screen.size());
        }
        Screen::Connecting(screen, _) => screen.draw(frame, atlas, font, mouse),
        Screen::Failed(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Credits(screen, drawn) => {
            // The time since the last frame, in ticks.
            let delta = drawn.elapsed().as_secs_f32() * 20.0;
            *drawn = Instant::now();
            screen.draw(frame, atlas, font, delta);
        }
    }
}
