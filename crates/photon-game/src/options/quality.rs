//! The quality options that the video screen holds and `options.txt` keeps: the graphics preset
//! and the options a preset controls beyond the render distance, smooth lighting, particles and
//! menu blur. They are stored and shown; nothing draws from them yet.

use super::json::{Json, double_value, int_value, java_double, write_string};

/// `GraphicsPreset`: what the preset slider says; Custom once a controlled option was changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsPreset {
    Fast,
    Fancy,
    Fabulous,
    Custom,
}

impl GraphicsPreset {
    pub const ALL: [Self; 4] = [Self::Fast, Self::Fancy, Self::Fabulous, Self::Custom];

    /// `getSerializedName`, which the file holds.
    pub const fn serialized(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Fancy => "fancy",
            Self::Fabulous => "fabulous",
            Self::Custom => "custom",
        }
    }

    /// The caption's translation key, which is also the slider's value.
    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::Fast => "options.graphics.fast",
            Self::Fancy => "options.graphics.fancy",
            Self::Fabulous => "options.graphics.fabulous",
            Self::Custom => "options.graphics.custom",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// `PrioritizeChunkUpdates`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrioritizeChunkUpdates {
    None,
    PlayerAffected,
    Nearby,
}

impl PrioritizeChunkUpdates {
    pub const ALL: [Self; 3] = [Self::None, Self::PlayerAffected, Self::Nearby];

    const fn id(self) -> i32 {
        self as i32
    }

    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::None => "options.prioritizeChunkUpdates.none",
            Self::PlayerAffected => "options.prioritizeChunkUpdates.byPlayer",
            Self::Nearby => "options.prioritizeChunkUpdates.nearby",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// `CloudStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudStatus {
    Off,
    Fast,
    Fancy,
}

impl CloudStatus {
    pub const ALL: [Self; 3] = [Self::Off, Self::Fast, Self::Fancy];

    /// `getSerializedName`: the older file's words for off and fancy.
    const fn serialized(self) -> &'static str {
        match self {
            Self::Off => "false",
            Self::Fast => "fast",
            Self::Fancy => "true",
        }
    }

    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::Off => "options.off",
            Self::Fast => "options.clouds.fast",
            Self::Fancy => "options.clouds.fancy",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// `TextureFilteringMethod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFiltering {
    None,
    Rgss,
    Anisotropic,
}

impl TextureFiltering {
    pub const ALL: [Self; 3] = [Self::None, Self::Rgss, Self::Anisotropic];

    const fn id(self) -> i32 {
        self as i32
    }

    pub const fn caption_key(self) -> &'static str {
        match self {
            Self::None => "options.textureFiltering.none",
            Self::Rgss => "options.textureFiltering.rgss",
            Self::Anisotropic => "options.textureFiltering.anisotropic",
        }
    }

    pub fn from_caption_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|value| value.caption_key() == key)
    }
}

/// The options of this module, by the file's key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum QualityKey {
    GraphicsPreset,
    BiomeBlendRadius,
    PrioritizeChunkUpdates,
    SimulationDistance,
    CloudStatus,
    MipmapLevels,
    EntityShadows,
    EntityDistanceScaling,
    CloudRange,
    CutoutLeaves,
    ImprovedTransparency,
    WeatherRadius,
    MaxAnisotropyBit,
    TextureFiltering,
}

/// The options, with the defaults the reference's `OptionInstance`s have.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QualityOptions {
    pub graphics_preset: GraphicsPreset,
    /// `biomeBlendRadius`, 0 to 7.
    pub biome_blend_radius: i32,
    pub prioritize_chunk_updates: PrioritizeChunkUpdates,
    /// `simulationDistance`, in chunks, 5 to 32.
    pub simulation_distance: i32,
    /// `renderClouds`.
    pub cloud_status: CloudStatus,
    /// `mipmapLevels`, 0 to 4.
    pub mipmap_levels: i32,
    pub entity_shadows: bool,
    /// `entityDistanceScaling`, a half to five in quarters.
    pub entity_distance_scaling: f64,
    /// `cloudRange`, in chunks, 2 to 128.
    pub cloud_range: i32,
    pub cutout_leaves: bool,
    pub improved_transparency: bool,
    /// `weatherRadius`, in blocks, 3 to 10.
    pub weather_radius: i32,
    /// `maxAnisotropyBit`, 1 to 3.
    pub max_anisotropy_bit: i32,
    pub texture_filtering: TextureFiltering,
}

