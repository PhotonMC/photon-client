use super::*;

/// `engine` (the sound family): the sound engine played through scenarios, with channels that write
/// their calls down.
pub(super) fn sound_engine_result(request: &Value) -> Result<Value, String> {
    use photon_audio::library::{Channel, ChannelSource, Pool};
    use photon_core::random::RandomSource;
    use photon_game::sounds::engine::{Attenuation, InstanceId, SoundEngine, SoundInstance};
    use photon_game::sounds::{SoundRegistry, SoundSource};
    use std::cell::RefCell;
    use std::rc::Rc;
    type Log = Rc<RefCell<Vec<String>>>;
    struct Recorder {
        id: usize,
        log: Log,
        stopped: bool,
        paused: bool,
        finish: Rc<RefCell<Vec<usize>>>,
        /// The pool it came from, which it goes back to when it is destroyed.
        pool: (Rc<RefCell<[usize; 2]>>, usize),
    }
    impl Recorder {
        fn note(&self, call: String) {
            self.log
                .borrow_mut()
                .push(format!("channel{} {call}", self.id));
        }
    }
    let hex = |value: f32| format!("{:x}", value.to_bits());
    let wide = |value: f64| format!("{:x}", value.to_bits());
    impl Channel for Recorder {
        fn set_pitch(&mut self, pitch: f32) {
            self.note(format!("pitch {:x}", pitch.to_bits()));
        }
        fn set_volume(&mut self, volume: f32) {
            self.note(format!("volume {:x}", volume.to_bits()));
        }
        fn linear_attenuation(&mut self, max_distance: f32) {
            self.note(format!("attenuation linear {:x}", max_distance.to_bits()));
        }
        fn disable_attenuation(&mut self) {
            self.note("attenuation none".into());
        }
        fn set_looping(&mut self, looping: bool) {
            self.note(format!("looping {looping}"));
        }
        fn set_position(&mut self, position: (f64, f64, f64)) {
            self.note(format!(
                "position {:x} {:x} {:x}",
                position.0.to_bits(),
                position.1.to_bits(),
                position.2.to_bits()
            ));
        }
        fn set_relative(&mut self, relative: bool) {
            self.note(format!("relative {relative}"));
        }
        fn attach_static_buffer(&mut self, path: &str) {
            self.log.borrow_mut().push(format!("buffer {path}"));
            self.note("static buffer".into());
        }
        fn attach_stream(&mut self, path: &str, looping: bool) {
            self.log
                .borrow_mut()
                .push(format!("stream {path} looping {looping}"));
            self.note("stream buffer".into());
        }
        fn play(&mut self) {
            self.note("play".into());
        }
        fn pause(&mut self) {
            if !self.stopped && !self.paused {
                self.paused = true;
                self.note("pause".into());
            }
        }
        fn unpause(&mut self) {
            if self.paused {
                self.paused = false;
                self.note("unpause".into());
            }
        }
        fn stop(&mut self) {
            if !self.stopped {
                self.stopped = true;
                self.note("stop".into());
            }
        }
        fn stopped(&self) -> bool {
            self.stopped || self.finish.borrow().contains(&self.id)
        }
        fn update_stream(&mut self) {}
        fn destroy(&mut self) {
            self.note("destroy".into());
            self.pool.0.borrow_mut()[self.pool.1] -= 1;
        }
        fn number(&self) -> Option<usize> {
            Some(self.id)
        }
    }
    struct Source {
        log: Log,
        created: usize,
        used: Rc<RefCell<[usize; 2]>>,
        limits: [usize; 2],
        finish: Rc<RefCell<Vec<usize>>>,
    }
    impl ChannelSource for Source {
        type Channel = Recorder;
        fn acquire(&mut self, pool: Pool) -> Option<Recorder> {
            let slot = usize::from(pool == Pool::Streaming);
            if self.used.borrow()[slot] >= self.limits[slot] {
                return None;
            }
            self.used.borrow_mut()[slot] += 1;
            self.created += 1;
            Some(Recorder {
                id: self.created - 1,
                log: Rc::clone(&self.log),
                stopped: false,
                paused: false,
                finish: Rc::clone(&self.finish),
                pool: (Rc::clone(&self.used), slot),
            })
        }
    }
    let source_named = |name: &str| -> Result<SoundSource, String> {
        SoundSource::ALL
            .iter()
            .copied()
            .find(|source| {
                source.name().eq_ignore_ascii_case(name)
                    || format!("{source:?}").eq_ignore_ascii_case(name)
            })
            .ok_or_else(|| format!("no sound source {name}"))
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let files: std::collections::HashSet<String> = row["files"]
            .as_array()
            .ok_or("files")?
            .iter()
            .filter_map(|file| file.as_str().map(|file| format!("minecraft:{file}")))
            .collect();
        let mut registry = SoundRegistry::default();
        registry
            .register_file("minecraft", row["json"].as_str().ok_or("json")?, &|path| {
                files.contains(path)
            })
            .map_err(|_| "invalid sounds")?;
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let finish = Rc::new(RefCell::new(Vec::new()));
        let source = Source {
            log: Rc::clone(&log),
            created: 0,
            used: Rc::new(RefCell::new([0, 0])),
            limits: [
                row["static_channels"].as_u64().ok_or("static")? as usize,
                row["streaming_channels"].as_u64().ok_or("streaming")? as usize,
            ],
            finish: Rc::clone(&finish),
        };
        let mut engine = SoundEngine::new(registry, source);
        for source in SoundSource::ALL {
            if let Some(volume) = row["volumes"][format!("{source:?}").to_uppercase()].as_f64() {
                engine.volumes.set(source, volume);
            }
        }
        let mut ids: std::collections::HashMap<i64, InstanceId> = std::collections::HashMap::new();
        let mut logged = 0;
        let mut steps = Vec::new();
        for step in row["steps"].as_array().ok_or("steps")? {
            let mut result = json!({});
            let float = |key: &str| -> Result<f32, String> {
                u32::from_str_radix(step[key].as_str().ok_or(key.to_string())?, 16)
                    .map(f32::from_bits)
                    .map_err(|error| error.to_string())
            };
            match step["op"].as_str().ok_or("op")? {
                "play" | "play_delayed" | "queue_ticking" => {
                    let seed: i64 = step["seed"]
                        .as_str()
                        .ok_or("seed")?
                        .parse()
                        .map_err(|_| "seed")?;
                    let mut instance = SoundInstance::new(
                        step["event"].as_str().ok_or("event")?,
                        source_named(step["source"].as_str().ok_or("source")?)?,
                        RandomSource::create(seed),
                    );
                    instance.volume = float("volume")?;
                    instance.pitch = float("pitch")?;
                    instance.looping = step["looping"].as_bool().ok_or("looping")?;
                    instance.delay = step["delay"].as_i64().ok_or("delay")? as i32;
                    instance.attenuation = if step["attenuation"] == "none" {
                        Attenuation::None
                    } else {
                        Attenuation::Linear
                    };
                    instance.relative = step["relative"].as_bool().ok_or("relative")?;
                    let at = step["pos"].as_array().ok_or("pos")?;
                    instance.position = (
                        at[0].as_f64().ok_or("x")?,
                        at[1].as_f64().ok_or("y")?,
                        at[2].as_f64().ok_or("z")?,
                    );
                    instance.can_start_silent = step["start_silent"].as_bool().ok_or("silent")?;
                    instance.can_play = step["can_play"].as_bool().ok_or("can_play")?;
                    if let Some(script) = step["tickable"].as_object() {
                        let numbers = |key: &str| -> Vec<f64> {
                            script[key]
                                .as_array()
                                .map(|list| list.iter().filter_map(Value::as_f64).collect())
                                .unwrap_or_default()
                        };
                        let (moves, volumes, pitches) = (
                            script["moves"].as_array().cloned().unwrap_or_default(),
                            numbers("volumes"),
                            numbers("pitches"),
                        );
                        let stop_at = script["stop_at"].as_i64().ok_or("stop_at")?;
                        let mut ticks = 0_i64;
                        instance.ticker = Some(Box::new(move |state| {
                            ticks += 1;
                            if (ticks as usize) <= moves.len() {
                                let to = &moves[ticks as usize - 1];
                                *state.position = (
                                    to[0].as_f64().unwrap_or_default(),
                                    to[1].as_f64().unwrap_or_default(),
                                    to[2].as_f64().unwrap_or_default(),
                                );
                                *state.volume = volumes[ticks as usize - 1] as f32;
                                *state.pitch = pitches[ticks as usize - 1] as f32;
                            }
                            if ticks >= stop_at {
                                *state.stopped = true;
                                *state.looping = false;
                            }
                        }));
                    }
                    let id = engine.add(instance);
                    ids.insert(step["id"].as_i64().ok_or("id")?, id);
                    match step["op"].as_str().unwrap_or_default() {
                        "play" => {
                            result["result"] = json!(match engine.play(id) {
                                photon_game::sounds::engine::PlayResult::Started => "STARTED",
                                photon_game::sounds::engine::PlayResult::StartedSilently =>
                                    "STARTED_SILENTLY",
                                photon_game::sounds::engine::PlayResult::NotStarted =>
                                    "NOT_STARTED",
                            });
                        }
                        "play_delayed" => {
                            engine.play_delayed(
                                id,
                                step["delay_ticks"].as_i64().ok_or("delay_ticks")? as i32,
                            );
                        }
                        _ => engine.queue_ticking_sound(id),
                    }
                }
                "stop" => engine.stop(
                    *ids.get(&step["id"].as_i64().ok_or("id")?)
                        .ok_or("no instance")?,
                ),
                "stop_named" => engine.stop_matching(
                    step["event"].as_str(),
                    step["source"].as_str().map(source_named).transpose()?,
                ),
                "tick" => engine.tick(step["paused"].as_bool().ok_or("paused")?),
                "finish" => {
                    // The channel of that instance ends by itself: the recorder is told by its number.
                    let id = *ids
                        .get(&step["id"].as_i64().ok_or("id")?)
                        .ok_or("no instance")?;
                    if let Some(channel) = engine.channel_number(id) {
                        finish.borrow_mut().push(channel);
                    }
                }
                "category_volume" => engine.update_category_volume(
                    source_named(step["source"].as_str().ok_or("source")?)?,
                    float("value")?,
                ),
                "slider" => {
                    let source = source_named(step["source"].as_str().ok_or("source")?)?;
                    engine
                        .volumes
                        .set(source, step["value"].as_f64().ok_or("value")?);
                    engine.refresh_category_volume(source);
                }
                "pause_all_except" => {
                    let ignored: Vec<SoundSource> = step["sources"]
                        .as_array()
                        .ok_or("sources")?
                        .iter()
                        .map(|name| source_named(name.as_str().unwrap_or_default()))
                        .collect::<Result<_, _>>()?;
                    engine.pause_all_except(&ignored);
                }
                "resume" => engine.resume(),
                "active" => {
                    let id = *ids
                        .get(&step["id"].as_i64().ok_or("id")?)
                        .ok_or("no instance")?;
                    result["active"] = json!(engine.is_active(id));
                }
                _ => return Err("unknown engine step".into()),
            }
            // What each channel was asked, in order, the channels in their own order (a line that names
            // no channel belongs to the one before).
            let mut by_channel: std::collections::BTreeMap<i64, Vec<String>> =
                std::collections::BTreeMap::new();
            let mut current = -1_i64;
            let lines = log.borrow();
            for line in &lines[logged..] {
                if let Some(rest) = line.strip_prefix("channel") {
                    current = rest
                        .split(' ')
                        .next()
                        .and_then(|number| number.parse().ok())
                        .unwrap_or(-1);
                }
                by_channel.entry(current).or_default().push(line.clone());
            }
            logged = lines.len();
            result["calls"] = json!(by_channel.into_values().flatten().collect::<Vec<_>>());
            steps.push(result);
        }
        let _ = (&hex, &wide);
        out.push(json!({"label": row["label"], "steps": steps}));
    }
    Ok(json!({"rows": out}))
}

