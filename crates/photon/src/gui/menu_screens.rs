//! The settings screens drawn over the game, and the blur behind them.

use super::*;

/// Where the screen goes and with what: its size and the mouse, the menu blurriness, the item drawer.
type ProgressArgs<'a> = (
    ((i32, i32), (i32, i32)),
    i32,
    &'a mut photon_render::gui::ItemDrawer<'a>,
    &'a mut photon_gui::screen::TooltipLines<'a>,
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
        (gui_size, item, tooltip): (
            (i32, i32),
            &mut photon_render::gui::ItemDrawer<'_>,
            &mut photon_gui::screen::TooltipLines<'_>,
        ),
    ) {
        // A link to confirm takes the screen's place (`ConfirmLinkScreen` is not drawn over its
        // parent), and so does a dialog (`setScreen` replaces the menu it was opened from).
        if controls.link_confirm.is_some() || controls.dialog.is_some() {
            return;
        }
        let mouse = (controls.gui_mouse.0 as i32, controls.gui_mouse.1 as i32);
        // `Screen.extractBackground`: a screen with no background of its own blurs what is behind it.
        let blurriness = controls.options.menu_background_blurriness;
        if Self::draw_progress_screen(
            (atlas, font, lang),
            controls,
            frame,
            ((gui_size, mouse), blurriness, item, tooltip),
        ) {
            return;
        }
        if Self::draw_option_screens((atlas, font, lang), controls, frame, (gui_size, mouse)) {
            return;
        }
        if let Some(pause) = controls.pause.as_mut() {
            if pause.size() != gui_size {
                *pause = pause.resized(font, lang, gui_size);
            }
            pause.draw(frame, atlas, font, mouse, gui_size, Some(blurriness));
        }
    }

    /// The options screen that is open, the topmost of those over it: whether one was drawn. In a
    /// level the screen under blurs the game; from the title the panorama is blurred by the
    /// backend instead.
    pub(crate) fn draw_option_screens(
        (atlas, font, lang): (&GuiAtlas, &Font, &Lang),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        (gui_size, mouse): ((i32, i32), (i32, i32)),
    ) -> bool {
        let blurriness = controls.options.menu_background_blurriness;
        let in_level = !controls.title_host;
        let blur = |frame: &mut photon_render::gui::GuiFrame| {
            if in_level {
                frame.blur_before_this_stratum(blurriness);
            }
        };
        macro_rules! sized {
            ($screen:expr) => {
                if let Some(screen) = $screen.as_mut() {
                    blur(frame);
                    if screen.size() != gui_size {
                        screen.resize(gui_size);
                    }
                    screen.draw(frame, atlas, font, mouse);
                    return true;
                }
            };
        }
        if let Some(screen) = controls.credits_attribution_screen.as_mut() {
            blur(frame);
            if screen.size() != gui_size {
                screen.resize(font, lang, gui_size);
            }
            screen.draw(frame, atlas, font, mouse);
            return true;
        }
        sized!(controls.font_screen);
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
            blur(frame);
            if options_screen.size() != gui_size {
                *options_screen =
                    options_screen.resized(font, lang, gui_size, controls.options.fov);
            }
            options_screen.draw(frame, atlas, font, mouse);
            return true;
        }
        false
    }

    /// The advancements or the statistics, when one is open: whether one was drawn.
    fn draw_progress_screen(
        (atlas, font, lang): (&GuiAtlas, &Arc<Font>, &Arc<Lang>),
        controls: &mut Controls,
        frame: &mut photon_render::gui::GuiFrame,
        ((gui_size, mouse), blurriness, item, tooltip): ProgressArgs<'_>,
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
            let mut player = |_: (i32, i32, i32, i32), _: f32, _: f32, _: (f32, f32)| Vec::new();
            let mut entity = |_: photon_core::ids::EntityId,
                              _: (i32, i32, i32, i32),
                              _: f32,
                              _: f32,
                              _: (f32, f32)| Vec::new();
            let (book, tags) = (
                photon_world::RecipeBook::default(),
                photon_world::block::tags::ItemTags::default(),
            );
            let mut draw = photon_gui::screen::ScreenDraw {
                held: (0, [false; 2]),
                frame,
                atlas,
                font,
                item,
                player: &mut player,
                entity: &mut entity,
                tooltip,
                lang,
                book: &book,
                tags: &tags,
                control_held: false,
                partial: 0.0,
                widget_tooltips: true,
            };
            stats.draw(&mut draw, mouse, millis);
        } else {
            return false;
        }
        true
    }
}
