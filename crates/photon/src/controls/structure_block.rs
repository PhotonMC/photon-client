use super::*;

use photon_gui::screen::structure_block::{
    StructureBlockAction, StructureBlockEdit as Done, StructureBlockScreen,
};
use photon_net::connection::{StructureBlockSettings, StructureFlags};

/// The structure block the player is editing: its screen and where the block is.
pub struct StructureBlockEdit {
    pub screen: StructureBlockScreen,
    pub pos: (i32, i32, i32),
}

impl Controls {
    /// Whether the command block's or the structure block's screen is up.
    pub(super) fn block_screen_open(&self) -> bool {
        self.command_block.is_some()
            || self.structure_block.is_some()
            || self.jigsaw_block.is_some()
            || self.test_block.is_some()
    }

    /// Whether a sign's or a block's text screen is up.
    pub fn text_edit_open(&self) -> bool {
        self.sign_edit.is_some() || self.block_screen_open()
    }

    /// The screens of the block or minecart just used open.
    pub fn open_block_screens(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        self.open_structure_block(session, surface, assets);
        self.open_jigsaw_block(session, surface, assets);
        self.open_test_block(session, surface, assets);
        self.open_command_block(session, surface, assets);
    }

    /// An event for whichever of the two block screens is up.
    pub(super) fn block_screen_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        if self.structure_block.is_some() {
            self.structure_block_event(event, session, surface, assets)
        } else if self.jigsaw_block.is_some() {
            self.jigsaw_block_event(event, session, surface, assets)
        } else if self.test_block.is_some() {
            self.test_block_event(event, session, surface, assets)
        } else {
            self.command_block_event(event, session, surface, assets)
        }
    }

    /// `Player.openStructureBlock`: the screen of the block just used opens over the game and the
    /// mouse is let go.
    pub fn open_structure_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pos) = session.take_opened_structure_block() else {
            return;
        };
        let Some(contents) = session.structure_block(pos) else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let screen =
            StructureBlockScreen::new((&assets.creative.font, &*assets.lang), &contents, size);
        self.structure_block = Some(StructureBlockEdit { screen, pos });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// `onClose`: Done, Save, Load and Detect send the block's settings; the mouse is grabbed again.
    fn close_structure_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        done: Option<Box<Done>>,
    ) -> Result {
        if let (Some(edit), Some(done)) = (self.structure_block.take(), done) {
            session.set_structure_block(edit.pos, &settings(&done))?;
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the structure block's screen.
    pub(super) fn structure_block_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, WindowEvent};
        let Some(edit) = self.structure_block.as_mut() else {
            return Ok(());
        };
        let lang = &assets.lang;
        let mouse = (self.gui_mouse.0 as i32, self.gui_mouse.1 as i32);
        let action = match event {
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let control = modifiers.intersects(Modifiers::Control | Modifiers::Super);
                let shift = modifiers.contains(Modifiers::Shift);
                edit.screen
                    .key_pressed(lang, key, (control, shift), &mut WindowClipboard(surface))
            }
            WindowEvent::Char(character) => {
                edit.screen.char_typed(character);
                StructureBlockAction::Stay
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                StructureBlockAction::Stay
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, modifiers) => edit
                .screen
                .clicked(lang, mouse, modifiers.contains(Modifiers::Shift)),
            WindowEvent::Scroll(_, y) => {
                edit.screen.scrolled(lang, y, mouse);
                StructureBlockAction::Stay
            }
            _ => StructureBlockAction::Stay,
        };
        match action {
            StructureBlockAction::Stay => Ok(()),
            StructureBlockAction::Cancel(_) => self.close_structure_block(session, surface, None),
            StructureBlockAction::Done(done) => {
                self.close_structure_block(session, surface, Some(done))
            }
        }
    }
}

/// What the packet carries of what the screen sends.
fn settings(done: &Done) -> StructureBlockSettings<'_> {
    StructureBlockSettings {
        update: done.update,
        mode: done.mode,
        name: &done.name,
        offset: done.offset,
        size: done.size,
        mirror: done.mirror,
        rotation: done.rotation,
        data: &done.data,
        flags: StructureFlags {
            ignore_entities: done.ignore_entities,
            strict: done.strict,
            show_air: done.show_air,
            show_bounding_box: done.show_bounding_box,
        },
        integrity: done.integrity,
        seed: done.seed,
    }
}

impl StructureBlockEdit {
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
