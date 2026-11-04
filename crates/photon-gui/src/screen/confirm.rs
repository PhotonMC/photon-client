//! The confirmation screen (`ConfirmScreen`): a title, a message wrapped and
//! centred under it and two buttons, over the menu background (or the death
//! screen's gradient, for `DeathScreen.TitleConfirmScreen`).

use photon_game::text::{Component, Lang, Run, Style};

mod layout;

use crate::layout::Arena;
use crate::screen::death;
use crate::widgets::Widget;
use layout::{Content, POPUP_BUTTON_SPACING, POPUP_CONTENT, draw_centred, lay_out, link_content};
use photon_render::gui::font::Font;
use photon_render::gui::{GuiAtlas, GuiFrame};

/// What the screen's callback is told.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Yes,
    No,
    /// `ConfirmLinkScreen`'s "Copy to Clipboard".
    Copy,
}

/// What stands behind the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmBackground {
    /// `extractMenuBackground`: the tiled texture, the in-world one when a
    /// level is loaded.
    Menu { in_world: bool },
    /// `DeathScreen.extractDeathBackground`.
    Death,
    /// `PopupScreen`: a dim over the screen drawn behind, and the popup's panel.
    Popup,
}

pub struct ConfirmScreen {
    arena: Arena,
    title: Widget,
    /// The message's lines, and the place of the box they stand in.
    message: Vec<Vec<Run>>,
    message_rect: usize,
    /// The title's lines, when it wraps.
    title_lines: Option<Vec<Vec<Run>>>,
    buttons: Vec<(Widget, ConfirmAction)>,
    extra: Option<Widget>,
    size: (i32, i32),
    /// `delayTicker`: the buttons wake, and Escape opens, when it runs out.
    delay: i32,
    focus: Option<usize>,
    background: ConfirmBackground,
    content: Content,
    /// The layout's rectangle, when a dark panel stands behind it.
    panel: Option<(i32, i32, i32, i32)>,
}

/// `Font.substrByWidth`: the start of `runs` that fits in `width`.
pub(crate) fn head_by_width(runs: &[Run], font: &Font, width: i32) -> Vec<Run> {
    let mut left = width as f32;
    let mut head = Vec::new();
    for run in runs {
        let mut style = run.style.clone();
        let mut kept = String::new();
        let mut chars = run.text.chars();
        while let Some(c) = chars.next() {
            // A code of the text styles what follows and takes no room.
            if c == '\u{a7}' {
                let Some(code) = chars.next() else { break };
                style = crate::runs::styled_by(style, code.to_ascii_lowercase(), &run.style);
                kept.push(c);
                kept.push(code);
                continue;
            }
            let bold = style.bold == Some(true);
            left -= font.glyph(c).map_or(0.0, |glyph| glyph.advance(bold));
            if left < 0.0 {
                if !kept.is_empty() {
                    head.push(Run {
                        text: kept,
                        style: run.style.clone(),
                    });
                }
                return head;
            }
            kept.push(c);
        }
        if !kept.is_empty() {
            head.push(Run {
                text: kept,
                style: run.style.clone(),
            });
        }
    }
    head
}

