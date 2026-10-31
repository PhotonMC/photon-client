//! The light screens drawn over the game: the social interactions and the debug options.

use super::*;

impl Gui {
    /// The social interactions screen or the debug options screen, at the size of the window.
    pub(super) fn draw_light_screen(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        (session, controls, gui_size): (&PlaySession, &mut Controls, (i32, i32)),
    ) {
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        let blurriness = controls.options.menu_background_blurriness;
        let columns = Self::debug_columns(session, controls);
        if let Some(options) = controls.debug_options.as_mut() {
            if options.size() != gui_size {
                options.resize(gui_size);
            }
            // `DebugOptionsScreen.extractBlurredBackground`: the overlay sits under the widgets.
            let overlay = |frame: &mut photon_render::gui::GuiFrame| {
                if let Some((left, right)) = columns.as_ref() {
                    photon_gui::hud::debug::draw(frame, atlas, font, gui_size.0, (left, right));
                }
            };
            options.draw(frame, (atlas, font), mouse, (blurriness, overlay));
        } else if controls.friends.is_some() {
            Self::draw_friends_overlay(frame, (atlas, font, lang), (controls, gui_size, mouse));
        } else if let Some(social) = controls.social.as_mut() {
            if social.size() != gui_size {
                *social = social.resized(font, lang, gui_size);
            }
            social.draw(frame, (atlas, font), mouse, blurriness);
        }
    }

    /// The friends overlay, with the pause menu it was opened from under it (`backgroundScreen`).
    pub(super) fn draw_friends_overlay(
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        (controls, gui_size, mouse): (&mut Controls, (i32, i32), (i32, i32)),
    ) {
        let blurriness = controls.options.menu_background_blurriness;
        if let Some(pause) = controls.pause_under.as_mut() {
            if pause.size() != gui_size {
                *pause = pause.resized(font, lang, gui_size);
            }
            pause.draw(frame, atlas, font, (-1, -1), gui_size, Some(blurriness));
        }
        if let Some(friends) = controls.friends.as_mut() {
            if friends.size() != gui_size {
                friends.resize(gui_size);
            }
            friends.draw(frame, (atlas, font), mouse, (blurriness, frame.millis));
        }
    }
}