impl Default for QualityOptions {
    fn default() -> Self {
        Self {
            graphics_preset: GraphicsPreset::Fancy,
            biome_blend_radius: 2,
            prioritize_chunk_updates: PrioritizeChunkUpdates::None,
            simulation_distance: 12,
            cloud_status: CloudStatus::Fancy,
            mipmap_levels: 4,
            entity_shadows: true,
            entity_distance_scaling: 1.0,
            cloud_range: 128,
            cutout_leaves: true,
            improved_transparency: false,
            weather_radius: 10,
            max_anisotropy_bit: 2,
            texture_filtering: TextureFiltering::None,
        }
    }
}

impl QualityOptions {
    /// The integer options and their `Codec.intRange`. The codec reaches one past the top, which
    /// the set then turns to the default: either way a value out of range leaves the default.
    fn int_slot(&mut self, key: QualityKey) -> Option<(&mut i32, std::ops::RangeInclusive<i32>)> {
        Some(match key {
            QualityKey::BiomeBlendRadius => (&mut self.biome_blend_radius, 0..=7),
            QualityKey::SimulationDistance => (&mut self.simulation_distance, 5..=32),
            QualityKey::MipmapLevels => (&mut self.mipmap_levels, 0..=4),
            QualityKey::CloudRange => (&mut self.cloud_range, 2..=128),
            QualityKey::WeatherRadius => (&mut self.weather_radius, 3..=10),
            QualityKey::MaxAnisotropyBit => (&mut self.max_anisotropy_bit, 1..=3),
            _ => return None,
        })
    }

    fn flag_slot(&mut self, key: QualityKey) -> Option<&mut bool> {
        Some(match key {
            QualityKey::EntityShadows => &mut self.entity_shadows,
            QualityKey::CutoutLeaves => &mut self.cutout_leaves,
            QualityKey::ImprovedTransparency => &mut self.improved_transparency,
            _ => return None,
        })
    }

    /// The value of a line of the file; one the option's codec does not take leaves the option.
    pub(super) fn read(&mut self, key: QualityKey, json: &Json<'_>) {
        if let Some((slot, range)) = self.int_slot(key) {
            if let Json::Number(number) = json {
                let value = int_value(number);
                *slot = if range.contains(&value) { value } else { *slot };
            }
        } else if let Some(slot) = self.flag_slot(key) {
            *slot = json.as_bool().unwrap_or(*slot);
        } else {
            self.read_choice(key, json);
        }
    }

    /// The options that are one of a few, by name or by id, and the entity distance.
    fn read_choice(&mut self, key: QualityKey, json: &Json<'_>) {
        match (key, json) {
            (QualityKey::GraphicsPreset | QualityKey::CloudStatus, _) => self.read_named(key, json),
            (QualityKey::PrioritizeChunkUpdates, Json::Number(number)) => {
                // `ByIdMap` wraps an id out of range.
                let id = int_value(number).rem_euclid(3) as usize;
                self.prioritize_chunk_updates = PrioritizeChunkUpdates::ALL[id];
            }
            (QualityKey::TextureFiltering, Json::Number(number)) => {
                let id = int_value(number).rem_euclid(3) as usize;
                self.texture_filtering = TextureFiltering::ALL[id];
            }
            (QualityKey::EntityDistanceScaling, Json::Number(number)) => {
                // `Codec.doubleRange(0.5, 5.0)`, then the set keeps the quarters of the slider.
                let value = double_value(number);
                if (0.5..=5.0).contains(&value) {
                    self.entity_distance_scaling = f64::from((value * 4.0) as i32) / 4.0;
                }
            }
            _ => {}
        }
    }

