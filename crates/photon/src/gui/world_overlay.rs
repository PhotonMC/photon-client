use super::*;
use photon_gui::screen::restrictions::RestrictionsScreen;

impl Gui {
    /// The restrictions or the game rules (or the question about leaving them), over the blurred
    /// game.
    pub(super) fn draw_world_overlay(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        (gui_size, mouse): ((i32, i32), (i32, i32)),
    ) {
        let blurriness = controls.options.menu_background_blurriness;
        if let Some(question) = controls.game_rules_confirm.as_mut() {
            if question.size() != gui_size {
                *question = question.resized(font, lang, gui_size);
            }
            question.draw(frame, (atlas, font), mouse, blurriness);
        } else if let Some(rules) = controls.game_rules.as_mut() {
            if rules.size() != gui_size {
                rules.resize(gui_size);
            }
            rules.draw(
                frame,
                (atlas, font),
                mouse,
                (blurriness, |_: &mut photon_render::gui::GuiFrame| {}),
            );
        } else if let Some(restrictions) = controls.restrictions.as_mut() {
            if restrictions.size() != gui_size {
                *restrictions = RestrictionsScreen::new(
                    font,
                    lang,
                    gui_size,
                    controls.chat_log.abilities(),
                    true,
                );
            }
            restrictions.draw(frame, (atlas, font), (mouse, blurriness));
        }
    }
}
