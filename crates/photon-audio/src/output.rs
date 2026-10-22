//! The sound device (`Library`): the mixer's stereo played by what the system gives, or silence
//! where it gives nothing.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::mixer::Mixer;

/// Why no sound can be made.
#[derive(Debug)]
pub struct OutputError(pub String);

fn device_name(device: &cpal::Device) -> Option<String> {
    device
        .description()
        .ok()
        .map(|description| description.name().to_owned())
}

/// The values of the sound device option: the system default (empty), then each device
/// (`Options.soundDevice`'s `LazyEnum`).
pub fn device_choices(devices: Vec<String>) -> Vec<String> {
    std::iter::once(String::new()).chain(devices).collect()
}

/// The output devices the system offers and the one it defaults to (`DeviceList`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeviceList {
    pub default_device: Option<String>,
    pub all_devices: Vec<String>,
}

impl DeviceList {
    /// `DeviceList.query`.
    pub fn query() -> Self {
        Self {
            default_device: cpal::default_host()
                .default_output_device()
                .and_then(|device| device_name(&device)),
            all_devices: Output::devices(),
        }
    }
}

/// `SoundEngine.shouldChangeDevice` after the device is known not to be lost: whether the engine
/// goes to another device, given the device it is on, the lists last seen and now, and the sound
/// device option (empty for the system default).
pub fn device_should_change(
    current: &str,
    (seen, now): (&DeviceList, &DeviceList),
    option: &str,
) -> bool {
    if seen == now {
        return false;
    }
    let listed = |name: &str| now.all_devices.iter().any(|device| device == name);
    // The device it is on has gone from the list (the system default counts as listed: the
    // output API names it apart from the devices it lists).
    let gone = !listed(current) && now.default_device.as_deref() != Some(current);
    let wanted = if option.is_empty() {
        // The system default has changed.
        now.default_device.as_deref() != Some(current)
    } else {
        // The preferred device has become available.
        current != option && listed(option)
    };
    gone || wanted
}

/// An open output: the mixer it plays, and the stream that plays it.
pub struct Output {
    pub mixer: Arc<Mutex<Mixer>>,
    /// The name of the device it plays on.
    pub name: String,
    /// Set when the system says the device is gone (`Library.isCurrentDeviceDisconnected`).
    pub lost: Arc<AtomicBool>,
    _stream: cpal::Stream,
}

impl Output {
    /// The names of the output devices the system offers
    /// (`SoundManager.getAvailableSoundDevices`).
    pub fn devices() -> Vec<String> {
        let Ok(devices) = cpal::default_host().output_devices() else {
            return Vec::new();
        };
        devices.filter_map(|device| device_name(&device)).collect()
    }

    /// Open the preferred output device, or the default one when it is empty, gone or will not
    /// open (`Library.openDeviceOrFallback`).
    pub fn open(preferred: &str) -> Result<Self, OutputError> {
        let named = Self::find(preferred).and_then(|device| Self::open_device(Some(device)).ok());
        named.map_or_else(|| Self::open_device(None), Ok)
    }

    fn find(name: &str) -> Option<cpal::Device> {
        let mut devices = cpal::default_host().output_devices().ok()?;
        devices.find(|device| device_name(device).as_deref() == Some(name))
    }

    /// Open `device`, or the default output device for none.
    fn open_device(device: Option<cpal::Device>) -> Result<Self, OutputError> {
        let device = device
            .or_else(|| cpal::default_host().default_output_device())
            .ok_or_else(|| OutputError("no output device".into()))?;
        let supported = device
            .default_output_config()
            .map_err(|error| OutputError(error.to_string()))?;
        let name = device_name(&device).unwrap_or_default();
        let lost = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&lost);
        let rate = supported.sample_rate();
        let channels = usize::from(supported.channels());
        let mixer = Arc::new(Mutex::new(Mixer::new(rate)));
        let shared = Arc::clone(&mixer);
        let mut stereo: Vec<f32> = Vec::new();
        let config = supported.config();
        let stream = match supported.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    Self::fill(&shared, &mut stereo, channels, data);
                },
                move |error| {
                    log::error!("sound output: {error}");
                    if error.kind() == cpal::ErrorKind::DeviceNotAvailable {
                        flag.store(true, Ordering::Relaxed);
                    }
                },
                None,
            ),
            other => return Err(OutputError(format!("unsupported sample format {other:?}"))),
        }
        .map_err(|error| OutputError(error.to_string()))?;
        stream
            .play()
            .map_err(|error| OutputError(error.to_string()))?;
        Ok(Self {
            mixer,
            name,
            lost,
            _stream: stream,
        })
    }

    /// One callback's worth: the mixer's stereo into the device's channels (the first two; any
    /// others silent).
    fn fill(mixer: &Mutex<Mixer>, stereo: &mut Vec<f32>, channels: usize, data: &mut [f32]) {
        let frames = data.len() / channels.max(1);
        stereo.resize(frames * 2, 0.0);
        match mixer.lock() {
            Ok(mut mixer) => mixer.render(stereo),
            Err(_) => stereo.fill(0.0),
        }
        for (frame, pair) in data
            .chunks_exact_mut(channels.max(1))
            .zip(stereo.as_chunks::<2>().0.iter())
        {
            frame.fill(0.0);
            frame[0] = pair[0];
            if channels > 1 {
                frame[1] = pair[1];
            } else {
                frame[0] = (pair[0] + pair[1]) * 0.5;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(default: &str, all: &[&str]) -> DeviceList {
        DeviceList {
            default_device: Some(default.to_owned()),
            all_devices: all.iter().map(|name| (*name).to_owned()).collect(),
        }
    }

    #[test]
    fn the_engine_moves_only_for_the_reasons_the_reference_names() {
        let seen = list("Speakers", &["Speakers", "Headset"]);
        // Nothing changed.
        assert!(!device_should_change("Speakers", (&seen, &seen), ""));
        // The device it is on is gone.
        let gone = list("Headset", &["Headset"]);
        assert!(device_should_change("Speakers", (&seen, &gone), "Speakers"));
        // The default changed while the option is the default; and not while it is named.
        let moved = list("Headset", &["Speakers", "Headset"]);
        assert!(device_should_change("Speakers", (&seen, &moved), ""));
        assert!(!device_should_change(
            "Speakers",
            (&seen, &moved),
            "Speakers"
        ));
        // The default device is not in the list under its own name, and is not gone for that.
        let named = list("Default Audio Device", &["Speakers"]);
        let other = list("Default Audio Device", &["Speakers", "Headset"]);
        assert!(!device_should_change(
            "Default Audio Device",
            (&named, &other),
            ""
        ));
        // The preferred device appears.
        let before = list("Speakers", &["Speakers"]);
        let after = list("Speakers", &["Speakers", "Headset"]);
        assert!(device_should_change(
            "Speakers",
            (&before, &after),
            "Headset"
        ));
        // A preferred device that is not listed stays unwanted.
        assert!(!device_should_change("Speakers", (&before, &after), "Dock"));
    }

    #[test]
    fn the_default_device_comes_first() {
        let choices = device_choices(vec!["Speakers".into(), "Headset".into()]);
        assert_eq!(choices, ["", "Speakers", "Headset"]);
        assert_eq!(device_choices(Vec::new()), [""]);
    }
}
