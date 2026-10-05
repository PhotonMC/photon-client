//! The game's sound definitions (`sounds.json`): which sound events there are, which files each
//! can play, and how one is picked by weight (`SoundManager`, `WeighedSoundEvents`, `Sound`).

use std::collections::HashMap;

use voxel_core::identifier::Identifier;
use voxel_core::json::{self, JsonObject, JsonValue};
use voxel_core::random::RandomSource;

/// A volume or a pitch (`SampledFloat`): a number the file gave, or the product of those an
/// event inherits from the one it points at.
#[derive(Debug, Clone, PartialEq)]
pub enum SampledFloat {
    Constant(f32),
    Multiplied(Vec<SampledFloat>),
}

impl SampledFloat {
    /// `SampledFloat.sample`: neither kind draws from the random source.
    pub fn sample(&self) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Multiplied(values) => {
                let mut result = 1.0_f32;
                for value in values {
                    result *= value.sample();
                }
                result
            }
        }
    }
}

/// One sound a event can play (`Sound`).
#[derive(Debug, Clone, PartialEq)]
pub struct Sound {
    /// The file's name under `sounds/` (without `.ogg`), or the event an entry points at.
    pub location: Identifier,
    pub volume: SampledFloat,
    pub pitch: SampledFloat,
    pub weight: i32,
    /// Streamed from the file as it plays rather than loaded whole.
    pub stream: bool,
    pub preload: bool,
    /// How far, in blocks, it can be heard (`attenuation_distance`).
    pub attenuation_distance: i32,
}

impl Sound {
    /// `SoundManager.EMPTY_SOUND`: what a pick that finds nothing gives.
    pub fn empty() -> Self {
        Self::file("minecraft:empty")
    }

    fn file(name: &str) -> Self {
        Self {
            location: Identifier::parse(name).expect("a fixed name"),
            volume: SampledFloat::Constant(1.0),
            pitch: SampledFloat::Constant(1.0),
            weight: 1,
            stream: false,
            preload: false,
            attenuation_distance: 16,
        }
    }

    /// The file (under the assets, `sounds/<name>.ogg`) a sound plays (`Sound.getPath`).
    pub fn path(&self) -> String {
        format!(
            "{}:sounds/{}.ogg",
            self.location.namespace(),
            self.location.path()
        )
    }
}

/// What an event holds: a file, or another event whose sounds it borrows with its own volume and pitch.
#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    File(Sound),
    Event(Sound),
}

/// A sound event (`WeighedSoundEvents`): its subtitle's translation key and what it can play.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SoundEvent {
    pub subtitle: Option<String>,
    pub entries: Vec<Entry>,
}

/// Why `sounds.json` is not accepted: the reference registers nothing from a file with anything wrong in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidSounds;

/// Every sound event of the loaded packs, by name (`SoundManager.registry`).
#[derive(Debug, Clone, Default)]
pub struct SoundRegistry {
    events: HashMap<String, SoundEvent>,
}

impl SoundRegistry {
    /// Register one pack's `sounds.json` (`SoundManager.prepare`): a file that does not parse, or
    /// holds an invalid entry, registers nothing. `exists` says whether a sound file
    /// (`namespace:sounds/name.ogg`) is among the assets; a file entry without its file is left out.
    pub fn register_file(
        &mut self,
        namespace: &str,
        text: &str,
        exists: &dyn Fn(&str) -> bool,
    ) -> Result<(), InvalidSounds> {
        let document = json::parse_object(text).ok_or(InvalidSounds)?;
        let mut parsed = Vec::new();
        for (key, value) in document.entries() {
            let name = String::from_utf16(key).map_err(|_| InvalidSounds)?;
            parsed.push((name, Registration::read(value)?));
        }
        for (name, registration) in parsed {
            self.register(&format!("{namespace}:{name}"), registration, exists);
        }
        Ok(())
    }

    /// `SoundManager.Preparations.handleRegistration`.
    fn register(&mut self, name: &str, registration: Registration, exists: &dyn Fn(&str) -> bool) {
        let event = self.events.entry(name.to_owned());
        let event = match event {
            std::collections::hash_map::Entry::Occupied(mut slot) => {
                if registration.replace {
                    slot.insert(SoundEvent {
                        subtitle: registration.subtitle.clone(),
                        entries: Vec::new(),
                    });
                }
                slot.into_mut()
            }
            std::collections::hash_map::Entry::Vacant(slot) => slot.insert(SoundEvent {
                subtitle: registration.subtitle.clone(),
                entries: Vec::new(),
            }),
        };
        for (sound, is_event) in registration.sounds {
            if is_event {
                event.entries.push(Entry::Event(sound));
            } else if exists(&sound.path()) {
                event.entries.push(Entry::File(sound));
            }
        }
    }

