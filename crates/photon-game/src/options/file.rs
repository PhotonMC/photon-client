use super::*;

impl Options {
    /// `Options.load`: `path` read over the defaults; a missing file leaves
    /// them, a line or a value that does not read is skipped.
    pub fn load(path: &Path) -> Self {
        let mut options = Self::default();
        let Ok(bytes) = std::fs::read(path) else {
            return options;
        };
        for line in String::from_utf8_lossy(&bytes).lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            match options.lines.iter_mut().find(|(known, _)| known == key) {
                Some(entry) => value.clone_into(&mut entry.1),
                None => options.lines.push((key.to_owned(), value.to_owned())),
            }
        }
        for (name, key) in KEYS {
            let value = options
                .lines
                .iter()
                .find(|(known, _)| known == name)
                .map(|(_, value)| value.clone());
            if let Some(value) = value {
                options.read(key, &value);
            }
        }
        options.read_key_bindings();
        options
    }

    /// `read` for the sound options.
    pub(super) fn read_sound(&mut self, key: Key, json: &Json) {
        match (key, json) {
            (Key::ShowSubtitles, Json::Bool(value)) => self.show_subtitles = *value,
            (Key::HideLightningFlashes, Json::Bool(value)) => self.hide_lightning_flashes = *value,
            (Key::DirectionalAudio, Json::Bool(value)) => self.directional_audio = *value,
            (Key::SoundDevice, Json::Text(device)) => self.sound_device.clone_from(device),
            (Key::MusicFrequency, Json::Text(name)) => {
                if let Some(value) = MusicFrequency::from_serialized(name) {
                    self.music_frequency = value;
                }
            }
            (Key::MusicToast, Json::Text(name)) => {
                if let Some(value) = MusicToast::from_serialized(name) {
                    self.music_toast = value;
                }
            }
            (Key::SoundCategory(source), Json::Number(number)) => {
                if let Some(value) = UnitDouble::validate_value(double_value(number)) {
                    self.sound_volumes[source.ordinal() as usize] = value;
                }
            }
            (Key::SoundCategory(source), Json::Bool(value)) => {
                self.sound_volumes[source.ordinal() as usize] = if *value { 1.0 } else { 0.0 };
            }
            _ => {}
        }
    }

    /// `read` for the main hand and the skin's parts.
    /// The options that come in groups: the skin's and the quality options.
    fn read_group(&mut self, key: Key, json: &Json) {
        match key {
            Key::Quality(key) => self.quality.read(key, json),
            Key::Online(key) => self.online.read(key, json),
            _ => self.read_skin(key, json),
        }
    }

    fn write_group(&self, key: Key) -> String {
        match key {
            Key::Quality(key) => self.quality.write(key),
            Key::Online(key) => self.online.write(key),
            Key::ModelPart(bit) => self.is_model_part_enabled(bit).to_string(),
            _ => String::new(),
        }
    }

    fn read_skin(&mut self, key: Key, json: &Json) {
        match (key, json) {
            (Key::MainHand, Json::Text(name)) => {
                if let Some(value) = MainHand::from_serialized(name) {
                    self.main_hand = value;
                }
            }
            (Key::ModelPart(bit), Json::Bool(shown)) => self.set_model_part(bit, *shown),
            _ => {}
        }
    }

    /// `option.codec().parse` and `set`: a value its codec refuses leaves
    /// the option as it was; one its value set refuses falls to the default.
    /// The options that are a number from zero to one, with the field each is kept in.
    fn unit_slots(&mut self) -> [(Key, &mut f64); 13] {
        [
            (Key::MouseSensitivity, &mut self.sensitivity),
            (Key::ChatOpacity, &mut self.chat_opacity),
            (Key::Gamma, &mut self.gamma),
            (Key::ScreenEffectScale, &mut self.screen_effect_scale),
            (Key::ChatLineSpacing, &mut self.chat_line_spacing),
            (
                Key::TextBackgroundOpacity,
                &mut self.text_background_opacity,
            ),
            (Key::ChatHeightFocused, &mut self.chat_height_focused),
            (Key::ChatHeightUnfocused, &mut self.chat_height_unfocused),
            (Key::ChatScale, &mut self.chat_scale),
            (Key::ChatWidth, &mut self.chat_width),
            (Key::GlintSpeed, &mut self.glint_speed),
            (Key::GlintStrength, &mut self.glint_strength),
            (Key::DamageTiltStrength, &mut self.damage_tilt_strength),
        ]
    }

    /// [`Self::unit_slots`]'s values.
    fn unit_values(&self) -> [(Key, f64); 13] {
        [
            (Key::MouseSensitivity, self.sensitivity),
            (Key::ChatOpacity, self.chat_opacity),
            (Key::Gamma, self.gamma),
            (Key::ScreenEffectScale, self.screen_effect_scale),
            (Key::ChatLineSpacing, self.chat_line_spacing),
            (Key::TextBackgroundOpacity, self.text_background_opacity),
            (Key::ChatHeightFocused, self.chat_height_focused),
            (Key::ChatHeightUnfocused, self.chat_height_unfocused),
            (Key::ChatScale, self.chat_scale),
            (Key::ChatWidth, self.chat_width),
            (Key::GlintSpeed, self.glint_speed),
            (Key::GlintStrength, self.glint_strength),
            (Key::DamageTiltStrength, self.damage_tilt_strength),
        ]
    }

    /// `read` for the field of view, the effect scales and the GUI scale.
    fn read_scale(&mut self, key: Key, json: &Json) {
        match (key, json) {
            (Key::Fov, Json::Number(number)) => {
                let degrees = (double_value(number) * 40.0 + 70.0) as i32;
                self.fov = FOV.validate_value(degrees).unwrap_or(FOV_DEFAULT);
            }
            // `Codec.doubleRange(0, 1)` of the real value, which `set` then takes through the
            // value set's `validateValue`: the square of the root.
            (Key::FovEffectScale, Json::Number(number)) => {
                let value = double_value(number);
                if (0.0..=1.0).contains(&value) {
                    self.fov_effect_scale = squared_through_root(value);
                }
            }
            // `UnitDouble.xmap(square, sqrt)`: the file holds the root (a boolean reads as one or
            // nothing, as the unit codec has it).
            (Key::DarknessEffectScale, json) => {
                if let Some(root) = unit_of(json) {
                    self.darkness_effect_scale = squared_through_root(root * root);
                }
            }
            (Key::GuiScale, Json::Number(number)) => {
                let scale = int_value(number);
                if (0..=GUI_SCALE_MAX + 1).contains(&scale) {
                    self.gui_scale =
                        ClampingLazyMaxIntRange::new(0, || GUI_SCALE_MAX, GUI_SCALE_MAX)
                            .validate_value(scale);
                }
            }
            _ => {}
        }
    }

    /// The options that are an integer in a range; a value outside it leaves the option as it was:
    /// `Codec.intRange(10, 260)`, `IntRange(2, 32)` and `IntRange(0, 10)`.
    fn read_int(&mut self, key: Key, json: &Json<'_>) {
        let (field, range) = match key {
            Key::MaxFps => (&mut self.framerate_limit, 10..=UNLIMITED_FRAMERATE_CUTOFF),
            Key::RenderDistance => (
                &mut self.render_distance,
                RENDER_DISTANCE_MIN..=RENDER_DISTANCE_MAX,
            ),
            _ => (&mut self.menu_background_blurriness, 0..=10),
        };
        *field = int_in(json, range).unwrap_or(*field);
    }

    /// The options that are a number from zero to one.
    fn read_unit(&mut self, key: Key, json: &Json) {
        let value = match *json {
            Json::Number(number) => UnitDouble::validate_value(double_value(number)),
            Json::Bool(value) => Some(if value { 1.0 } else { 0.0 }),
            Json::Text(_) | Json::Other => None,
        };
        if let (Some(value), Some((_, slot))) = (
            value,
            self.unit_slots().into_iter().find(|(own, _)| *own == key),
        ) {
            *slot = value;
        }
    }

    pub(super) fn read(&mut self, key: Key, text: &str) {
        let json = parse_json(text);
        let boolean = || json.as_bool();
        match key {
            Key::Fov | Key::FovEffectScale | Key::DarknessEffectScale | Key::GuiScale => {
                self.read_scale(key, &json);
            }
            Key::AmbientOcclusion | Key::EnableVsync => self.read_switch(key, boolean()),
            Key::MaxFps | Key::RenderDistance | Key::MenuBackgroundBlurriness => {
                self.read_int(key, &json);
            }
            Key::InactivityFpsLimit => self.read_inactivity_limit(&json),
            Key::InvertXMouse => self.invert_mouse_x = boolean().unwrap_or(self.invert_mouse_x),
            Key::SaveChatDrafts
            | Key::ChatColors
            | Key::BackgroundForChatOnly
            | Key::AdvancedItemTooltips
            | Key::PauseOnLostFocus
            | Key::ViewBobbing
            | Key::ForceUnicodeFont
            | Key::JapaneseGlyphVariants
            | Key::HighContrastBlockOutline => self.read_flag(key, boolean()),
            Key::InvertYMouse => self.invert_mouse_y = boolean().unwrap_or(self.invert_mouse_y),
            Key::AttackIndicator | Key::ChatVisibility | Key::Particles => {
                self.read_choice(key, &json);
            }
            Key::ShowSubtitles
            | Key::HideLightningFlashes
            | Key::DirectionalAudio
            | Key::SoundDevice
            | Key::MusicFrequency
            | Key::MusicToast
            | Key::SoundCategory(_) => self.read_sound(key, &json),
            Key::Lang => text.clone_into(&mut self.language_code),
            Key::LastServer | Key::JoinedFirstServer | Key::TutorialStep => {
                self.read_server(key, text, boolean());
            }
            Key::MouseSensitivity
            | Key::ChatOpacity
            | Key::Gamma
            | Key::ScreenEffectScale
            | Key::ChatLineSpacing
            | Key::TextBackgroundOpacity
            | Key::ChatHeightFocused
            | Key::ChatHeightUnfocused
            | Key::ChatScale
            | Key::ChatWidth
            | Key::GlintSpeed
            | Key::GlintStrength
            | Key::DamageTiltStrength => {
                self.read_unit(key, &json);
            }
            Key::MainHand | Key::ModelPart(_) | Key::Quality(_) | Key::Online(_) => {
                self.read_group(key, &json);
            }
            Key::MouseWheelSensitivity | Key::NotificationDisplayTime | Key::ChatDelay => {
                self.read_range(key, &json);
            }
        }
    }

    /// `isModelPartEnabled`, by the part's bit.
    pub fn is_model_part_enabled(&self, bit: usize) -> bool {
        self.model_parts & (1 << bit) != 0
    }

    /// `setModelPart`.
    pub fn set_model_part(&mut self, bit: usize, shown: bool) {
        if shown {
            self.model_parts |= 1 << bit;
        } else {
            self.model_parts &= !(1 << bit);
        }
    }

    /// `inactivityFpsLimit`: a known name leaves the limit as it is read, anything else keeps it.
    fn read_inactivity_limit(&mut self, json: &Json) {
        if let Json::Text(name) = json
            && let Some(value) = InactivityFpsLimit::from_serialized(name)
        {
            self.inactivity_fps_limit = value;
        }
    }

    /// A plain switch the file holds; anything but a boolean leaves it. They are the chat's two
    /// (saving drafts, the text background being the chat's alone) and F3+H and F3+P's.
    fn read_flag(&mut self, key: Key, value: Option<bool>) {
        let flag = match key {
            Key::SaveChatDrafts => &mut self.save_chat_drafts,
            Key::ChatColors => &mut self.chat_colors,
            Key::AdvancedItemTooltips => &mut self.advanced_item_tooltips,
            Key::PauseOnLostFocus => &mut self.pause_on_lost_focus,
            Key::ViewBobbing => &mut self.view_bobbing,
            Key::HighContrastBlockOutline => &mut self.high_contrast_block_outline,
            Key::ForceUnicodeFont => &mut self.font.force_unicode,
            Key::JapaneseGlyphVariants => &mut self.font.japanese_glyph_variants,
            _ => &mut self.background_for_chat_only,
        };
        *flag = value.unwrap_or(*flag);
    }

    fn flag(&self, key: Key) -> bool {
        match key {
            Key::SaveChatDrafts => self.save_chat_drafts,
            Key::ChatColors => self.chat_colors,
            Key::AdvancedItemTooltips => self.advanced_item_tooltips,
            Key::PauseOnLostFocus => self.pause_on_lost_focus,
            Key::ViewBobbing => self.view_bobbing,
            Key::HighContrastBlockOutline => self.high_contrast_block_outline,
            Key::ForceUnicodeFont => self.font.force_unicode,
            Key::JapaneseGlyphVariants => self.font.japanese_glyph_variants,
            _ => self.background_for_chat_only,
        }
    }

    fn range_value(&self, key: Key) -> f64 {
        match key {
            Key::NotificationDisplayTime => self.notification_display_time,
            Key::ChatDelay => self.chat_delay,
            _ => self.mouse_wheel_sensitivity,
        }
    }

    /// The options of a range: the mouse wheel's (`Codec.doubleRange` of its log scale, in steps)
    /// and the notification time's (`Codec.doubleRange(0.5, 10.0)`); a value out of its range
    /// leaves the option as it was.
    fn read_range(&mut self, key: Key, json: &Json<'_>) {
        let Json::Number(number) = json else {
            return;
        };
        let value = double_value(number);
        if key == Key::ChatDelay {
            // The codec takes 0 to 6; the value set then keeps the tenths (`xmap` of an
            // `IntRange(0, 60)`), cut off rather than rounded.
            if (0.0..=6.0).contains(&value) {
                self.chat_delay = f64::from((value * 10.0) as i32) / 10.0;
            }
        } else if key == Key::NotificationDisplayTime {
            if (0.5..=10.0).contains(&value) {
                self.notification_display_time = value;
            }
        } else if (log_mouse(-200)..=log_mouse(100)).contains(&value) {
            self.mouse_wheel_sensitivity = mouse_wheel_step(value).unwrap_or(log_mouse(0));
        }
    }

    fn read_switch(&mut self, key: Key, value: Option<bool>) {
        let switch = if key == Key::AmbientOcclusion {
            &mut self.ambient_occlusion
        } else {
            &mut self.enable_vsync
        };
        *switch = value.unwrap_or(*switch);
    }

    fn write_switch(&self, key: Key) -> String {
        if key == Key::AmbientOcclusion {
            self.ambient_occlusion
        } else {
            self.enable_vsync
        }
        .to_string()
    }

    /// The options that are one of a few, by their id.
    /// The id of an option that is one of three choices.
    fn choice_id(&self, key: Key) -> i32 {
        match key {
            Key::AttackIndicator => self.attack_indicator.id(),
            Key::Particles => self.particles.id(),
            _ => self.chat_visibility.id(),
        }
    }

    fn read_choice(&mut self, key: Key, json: &Json) {
        let Json::Number(number) = json else {
            return;
        };
        // `ByIdMap` wraps an id out of range.
        let id = int_value(number).rem_euclid(3);
        match key {
            Key::AttackIndicator => {
                self.attack_indicator = AttackIndicator::ALL
                    .into_iter()
                    .find(|value| value.id() == id)
                    .unwrap_or(self.attack_indicator);
            }
            Key::Particles => {
                self.particles = ParticleStatus::from_id(id).unwrap_or(self.particles);
            }
            _ => self.chat_visibility = ChatVisibility::from_id(id).unwrap_or(self.chat_visibility),
        }
    }

    /// The options about what the player has done: the last server joined, whether one was joined at
    /// all, and the tutorial's step.
    fn read_server(&mut self, key: Key, text: &str, flag: Option<bool>) {
        match key {
            Key::LastServer => text.clone_into(&mut self.last_server),
            Key::TutorialStep => self.tutorial_step = TutorialStep::by_name(text),
            _ => self.joined_first_server = flag.unwrap_or(self.joined_first_server),
        }
    }

    fn server_state(&self, key: Key) -> String {
        match key {
            Key::LastServer => self.last_server.clone(),
            Key::TutorialStep => self.tutorial_step.name().to_owned(),
            _ => self.joined_first_server.to_string(),
        }
    }

    /// `write` for the field of view and the effect scales.
    fn write_scale(&self, key: Key) -> String {
        java_double(match key {
            Key::Fov => (f64::from(self.fov) - 70.0) / 40.0,
            Key::FovEffectScale => self.fov_effect_scale,
            _ => self.darkness_effect_scale.sqrt(),
        })
    }

    /// The value of an option that is an integer in a range ([`Self::read_int`]).
    fn int_value(&self, key: Key) -> i32 {
        match key {
            Key::MaxFps => self.framerate_limit,
            Key::RenderDistance => self.render_distance,
            _ => self.menu_background_blurriness,
        }
    }

    /// The value the file holds for an option.
    pub(super) fn write(&self, key: Key) -> String {
        match key {
            Key::AmbientOcclusion | Key::EnableVsync => self.write_switch(key),
            Key::MaxFps | Key::RenderDistance | Key::MenuBackgroundBlurriness => {
                self.int_value(key).to_string()
            }
            Key::InactivityFpsLimit => write_string(self.inactivity_fps_limit.serialized()),
            Key::Fov | Key::FovEffectScale | Key::DarknessEffectScale => self.write_scale(key),
            Key::GuiScale => self.gui_scale.to_string(),
            Key::InvertXMouse => self.invert_mouse_x.to_string(),
            Key::SaveChatDrafts
            | Key::ChatColors
            | Key::BackgroundForChatOnly
            | Key::AdvancedItemTooltips
            | Key::PauseOnLostFocus
            | Key::ViewBobbing
            | Key::ForceUnicodeFont
            | Key::JapaneseGlyphVariants
            | Key::HighContrastBlockOutline => self.flag(key).to_string(),
            Key::InvertYMouse => self.invert_mouse_y.to_string(),
            Key::Lang => self.language_code.clone(),
            Key::LastServer | Key::JoinedFirstServer | Key::TutorialStep => self.server_state(key),
            Key::ChatVisibility | Key::Particles | Key::AttackIndicator => {
                self.choice_id(key).to_string()
            }
            Key::MouseSensitivity
            | Key::ChatOpacity
            | Key::Gamma
            | Key::ScreenEffectScale
            | Key::ChatLineSpacing
            | Key::TextBackgroundOpacity
            | Key::ChatHeightFocused
            | Key::ChatHeightUnfocused
            | Key::ChatScale
            | Key::ChatWidth
            | Key::GlintSpeed
            | Key::GlintStrength
            | Key::DamageTiltStrength => {
                let value = self.unit_values().into_iter().find(|(own, _)| *own == key);
                java_double(value.map_or(0.0, |(_, value)| value))
            }
            Key::MainHand => write_string(self.main_hand.serialized()),
            Key::ModelPart(_) | Key::Quality(_) | Key::Online(_) => self.write_group(key),
            Key::MouseWheelSensitivity | Key::NotificationDisplayTime | Key::ChatDelay => {
                java_double(self.range_value(key))
            }
            Key::ShowSubtitles => self.show_subtitles.to_string(),
            Key::HideLightningFlashes => self.hide_lightning_flashes.to_string(),
            Key::DirectionalAudio => self.directional_audio.to_string(),
            Key::SoundDevice => write_string(&self.sound_device),
            Key::MusicFrequency => write_string(self.music_frequency.serialized()),
            Key::MusicToast => write_string(self.music_toast.serialized()),
            Key::SoundCategory(source) => {
                java_double(self.sound_volumes[source.ordinal() as usize])
            }
        }
    }

    /// `Options.save`: the version, then the lines as they were read with
    /// the values this holds, then the options the file did not have.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut text = format!("version:{DATA_VERSION}\n");
        let mut written = Vec::new();
        for (name, value) in &self.lines {
            if name == "version" {
                continue;
            }
            let known = KEYS.iter().find(|(known, _)| known == name);
            let value = known.map_or_else(
                || self.key_binding_line(name).unwrap_or_else(|| value.clone()),
                |(_, key)| self.write(*key),
            );
            text.push_str(&format!("{name}:{value}\n"));
            written.push(name.as_str());
        }
        for (name, key) in KEYS {
            if !written.contains(&name) {
                text.push_str(&format!("{name}:{}\n", self.write(key)));
            }
        }
        for (name, value) in self.missing_key_lines() {
            text.push_str(&format!("{name}:{value}\n"));
        }
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        std::fs::write(path, text)
    }
}

