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
    host: &mut crate::controls::Controls,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, mouse, size): (&Lang, (i32, i32), (i32, i32)),
) {
    match screen {
        Screen::Title(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Servers(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Form(screen, _) => screen.draw(frame, atlas, font, mouse),
        // Not over a level, so the panorama is what blurs, not this screen.
        Screen::Confirm(screen, _) => screen.draw(frame, (atlas, font), mouse, 0),
        // In the level: `LevelLoadingScreen` until it has loaded.
        Screen::Connecting(screen, task, _) if task.status.loading() => {
            draw_level_loading(frame, atlas, font, lang, screen.size());
        }
        Screen::Connecting(screen, _, question) => {
            screen.draw(frame, atlas, font, mouse);
            draw_pack_question(question.as_mut(), frame, (atlas, font), mouse);
        }
        Screen::Failed(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Settings => draw_settings(host, (frame, atlas, font), (lang, size)),
        Screen::Credits(screen, drawn) => {
            // The time since the last frame, in ticks.
            let delta = drawn.elapsed().as_secs_f32() * 20.0;
            *drawn = Instant::now();
            screen.draw(frame, atlas, font, delta);
        }
    }
}

/// The question about the server's packs, over the connect screen when one is up.
fn draw_pack_question(
    question: Option<&mut photon_gui::screen::confirm::ConfirmScreen>,
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &Font),
    mouse: (i32, i32),
) {
    if let Some(question) = question {
        question.draw(frame, (atlas, font), mouse, 0);
    }
}

/// The options screens of the title, or the question one of them asked.
fn draw_settings(
    host: &mut crate::controls::Controls,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, size): (&Lang, (i32, i32)),
) {
    let mouse = (host.gui_mouse.0 as i32, host.gui_mouse.1 as i32);
    if let Some((link, _)) = host.link_confirm.as_mut() {
        if link.size() != size {
            *link = link.resized(font, lang, size);
        }
        link.draw(frame, (atlas, font), mouse, 0);
    } else {
        crate::gui::Gui::draw_option_screens((atlas, font, lang), host, frame, (size, mouse));
    }
}
