use super::*;

impl VideoSettingsScreen {
    /// `init`, with the GUI scale as it is set, and the most the window
    /// allows.
    pub fn new(
        font: &Font,
        lang: &Lang,
        size: (i32, i32),
        in_world: bool,
        gui_scale: i32,
        max_gui_scale: i32,
        values: VideoValues,
    ) -> Self {
        let screen = SubScreen::new(font, lang, size, "options.videoTitle", in_world, |list| {
            list.add_header(font, lang, "options.video.display.header");
            list.add_big(
                lang,
                PLAIN,
                slider(
                    "options.fullscreen.resolution",
                    IntRange::new(-1, -1),
                    fullscreen_resolution,
                    -1,
                ),
            );
            list.add_small(lang, display_options(gui_scale, max_gui_scale, &values));
            list.add_header(font, lang, "options.video.quality.header");
            list.add_big(
                lang,
                GRAPHICS_PRESET,
                slider_tip(
                    "options.graphics.preset",
                    SliderableEnum::new(vec![
                        "options.graphics.fast",
                        "options.graphics.fancy",
                        "options.graphics.fabulous",
                        "options.graphics.custom",
                    ]),
                    graphics_preset,
                    "options.graphics.fancy",
                    constant_tip("options.graphics.preset.tooltip"),
                ),
            );
            list.add_small(lang, quality_options());
            list.add_header(font, lang, "options.video.preferences.header");
            list.add_small(lang, preference_options(values.attack_indicator));
        });
        let style = Style {
            colour: Some(RESTART_COLOUR),
            ..Style::default()
        };
        let restart = lang.runs(&Component::translatable("options.restartRequired"), &style);
        let restart_width = font.width_of_runs(&restart);
        let mut video = Self {
            screen,
            gui_scale,
            max_gui_scale,
            restart: (restart, restart_width),
        };
        video.tick();
        video
    }

    /// `tick`: the anisotropy slider is for the filtering that uses it, and
    /// the header says when a restart is needed
    /// (`isRestartRequiredToApplyVideoSettings`).
    pub fn tick(&mut self) {
        let list = &self.screen.list;
        let exclusive = list
            .find(EXCLUSIVE_FULLSCREEN)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
            .is_some_and(|button| *button.value());
        let api_changed = list
            .find(GRAPHICS_API)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
            .is_some_and(|button| *button.value() != "options.graphicsApi.default");
        self.screen
            .set_header_line((&self.restart.0, self.restart.1), exclusive || api_changed);
        let anisotropic = self
            .screen
            .list
            .find(TEXTURE_FILTERING)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
            .is_some_and(|button| *button.value() == "options.textureFiltering.anisotropic");
        if let Some(control) = self.screen.list.find_mut(MAX_ANISOTROPY) {
            control.set_active(anisotropic);
        }
    }

    pub fn size(&self) -> (i32, i32) {
        self.screen.size()
    }

    pub fn resize(&mut self, size: (i32, i32)) {
        self.screen.resize(size);
    }

    pub fn scroll_amount(&self) -> f64 {
        self.screen.list.scroll_amount()
    }