/// `decode` (the sound family): sound files decoded, held against the reference's own samples
/// (written by its row) one at a time, within a least significant bit.
pub(super) fn sound_decode_result(request: &Value) -> Result<Value, String> {
    let assets =
        std::path::Path::new(&std::env::var("PHOTON_ASSETS").map_err(|error| error.to_string())?)
            .parent()
            .and_then(std::path::Path::parent)
            .ok_or("assets")?
            .to_path_buf();
    let index = photon_core::json::read_object(&assets.join("indexes/32.json")).ok_or("index")?;
    let objects = index.object("objects").ok_or("objects")?;
    let reference = assets.parent().ok_or("assets")?.join("reference-pcm");
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let hash = objects
            .object(row["object"].as_str().ok_or("object")?)
            .and_then(|entry| entry.text("hash"))
            .ok_or("no such object")?;
        let data = std::fs::read(assets.join("objects").join(&hash[..2]).join(&hash))
            .map_err(|error| error.to_string())?;
        let pcm = photon_audio::decode::decode_ogg(&data).map_err(|error| error.0)?;
        let label = row["label"].as_str().ok_or("label")?;
        let expected = std::fs::read(reference.join(format!("{label}.pcm"))).map_err(|error| {
            format!("{label}: the reference's samples are missing ({error}); capture first")
        })?;
        let within = expected.len() == pcm.samples.len() * 2
            && expected
                .as_chunks::<2>()
                .0
                .iter()
                .zip(&pcm.samples)
                .all(|(bytes, sample)| {
                    (i32::from(i16::from_le_bytes([bytes[0], bytes[1]])) - i32::from(*sample)).abs()
                        <= 1
                });
        out.push(
            json!({"label": label, "channels": pcm.channels, "rate": pcm.rate,
            "samples": pcm.samples.len(), "within_one_lsb": within}),
        );
    }
    Ok(json!({"rows": out}))
}

