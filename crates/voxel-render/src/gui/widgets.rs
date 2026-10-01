//! Buttons and labels (`Button`, `AbstractButton`, `SpriteIconButton`,
//! `StringWidget`): how each is drawn, hovered, focused and pressed.

use voxel_game::text::Run;

use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::{GuiAtlas, GuiFrame};

/// What a widget is.
#[derive(Debug, Clone)]
pub enum Kind {
    /// `Button.Plain`: a label in the button sprite.
    Button { label: Vec<Run> },
    /// `SpriteIconButton.CenteredIcon`: a sprite in the middle of the button.
    Icon { sprite: String, size: (i32, i32) },
    /// `StringWidget`: text, not drawn over a button.
    Text { text: Vec<Run> },
}

#[derive(Debug, Clone)]
pub struct Widget {
    /// Its place in the layout's arena.
    pub rect: usize,
    pub active: bool,
    pub kind: Kind,
}

/// `AbstractButton.SPRITES`: the sprite for a button's state.
fn button_sprite(active: bool, highlighted: bool) -> &'static str {
    match (active, highlighted) {
        (true, true) => "widget/button_highlighted",
        (true, false) => "widget/button",
        (false, _) => "widget/button_disabled",
    }
}

/// `-6250336`: the colour a label takes when its button is not active.
const INACTIVE_COLOUR: u32 = 0xA0_A0A0;

impl Widget {
    /// A widget can be focused and pressed: it is a button, and active.
    pub fn is_active(&self) -> bool {
        self.active && !matches!(self.kind, Kind::Text { .. })
    }

    /// `AbstractWidget.areCoordinatesInRectangle`.
    pub fn contains(&self, arena: &Arena, (x, y): (i32, i32)) -> bool {
        let rect = arena.0[self.rect];
        x >= rect.x && y >= rect.y && x < rect.x + rect.width && y < rect.y + rect.height
    }

    /// `extractRenderState`: the sprite and what is on it.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        font: &Font,
        arena: &Arena,
        highlighted: bool,
    ) {
        let rect = arena.0[self.rect];
        match &self.kind {
            Kind::Button { label } => {
                self.draw_sprite(frame, atlas, arena, highlighted);
                draw_label(
                    frame,
                    atlas,
                    font,
                    (label, self.active),
                    (rect.x + 2, rect.x + rect.width - 2),
                    (rect.y, rect.y + rect.height),
                );
            }
            Kind::Icon { sprite, size } => {
                self.draw_sprite(frame, atlas, arena, highlighted);
                let x = rect.x + rect.width / 2 - size.0 / 2;
                let y = rect.y + rect.height / 2 - size.1 / 2;
                frame.sprite(atlas, sprite, x, y, size.0, size.1);
            }
            Kind::Text { text } => {
                let y = rect.y + (rect.height - 9) / 2;
                frame.styled_text(atlas, font, text, rect.x, y, 0xFFFF_FFFF, true);
            }
        }
    }

    /// `extractDefaultSprite`: the button's sprite over its whole rectangle.
    fn draw_sprite(
        &self,
        frame: &mut GuiFrame,
        atlas: &GuiAtlas,
        arena: &Arena,
        highlighted: bool,
    ) {
        let rect = arena.0[self.rect];
        frame.nine_slice(
            atlas,
            button_sprite(self.active, highlighted),
            (200, 20),
            3,
            false,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        );
    }
}

/// `extractScrollingStringOverContents`: the label centred in what is
/// between the margins; a label too wide slides to and fro behind the
/// margins, by the frame's clock. A label of a widget that is not active
/// takes the grey of one that is not.
pub fn draw_label(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    font: &Font,
    (label, active): (&[Run], bool),
    (left, right): (i32, i32),
    (top, bottom): (i32, i32),
) {
    let mut runs = label.to_vec();
    if !active {
        for run in &mut runs {
            run.style.colour.get_or_insert(INACTIVE_COLOUR);
        }
    }
    let width = font.width_of_runs(&runs);
    let text_top = (top + bottom - 9) / 2 + 1;
    if width > right - left {
        // `ActiveTextCollector.defaultScrollingHelper`.
        let max_position = f64::from(width - (right - left));
        let time = frame.millis as f64 / 1000.0;
        let period = (max_position * 0.5).max(3.0);
        let alpha = ((std::f64::consts::PI / 2.0)
            * (std::f64::consts::PI * 2.0 * time / period).cos())
        .sin()
            / 2.0
            + 0.5;
        let position = alpha * max_position;
        let mark = frame.mark();
        frame.styled_text(
            atlas,
            font,
            &runs,
            left - position as i32,
            text_top,
            0xFFFF_FFFF,
            true,
        );
        frame.clip_from(mark, (left, top, right, bottom));
    } else {
        let centre = (left + right) / 2;
        let x = centre.clamp(left + width / 2, right - width / 2) - width / 2;
        frame.styled_text(atlas, font, &runs, x, text_top, 0xFFFF_FFFF, true);
    }
}