    /// What the buttons and the slider say, over `fallback` for any that is missing.
    pub fn values(&self, fallback: VideoValues) -> VideoValues {
        let list = &self.screen.list;
        VideoValues {
            attack_indicator: list
                .find(ATTACK_INDICATOR)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
                .and_then(|button| AttackIndicator::from_caption_key(button.value()))
                .unwrap_or(fallback.attack_indicator),
            framerate_limit: list
                .find(FRAMERATE_LIMIT)
                .and_then(|control| {
                    control
                        .as_any()
                        .downcast_ref::<OptionSlider<FramerateValues>>()
                })
                .map_or(fallback.framerate_limit, |slider| *slider.value()),
            vsync: list
                .find(VSYNC)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<bool>>())
                .map_or(fallback.vsync, |button| *button.value()),
            inactivity_fps_limit: list
                .find(INACTIVITY_FPS_LIMIT)
                .and_then(|control| control.as_any().downcast_ref::<CycleButton<Named>>())
                .and_then(|button| InactivityFpsLimit::from_caption_key(button.value()))
                .unwrap_or(fallback.inactivity_fps_limit),
        }
    }

    pub fn draw(&self, frame: &mut GuiFrame, atlas: &GuiAtlas, font: &Font, mouse: (i32, i32)) {
        self.screen.draw(frame, atlas, font, mouse);
    }

    /// `setGraphicsPresetToCustom`: the preset says Custom.
    pub(super) fn customise(&mut self, lang: &Lang, changed: Option<u8>) {
        if changed.is_none_or(|tag| tag & CUSTOMISING == 0) {
            return;
        }
        if let Some(preset) = self
            .screen
            .list
            .find_mut(GRAPHICS_PRESET)
            .and_then(|control| {
                control
                    .as_any_mut()
                    .downcast_mut::<OptionSlider<SliderableEnum<Named>>>()
            })
        {
            preset.set_current(lang, "options.graphics.custom");
        }
    }

    /// The GUI scale the button says, if it is another than the screen
    /// last took: the window follows, and the screen is laid out again.
    pub(super) fn follow_gui_scale(&mut self, resize: &mut dyn FnMut(i32) -> (i32, i32)) {
        let shown = self
            .screen
            .list
            .find(GUI_SCALE)
            .and_then(|control| control.as_any().downcast_ref::<CycleButton<i32>>())
            .map(|button| *button.value());
        if let Some(shown) = shown
            && shown != self.gui_scale
        {
            self.gui_scale = shown;
            let size = resize(shown);
            self.screen.resize(size);
        }
    }

    /// A press of the primary button. A new GUI scale is given to `resize`,
    /// which answers with the GUI size the window now has.
    pub fn clicked(
        &mut self,
        lang: &Lang,
        mouse: (f64, f64),
        shift: bool,
        resize: &mut dyn FnMut(i32) -> (i32, i32),
    ) -> SubScreenInput {
        let input = self.screen.clicked(lang, mouse, shift);
        self.customise(lang, input.changed);
        self.follow_gui_scale(resize);
        self.tick();
        input
    }

    /// Tab (and Shift+Tab).
    pub fn tab(&mut self, forward: bool) {
        self.screen.tab(forward);
        self.tick();
    }

    /// An arrow, Enter or Space: a new GUI scale is given to `resize`.
    pub fn key_pressed(
        &mut self,
        lang: &Lang,
        key: SliderKey,
        shift: bool,
        resize: &mut dyn FnMut(i32) -> (i32, i32),
    ) -> SubScreenInput {
        let input = self.screen.key_pressed(lang, key, shift);
        self.customise(lang, input.changed);
        self.follow_gui_scale(resize);
        self.tick();
        input
    }

    pub fn dragged(&mut self, lang: &Lang, mouse: (f64, f64), dy: f64) {
        let changed = self.screen.dragged(lang, mouse, dy);
        self.customise(lang, changed);
        self.tick();
    }

    pub fn released(&mut self, lang: &Lang) {
        self.screen.released(lang);
        self.tick();
    }

    /// `mouseScrolled`: the wheel scrolls the list; with Control it steps
    /// the GUI scale, from the automatic one down from the largest.
    pub fn scrolled(
        &mut self,
        lang: &Lang,
        mouse: (f64, f64),
        scroll_y: f64,
        control: bool,
        resize: &mut dyn FnMut(i32) -> (i32, i32),
    ) {
        if !control {
            self.screen.list.scrolled(mouse, scroll_y);
            return;
        }
        let old = self.gui_scale;
        let adjusted = if old == 0 {
            self.max_gui_scale + 1
        } else {
            old
        };
        let new = adjusted + scroll_y.signum() as i32 * i32::from(scroll_y != 0.0);
        if new != 0 && new <= self.max_gui_scale && new >= 0 {
            self.gui_scale = new;
            let size = resize(new);
            self.screen.resize(size);
            if let Some(button) = self
                .screen
                .list
                .find_mut(GUI_SCALE)
                .and_then(|control| control.as_any_mut().downcast_mut::<CycleButton<i32>>())
            {
                button.set_value(lang, new);
            }
            self.screen.list.set_scroll_amount(0.0);
        }
    }
}
