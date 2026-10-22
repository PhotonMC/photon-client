//! `PollingDeviceTracker`: the output devices the system offers, asked again in the background
//! once a second, and what `SoundEngine.shouldChangeDevice` decides from them.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use photon_audio::output::{DeviceList, device_should_change};
use photon_game::options::Options;

use super::SoundSystem;

/// `PollingDeviceTracker.DEFAULT_DEVICE_CHECK_INTERVAL_MS`.
const CHECK_INTERVAL: Duration = Duration::from_millis(1000);

pub struct DeviceTracker {
    /// What the last query found (`AbstractDeviceTracker.deviceList`).
    current: Arc<Mutex<DeviceList>>,
    pending: Arc<AtomicBool>,
    last_check: Instant,
    /// `SoundEngine.lastSeenDevices`.
    seen: DeviceList,
}

impl DeviceTracker {
    pub fn new() -> Self {
        let list = DeviceList::query();
        Self {
            seen: list.clone(),
            current: Arc::new(Mutex::new(list)),
            pending: Arc::new(AtomicBool::new(false)),
            last_check: Instant::now(),
        }
    }

    /// `forceRefresh`: ask now, on this thread.
    pub fn force_refresh(&mut self) {
        self.last_check = Instant::now();
        if let Ok(mut current) = self.current.lock() {
            *current = DeviceList::query();
        }
    }

    /// `tick`: once the interval has passed, ask in the background unless a query is still out.
    pub fn tick(&mut self) {
        if self.last_check.elapsed() < CHECK_INTERVAL {
            return;
        }
        self.last_check = Instant::now();
        if self.pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let (current, pending) = (Arc::clone(&self.current), Arc::clone(&self.pending));
        std::thread::spawn(move || {
            let list = DeviceList::query();
            if let Ok(mut current) = current.lock() {
                *current = list;
            }
            pending.store(false, Ordering::Release);
        });
    }

    /// `SoundEngine.shouldChangeDevice` after the device is known not to be lost: whether the
    /// lists seen since call for another device; the lists are remembered when they differ.
    pub fn should_change(&mut self, on: &str, option: &str) -> bool {
        let Ok(now) = self.current.lock().map(|list| list.clone()) else {
            return false;
        };
        let change = device_should_change(on, (&self.seen, &now), option);
        self.seen = now;
        change
    }
}

impl SoundSystem {
    /// `SoundManager.reload` after the sound device option changed: every sound stops and the
    /// engine starts on the device now chosen, with a UI click as its first sound. Nothing
    /// happens while the option is as the engine was opened for.
    pub fn reload_for_device(sound: &mut Option<Self>, options: &Options) {
        let Some(device) = sound.as_ref().map(|sound| sound.device.0.clone()) else {
            return;
        };
        if device != options.sound_device {
            Self::reload(sound, options);
            if let Some(sound) = sound.as_mut() {
                sound.play_ui("minecraft:ui.button.click", 1.0, 1.0);
            }
        }
    }

    /// `SoundEngine.tick`'s check of the device: a device that is lost, gone or preferred again
    /// reloads the engine, without the click an option change plays.
    pub fn track_device(sound: &mut Option<Self>, options: &Options) {
        let Some(this) = sound.as_mut() else {
            return;
        };
        let change = if this.output.lost.load(Ordering::Relaxed) {
            log::info!("audio device was lost");
            this.tracker.force_refresh();
            true
        } else {
            this.tracker.tick();
            let on = this.output.name.clone();
            this.tracker.should_change(&on, &options.sound_device)
        };
        if change {
            Self::reload(sound, options);
        }
    }

    /// `SoundManager.reload`: every sound stops and the engine starts again on the device now
    /// chosen, keeping the tracker's view of the devices.
    fn reload(sound: &mut Option<Self>, options: &Options) {
        let Some(old) = sound.take() else {
            return;
        };
        let assets = old.device.1.clone();
        let tracker = old.tracker;
        // `SoundEngine.reload` destroys the library before it loads the next, so the old stream
        // lets go of the device first.
        drop(old.output);
        drop(old.engine);
        *sound = Self::open(&assets, options);
        if let Some(sound) = sound.as_mut() {
            sound.tracker = tracker;
        }
    }
}
