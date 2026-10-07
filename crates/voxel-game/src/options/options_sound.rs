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

    /// `option.codec().parse` and `set`: a value its codec refuses leaves
    /// the option as it was; one its value set refuses falls to the default.
    pub(super) fn read(&mut self, key: Key, text: &str) {
        let json = parse_json(text);
        let unit = || match json {
            Json::Number(number) => UnitDouble::validate_value(double_value(number)),
            Json::Bool(value) => Some(if value { 1.0 } else { 0.0 }),
            Json::Text(_) | Json::Other => None,
        };
        let boolean = || match json {
            Json::Bool(value) => Some(value),
            _ => None,
        };
        match key {
            Key::Fov => {
                if let Json::Number(number) = json {
                    let degrees = (double_value(number) * 40.0 + 70.0) as i32;
                    self.fov = FOV.validate_value(degrees).unwrap_or(FOV_DEFAULT);
                }
            }
            Key::GuiScale => {
                if let Json::Number(number) = json {
                    let scale = int_value(number);
                    if (0..=GUI_SCALE_MAX + 1).contains(&scale) {
                        self.gui_scale =
                            ClampingLazyMaxIntRange::new(0, || GUI_SCALE_MAX, GUI_SCALE_MAX)
                                .validate_value(scale);
                    }
                }
            }
            Key::EnableVsync => self.enable_vsync = boolean().unwrap_or(self.enable_vsync),
            Key::MaxFps => {
                // `Codec.intRange(10, 260)`.
                if let Json::Number(number) = json {
                    let limit = int_value(number);
                    if (10..=UNLIMITED_FRAMERATE_CUTOFF).contains(&limit) {
                        self.framerate_limit = limit;
                    }
                }
            }
            Key::InactivityFpsLimit => {
                if let Json::Text(name) = &json
                    && let Some(value) = InactivityFpsLimit::from_serialized(name)
                {
                    self.inactivity_fps_limit = value;
                }
            }
            Key::InvertXMouse => self.invert_mouse_x = boolean().unwrap_or(self.invert_mouse_x),
            Key::SaveChatDrafts => {
                self.save_chat_drafts = boolean().unwrap_or(self.save_chat_drafts);
            }
            Key::InvertYMouse => self.invert_mouse_y = boolean().unwrap_or(self.invert_mouse_y),
            Key::AttackIndicator => {
                if let Json::Number(number) = json {
                    // `ByIdMap` wraps an id out of range.
                    let id = int_value(number).rem_euclid(3);
                    self.attack_indicator = AttackIndicator::ALL
                        .into_iter()
                        .find(|value| value.id() == id)
                        .unwrap_or(self.attack_indicator);
                }
            }
            Key::ShowSubtitles
            | Key::HideLightningFlashes
            | Key::DirectionalAudio
            | Key::SoundDevice
            | Key::MusicFrequency
            | Key::MusicToast
            | Key::SoundCategory(_) => self.read_sound(key, &json),
            Key::MouseSensitivity => self.sensitivity = unit().unwrap_or(self.sensitivity),
            Key::Lang => text.clone_into(&mut self.language_code),
            Key::ChatVisibility => {
                if let Json::Number(number) = json {
                    // `ByIdMap` wraps an id out of range.
                    let id = int_value(number).rem_euclid(3);
                    self.chat_visibility =
                        ChatVisibility::from_id(id).unwrap_or(self.chat_visibility);
                }
            }
            Key::ChatOpacity => self.chat_opacity = unit().unwrap_or(self.chat_opacity),
            Key::ChatLineSpacing => {
                self.chat_line_spacing = unit().unwrap_or(self.chat_line_spacing)
            }
            Key::TextBackgroundOpacity => {
                self.text_background_opacity = unit().unwrap_or(self.text_background_opacity);
            }
            Key::ChatHeightFocused => {
                self.chat_height_focused = unit().unwrap_or(self.chat_height_focused);
            }
            Key::ChatHeightUnfocused => {
                self.chat_height_unfocused = unit().unwrap_or(self.chat_height_unfocused);
            }
            Key::ChatScale => self.chat_scale = unit().unwrap_or(self.chat_scale),
            Key::ChatWidth => self.chat_width = unit().unwrap_or(self.chat_width),
            Key::MouseWheelSensitivity => {
                if let Json::Number(number) = json {
                    let value = double_value(number);
                    if (log_mouse(-200)..=log_mouse(100)).contains(&value) {
                        self.mouse_wheel_sensitivity =
                            mouse_wheel_step(value).unwrap_or(log_mouse(0));
                    }
                }
            }
        }
    }

    /// The value the file holds for an option.
    pub(super) fn write(&self, key: Key) -> String {
        match key {
            Key::EnableVsync => self.enable_vsync.to_string(),
            Key::MaxFps => self.framerate_limit.to_string(),
            Key::InactivityFpsLimit => write_string(self.inactivity_fps_limit.serialized()),
            Key::Fov => java_double((f64::from(self.fov) - 70.0) / 40.0),
            Key::GuiScale => self.gui_scale.to_string(),
            Key::InvertXMouse => self.invert_mouse_x.to_string(),
            Key::SaveChatDrafts => self.save_chat_drafts.to_string(),
            Key::InvertYMouse => self.invert_mouse_y.to_string(),
            Key::MouseSensitivity => java_double(self.sensitivity),
            Key::Lang => self.language_code.clone(),
            Key::ChatVisibility => self.chat_visibility.id().to_string(),
            Key::ChatOpacity => java_double(self.chat_opacity),
            Key::ChatLineSpacing => java_double(self.chat_line_spacing),
            Key::TextBackgroundOpacity => java_double(self.text_background_opacity),
            Key::ChatHeightFocused => java_double(self.chat_height_focused),
            Key::ChatHeightUnfocused => java_double(self.chat_height_unfocused),
            Key::ChatScale => java_double(self.chat_scale),
            Key::ChatWidth => java_double(self.chat_width),
            Key::MouseWheelSensitivity => java_double(self.mouse_wheel_sensitivity),
            Key::AttackIndicator => self.attack_indicator.id().to_string(),
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

    /// `Options.getSoundSourceVolume`: what the player set a sound category to.
    pub fn sound_category_volume(&self, source: SoundSource) -> f64 {
        self.sound_volumes[source.ordinal() as usize]
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
            let value = known.map_or_else(|| value.clone(), |(_, key)| self.write(*key));
            text.push_str(&format!("{name}:{value}\n"));
            written.push(name.as_str());
        }
        for (name, key) in KEYS {
            if !written.contains(&name) {
                text.push_str(&format!("{name}:{}\n", self.write(key)));
            }
        }
        if let Some(directory) = path.parent() {
            std::fs::create_dir_all(directory)?;
        }
        std::fs::write(path, text)
    }
}