/// `UnitDouble.validateValue` of the root, mapped back by `square`: what `set` keeps of a value
/// of a `UnitDouble.xmap(square, sqrt)` option.
fn squared_through_root(value: f64) -> f64 {
    let root = value.sqrt();
    root * root
}

/// `UnitDouble.codec`: a number from zero to one, or a boolean as one or nothing.
fn unit_of(json: &Json<'_>) -> Option<f64> {
    match json {
        Json::Number(number) => UnitDouble::validate_value(double_value(number)),
        Json::Bool(value) => Some(if *value { 1.0 } else { 0.0 }),
        Json::Text(_) | Json::Other => None,
    }
}

/// A whole number of the file within `range` (`Codec.intRange`); none for one outside it.
fn int_in(json: &Json<'_>, range: std::ops::RangeInclusive<i32>) -> Option<i32> {
    match json {
        Json::Number(number) => Some(int_value(number)).filter(|value| range.contains(value)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaded(text: &str) -> Options {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let number = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("photon-options-{}-{number}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("options.txt");
        std::fs::write(&path, text).unwrap();
        let options = Options::load(&path);
        std::fs::remove_dir_all(&directory).unwrap();
        options
    }

    #[test]
    fn the_online_options_read_and_write_as_the_file_holds_them() {
        let options = loaded(
            "realmsNotifications:false\ninGameNotification:true\nsharePresence:\"limited\"\n",
        );
        assert!(!options.online.realms_notifications);
        assert!(options.online.in_game_notification);
        assert_eq!(options.online.share_presence, PresenceSharing::Limited);
        assert_eq!(
            options.write(Key::Online(OnlineKey::SharePresence)),
            "\"limited\""
        );
        let refused = loaded("realmsNotifications:1\nsharePresence:\"everyone\"\n");
        assert_eq!(refused.online, OnlineOptions::default());
    }

    #[test]
    fn the_menu_blurriness_reads_in_range_and_keeps_the_default_outside_it() {
        assert_eq!(
            loaded("menuBackgroundBlurriness:0\n").menu_background_blurriness,
            0
        );
        assert_eq!(
            loaded("menuBackgroundBlurriness:10\n").menu_background_blurriness,
            10
        );
        assert_eq!(
            loaded("menuBackgroundBlurriness:11\n").menu_background_blurriness,
            5
        );
        assert_eq!(
            loaded("menuBackgroundBlurriness:-1\n").menu_background_blurriness,
            5
        );
        assert_eq!(loaded("").menu_background_blurriness, 5);
    }

    #[test]
    fn a_key_line_rebinds_its_mapping_and_every_mapping_is_written() {
        use photon_data::keys::InputKey;
        let options = loaded(
            "key_key.jump:key.keyboard.j\nkey_key.attack:key.mouse.4\nkey_key.drop:nonsense\n",
        );
        assert_eq!(options.key_binding("key.jump"), Some(InputKey::Keysym(74)));
        assert_eq!(options.key_binding("key.attack"), Some(InputKey::Mouse(3)));
        // A name that does not read leaves the default.
        assert_eq!(options.key_binding("key.drop"), Some(InputKey::Keysym(81)));
        assert_eq!(
            options.key_binding("key.forward"),
            Some(InputKey::Keysym(87))
        );
        assert_eq!(
            options
                .mappings_of(InputKey::Keysym(74))
                .collect::<Vec<_>>(),
            ["key.jump"]
        );
        assert_eq!(
            options.key_binding_line("key_key.jump").as_deref(),
            Some("key.keyboard.j")
        );
        assert!(
            options
                .missing_key_lines()
                .contains(&("key_key.forward".to_owned(), "key.keyboard.w".to_owned()))
        );
    }

    #[test]
    fn the_skin_parts_and_the_main_hand_are_read_and_written() {
        let options = loaded(
            "modelPart_cape:false\nmodelPart_hat:false\nmainHand:\"left\"\nmodelPart_jacket:1",
        );
        assert_eq!(options.model_parts, 0b011_1110 & !0b100_0000);
        assert_eq!(options.main_hand, MainHand::Left);
        assert_eq!(options.write(Key::ModelPart(0)), "false");
        assert_eq!(options.write(Key::ModelPart(1)), "true");
        assert_eq!(options.write(Key::MainHand), "\"left\"");
        // An unknown hand leaves the default.
        assert_eq!(loaded("mainHand:\"up\"").main_hand, MainHand::Right);
    }

    #[test]
    fn the_defaults_are_the_references() {
        let options = Options::default();
        assert_eq!((options.fov, options.gui_scale), (70, 0));
        assert_eq!(
            (options.sensitivity, options.mouse_wheel_sensitivity),
            (0.5, 1.0)
        );
        assert_eq!(
            (options.chat_opacity, options.chat_scale, options.chat_width),
            (1.0, 1.0, 1.0)
        );
        assert_eq!(options.chat_height_unfocused, 0.4375);
        assert_eq!(options.text_background_opacity, 0.5);
        assert_eq!(options.language_code, "en_us");
    }

    #[test]
    fn the_field_of_view_is_stored_as_a_share_of_its_range() {
        // `(int)(value * 40.0 + 70.0)`; outside 30 to 110 the default holds.
        assert_eq!(loaded("fov:0.5").fov, 90);
        assert_eq!(loaded("fov:-1.0").fov, 30);
        assert_eq!(loaded("fov:1.0").fov, 110);
        assert_eq!(loaded("fov:2.0").fov, 70);
        assert_eq!(loaded("fov:-2.0").fov, 70);
        assert_eq!(loaded("fov:true").fov, 70);
    }

    #[test]
    fn the_chat_delay_keeps_its_tenths_and_stays_within_six_seconds() {
        // The codec takes 0 to 6, and the value set cuts off at a tenth.
        assert_eq!(
            loaded("tutorialStep:punch_tree").tutorial_step,
            TutorialStep::PunchTree
        );
        assert_eq!(
            loaded("tutorialStep:bogus").tutorial_step,
            TutorialStep::None
        );
        assert_eq!(loaded("").tutorial_step, TutorialStep::Movement);
        assert_eq!(loaded("chatDelay:2.5").chat_delay, 2.5);
        assert_eq!(loaded("chatDelay:0.29").chat_delay, 0.2);
        assert_eq!(loaded("chatDelay:6.0").chat_delay, 6.0);
        assert_eq!(loaded("chatDelay:6.5").chat_delay, 0.0);
        assert_eq!(loaded("chatDelay:-0.5").chat_delay, 0.0);
        assert_eq!(loaded("chatDelay:true").chat_delay, 0.0);
    }

    #[test]
    fn a_unit_value_outside_its_range_leaves_the_option() {
        assert_eq!(loaded("mouseSensitivity:0.25").sensitivity, 0.25);
        assert_eq!(loaded("mouseSensitivity:1.5").sensitivity, 0.5);
        assert_eq!(loaded("mouseSensitivity:true").sensitivity, 1.0);
        assert_eq!(loaded("chatOpacity:false").chat_opacity, 0.0);
        assert_eq!(loaded("chatOpacity:nonsense").chat_opacity, 1.0);
        assert_eq!(loaded("gamma:0.25").gamma, 0.25);
        assert_eq!(loaded("gamma:1.5").gamma, 0.5);
        assert_eq!(loaded("screenEffectScale:0.25").screen_effect_scale, 0.25);
        assert_eq!(loaded("screenEffectScale:2.0").screen_effect_scale, 1.0);
    }

    #[test]
    fn the_gui_scale_and_the_visibility_read_as_integers() {
        assert_eq!(loaded("guiScale:3").gui_scale, 3);
        assert_eq!(loaded("guiScale:-1").gui_scale, 0);
        assert_eq!(loaded("guiScale:2.9").gui_scale, 2);
        assert_eq!(
            loaded("chatVisibility:1").chat_visibility,
            ChatVisibility::System
        );
        // The id wraps round.
        assert_eq!(
            loaded("chatVisibility:3").chat_visibility,
            ChatVisibility::Full
        );
        assert_eq!(
            loaded("chatVisibility:-1").chat_visibility,
            ChatVisibility::Hidden
        );
    }

    #[test]
    fn the_information_options_read_wrap_and_write_back() {
        let options = loaded("chatColors:false\nallowServerListing:false\nparticles:4\n");
        assert!(!options.chat_colors);
        assert!(!options.online.allow_server_listing);
        // The id wraps round, as `ByIdMap` does.
        assert_eq!(options.particles, ParticleStatus::Decreased);
        assert_eq!(loaded("particles:-1").particles, ParticleStatus::Minimal);
        assert!(loaded("chatColors:nonsense").chat_colors);
        let path = std::env::temp_dir().join(format!("photon-info-{}.txt", std::process::id()));
        let fresh = Options {
            chat_colors: false,
            online: OnlineOptions {
                allow_server_listing: false,
                ..OnlineOptions::default()
            },
            particles: ParticleStatus::Decreased,
            ..Options::default()
        };
        fresh.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let line = |key: &str| text.lines().position(|line| line.starts_with(key)).unwrap();
        assert!(text.contains("particles:1\n"));
        assert!(text.contains("chatColors:false\n"));
        assert!(text.contains("allowServerListing:false\n"));
        // The write order is `processOptions`'s.
        assert!(line("particles:") < line("chatColors:"));
        assert!(line("chatColors:") < line("invertXMouse:"));
        assert!(line("joinedFirstServer:") < line("allowServerListing:"));
        assert!(line("allowServerListing:") < line("saveChatDrafts:"));
    }

    #[test]
    fn the_wheel_takes_the_step_at_or_below_its_value() {
        assert_eq!(
            loaded("mouseWheelSensitivity:1.0").mouse_wheel_sensitivity,
            1.0
        );
        assert_eq!(
            loaded("mouseWheelSensitivity:10.0").mouse_wheel_sensitivity,
            10.0
        );
        assert_eq!(
            loaded("mouseWheelSensitivity:11.0").mouse_wheel_sensitivity,
            1.0
        );
        let stepped = loaded("mouseWheelSensitivity:0.5").mouse_wheel_sensitivity;
        assert!((stepped - 10.0_f64.powf(-0.31)).abs() < 1e-12);
    }

    #[test]
    fn booleans_and_the_language_read_as_they_are() {
        let options = loaded("invertYMouse:true\ninvertXMouse:1\nlang:tr_tr");
        assert!(options.invert_mouse_y);
        assert!(!options.invert_mouse_x);
        assert_eq!(options.language_code, "tr_tr");
    }

    #[test]
    fn the_focus_pause_is_on_and_the_advanced_tooltips_off_until_the_file_says_otherwise() {
        let options = loaded("");
        assert!(options.pause_on_lost_focus && !options.advanced_item_tooltips);
        let options = loaded("pauseOnLostFocus:false\nadvancedItemTooltips:true");
        assert!(!options.pause_on_lost_focus && options.advanced_item_tooltips);
        assert_eq!(options.write(Key::PauseOnLostFocus), "false");
    }

    #[test]
    fn the_block_outline_contrast_is_a_switch_that_is_off_by_default() {
        assert!(!loaded("").high_contrast_block_outline);
        let options = loaded("highContrastBlockOutline:true");
        assert!(options.high_contrast_block_outline);
        assert_eq!(options.write(Key::HighContrastBlockOutline), "true");
        assert!(!loaded("highContrastBlockOutline:1").high_contrast_block_outline);
    }

    #[test]
    fn the_accessibility_scales_read_with_their_codecs() {
        let options = loaded("");
        assert!(options.view_bobbing);
        assert_eq!(
            (options.fov_effect_scale, options.darkness_effect_scale),
            (1.0, 1.0)
        );
        assert_eq!((options.glint_speed, options.glint_strength), (0.5, 0.75));
        let options = loaded(
            "bobView:false\nfovEffectScale:0.25\ndarknessEffectScale:0.5\nglintSpeed:2.0\nglintStrength:0.1\ndamageTiltStrength:0.0",
        );
        assert!(!options.view_bobbing);
        assert_eq!(options.fov_effect_scale, 0.25);
        // The file holds the root of the darkness scale.
        assert_eq!(options.darkness_effect_scale, 0.25);
        assert_eq!(options.write(Key::DarknessEffectScale), "0.5");
        // A value its codec refuses leaves the default.
        assert_eq!(options.glint_speed, 0.5);
        assert_eq!(options.glint_strength, 0.1);
        assert_eq!(options.damage_tilt_strength, 0.0);
        assert_eq!(loaded("fovEffectScale:1.5").fov_effect_scale, 1.0);
        assert_eq!(
            loaded("darknessEffectScale:true").darkness_effect_scale,
            1.0
        );
        assert_eq!(
            loaded("darknessEffectScale:false").darkness_effect_scale,
            0.0
        );
        // The value set takes the value through its root and back.
        let root = 0.3_f64.sqrt();
        assert_eq!(loaded("fovEffectScale:0.3").fov_effect_scale, root * root);
        assert_eq!(
            loaded("darknessEffectScale:0.3").darkness_effect_scale,
            (0.3_f64 * 0.3).sqrt().powi(2)
        );
    }

    #[test]
    fn a_double_is_written_as_java_writes_it() {
        assert_eq!(java_double(0.0), "0.0");
        assert_eq!(java_double(1.0), "1.0");
        assert_eq!(java_double(0.4375), "0.4375");
        assert_eq!(java_double(-0.5), "-0.5");
        assert_eq!(java_double(1.0e-4), "1.0E-4");
        assert_eq!(java_double(1.5e-5), "1.5E-5");
        assert_eq!(java_double(12_345_678.0), "1.2345678E7");
        assert_eq!(java_double(0.001), "0.001");
    }

    #[test]
    fn what_is_saved_reads_back_and_what_is_not_known_stays() {
        let directory = std::env::temp_dir().join("photon-options-roundtrip");
        let path = directory.join("options.txt");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            &path,
            "version:1\nkey_key.jump:key.keyboard.space\nfov:0.5\nbad line\nsoundCategory_master:0.8\n",
        )
        .unwrap();
        let mut options = Options::load(&path);
        assert_eq!(options.fov, 90);
        options.fov = 100;
        options.chat_width = 0.25;
        options.chat_delay = 1.5;
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "version:4903");
        assert_eq!(lines[1], "key_key.jump:key.keyboard.space");
        assert_eq!(lines[2], "fov:0.75");
        assert_eq!(lines[3], "soundCategory_master:0.8");
        assert!(lines.contains(&"chatWidth:0.25"));
        assert!(lines.contains(&"chatDelay:1.5"));
        assert!(lines.contains(&"tutorialStep:movement"));
        assert!(lines.contains(&"guiScale:0"));
        let again = Options::load(&path);
        assert_eq!((again.fov, again.chat_width), (100, 0.25));
    }

    #[test]
    fn the_sound_options_read_and_write_as_the_file_holds_them() {
        let directory = std::env::temp_dir().join("photon-options-sound");
        let path = directory.join("options.txt");
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            &path,
            "soundCategory_music:0.25\nsoundCategory_ui:2.0\nshowSubtitles:true\nmusicFrequency:\"CONSTANT\"\nmusicToast:\"pause_and_toast\"\nsoundDevice:\"Speakers\"\n",
        )
        .unwrap();
        let options = Options::load(&path);
        assert_eq!(options.sound_category_volume(SoundSource::Music), 0.25);
        // Out of range: the option keeps its default.
        assert_eq!(options.sound_category_volume(SoundSource::Ui), 1.0);
        assert!(options.show_subtitles);
        assert_eq!(options.music_frequency, MusicFrequency::Constant);
        assert_eq!(options.music_toast, MusicToast::PauseAndToast);
        assert_eq!(options.sound_device, "Speakers");
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("musicFrequency:\"CONSTANT\"\n"));
        assert!(text.contains("soundCategory_music:0.25\n"));
        assert!(text.contains("soundDevice:\"Speakers\"\n"));
    }

    #[test]
    fn the_attack_indicator_reads_by_id_and_wraps() {
        let options = loaded("attackIndicator:2");
        assert_eq!(options.attack_indicator, AttackIndicator::Hotbar);
        assert_eq!(
            loaded("attackIndicator:5").attack_indicator,
            AttackIndicator::Hotbar
        );
        assert_eq!(
            loaded("attackIndicator:3").attack_indicator,
            AttackIndicator::Off
        );
        assert_eq!(
            loaded("attackIndicator:true").attack_indicator,
            AttackIndicator::Crosshair
        );
    }

    #[test]
    fn the_display_options_read_and_write_as_the_game_does() {
        let mut options = loaded("enableVsync:false\nmaxFps:30\ninactivityFpsLimit:\"minimized\"");
        assert!(!options.enable_vsync);
        assert_eq!(options.framerate_limit, 30);
        assert_eq!(options.inactivity_fps_limit, InactivityFpsLimit::Minimized);
        // A limit outside `intRange(10, 260)` and a value that is not a name leave the option.
        let refused = loaded("maxFps:5\ninactivityFpsLimit:\"never\"");
        assert_eq!(refused.framerate_limit, FRAMERATE_LIMIT_DEFAULT);
        assert_eq!(refused.inactivity_fps_limit, InactivityFpsLimit::Afk);
        assert_eq!(loaded("maxFps:260").framerate_limit, 260);
        options.framerate_limit = UNLIMITED_FRAMERATE_CUTOFF;
        let path = std::env::temp_dir().join(format!("photon-display-{}.txt", std::process::id()));
        options.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("enableVsync:false\n"));
        assert!(text.contains("maxFps:260\n"));
        assert!(text.contains("inactivityFpsLimit:\"minimized\"\n"));
    }

    #[test]
    fn ambient_occlusion_is_on_unless_the_file_turns_it_off() {
        assert!(loaded("").ambient_occlusion);
        let mut options = loaded("ao:false");
        assert!(!options.ambient_occlusion);
        options.ambient_occlusion = true;
        assert_eq!(options.write(Key::AmbientOcclusion), "true");
    }

    #[test]
    fn the_font_options_read_and_write_as_flags() {
        let mut options = loaded("forceUnicodeFont:true\njapaneseGlyphVariants:true\n");
        assert!(options.font.force_unicode);
        assert!(options.font.japanese_glyph_variants);
        options.font.force_unicode = false;
        assert_eq!(options.write(Key::ForceUnicodeFont), "false");
        assert_eq!(options.write(Key::JapaneseGlyphVariants), "true");
        let unset = loaded("");
        assert!(!unset.font.force_unicode && !unset.font.japanese_glyph_variants);
    }
}
