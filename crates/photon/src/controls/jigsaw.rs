use super::*;

use photon_gui::screen::jigsaw::{JigsawAction, JigsawEdit as Done, JigsawScreen};
use photon_net::connection::JigsawBlockSettings;

/// The jigsaw block the player is editing: its screen and where the block is.
pub struct JigsawEdit {
    pub screen: JigsawScreen,
    pub pos: (i32, i32, i32),
    /// Whether the primary button is down on the screen, which drags the levels slider.
    dragging: bool,
}

impl Controls {
    /// `Player.openJigsawBlock`: the screen of the block just used opens over the game and the
    /// mouse is let go.
    pub fn open_jigsaw_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pos) = session.take_opened_jigsaw_block() else {
            return;
        };
        let Some(contents) = session.jigsaw_block(pos) else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let screen = JigsawScreen::new((&assets.creative.font, &*assets.lang), &contents, size);
        self.jigsaw_block = Some(JigsawEdit {
            screen,
            pos,
            dragging: false,
        });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// `onClose`: Done and Generate send the block's settings, Generate then the levels; the mouse
    /// is grabbed again.
    fn close_jigsaw_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        sent: Option<Sent>,
    ) -> Result {
        if let (Some(edit), Some(sent)) = (self.jigsaw_block.take(), sent) {
            session.set_jigsaw_block(edit.pos, &settings(&sent.done))?;
            if let Some((levels, keep)) = sent.generate {
                session.generate_jigsaw(edit.pos, levels, keep)?;
            }
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the jigsaw block's screen.
    pub(super) fn jigsaw_block_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let lang = &assets.lang;
        if let glfw::WindowEvent::CursorPos(x, y) = event {
            self.move_gui_mouse((x, y), surface);
            let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
            if let Some(edit) = self.jigsaw_block.as_mut().filter(|edit| edit.dragging) {
                edit.screen.dragged(lang, mouse);
            }
            return Ok(());
        }
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let Some(edit) = self.jigsaw_block.as_mut() else {
            return Ok(());
        };
        match edit.act(event, (lang, mouse), surface) {
            JigsawAction::Stay => Ok(()),
            JigsawAction::Cancel => self.close_jigsaw_block(session, surface, None),
            JigsawAction::Done(done) => {
                self.close_jigsaw_block(session, surface, Some(Sent::new(done, None)))
            }
            JigsawAction::Generate(done, levels, keep) => self.close_jigsaw_block(
                session,
                surface,
                Some(Sent::new(done, Some((levels, keep)))),
            ),
        }
    }
}

/// What a closing screen sends: the block's settings, and for Generate the levels and whether to
/// keep the jigsaws.
struct Sent {
    done: Box<Done>,
    generate: Option<(i32, bool)>,
}

impl Sent {
    fn new(done: Box<Done>, generate: Option<(i32, bool)>) -> Self {
        Self { done, generate }
    }
}

/// What the packet carries of what the screen sends.
fn settings(done: &Done) -> JigsawBlockSettings<'_> {
    JigsawBlockSettings {
        name: &done.name,
        target: &done.target,
        pool: &done.pool,
        final_state: &done.final_state,
        joint: done.joint,
        selection_priority: done.selection_priority,
        placement_priority: done.placement_priority,
    }
}

impl JigsawEdit {
    /// What a key, a character, a button or the wheel does to the screen.
    fn act(
        &mut self,
        event: glfw::WindowEvent,
        (lang, mouse): (&photon_game::text::Lang, (i32, i32)),
        surface: &mut WindowSurface,
    ) -> JigsawAction {
        use glfw::{Action, Modifiers, MouseButton, WindowEvent};
        match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                self.screen
                    .key_pressed(lang, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                self.dragging = true;
                self.screen
                    .clicked(lang, mouse, modifiers.contains(Modifiers::Shift))
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                self.dragging = false;
                self.screen.released();
                JigsawAction::Stay
            }
            WindowEvent::Char(character) => {
                self.screen.char_typed(character);
                JigsawAction::Stay
            }
            WindowEvent::Scroll(_, y) => {
                self.screen.scrolled(lang, y, mouse);
                JigsawAction::Stay
            }
            _ => JigsawAction::Stay,
        }
    }

    /// The screen drawn at the size of the window.
    pub fn draw(
        &mut self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (
            &photon_render::gui::GuiAtlas,
            &photon_render::gui::font::Font,
            &photon_game::text::Lang,
        ),
        (mouse, size): ((i32, i32), (i32, i32)),
    ) {
        if self.screen.size() != size {
            self.screen.resize(lang, size);
        }
        self.screen.draw((frame, atlas, font), mouse, lang);
    }
}
