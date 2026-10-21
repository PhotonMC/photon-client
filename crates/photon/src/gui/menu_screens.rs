//! The settings screens drawn over the game, and the blur behind them.

use super::*;

/// Where the screen goes and with what: its size and the mouse, the menu blurriness, the item drawer.
type ProgressArgs<'a> = (
    ((i32, i32), (i32, i32)),
    i32,
    &'a mut photon_render::gui::ItemDrawer<'a>,
);

impl Gui {
    /// The open confirmation screen, laid out again for the window.
    pub(super) fn draw_confirm(
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        gui_size: (i32, i32),
    ) {
        let blurriness = controls.options.menu_background_blurriness;
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        let Some(confirm) = controls.confirm.as_mut() else {
            return;
        };
        if confirm.size() != gui_size {
            *confirm = confirm.resized(font, lang, gui_size);
        }
        confirm.draw(frame, (atlas, font), mouse, blurriness);
    }

    /// The settings screen that is open, the topmost of those over the
    /// options, or the options screen, or the pause menu.
    pub(super) fn draw_menu(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        (gui_size, item): ((i32, i32), &mut photon_render::gui::ItemDrawer<'_>),
    ) {
        // A link to confirm takes the screen's place (`ConfirmLinkScreen` is not drawn over its
        // parent).
        if controls.link_confirm.is_some() {
            return;
        }
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        // `Screen.extractBackground`: a screen with no background of its own blurs what is behind it.
        let blurriness = controls.options.menu_background_blurriness;
        if Self::draw_progress_screen(
            (atlas, font, lang),
            controls,
            frame,
            ((gui_size, mouse), blurriness, item),
        ) {
            return;
        }
        macro_rules! sized {
            ($screen:expr) => {
                if let Some(screen) = $screen.as_mut() {
                    frame.blur_before_this_stratum(blurriness);
                    if screen.size() != gui_size {
                        screen.resize(gui_size);
                    }
                    screen.draw(frame, atlas, font, mouse);
                    return;
                }
            };
        }
        sized!(controls.language_screen);
        sized!(controls.key_binds_screen);
        sized!(controls.mouse_screen);
        sized!(controls.controls_screen);
        sized!(controls.skin_screen);
        sized!(controls.chat_options_screen);
        sized!(controls.accessibility_screen);
        sized!(controls.sound_screen);
        sized!(controls.video_screen);
        if let Some(options_screen) = controls.options_screen.as_mut() {
            frame.blur_before_this_stratum(blurriness);
            if options_screen.size() != gui_size {
                *options_screen =
                    options_screen.resized(font, lang, gui_size, controls.options.fov);
            }
            options_screen.draw(frame, atlas, font, mouse);
        } else if let Some(pause) = controls.pause.as_mut() {
            if pause.size() != gui_size {
                *pause = PauseScreen::new(font, lang, gui_size);
            }
            pause.draw(frame, atlas, font, mouse, gui_size, Some(blurriness));
        }
    }

    /// The advancements or the statistics, when one is open: whether one was drawn.
    fn draw_progress_screen(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        ((gui_size, mouse), blurriness, item): ProgressArgs<'_>,
    ) -> bool {
        if let Some(open) = controls.advancements.as_mut() {
            frame.blur_before_this_stratum(blurriness);
            if open.screen.size() != gui_size {
                open.screen.resize(gui_size);
            }
            open.screen.draw(frame, (atlas, font, lang), mouse, item);
        } else if let Some(stats) = controls.stats.as_mut() {
            frame.blur_before_this_stratum(blurriness);
            if stats.size() != gui_size {
                stats.resize(gui_size);
            }
            let millis = frame.millis;
            stats.draw(frame, (atlas, font), mouse, (millis, item));
        } else {
            return false;
        }
        true
    }
}