    /// The preset and the clouds, which the file holds by name; the clouds also take the older
    /// switch (`withAlternative(.., Codec.BOOL, ..)`).
    fn read_named(&mut self, key: QualityKey, json: &Json<'_>) {
        match (key, json) {
            (QualityKey::GraphicsPreset, Json::Text(name)) => {
                let found = GraphicsPreset::ALL
                    .into_iter()
                    .find(|value| value.serialized() == name);
                self.graphics_preset = found.unwrap_or(self.graphics_preset);
            }
            (QualityKey::CloudStatus, Json::Text(name)) => {
                let found = CloudStatus::ALL
                    .into_iter()
                    .find(|value| value.serialized() == name);
                self.cloud_status = found.unwrap_or(self.cloud_status);
            }
            (QualityKey::CloudStatus, Json::Bool(on)) => {
                self.cloud_status = if *on {
                    CloudStatus::Fancy
                } else {
                    CloudStatus::Off
                };
            }
            _ => {}
        }
    }

    /// The value the file holds for an option.
    pub(super) fn write(&self, key: QualityKey) -> String {
        let mut own = *self;
        if let Some((slot, _)) = own.int_slot(key) {
            return slot.to_string();
        }
        if let Some(slot) = own.flag_slot(key) {
            return slot.to_string();
        }
        match key {
            QualityKey::GraphicsPreset => write_string(self.graphics_preset.serialized()),
            QualityKey::CloudStatus => write_string(self.cloud_status.serialized()),
            QualityKey::PrioritizeChunkUpdates => self.prioritize_chunk_updates.id().to_string(),
            QualityKey::TextureFiltering => self.texture_filtering.id().to_string(),
            _ => java_double(self.entity_distance_scaling),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::json::parse_json;
    use super::*;

    fn read(key: QualityKey, text: &str) -> QualityOptions {
        let mut options = QualityOptions::default();
        options.read(key, &parse_json(text));
        options
    }

    #[test]
    fn the_defaults_are_the_option_instances_not_a_preset() {
        let options = QualityOptions::default();
        assert_eq!(options.graphics_preset, GraphicsPreset::Fancy);
        assert_eq!(options.simulation_distance, 12);
        assert_eq!(options.cloud_range, 128);
        assert_eq!(options.max_anisotropy_bit, 2);
        assert_eq!(options.texture_filtering, TextureFiltering::None);
        assert_eq!(
            options.prioritize_chunk_updates,
            PrioritizeChunkUpdates::None
        );
    }

    #[test]
    fn the_names_and_ids_read_and_write_as_the_codecs_have_them() {
        let preset = read(QualityKey::GraphicsPreset, "\"fast\"");
        assert_eq!(preset.graphics_preset, GraphicsPreset::Fast);
        assert_eq!(preset.write(QualityKey::GraphicsPreset), "\"fast\"");
        // A preset is a name, never an id.
        let id = read(QualityKey::GraphicsPreset, "0");
        assert_eq!(id.graphics_preset, GraphicsPreset::Fancy);
        let filtering = read(QualityKey::TextureFiltering, "5");
        assert_eq!(filtering.texture_filtering, TextureFiltering::Anisotropic);
        assert_eq!(filtering.write(QualityKey::TextureFiltering), "2");
        let priority = read(QualityKey::PrioritizeChunkUpdates, "-1");
        assert_eq!(
            priority.prioritize_chunk_updates,
            PrioritizeChunkUpdates::Nearby
        );
    }

    #[test]
    fn a_name_without_quotes_reads_as_the_lenient_parser_has_it() {
        let preset = read(QualityKey::GraphicsPreset, "fabulous");
        assert_eq!(preset.graphics_preset, GraphicsPreset::Fabulous);
        // Not a word: it has a separator in it.
        let split = read(QualityKey::GraphicsPreset, "fast,slow");
        assert_eq!(split.graphics_preset, GraphicsPreset::Fancy);
        assert_eq!(
            read(QualityKey::CloudStatus, "fast").cloud_status,
            CloudStatus::Fast
        );
    }

    #[test]
    fn the_clouds_take_their_names_and_the_older_switch() {
        assert_eq!(
            read(QualityKey::CloudStatus, "\"fast\"").cloud_status,
            CloudStatus::Fast
        );
        assert_eq!(
            read(QualityKey::CloudStatus, "\"false\"").cloud_status,
            CloudStatus::Off
        );
        assert_eq!(
            read(QualityKey::CloudStatus, "false").cloud_status,
            CloudStatus::Off
        );
        assert_eq!(
            read(QualityKey::CloudStatus, "true").cloud_status,
            CloudStatus::Fancy
        );
        let off = read(QualityKey::CloudStatus, "false");
        assert_eq!(off.write(QualityKey::CloudStatus), "\"false\"");
        // The older words are the names; "fancy" is not one, so the option stays as it was.
        let mut fast = QualityOptions {
            cloud_status: CloudStatus::Fast,
            ..QualityOptions::default()
        };
        fast.read(QualityKey::CloudStatus, &parse_json("\"fancy\""));
        assert_eq!(fast.cloud_status, CloudStatus::Fast);
    }

    #[test]
    fn a_number_out_of_its_range_leaves_the_default() {
        assert_eq!(
            read(QualityKey::SimulationDistance, "4").simulation_distance,
            12
        );
        assert_eq!(
            read(QualityKey::SimulationDistance, "20").simulation_distance,
            20
        );
        assert_eq!(
            read(QualityKey::SimulationDistance, "33").simulation_distance,
            12
        );
        assert_eq!(
            read(QualityKey::BiomeBlendRadius, "0").biome_blend_radius,
            0
        );
        assert_eq!(
            read(QualityKey::BiomeBlendRadius, "8").biome_blend_radius,
            2
        );
        assert_eq!(
            read(QualityKey::MaxAnisotropyBit, "0").max_anisotropy_bit,
            2
        );
        assert_eq!(read(QualityKey::WeatherRadius, "2").weather_radius, 10);
        assert_eq!(read(QualityKey::CloudRange, "\"x\"").cloud_range, 128);
        assert_eq!(
            read(QualityKey::MipmapLevels, "3").write(QualityKey::MipmapLevels),
            "3"
        );
    }

    #[test]
    fn the_entity_distance_keeps_the_quarters_of_its_slider() {
        let scaled = read(QualityKey::EntityDistanceScaling, "0.8");
        assert_eq!(scaled.entity_distance_scaling, 0.75);
        assert_eq!(scaled.write(QualityKey::EntityDistanceScaling), "0.75");
        let low = read(QualityKey::EntityDistanceScaling, "0.4");
        assert_eq!(low.entity_distance_scaling, 1.0);
        let high = read(QualityKey::EntityDistanceScaling, "5.0");
        assert_eq!(high.entity_distance_scaling, 5.0);
    }

    #[test]
    fn the_switches_read_as_booleans() {
        assert!(!read(QualityKey::EntityShadows, "false").entity_shadows);
        assert!(read(QualityKey::ImprovedTransparency, "true").improved_transparency);
        // `JsonOps.getBooleanValue` takes a boolean only, as `getNumberValue` takes a number only.
        assert!(!read(QualityKey::ImprovedTransparency, "1").improved_transparency);
        let off = read(QualityKey::CutoutLeaves, "false");
        assert_eq!(off.write(QualityKey::CutoutLeaves), "false");
    }

    #[test]
    fn the_file_keeps_the_preset_and_what_it_controls_across_a_save() {
        let directory = std::env::temp_dir().join(format!("photon-quality-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("options.txt");
        std::fs::write(
            &path,
            "graphicsPreset:\"fast\"\nsimulationDistance:20\nrenderClouds:false\ntextureFiltering:2\n",
        )
        .unwrap();
        let options = super::super::Options::load(&path);
        assert_eq!(options.quality.graphics_preset, GraphicsPreset::Fast);
        assert_eq!(options.quality.simulation_distance, 20);
        assert_eq!(options.quality.cloud_status, CloudStatus::Off);
        options.save(&path).unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("graphicsPreset:\"fast\"\n"));
        assert!(saved.contains("renderClouds:\"false\"\n"));
        assert!(saved.contains("weatherRadius:10\n"));
        assert_eq!(super::super::Options::load(&path).quality, options.quality);
    }
}
