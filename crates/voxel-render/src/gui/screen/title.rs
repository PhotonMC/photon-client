//! The title screen (`TitleScreen`): the logo, the yellow splash, the menu's buttons, the version
//! and the copyright line, over the panorama's overlay.

use voxel_game::text::{Lang, Run, Style};

use crate::gui::font::Font;
use crate::gui::widgets::Kind;
use crate::gui::widgets::button_set::ButtonSet;
use crate::gui::{GuiAtlas, GuiFrame};

/// What pressing a button asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleAction {
    Singleplayer,
    Multiplayer,
    Realms,
    Friends,
    Language,
    Accessibility,
    Options,
    Quit,
    Credits,
}

/// `SplashRenderer`'s place: right of the middle, below the logo, turned by a ninth of a half turn.
const SPLASH_OFFSET: (f32, f32) = (123.0, 69.0);
const SPLASH_ANGLE: f32 = (-std::f64::consts::PI / 9.0) as f32;

pub struct TitleScreen {
    buttons: ButtonSet<TitleAction>,
    splash: Option<Vec<Run>>,
    version: String,
}

fn runs(lang: &Lang, key: &str) -> Vec<Run> {
    lang.runs(
        &voxel_game::text::Component::translatable(key),
        &Style::default(),
    )
}

impl TitleScreen {
    /// `TitleScreen.init` and `createNormalMenuOptions` in a screen of `width` by `height` units,
    /// for a client that may play online (the Realms and friends services are not reached).
    pub fn new(
        font: &Font,
        lang: &Lang,
        (width, height): (i32, i32),
        splash: Option<&str>,
    ) -> Self {
        let mut buttons = ButtonSet::new((width, height));
        let button = |key: &str| Kind::Button {
            label: runs(lang, key),
        };
        let icon = |sprite: &str| Kind::Icon {
            sprite: sprite.to_owned(),
            size: (15, 15),
        };
        let mut top = height / 4 + 48;
        let left = width / 2 - 100;
        buttons.add(
            (left, top, 200, 20),
            (button("menu.singleplayer"), true),
            TitleAction::Singleplayer,
            None,
        );
        top += 24;
        buttons.add(
            (left, top, 200, 20),
            (button("menu.multiplayer"), true),
            TitleAction::Multiplayer,
            None,
        );
        top += 24;
        buttons.add(
            (left, top, 200, 20),
            (button("menu.online"), true),
            TitleAction::Realms,
            None,
        );
        // `getHorizontalPosition`: three icons twenty wide, four apart, centred.
        top += 24;
        let icons = [
            (
                "friends/friends",
                TitleAction::Friends,
                "gui.friends.open.tooltip",
            ),
            (
                "icon/language",
                TitleAction::Language,
                "options.language.tooltip",
            ),
            (
                "icon/accessibility",
                TitleAction::Accessibility,
                "options.accessibility.tooltip",
            ),
        ];
        for (index, (sprite, action, tooltip)) in (0..).zip(icons) {
            let x = width / 2 - (3 * 20 + 2 * 4) / 2 + index * 24;
            buttons.add(
                (x, top, 20, 20),
                (icon(sprite), true),
                action,
                Some((lang, tooltip)),
            );
        }
        top += 24;
        buttons.add(
            (left, top, 98, 20),
            (button("menu.options"), true),
            TitleAction::Options,
            None,
        );
        buttons.add(
            (width / 2 + 2, top, 98, 20),
            (button("menu.quit"), true),
            TitleAction::Quit,
            None,
        );
        let copyright = runs(lang, "title.credits");
        let copyright_width = font.width_of_runs(&copyright);
        buttons.add(
            (
                width - copyright_width - 2,
                height - 10,
                copyright_width,
                10,
            ),
            (Kind::PlainText { text: copyright }, true),
            TitleAction::Credits,
            None,
        );
        let splash = splash.map(|text| {
            vec![Run {
                text: text.to_owned(),
                style: Style {
                    colour: Some(voxel_game::splash::SPLASH_COLOUR & 0xFF_FFFF),
                    ..Style::default()
                },
            }]
        });
        Self {
            buttons,
            splash,
            version: "Minecraft 26.2".to_owned(),
        }
    }

    pub fn buttons(&mut self) -> &mut ButtonSet<TitleAction> {
        &mut self.buttons
    }

    /// The screen size the layout was made for.
    pub fn size(&self) -> (i32, i32) {
        self.buttons.size()
    }

    /// `extractRenderState`: the panorama's overlay, the buttons, the logo, the splash, the
    /// version, then the tooltip asked for.
    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        let (width, height) = self.size();
        // `Panorama.extractRenderState`: the overlay over the whole screen (the cube is not drawn).
        frame.blit_texture_stretched(
            atlas,
            "title/background/panorama_overlay",
            (16, 128),
            (0, 0, 16, 128),
            (0, 0, width, height),
        );
        self.buttons.draw(frame, atlas, font, mouse);
        // `LogoRenderer`: the logo thirty units down, the edition overlapping its foot by seven.
        frame.blit_texture(
            atlas,
            "title/minecraft",
            (256, 64),
            (0, 0),
            width / 2 - 128,
            30,
            256,
            44,
        );
        frame.blit_texture(
            atlas,
            "title/edition",
            (128, 16),
            (0, 0),
            width / 2 - 64,
            30 + 44 - 7,
            128,
            14,
        );
        if let Some(splash) = &self.splash {
            draw_splash(frame, atlas, font, splash, width);
        }
        frame.text_with_shadow(atlas, font, &self.version, 2, height - 10, 0xFFFF_FFFF);
        self.buttons.draw_tooltip(frame, atlas, font, mouse);
    }
}

/// `SplashRenderer.extractRenderState`: the text turned and scaled about its place, its size
/// beating once a second.
fn draw_splash(frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, splash: &[Run], width: i32) {
    let text_width = font.width_of_runs(splash);
    let angle =
        ((frame.millis % 1000) as f32 / 1000.0 * (std::f64::consts::PI * 2.0) as f32).into();
    let phase = 1.8_f32 - (voxel_core::math::mth_sin_f64(angle) * 0.1_f32).abs();
    let scale = phase * 100.0 / (text_width + 32) as f32;
    // `Matrix3x2f().translate(...).rotate(...).scale(...)`.
    let (sin, cos) = (
        f64::from(SPLASH_ANGLE).sin() as f32,
        f64::from(SPLASH_ANGLE).cos() as f32,
    );
    let pose = [
        cos * scale,
        sin * scale,
        -sin * scale,
        cos * scale,
        width as f32 / 2.0 + SPLASH_OFFSET.0,
        SPLASH_OFFSET.1,
    ];
    let mark = frame.mark();
    frame.styled_text(atlas, font, splash, -text_width / 2, -8, 0xFFFF_FFFF, true);
    frame.transform_from(mark, pose);
}