fn reg() -> std::sync::Arc<photon_data::version_registries::VersionRegistries> {
    photon_data::version_registries::VersionRegistries::newest()
}

/// `packets` (the sound family): the three sound packets read from the bytes a scenario gives.
pub(super) fn sound_packets_result(request: &Value) -> Result<Value, String> {
    use photon_game::sounds::SoundHolder;
    use photon_net::connection::{
        read_sound_entity_packet, read_sound_packet, read_stop_sound_packet,
    };
    let bits = |value: f32| format!("{:08x}", value.to_bits());
    let wide = |value: f64| format!("{:016x}", value.to_bits());
    let holder = |out: &mut Value, sound: &SoundHolder| match sound {
        SoundHolder::Registered(id) => out["registered"] = json!(id),
        SoundHolder::Direct {
            location,
            fixed_range,
        } => {
            out["direct"] = json!(location);
            if let Some(range) = fixed_range {
                out["range"] = json!(bits(*range));
            }
        }
    };
    let mut out = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let hex = row["hex"].as_str().ok_or("hex")?;
        let bytes: Vec<u8> = (0..hex.len() / 2)
            .map(|index| u8::from_str_radix(&hex[2 * index..2 * index + 2], 16))
            .collect::<Result<_, _>>()
            .map_err(|error| error.to_string())?;
        let mut input = bytes.as_slice();
        let mut described = json!({"label": row["label"]});
        let read = match row["kind"].as_str().ok_or("kind")? {
            "sound" => read_sound_packet(&mut input, &reg()).map(|packet| {
                holder(&mut described, &packet.sound);
                described["source"] = json!(packet.source.ordinal());
                described["x"] = json!(wide(packet.position.0));
                described["y"] = json!(wide(packet.position.1));
                described["z"] = json!(wide(packet.position.2));
                described["volume"] = json!(bits(packet.volume));
                described["pitch"] = json!(bits(packet.pitch));
                described["seed"] = json!(packet.seed.to_string());
            }),
            "sound_entity" => read_sound_entity_packet(&mut input, &reg()).map(|packet| {
                holder(&mut described, &packet.sound);
                described["source"] = json!(packet.source.ordinal());
                described["entity"] = json!(packet.entity_id.0);
                described["volume"] = json!(bits(packet.volume));
                described["pitch"] = json!(bits(packet.pitch));
                described["seed"] = json!(packet.seed.to_string());
            }),
            _ => read_stop_sound_packet(&mut input).map(|packet| {
                if let Some(name) = &packet.name {
                    described["name"] = json!(name);
                }
                if let Some(source) = packet.source {
                    described["source"] = json!(source.ordinal());
                }
            }),
        };
        if read.is_ok() {
            described["trailing"] = json!(input.len());
        } else {
            described = json!({"label": row["label"], "error": true});
        }
        out.push(described);
    }
    Ok(json!({"rows": out}))
}
