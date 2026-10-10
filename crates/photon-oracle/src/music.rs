//! `music_manager`: the music manager ticked with what a scenario says the game wants.

use photon_core::random::RandomSource;
use photon_game::options::MusicFrequency;
use photon_game::sounds::music::{Music, MusicHost, MusicManager, Started};
use serde_json::{Value, json};

struct Host {
    music: Option<Music>,
    volume: f32,
    active: bool,
    calls: Vec<String>,
}

impl MusicHost for Host {
    fn volume(&self) -> f32 {
        self.volume
    }
    fn situational_music(&self) -> Option<Music> {
        self.music.clone()
    }
    fn loading_screen(&self) -> bool {
        false
    }
    fn is_active(&self) -> bool {
        self.active
    }
    fn play(&mut self, music: &Music) -> Started {
        self.calls.push(format!("play {}", music.sound));
        self.active = true;
        Started::No
    }
    fn stop(&mut self, event: &str) {
        self.calls.push(format!("stop {event}"));
        self.active = false;
    }
    fn update_music_volume(&mut self, gain: f32) {
        self.calls.push(format!("volume {:08x}", gain.to_bits()));
    }
}

fn bits(value: &Value) -> Result<f32, String> {
    let text = value.as_str().ok_or("bits")?;
    u32::from_str_radix(text, 16)
        .map(f32::from_bits)
        .map_err(|error| error.to_string())
}

pub(super) fn music_manager_result(request: &Value) -> Result<Value, String> {
    let mut rows = Vec::new();
    for row in request["rows"].as_array().ok_or("rows")? {
        let frequency = match row["frequency"].as_str().ok_or("frequency")? {
            "FREQUENT" => MusicFrequency::Frequent,
            "CONSTANT" => MusicFrequency::Constant,
            _ => MusicFrequency::Default,
        };
        let mut manager = MusicManager::new(
            RandomSource::create(row["seed"].as_i64().ok_or("seed")?),
            frequency,
        );
        let mut host = Host {
            music: None,
            volume: 1.0,
            active: false,
            calls: Vec::new(),
        };
        let mut ticks = Vec::new();
        for tick in row["ticks"].as_array().ok_or("ticks")? {
            host.music = tick["music"].as_object().map(|music| {
                Music::new(
                    music["sound"].as_str().unwrap_or_default(),
                    music["min"].as_i64().unwrap_or(0) as i32,
                    music["max"].as_i64().unwrap_or(0) as i32,
                    music["replace"].as_bool().unwrap_or(false),
                )
            });
            host.volume = bits(&tick["volume"])?;
            if let Some(active) = tick["active"].as_bool() {
                host.active = active;
            }
            host.calls.clear();
            manager.tick(&mut host);
            let current = manager.current().unwrap_or_default().to_owned();
            let calls = host.calls.clone();
            ticks.push(json!({
                "delay": manager.next_song_delay(),
                "current": current,
                "gain": format!("{:08x}", manager.gain().to_bits()),
                "calls": calls,
            }));
        }
        rows.push(json!({"label": row["label"], "ticks": ticks}));
    }
    Ok(json!({"rows": rows}))
}
