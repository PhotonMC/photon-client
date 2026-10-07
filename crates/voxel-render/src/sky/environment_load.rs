use super::*;

impl Environment {
    /// Read a dimension's sky data from a data pack folder (the one holding
    /// `minecraft/`), with the biomes numbered as the server numbered them.
    #[expect(
        clippy::too_many_lines,
        reason = "one read per data pack file the sky needs"
    )]
    pub fn load(data_root: &Path, dimension_type: &str, biome_names: &[String]) -> Option<Self> {
        let dimension = voxel_core::json::read_object(&data_path(
            data_root,
            "dimension_type",
            dimension_type,
            ".json",
        ))?;
        let skybox = match dimension.text("skybox").as_deref() {
            Some("end") => Skybox::End,
            Some("none") => Skybox::None,
            _ => Skybox::Overworld,
        };
        let dimension_map = parse_attribute_map(dimension.object("attributes"));
        let weather = dimension.bool("has_skylight").unwrap_or(false)
            && !dimension.bool("has_ceiling").unwrap_or(false)
            && dimension_type != "minecraft:the_end";
        let biome_files: Vec<Option<JsonObject>> = biome_names
            .iter()
            .map(|name| {
                voxel_core::json::read_object(&data_path(
                    data_root,
                    "worldgen/biome",
                    name,
                    ".json",
                ))
            })
            .collect();
        let precipitation = biome_files
            .iter()
            .map(|biome| {
                biome
                    .as_ref()
                    .and_then(|biome| biome.bool("has_precipitation"))
                    .unwrap_or(false)
            })
            .collect();
        let biomes: Vec<AttributeMap> = biome_files
            .iter()
            .map(|biome| {
                parse_attribute_map(biome.as_ref().and_then(|biome| biome.object("attributes")))
            })
            .collect();
        let plains = biome_names
            .iter()
            .position(|name| name == "minecraft:plains")
            .unwrap_or(0);
        let mut biome_attributes = Vec::new();
        for map in &biomes {
            for attribute in map.keys() {
                if !biome_attributes.contains(attribute) {
                    biome_attributes.push(*attribute);
                }
            }
        }
        let mut timeline_names = Vec::new();
        match dimension.get_str("timelines") {
            Some(JsonValue::String(text)) => {
                let text = String::from_utf16(text).ok()?;
                match text.strip_prefix('#') {
                    Some(tag) => expand_tag(data_root, "timeline", tag, &mut timeline_names),
                    None => timeline_names.push(text),
                }
            }
            Some(JsonValue::Array(list)) => {
                for value in list {
                    if let JsonValue::String(text) = value {
                        timeline_names.push(String::from_utf16(text).ok()?);
                    }
                }
            }
            _ => {}
        }
        let mut timelines = Vec::new();
        for name in timeline_names {
            let Some(timeline) =
                voxel_core::json::read_object(&data_path(data_root, "timeline", &name, ".json"))
            else {
                continue;
            };
            let clock = timeline.text("clock").unwrap_or_default();
            let period = timeline.i32("period_ticks").map(i64::from);
            let mut tracks = Vec::new();
            if let Some(object) = timeline.object("tracks") {
                for attribute in ATTRIBUTES {
                    if let Some(JsonValue::Object(track)) = object.get_str(attribute.name())
                        && let Some(track) = Track::parse(attribute, period, track)
                    {
                        tracks.push(track);
                    }
                }
            }
            timelines.push(Timeline { clock, tracks });
        }
        Some(Self {
            skybox,
            dimension: dimension_map,
            biomes,
            plains,
            biome_attributes,
            timelines,
            weather,
            precipitation,
        })
    }

    /// Whether the biome with this network number has precipitation; an
    /// unknown number reads as plains.
    pub fn has_precipitation(&self, biome: Option<i32>) -> bool {
        let biome = biome
            .and_then(|id| usize::try_from(id).ok())
            .filter(|id| *id < self.precipitation.len())
            .unwrap_or(self.plains);
        self.precipitation.get(biome).copied().unwrap_or(false)
    }

    pub fn skybox(&self) -> Skybox {
        self.skybox
    }

    /// Every attribute at a camera position and the clocks' current ticks.
    ///
    /// `biome_at` answers the network biome number at a quart position, or
    /// nothing where no chunk is loaded, which the client reads as plains.
    #[expect(
        clippy::too_many_lines,
        reason = "kept whole to read against the reference method line by line"
    )]
    pub fn evaluate(
        &self,
        eye: [f64; 3],
        biome_at: impl Fn(i32, i32, i32) -> Option<i32>,
        clock_ticks: impl Fn(&str) -> i64,
        weather: Weather,
        sky_flash: bool,
    ) -> Attributes {
        // GaussianSampler over the biomes around the camera, a quarter scale.
        let mut weights: Vec<(usize, f64)> = Vec::new();
        pub(super) const KERNEL: [f64; 7] = [0.0, 1.0, 4.0, 6.0, 4.0, 1.0, 0.0];
        let position = [
            eye[0] * 0.25 - 0.5,
            eye[1] * 0.25 - 0.5,
            eye[2] * 0.25 - 0.5,
        ];
        let integral = position.map(|v| v.floor() as i32);
        let relative = [
            position[0] - f64::from(integral[0]),
            position[1] - f64::from(integral[1]),
            position[2] - f64::from(integral[2]),
        ];
        let weight = |axis: usize, step: usize| {
            KERNEL[step + 1] + relative[axis] * (KERNEL[step] - KERNEL[step + 1])
        };
        for z in 0..6 {
            let weight_z = weight(2, z);
            for x in 0..6 {
                let weight_x = weight(0, x);
                for y in 0..6 {
                    let weight_y = weight(1, y);
                    let biome = biome_at(
                        integral[0] - 2 + x as i32,
                        integral[1] - 2 + y as i32,
                        integral[2] - 2 + z as i32,
                    )
                    .and_then(|id| usize::try_from(id).ok())
                    .filter(|id| *id < self.biomes.len())
                    .unwrap_or(self.plains);
                    let sample = weight_x * weight_y * weight_z;
                    match weights.iter_mut().find(|(id, _)| *id == biome) {
                        Some((_, total)) => *total += sample,
                        None => weights.push((biome, sample)),
                    }
                }
            }
        }

        let mut values = BTreeMap::new();
        for attribute in ATTRIBUTES {
            let kind = attribute.kind();
            let mut value = attribute.default_value();
            if let Some(entry) = self.dimension.get(&attribute) {
                value = apply(kind, entry.operation, value, entry.argument);
            }
            if self.biome_attributes.contains(&attribute) {
                let from_biome = |biome: usize, base: Value| match self
                    .biomes
                    .get(biome)
                    .and_then(|map| map.get(&attribute))
                {
                    Some(entry) => apply(kind, entry.operation, base, entry.argument),
                    None => base,
                };
                value = match weights.as_slice() {
                    [] => value,
                    [(biome, _)] => from_biome(*biome, value),
                    _ if kind == Kind::Phase => from_biome(weights[0].0, value),
                    _ => {
                        let mut result: Option<Value> = None;
                        let mut accumulated = 0.0_f64;
                        for (biome, weight) in &weights {
                            let source = from_biome(*biome, value);
                            accumulated += weight;
                            result = Some(match result {
                                None => source,
                                Some(previous) => {
                                    lerp(kind, (weight / accumulated) as f32, previous, source)
                                }
                            });
                        }
                        result.unwrap_or(value)
                    }
                };
            }
            for timeline in &self.timelines {
                for track in timeline
                    .tracks
                    .iter()
                    .filter(|track| track.attribute == attribute)
                {
                    let argument = track.sample(clock_ticks(&timeline.clock));
                    value = apply(kind, track.operation, value, argument);
                }
            }
            // The weather layer: the rain's change blended in by the rain
            // not already thunder, then the thunder's by the thunder.
            if self.weather {
                let rain = weather.rain - weather.thunder;
                if rain > 0.0
                    && let Some(rained) = weather_modifier(attribute, false, value)
                {
                    value = lerp(kind, rain, value, rained);
                }
                if weather.thunder > 0.0
                    && let Some(thundered) = weather_modifier(attribute, true, value)
                {
                    value = lerp(kind, weather.thunder, value, thundered);
                }
            }
            // `ClientLevel`'s own layers, last: a lightning flash lights the sky.
            if sky_flash {
                value = match attribute {
                    Attribute::SkyColor => {
                        Value::Color(srgb_lerp(0.22, value.color(), FLASH_COLOUR))
                    }
                    Attribute::SkyLightFactor => Value::Float(1.0),
                    _ => value,
                };
            }
            values.insert(attribute, attribute.sanitize(value));
        }
        Attributes { values }
    }
}
