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
        let entries = &controls.debug.entries;
        if let Some(options) = controls.debug_options.as_mut() {
            if options.size() != gui_size {
                let reduced = session.reduced_debug_info();
                *options = options.resized(font, lang, gui_size, entries, reduced);
            }
            options.draw(frame, atlas, font, mouse);
        } else if let Some(social) = controls.social.as_mut() {
            if social.size() != gui_size {
                *social = social.resized(font, lang, gui_size);
            }
            social.draw(frame, atlas, font, mouse);
        }
    }
}