impl ConfirmScreen {
    /// `ConfirmScreen.init`, for a title, a message and the two buttons'
    /// labels, in a screen of `width` by `height` units.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        title: &Component,
        message: &Component,
        labels: (&Component, &Component),
    ) -> Self {
        Self::two_buttons(font, lang, size, (title, message), labels, false)
    }

    /// `FriendsListConfirmScreen`: the same question over a dark panel.
    pub fn friends(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (title, message): (&Component, &Component),
        labels: (&Component, &Component),
    ) -> Self {
        Self::two_buttons(font, lang, size, (title, message), labels, true)
    }

    /// `PopupScreen` with one message and two buttons, over the screen that stays behind it.
    pub fn popup(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (title, message): (&Component, &Component),
        (yes, no): (&Component, &Component),
    ) -> Self {
        let bold = Style {
            bold: Some(true),
            ..Style::default()
        };
        let mut screen = Self::build(
            font,
            lang,
            size,
            Content {
                title: title.clone().with_style(&bold),
                message: message.clone(),
                buttons: vec![
                    (yes.clone(), ConfirmAction::Yes),
                    (no.clone(), ConfirmAction::No),
                ],
                button_width: (POPUP_CONTENT - POPUP_BUTTON_SPACING) / 2,
                extra: None,
                dark_panel: false,
                wrapped_title: true,
                popup: true,
            },
        );
        screen.background = ConfirmBackground::Popup;
        screen
    }

    /// A message with one button, which only closes it.
    pub fn notice(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (title, message): (&Component, &Component),
        label: &Component,
    ) -> Self {
        Self::build(
            font,
            lang,
            size,
            Content {
                title: title.clone(),
                message: message.clone(),
                buttons: vec![(label.clone(), ConfirmAction::No)],
                button_width: 150,
                extra: None,
                dark_panel: false,
                wrapped_title: false,
                popup: false,
            },
        )
    }

    fn two_buttons(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (title, message): (&Component, &Component),
        (yes, no): (&Component, &Component),
        dark_panel: bool,
    ) -> Self {
        Self::build(
            font,
            lang,
            size,
            Content {
                title: title.clone(),
                message: message.clone(),
                buttons: vec![
                    (yes.clone(), ConfirmAction::Yes),
                    (no.clone(), ConfirmAction::No),
                ],
                button_width: 150,
                extra: None,
                dark_panel,
                wrapped_title: false,
                popup: false,
            },
        )
    }

    /// `ClientCommonPacketListenerImpl.PackConfirmScreen`: the question about the server's
    /// resource packs, in the wording of a pack that is required or not, with the server's own
    /// prompt under it when it sent one.
    pub fn server_pack(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        required: bool,
        prompt: Option<&Component>,
    ) -> Self {
        let translated = Component::translatable;
        let (title, header, yes, no) = if required {
            (
                translated("multiplayer.requiredTexturePrompt.line1"),
                translated("multiplayer.requiredTexturePrompt.line2").with_style(&Style {
                    colour: photon_game::text::named_colour("yellow"),
                    bold: Some(true),
                    ..Style::default()
                }),
                translated("gui.proceed"),
                translated("menu.disconnect"),
            )
        } else {
            (
                translated("multiplayer.texturePrompt.line1"),
                translated("multiplayer.texturePrompt.line2"),
                translated("gui.yes"),
                translated("gui.no"),
            )
        };
        let message = match prompt {
            Some(prompt) => Component::translatable_with(
                "multiplayer.texturePrompt.serverPrompt",
                vec![header, prompt.clone()],
            ),
            None => header,
        };
        Self::new(font, lang, size, &title, &message, (&yes, &no))
    }

    /// `ClientPacketListener.openSendConfirmationWindow`: the message `message_key` names with the
    /// command in yellow, the accept button's key, and a way back (`GUI_BACK` over a screen,
    /// `GUI_CANCEL` over none).
    pub fn command(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        (message_key, command, accept): (&str, &str, &str),
        screen_below: bool,
    ) -> Self {
        let translated = Component::translatable;
        let message = Component::translatable_with(
            message_key,
            vec![Component::literal(command).with_colour("yellow")],
        );
        let back = translated(if screen_below {
            "gui.back"
        } else {
            "gui.cancel"
        });
        Self::new(
            font,
            lang,
            size,
            &translated("multiplayer.confirm_command.title"),
            &message,
            (&translated(accept), &back),
        )
    }

    /// `new ConfirmLinkScreen(callback, url, trusted)`: the link asked
    /// about, with "Copy to Clipboard" between the answers, and a warning
    /// when the link is not trusted.
    pub fn link(font: &Font, lang: &Lang, size: (i32, i32), url: &str, trusted: bool) -> Self {
        Self::build(font, lang, size, link_content(url, trusted))
    }

    /// `PrivacyConfirmLinkScreen`: the account's privacy page, with a yellow underlined title and
    /// the link under the message.
    pub fn privacy_link(font: &Font, lang: &Lang, size: (i32, i32), url: &str) -> Self {
        let mut content = link_content(url, true);
        content.title = Component::translatable("gui.privacy_link.title").with_style(&Style {
            colour: Some(0xFF_FF00),
            underlined: Some(true),
            ..Style::default()
        });
        content.message = Component::translatable("gui.privacy_link.message");
        content.extra = Some(Component::literal(url).with_style(&Style {
            colour: Some(0xFF_FFFF),
            ..Style::default()
        }));
        content.wrapped_title = true;
        Self::build(font, lang, size, content)
    }

    fn build(font: &Font, lang: &Lang, size: (i32, i32), content: Content) -> Self {
        let laid = lay_out(font, lang, size, &content);
        Self {
            title: laid.title,
            message: laid.message,
            message_rect: laid.message_rect,
            title_lines: laid.title_lines,
            buttons: laid.buttons,
            extra: laid.extra,
            arena: laid.arena,
            size,
            delay: 0,
            focus: None,
            background: ConfirmBackground::Menu { in_world: false },
            content,
            panel: laid.panel,
        }
    }

    #[must_use]
    pub fn with_background(mut self, background: ConfirmBackground) -> Self {
        self.background = background;
        self
    }

    /// `setDelay`: the buttons sleep, and Escape is shut, for `ticks`.
    pub fn set_delay(&mut self, ticks: i32) {
        self.delay = ticks;
        for (widget, _) in &mut self.buttons {
            widget.active = false;
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.size
    }

    /// The screen again for a screen of another size: as `rebuildWidgets`
    /// leaves it, with the delay as it was and the buttons new (and awake).
    #[must_use]
    pub fn resized(&self, font: &Font, lang: &Lang, size: (i32, i32)) -> Self {
        let mut screen =
            Self::build(font, lang, size, self.content.clone()).with_background(self.background);
        screen.delay = self.delay;
        screen
    }

    /// `tick`: the buttons wake when the delay runs out.
    pub fn tick(&mut self) {
        self.delay -= 1;
        if self.delay == 0 {
            for (widget, _) in &mut self.buttons {
                widget.active = true;
            }
        }
    }

    /// The background, then the panel behind the text when there is one.
    fn draw_backdrop(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, blurriness: i32) {
        let (width, height) = self.size;
        match self.background {
            ConfirmBackground::Menu { in_world } => {
                if in_world {
                    frame.blur_before_this_stratum(blurriness);
                }
                let texture = if in_world {
                    "inworld_menu_background"
                } else {
                    "menu_background"
                };
                frame.tile_texture(atlas, texture, (32, 32), (0, 0, width, height));
            }
            ConfirmBackground::Death => death::draw_background(frame, atlas, self.size),
            ConfirmBackground::Popup => {
                frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
            }
        }
        let Some((x, y, width, height)) = self.panel else {
            return;
        };
        if self.background == ConfirmBackground::Popup {
            let rect = (x - 18, y - 18, width + 36, height + 36);
            frame.nine_slice(atlas, "popup/background", (236, 34), 6, false, rect);
        } else {
            let rect = (x - 18, y - 18, width + 37, height + 37);
            frame.nine_slice(atlas, "friends/background_dark", (236, 34), 8, false, rect);
        }
    }

    /// The background (over a game, with the frame behind it blurred first), the title, the
    /// message, then the buttons.
    pub fn draw(
        &self,
        frame: &mut GuiFrame,
        (atlas, font): (&GuiAtlas, &Font),
        mouse: (i32, i32),
        blurriness: i32,
    ) {
        self.draw_backdrop(frame, atlas, blurriness);
        match &self.title_lines {
            Some(lines) => {
                let rect = self.arena.0[self.title.rect];
                draw_centred(
                    frame,
                    (atlas, font),
                    lines,
                    (rect.x + rect.width / 2, rect.y),
                );
            }
            None => self.title.draw(frame, atlas, font, &self.arena, false),
        }
        if let Some(extra) = &self.extra {
            extra.draw(frame, atlas, font, &self.arena, false);
        }
        let rect = self.arena.0[self.message_rect];
        let top = (rect.x + rect.width / 2, rect.y);
        draw_centred(frame, (atlas, font), &self.message, top);
        for (index, (widget, _)) in self.buttons.iter().enumerate() {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == Some(index);
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
    }

    /// A press of the primary button at `mouse`: the button it was on, if
    /// it was awake.
    pub fn clicked(&self, mouse: (i32, i32)) -> Option<ConfirmAction> {
        let (_, action) = self
            .buttons
            .iter()
            .find(|(widget, _)| widget.is_active() && widget.contains(&self.arena, mouse))?;
        crate::ui_sound::play_down_sound();
        Some(*action)
    }

    /// Tab (and Shift+Tab): the focus moves to the next awake button.
    pub fn tab(&mut self, forward: bool) {
        let awake: Vec<usize> = (0..self.buttons.len())
            .filter(|&index| self.buttons[index].0.is_active())
            .collect();
        let (Some(&first), Some(&last)) = (awake.first(), awake.last()) else {
            return;
        };
        let position = self
            .focus
            .and_then(|focus| awake.iter().position(|&index| index == focus));
        self.focus = Some(match (position, forward) {
            (None, true) => first,
            (None, false) => last,
            (Some(at), true) => awake.get(at + 1).copied().unwrap_or(first),
            (Some(at), false) => at.checked_sub(1).map_or(last, |before| awake[before]),
        });
    }

    /// Enter or Space: the focused button, if it is awake.
    pub fn pressed(&self) -> Option<ConfirmAction> {
        let (widget, action) = self.buttons.get(self.focus?)?;
        if !widget.is_active() {
            return None;
        }
        crate::ui_sound::play_down_sound();
        Some(*action)
    }

    /// Escape: "No", once the delay has run out.
    pub fn escape(&self) -> Option<ConfirmAction> {
        (self.delay <= 0).then_some(ConfirmAction::No)
    }
}
