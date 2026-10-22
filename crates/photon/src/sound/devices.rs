//! `PollingDeviceTracker`: the output devices the system offers, asked again in the background
//! once a second, and what `SoundEngine.shouldChangeDevice` decides from them.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use photon_audio::output::{DeviceList, NO_DEVICE, device_should_change};
use photon_game::options::Options;

use super::SoundSystem;

/// The sound engine, or what stays of it while it has no device: the tracker goes on watching the
/// devices, so one that appears starts the engine (`SoundEngine` keeps its tracker when the
/// library failed to load).
struct Idle {
    tracker: DeviceTracker,
    assets: std::path::PathBuf,
}

/// The sound engine with what watches for a device in its place when it has none; it reads as
/// the engine's `Option`.
pub struct SoundSlot {
    system: Option<SoundSystem>,
    idle: Option<Idle>,
}

impl std::ops::Deref for SoundSlot {
    type Target = Option<SoundSystem>;

    fn deref(&self) -> &Self::Target {
        &self.system
    }
}

impl std::ops::DerefMut for SoundSlot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.system
    }
}

impl SoundSlot {
    /// `SoundEngine.reload` at the start: the engine, or what keeps watching for a device.
    pub fn open(assets: &std::path::Path, options: &Options) -> Self {
        let system = SoundSystem::open(assets, options);
        let idle = system.is_none().then(|| Idle {
            tracker: DeviceTracker::new(),
            assets: assets.to_path_buf(),
        });
        Self { system, idle }
    }

    /// The sound device option changed: reload on the device now chosen, with a click.
    pub fn reload_for_device(&mut self, options: &Options) {
        SoundSystem::reload_for_device(&mut self.system, &mut self.idle, options);
    }

    /// `SoundEngine.tick`'s check of the devices.
    pub fn track_device(&mut self, options: &Options) {
        SoundSystem::track_device(&mut self.system, &mut self.idle, options);
    }
}

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
    fn reload_for_device(sound: &mut Option<Self>, idle: &mut Option<Idle>, options: &Options) {
        let Some(device) = sound.as_ref().map(|sound| sound.device.0.clone()) else {
            return;
        };
        if device != options.sound_device {
            Self::reload(sound, idle, options);
            if let Some(sound) = sound.as_mut() {
                sound.play_ui("minecraft:ui.button.click", 1.0, 1.0);
            }
        }
    }

    /// `SoundEngine.tick`'s check of the device: a device that is lost, gone or preferred again
    /// reloads the engine, without the click an option change plays. With no engine, a change of
    /// the devices tries to start it.
    fn track_device(sound: &mut Option<Self>, idle: &mut Option<Idle>, options: &Options) {
        if let Some(waiting) = idle.as_mut() {
            waiting.tracker.tick();
            if waiting
                .tracker
                .should_change(NO_DEVICE, &options.sound_device)
                && let Some(waiting) = idle.take()
            {
                Self::start(sound, idle, (waiting.tracker, waiting.assets), options);
            }
            return;
        }
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
            Self::reload(sound, idle, options);
        }
    }

    /// `SoundManager.reload`: every sound stops and the engine starts again on the device now
    /// chosen, keeping the tracker's view of the devices.
    fn reload(sound: &mut Option<Self>, idle: &mut Option<Idle>, options: &Options) {
        let Some(old) = sound.take() else {
            return;
        };
        let assets = old.device.1.clone();
        let tracker = old.tracker;
        // `SoundEngine.reload` destroys the library before it loads the next, so the old stream
        // lets go of the device first.
        drop(old.output);
        drop(old.engine);
        Self::start(sound, idle, (tracker, assets), options);
    }

    /// The engine started with the tracker it had, or watching for a device when it will not.
    fn start(
        sound: &mut Option<Self>,
        idle: &mut Option<Idle>,
        (tracker, assets): (DeviceTracker, std::path::PathBuf),
        options: &Options,
    ) {
        *sound = Self::open(&assets, options);
        match sound.as_mut() {
            Some(sound) => sound.tracker = tracker,
            None => *idle = Some(Idle { tracker, assets }),
        }
    }
}
