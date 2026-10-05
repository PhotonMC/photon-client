//! The sound device (`Library`): the mixer's stereo played by what the system gives, or silence
//! where it gives nothing.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::mixer::Mixer;

/// Why no sound can be made.
#[derive(Debug)]
pub struct OutputError(pub String);

/// An open output: the mixer it plays, and the stream that plays it.
pub struct Output {
    pub mixer: Arc<Mutex<Mixer>>,
    _stream: cpal::Stream,
}

impl Output {
    /// Open the default output device.
    pub fn open() -> Result<Self, OutputError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
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
                |error| eprintln!("sound output: {error}"),
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
            .zip(stereo.chunks_exact(2))
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
