use super::*;

use photon_game::text::Lang;
use photon_gui::screen::command_block::{
    CommandBlockAction, CommandBlockScreen, SuggestionSources,
};
use photon_net::connection::CommandBlockSettings;

/// The command block the player is editing: its screen and its position.
pub struct CommandBlockEdit {
    pub screen: CommandBlockScreen,
    pub pos: (i32, i32, i32),
}

impl CommandBlockEdit {
    /// The screen drawn at the size of the window.
    pub fn draw(
        &mut self,
        frame: &mut photon_render::gui::GuiFrame,
        (atlas, font, lang): (
            &photon_render::gui::GuiAtlas,
            &photon_render::gui::font::Font,
            &Lang,
        ),
        (mouse, size, commands): (
            (i32, i32),
            (i32, i32),
            Option<&photon_game::commands::CommandTree>,
        ),
    ) {
        if self.screen.size() != size {
            self.screen.resize(lang, size);
        }
        self.screen
            .draw((frame, atlas, font), mouse, (commands, lang));
    }
}

impl Controls {
    /// `Player.openCommandBlock`: the screen of the block just used opens over the game and the
    /// mouse is let go. Its controls wait for the block's data from the server.
    pub fn open_command_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let Some(pos) = session.take_opened_command_block() else {
            return;
        };
        let Some(block) = session.command_block(pos) else {
            return;
        };
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        self.command_block = Some(CommandBlockEdit {
            screen: CommandBlockScreen::new(
                (&assets.creative.font, &assets.lang),
                &block.contents,
                size,
            ),
            pos,
        });
        self.chat = None;
        self.pause = None;
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        let (width, height) = surface.window_size();
        surface.set_cursor_position(f64::from(width) / 2.0, f64::from(height) / 2.0);
    }

    /// `CommandBlockEditScreen.tick` and `handleBlockEntityData`: the screen closes once the block
    /// is gone, takes the data that arrived, and trades suggestions with the server.
    pub fn command_block_tick(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        let Some(edit) = self.command_block.as_mut() else {
            return Ok(());
        };
        let Some(block) = session.command_block(edit.pos) else {
            self.close_command_block(session, surface, None)?;
            return Ok(());
        };
        if session.take_command_block_data() {
            edit.screen.update_gui(
                &assets.lang,
                (block.mode, block.conditional),
                &block.contents,
            );
        }
        edit.screen.set_suggestion_sources(SuggestionSources {
            commands: session.commands().cloned(),
            selectors: session.can_use_selectors(),
            players: session.online_player_names(),
            teams: session.scoreboard().team_names(),
            selected_entities: session.selected_entities(),
        });
        if let Some(answer) = session.take_suggestions() {
            edit.screen
                .suggested(answer.id, answer.start, answer.length, answer.matches);
        }
        if let Some((id, text)) = edit.screen.take_request() {
            session.request_suggestions(id, &text)?;
        }
        Ok(())
    }

    /// `onClose`: Done sends the block's settings; the mouse is grabbed again.
    fn close_command_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        done: Option<photon_gui::screen::command_block::CommandBlockEdit>,
    ) -> Result {
        if let Some(edit) = self.command_block.take()
            && let Some(done) = done
        {
            session.set_command_block(
                edit.pos,
                &CommandBlockSettings {
                    command: &done.command,
                    mode: done.mode,
                    track_output: done.track_output,
                    conditional: done.conditional,
                    automatic: done.automatic,
                },
            )?;
        }
        self.captured = true;
        self.last_cursor = None;
        surface.set_cursor_captured(true);
        Ok(())
    }

    /// An event for the command block's screen.
    pub(super) fn command_block_event(
        &mut self,
        event: glfw::WindowEvent,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> Result {
        use glfw::{Action, Modifiers, WindowEvent};
        let Some(edit) = self.command_block.as_mut() else {
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
                CommandBlockAction::Stay
            }
            WindowEvent::CursorPos(x, y) => {
                self.move_gui_mouse((x, y), surface);
                CommandBlockAction::Stay
            }
            WindowEvent::MouseButton(glfw::MouseButton::Button1, Action::Press, modifiers) => edit
                .screen
                .clicked(lang, mouse, modifiers.contains(Modifiers::Shift)),
            WindowEvent::Scroll(_, y) => {
                edit.screen.scrolled(lang, y, mouse);
                CommandBlockAction::Stay
            }
            _ => CommandBlockAction::Stay,
        };
        match action {
            CommandBlockAction::Stay => Ok(()),
            CommandBlockAction::Close => self.close_command_block(session, surface, None),
            CommandBlockAction::Done(done) => {
                self.close_command_block(session, surface, Some(done))
            }
        }
    }
}
