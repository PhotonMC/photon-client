//! The screens shown while the connection is made over again: `ServerReconfigScreen` while the
//! configuration phase runs again and `ConnectScreen` while a transfer connects. The window
//! keeps drawing and taking input; the connection works on its own thread.

use photon_gui::screen::connect::{ConnectScreen, ReconfigScreen};
use photon_net::connection::SwitchKind;

use super::*;

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The screen a switch shows, made for a window size.
enum Shown {
    Reconfiguring(ReconfigScreen),
    Transferring(ConnectScreen),
}

impl Shown {
    fn size(&self) -> (i32, i32) {
        match self {
            Self::Reconfiguring(screen) => screen.size(),
            Self::Transferring(screen) => screen.size(),
        }
    }
}

/// A switch in view: its screen, the ticks it has lasted and the mouse over it.
pub(crate) struct SwitchView {
    screen: Shown,
    ticks: u32,
    mouse: (i32, i32),
}

impl Play {
    /// The configuration phase again, a respawn or a change of dimension: what was open over the
    /// old level is closed, and nothing of the old level stays.
    pub(super) fn reset_level(&mut self) -> Result<()> {
        if self.session.take_reconfigured() {
            self.close_screens();
            // `Hud.onDisconnected`: the toasts go with the level.
            self.gui.clear_toasts();
        }
        self.end_switch();
        if self.session.take_level_reset() {
            self.backend.clear_sections()?;
            self.sections.clear();
            self.sky.reset(&self.session);
            self.particles.clear();
        }
        if self.session.take_level_swapped() {
            self.particles.forget_jukeboxes();
            if let Some(sound) = self.sound.as_mut() {
                sound.reset_level();
            }
        }
        Ok(())
    }

    fn switch_screen(&self, kind: SwitchKind, size: (i32, i32)) -> Shown {
        let (font, _) = self.gui.text_assets();
        let lang = self.gui.lang();
        match kind {
            SwitchKind::Reconfiguring => {
                let mut screen = ReconfigScreen::new(lang, font, size);
                screen.set_ticks(0);
                Shown::Reconfiguring(screen)
            }
            SwitchKind::Transferring => {
                let mut screen = ConnectScreen::new(lang, size);
                screen.set_status(lang, "connect.transferring");
                Shown::Transferring(screen)
            }
        }
    }

    /// An event of the window while a switch goes on: the screen takes the mouse, and the game
    /// below takes no input. Whether the event was taken.
    pub(super) fn switch_event(&mut self, event: &glfw::WindowEvent) -> bool {
        if self.session.switching().is_none() || !is_input(event) {
            return false;
        }
        let Some(view) = self.controls.switch.as_mut() else {
            return true;
        };
        match event {
            glfw::WindowEvent::CursorPos(x, y) => {
                let scale = self.surface.scale();
                let (width, height) = self.surface.window_size();
                view.mouse = (
                    (x * f64::from(scale.gui_scaled_width()) / f64::from(width.max(1))) as i32,
                    (y * f64::from(scale.gui_scaled_height()) / f64::from(height.max(1))) as i32,
                );
            }
            glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1, glfw::Action::Press, _) => {
                match &view.screen {
                    Shown::Reconfiguring(screen) if screen.disconnected(view.mouse) => {
                        // `connection.disconnect(ABORT_CONNECTION)`: the disconnected screen.
                        self.session.abort_switch();
                    }
                    Shown::Transferring(screen) if screen.cancelled(view.mouse) => {
                        // `ConnectScreen`'s Cancel goes back to the screen it came from.
                        self.session.abort_switch();
                        self.controls.leave = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        true
    }

    /// What was open over the old level is closed.
    pub(super) fn close_screens(&mut self) {
        self.controls.screen = None;
        self.controls.chat = None;
        self.controls.dialog = None;
        self.controls.dialog_warning = None;
    }

    /// The switch is over: its screen goes. A new level takes the mouse the screen let go of; a
    /// failure leaves it to the disconnected screen.
    pub(super) fn end_switch(&mut self) {
        if self.session.switching().is_some() || self.controls.switch.take().is_none() {
            return;
        }
        if self.controls.disconnected.is_none() {
            self.controls.captured = true;
            self.surface.set_cursor_captured(true);
        }
    }

    /// The ticks a switch lasts, which count to the Disconnect button; none run for the game.
    pub(super) fn tick_switch(&mut self) -> bool {
        if self.session.switching().is_none() {
            return false;
        }
        if self.accumulated >= TICK_LENGTH {
            self.accumulated -= TICK_LENGTH;
            if let Some(view) = self.controls.switch.as_mut() {
                view.ticks += 1;
                if let Shown::Reconfiguring(screen) = &mut view.screen {
                    screen.set_ticks(view.ticks);
                }
            }
        }
        true
    }

    /// A frame of the switch's screen, in the place of the level the switch cleared. The first
    /// frame closes what was open over the level and lets go of the mouse.
    pub(super) fn draw_switch(&mut self, kind: SwitchKind) -> Result<Flow> {
        let scale = self.surface.scale();
        let size = (scale.gui_scaled_width(), scale.gui_scaled_height());
        if self.controls.switch.is_none() {
            self.close_screens();
            self.controls.captured = false;
            self.surface.set_cursor_captured(false);
            self.controls.switch = Some(SwitchView {
                screen: self.switch_screen(kind, size),
                ticks: 0,
                mouse: (0, 0),
            });
        }
        if self
            .controls
            .switch
            .as_ref()
            .is_some_and(|view| view.screen.size() != size)
        {
            let ticks = self.controls.switch.as_ref().map_or(0, |view| view.ticks);
            let mut screen = self.switch_screen(kind, size);
            if let Shown::Reconfiguring(screen) = &mut screen {
                screen.set_ticks(ticks);
            }
            if let Some(view) = self.controls.switch.as_mut() {
                view.screen = screen;
            }
        }
        if let Some(view) = &self.controls.switch {
            self.gui
                .draw_menu_screen(
                    &mut self.backend,
                    &self.surface,
                    |frame, atlas, font| match &view.screen {
                        Shown::Reconfiguring(screen) => screen.draw(frame, atlas, font, view.mouse),
                        Shown::Transferring(screen) => screen.draw(frame, atlas, font, view.mouse),
                    },
                )?;
            self.backend.draw_scene([0.0, 0.0, 0.0, 1.0], IDENTITY)?;
        }
        Ok(Flow::Next)
    }
}
