use super::*;

use photon_core::ids::EntityId;
use photon_game::text::Lang;
use photon_gui::screen::command_block::{
    CommandBlockAction, CommandBlockScreen, SuggestionSources,
};
use photon_net::connection::CommandBlockSettings;

/// What the screen edits: a command block, or a command block minecart.
#[derive(Clone, Copy)]
pub enum CommandBlockTarget {
    Block((i32, i32, i32)),
    Minecart(EntityId),
}

/// The command block the player is editing: its screen and what it belongs to.
pub struct CommandBlockEdit {
    pub screen: CommandBlockScreen,
    pub target: CommandBlockTarget,
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
    /// `Player.openCommandBlock` and `openMinecartCommandBlock`: the screen of the block or
    /// minecart just used opens over the game and the mouse is let go. The block's controls wait
    /// for its data from the server.
    pub fn open_command_block(
        &mut self,
        session: &mut PlaySession,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        let size = surface.scale();
        let size = (size.gui_scaled_width(), size.gui_scaled_height());
        let fonts = (&assets.creative.font, &*assets.lang);
        let edit = if let Some(pos) = session.take_opened_command_block() {
            session.command_block(pos).map(|block| CommandBlockEdit {
                screen: CommandBlockScreen::new(fonts, &block.contents, size),
                target: CommandBlockTarget::Block(pos),
            })
        } else if let Some(id) = session.take_opened_command_minecart() {
            session
                .command_minecart(id)
                .map(|contents| CommandBlockEdit {
                    screen: CommandBlockScreen::minecart(fonts, &contents, size),
                    target: CommandBlockTarget::Minecart(id),
                })
        } else {
            None
        };
        let Some(edit) = edit else {
            return;
        };
        self.command_block = Some(edit);
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
        match edit.target {
            CommandBlockTarget::Block(pos) => {
                let Some(block) = session.command_block(pos) else {
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
            }
            CommandBlockTarget::Minecart(id) => {
                if session.command_minecart(id).is_none() {
                    self.close_command_block(session, surface, None)?;
                    return Ok(());
                }
            }
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
        if let Some(edit) = self.command_block.take() {
            if let CommandBlockTarget::Minecart(id) = edit.target {
                session.set_minecart_track_output(id, edit.screen.track_output());
            }
            if let Some(done) = done {
                send_command_block(session, edit.target, &done)?;
            }
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

/// What Done sends: the block's settings, or the minecart's command.
fn send_command_block(
    session: &mut PlaySession,
    target: CommandBlockTarget,
    done: &photon_gui::screen::command_block::CommandBlockEdit,
) -> Result {
    Ok(match (target, done.block) {
        (CommandBlockTarget::Block(pos), Some(block)) => session.set_command_block(
            pos,
            &CommandBlockSettings {
                command: &done.command,
                mode: block.mode,
                track_output: done.track_output,
                conditional: block.conditional,
                automatic: block.automatic,
            },
        ),
        (CommandBlockTarget::Minecart(id), _) => {
            session.set_command_minecart(id, &done.command, done.track_output)
        }
        (CommandBlockTarget::Block(_), None) => Ok(()),
    }?)
}
