//! The tooltips of the friends overlay's buttons (`Tooltip.create`, with no delay), shown while the
//! cursor is over the button.

use super::entries::{Button, Rect};
use super::*;
use crate::FrameRuns;
use photon_render::gui::{GuiAtlas, GuiFrame, Positioner, ScreenRect};

/// The buttons' tooltip texts as runs, made once.
pub(super) struct TooltipTexts {
    send: Vec<Run>,
    pub(super) empty_name: Vec<Run>,
    copy: Vec<Run>,
    unfriend: Vec<Run>,
    accept: Vec<Run>,
    decline: Vec<Run>,
    cancel: Vec<Run>,
    loading: Vec<Run>,
}

impl TooltipTexts {
    pub(super) fn new(lang: &Lang) -> Self {
        let key = |name: &str| lang.runs(&Component::translatable(name), &Style::default());
        Self {
            send: key("gui.friends.send_request"),
            empty_name: key("gui.friends.empty_nickname"),
            copy: key("gui.friends.copy_to_clipboard"),
            unfriend: key("gui.friends.unfriend"),
            accept: key("gui.friends.accept"),
            decline: key("gui.friends.decline"),
            cancel: key("gui.friends.cancel_request"),
            loading: key("gui.friends.button.loading"),
        }
    }
}

impl FriendsOverlay {
    /// The tooltip of the button under the cursor, and where the button is.
    pub(super) fn tooltip_at(&self, at: (i32, i32), font: &Font) -> Option<(&[Run], Rect)> {
        let texts = &self.texts.tooltips;
        if let Some((button, id, rect)) = self.button_under(at) {
            if self.pressed == Some(Pressed::Entry(button, id)) {
                return Some((&texts.loading, rect));
            }
            let lines = match button {
                Button::Remove => &texts.unfriend,
                Button::Accept => &texts.accept,
                Button::Decline => &texts.decline,
                Button::Revoke => &texts.cancel,
            };
            return Some((lines, rect));
        }
        let send = self.button_rect();
        if draw::inside(at, send) {
            if self.pressed == Some(Pressed::Send) {
                return Some((&texts.loading, send));
            }
            let empty = self.box_active() && self.edit.value().trim().is_empty();
            return Some((
                if empty {
                    &texts.empty_name
                } else {
                    &texts.send
                },
                send,
            ));
        }
        let name = self.name_rect(font);
        draw::inside(at, name).then_some((texts.copy.as_slice(), name))
    }

    pub(super) fn draw_tooltip(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
    ) {
        let Some((lines, (x, y, width, height))) = self.tooltip_at(mouse, font) else {
            return;
        };
        let rect = ScreenRect {
            x,
            y,
            width,
            height,
        };
        frame.tooltip_with(
            atlas,
            font,
            &[lines.to_vec()],
            &Positioner::Menu(rect),
            mouse,
            self.size,
        );
    }
}
