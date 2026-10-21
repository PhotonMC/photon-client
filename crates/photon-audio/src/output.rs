//! The sound device (`Library`): the mixer's stereo played by what the system gives, or silence
//! where it gives nothing.

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

/// An open output: the mixer it plays, and the stream that plays it.
pub struct Output {
    pub mixer: Arc<Mutex<Mixer>>,
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

    /// Open the preferred output device, or the default one when it is empty or gone
    /// (`Library.openDeviceOrFallback`).
    pub fn open(preferred: &str) -> Result<Self, OutputError> {
        let host = cpal::default_host();
        let named = (!preferred.is_empty())
            .then(|| host.output_devices().ok())
            .flatten()
            .and_then(|mut devices| {
                devices.find(|device| device_name(device).as_deref() == Some(preferred))
            });
        let device = named
            .or_else(|| host.default_output_device())
            .ok_or_else(|| OutputError("no output device".into()))?;
        let supported = device
            .default_output_config()
            .map_err(|error| OutputError(error.to_string()))?;
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
                |error| log::error!("sound output: {error}"),
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

    #[test]
    fn the_default_device_comes_first() {
        let choices = device_choices(vec!["Speakers".into(), "Headset".into()]);
        assert_eq!(choices, ["", "Speakers", "Headset"]);
        assert_eq!(device_choices(Vec::new()), [""]);
    }
}