    /// The event of this name, if the packs define it (`SoundManager.getSoundEvent`).
    pub fn event(&self, name: &str) -> Option<&SoundEvent> {
        self.events.get(name)
    }

    /// Every event's name.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.events.keys().map(String::as_str)
    }

    /// `WeighedSoundEvents.getWeight`: the weights of what an event can play, added.
    pub fn weight(&self, event: &SoundEvent) -> i32 {
        event
            .entries
            .iter()
            .map(|entry| self.entry_weight(entry))
            .fold(0, i32::wrapping_add)
    }

    /// An entry's weight: a file's own, an event entry's what the event it points at has now.
    pub fn entry_weight(&self, entry: &Entry) -> i32 {
        match entry {
            Entry::File(sound) => sound.weight,
            Entry::Event(sound) => self
                .events
                .get(&sound.location.as_string())
                .map_or(0, |target| self.weight(target)),
        }
    }

    /// `WeighedSoundEvents.getSound`: one of its sounds, by weight.
    pub fn pick(&self, event: &SoundEvent, random: &mut RandomSource) -> Sound {
        let weight = self.weight(event);
        if event.entries.is_empty() || weight == 0 {
            return Sound::empty();
        }
        let Ok(mut index) = random.next_int_bound(weight) else {
            return Sound::empty();
        };
        for entry in &event.entries {
            index = index.wrapping_sub(self.entry_weight(entry));
            if index < 0 {
                return self.resolve(entry, random);
            }
        }
        Sound::empty()
    }

    /// `Weighted.getSound` of one entry: a file is itself; an event entry is what the event it
    /// points at picks, with the volumes and pitches multiplied.
    fn resolve(&self, entry: &Entry, random: &mut RandomSource) -> Sound {
        match entry {
            Entry::File(sound) => sound.clone(),
            Entry::Event(sound) => {
                let Some(target) = self.events.get(&sound.location.as_string()) else {
                    return Sound::empty();
                };
                let wrapped = self.pick(target, random);
                Sound {
                    location: wrapped.location.clone(),
                    volume: SampledFloat::Multiplied(vec![wrapped.volume, sound.volume.clone()]),
                    pitch: SampledFloat::Multiplied(vec![wrapped.pitch, sound.pitch.clone()]),
                    weight: sound.weight,
                    stream: wrapped.stream || sound.stream,
                    preload: wrapped.preload,
                    attenuation_distance: wrapped.attenuation_distance,
                }
            }
        }
    }
}

/// One event as the file gives it (`SoundEventRegistration`).
struct Registration {
    sounds: Vec<(Sound, bool)>,
    replace: bool,
    subtitle: Option<String>,
}

impl Registration {
    /// `SoundEventRegistrationSerializer.deserialize`.
    fn read(value: &JsonValue) -> Result<Self, InvalidSounds> {
        let JsonValue::Object(object) = value else {
            return Err(InvalidSounds);
        };
        let replace = boolean(object, "replace", false)?;
        let subtitle = match object.get_str("subtitle") {
            None => None,
            Some(value) => Some(primitive_text(value)?),
        };
        let mut sounds = Vec::new();
        if let Some(listed) = object.get_str("sounds") {
            let JsonValue::Array(items) = listed else {
                return Err(InvalidSounds);
            };
            for item in items {
                sounds.push(match item {
                    JsonValue::String(units) => {
                        let name = String::from_utf16(units).map_err(|_| InvalidSounds)?;
                        let name = Identifier::parse(&name).map_err(|_| InvalidSounds)?;
                        (Sound::file(&name.as_string()), false)
                    }
                    JsonValue::Object(sound) => read_sound(sound)?,
                    _ => return Err(InvalidSounds),
                });
            }
        }
        Ok(Self {
            sounds,
            replace,
            subtitle,
        })
    }
}

/// `SoundEventRegistrationSerializer.getSound`: the sound, and whether it names an event.
fn read_sound(object: &JsonObject) -> Result<(Sound, bool), InvalidSounds> {
    let name = primitive_text(object.get_str("name").ok_or(InvalidSounds)?)?;
    let location = Identifier::parse(&name).map_err(|_| InvalidSounds)?;
    let is_event = match object.get_str("type") {
        None => false,
        Some(value) => match primitive_text(value)?.as_str() {
            "file" => false,
            "event" => true,
            _ => return Err(InvalidSounds),
        },
    };
    let volume = float(object, "volume", 1.0)?;
    if volume.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Err(InvalidSounds);
    }
    let pitch = float(object, "pitch", 1.0)?;
    if pitch.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
        return Err(InvalidSounds);
    }
    let weight = int(object, "weight", 1)?;
    if weight <= 0 {
        return Err(InvalidSounds);
    }
    let preload = boolean(object, "preload", false)?;
    let stream = boolean(object, "stream", false)?;
    let attenuation_distance = int(object, "attenuation_distance", 16)?;
    Ok((
        Sound {
            location,
            volume: SampledFloat::Constant(volume),
            pitch: SampledFloat::Constant(pitch),
            weight,
            stream,
            preload,
            attenuation_distance,
        },
        is_event,
    ))
}

