//! One frame of the open menu screen.

use std::time::Instant;

use photon_game::text::Lang;
use photon_gui::screen::connect::draw_level_loading;
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

use super::{Menu, Screen};
use crate::play::Client;

/// The open screen drawn into `frame`.
pub(super) fn draw_screen(
    screen: &mut Screen,
    host: &mut crate::controls::Controls,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, mouse, size): (&Lang, (i32, i32), (i32, i32)),
) {
    match screen {
        Screen::Home(screen) => {
            draw_title(screen, host, (frame, atlas, font), (lang, size, mouse));
        }
        Screen::Servers(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Form(screen, _) => screen.draw(frame, atlas, font, mouse),
        // Not over a level, so the panorama is what blurs, not this screen.
        Screen::Confirm(screen, _) => screen.draw(frame, (atlas, font), mouse, 0),
        Screen::Connecting(screen, task, question) => {
            draw_connecting(screen, task, question, (frame, atlas, font), (lang, mouse));
        }
        Screen::Linking(screen) => screen.draw(frame, atlas, font, mouse),
        Screen::Failed(screen, _) | Screen::Ask(screen, _) => {
            screen.draw(frame, atlas, font, mouse);
        }
        Screen::Settings => draw_settings(host, (frame, atlas, font), (lang, size)),
        Screen::Credits(screen, drawn) => {
            // The time since the last frame, in ticks.
            let delta = drawn.elapsed().as_secs_f32() * 20.0;
            *drawn = Instant::now();
            screen.draw(frame, atlas, font, delta);
        }
    }
}

/// The connect screen: `LevelLoadingScreen` in the level until it has loaded, else the connect
/// screen with the question about the server's packs when one is up.
fn draw_connecting(
    screen: &photon_gui::screen::loading::LoadingScreen,
    task: &super::JoinTask,
    question: &mut Option<photon_gui::screen::confirm::ConfirmScreen>,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, mouse): (&Lang, (i32, i32)),
) {
    if task.status.loading() {
        draw_level_loading(frame, atlas, font, lang, screen.size());
        return;
    }
    screen.draw(frame, atlas, font, mouse);
    draw_pack_question(question.as_mut(), frame, (atlas, font), mouse);
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

/// The title, and what opens over it: the friends overlay and its question.
fn draw_title(
    screen: &photon_gui::screen::home::HomeScreen,
    host: &mut crate::controls::Controls,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, size, mouse): (&Lang, (i32, i32), (i32, i32)),
) {
    // A screen opened over the title replaces it; the friends overlay (and its popup)
    // draw it as their background, with no mouse over it (`extractRenderState(-1, -1)`).
    let popup = matches!(
        host.link_confirm,
        Some((_, crate::controls::ConfirmTarget::RemoveFriend(_)))
    );
    let background = host.friends.is_some() && (!host.question_open() || popup);
    if !host.host_open() {
        screen.draw(frame, atlas, font, mouse);
    } else if background {
        screen.draw(frame, atlas, font, (-1, -1));
    }
    draw_over_title(host, (frame, atlas, font), (lang, size));
}

fn draw_over_title(
    host: &mut crate::controls::Controls,
    at: (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, size): (&Lang, (i32, i32)),
) {
    if host.host_open() {
        draw_settings(host, at, (lang, size));
    }
}

/// The options screens of the title, or the question one of them asked.
fn draw_settings(
    host: &mut crate::controls::Controls,
    (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
    (lang, size): (&Lang, (i32, i32)),
) {
    let mouse = (host.gui_mouse.0 as i32, host.gui_mouse.1 as i32);
    // `PopupScreen` draws the screen it was opened over, `(-1, -1)` for the mouse.
    let popup = matches!(
        host.link_confirm,
        Some((_, crate::controls::ConfirmTarget::RemoveFriend(_)))
    );
    let shown = !host.question_open() || popup;
    if let Some(friends) = host.friends.as_mut().filter(|_| shown) {
        if friends.size() != size {
            friends.resize(size);
        }
        let blurriness = host.options.menu_background_blurriness;
        let at = if popup { (-1, -1) } else { mouse };
        friends.draw(frame, (atlas, font), at, (blurriness, frame.millis));
    }
    let friends_up = host.friends.is_some();
    if let Some((link, _)) = host.link_confirm.as_mut() {
        if link.size() != size {
            *link = link.resized(font, lang, size);
        }
        link.draw(frame, (atlas, font), mouse, 0);
    } else if !friends_up {
        let at = (atlas, font, lang);
        if !crate::gui::Gui::draw_packs(at, host, frame, (size, mouse)) {
            crate::gui::Gui::draw_option_screens(at, host, frame, (size, mouse));
        }
    }
}

impl Menu {
    /// One frame of the screen. The colour the frame is cleared to.
    pub(super) fn draw_frame(
        &mut self,
        client: &mut Client,
        size: (i32, i32),
    ) -> Result<[f32; 4], Box<dyn std::error::Error>> {
        let mouse = self.mouse;
        let screen = &mut self.screen;
        let lang = self.gui.lang();
        let host = &mut self.host;
        self.gui.draw_menu_screen(
            &mut client.backend,
            &client.surface,
            |frame, atlas, font| {
                draw_screen(screen, host, (frame, atlas, font), (lang, mouse, size));
            },
        )?;
        Ok([0.0, 0.0, 0.0, 1.0])
    }
}