/// `GsonHelper.convertToString`: the text of a string, a number (as written) or a boolean.
fn primitive_text(value: &JsonValue) -> Result<String, InvalidSounds> {
    match value {
        JsonValue::String(units) => String::from_utf16(units).map_err(|_| InvalidSounds),
        JsonValue::Number(literal) => Ok(literal.clone()),
        JsonValue::Bool(value) => Ok(value.to_string()),
        _ => Err(InvalidSounds),
    }
}

/// `GsonHelper.getAsBoolean`: the default when absent; of a primitive, a boolean as it is and
/// anything else whether its text is `true` (`Boolean.parseBoolean`); an error for the rest.
fn boolean(object: &JsonObject, key: &str, default: bool) -> Result<bool, InvalidSounds> {
    match object.get_str(key) {
        None => Ok(default),
        Some(JsonValue::Bool(value)) => Ok(*value),
        Some(value) => Ok(primitive_text(value)?.eq_ignore_ascii_case("true")),
    }
}

/// `GsonHelper.getAsFloat`.
fn float(object: &JsonObject, key: &str, default: f32) -> Result<f32, InvalidSounds> {
    match object.get_str(key) {
        None => Ok(default),
        Some(value @ JsonValue::Number(_)) => value.as_f32().ok_or(InvalidSounds),
        Some(_) => Err(InvalidSounds),
    }
}

/// `GsonHelper.getAsInt`.
fn int(object: &JsonObject, key: &str, default: i32) -> Result<i32, InvalidSounds> {
    match object.get_str(key) {
        None => Ok(default),
        Some(value @ JsonValue::Number(_)) => value.as_i32().ok_or(InvalidSounds),
        Some(_) => Err(InvalidSounds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(text: &str, files: &[&str]) -> SoundRegistry {
        let mut registry = SoundRegistry::default();
        let files: Vec<String> = files
            .iter()
            .map(|file| format!("minecraft:sounds/{file}.ogg"))
            .collect();
        registry
            .register_file("minecraft", text, &|path| {
                files.iter().any(|file| file == path)
            })
            .expect("valid");
        registry
    }

    #[test]
    fn a_pick_follows_the_weights() {
        let registry = registry(
            r#"{"event": {"sounds": [{"name": "a", "weight": 3}, "b"]}}"#,
            &["a", "b"],
        );
        let event = registry.event("minecraft:event").expect("registered");
        assert_eq!(registry.weight(event), 4);
        let mut random = RandomSource::create(1);
        let picked: Vec<String> = (0..20)
            .map(|_| registry.pick(event, &mut random).location.as_string())
            .collect();
        assert!(picked.iter().any(|name| name == "minecraft:a"));
        assert!(picked.iter().any(|name| name == "minecraft:b"));
    }

    #[test]
    fn an_event_entry_borrows_another_events_sounds_and_multiplies_its_volume() {
        let registry = registry(
            r#"{"base": {"sounds": [{"name": "x", "volume": 0.5}]},
                "wrap": {"sounds": [{"name": "base", "type": "event", "volume": 0.5}]}}"#,
            &["x"],
        );
        let event = registry.event("minecraft:wrap").expect("registered");
        let sound = registry.pick(event, &mut RandomSource::create(0));
        assert_eq!(sound.location.as_string(), "minecraft:x");
        assert_eq!(sound.volume.sample(), 0.25);
    }

    #[test]
    fn a_file_with_anything_wrong_registers_nothing_and_a_missing_file_is_left_out() {
        let mut bad = SoundRegistry::default();
        assert_eq!(
            bad.register_file(
                "minecraft",
                r#"{"good": {"sounds": ["a"]}, "bad": {"sounds": [{"name": "a", "volume": 0}]}}"#,
                &|_| true,
            ),
            Err(InvalidSounds)
        );
        assert_eq!(bad.names().count(), 0);
        let registry = registry(r#"{"event": {"sounds": ["gone", "here"]}}"#, &["here"]);
        assert_eq!(
            registry
                .event("minecraft:event")
                .expect("registered")
                .entries
                .len(),
            1
        );
    }
}
