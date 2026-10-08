//! Ogg/Vorbis decoding (`JOrbisAudioStream`, `FloatSampleSource`, `ChunkedSampleByteBuf`,
//! `LoopingAudioStream`): a file into the 16-bit signed samples (interleaved) its channels hold,
//! whole or a chunk at a time.

use std::io::Cursor;

use lewton::inside_ogg::OggStreamReader;
use lewton::samples::InterleavedSamples;

/// A decoded sound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcm {
    pub channels: u8,
    /// Samples per second, for each channel.
    pub rate: u32,
    /// Interleaved: one sample of every channel in turn.
    pub samples: Vec<i16>,
}

/// Why a file did not decode.
#[derive(Debug)]
pub struct DecodeError(pub String);

/// `ChunkedSampleByteBuf.accept`: a float sample as 16 bits, rounded the reference's way.
pub fn to_i16(sample: f32) -> i16 {
    ((sample * 32767.5 - 0.5) as i32).clamp(-32768, 32767) as i16
}

/// What the last page of a stream says it holds, in samples of each channel
/// (`JOrbisAudioStream`'s `totalSamplesInStream`): the granule position of the page that
/// ends the stream, if one does.
fn frames_in_stream(bytes: &[u8]) -> Option<u64> {
    const CAPTURE: &[u8; 4] = b"OggS";
    let at = bytes
        .windows(CAPTURE.len())
        .rposition(|window| window == CAPTURE)?;
    let header = bytes.get(at..at + 14)?;
    // Bit two of the header type is "last page of the stream".
    if header[5] & 4 == 0 {
        return None;
    }
    Some(u64::from_le_bytes(header[6..14].try_into().ok()?))
}

/// One Ogg/Vorbis file being read (`JOrbisAudioStream`).
pub struct OggStream {
    reader: OggStreamReader<Cursor<Vec<u8>>>,
    channels: u8,
    rate: u32,
    /// How many samples the stream holds in all, counting every channel.
    total_samples: u64,
    written: u64,
}

impl OggStream {
    pub fn new(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let frames = frames_in_stream(&bytes);
        let reader = OggStreamReader::new(Cursor::new(bytes))
            .map_err(|error| DecodeError(format!("{error:?}")))?;
        let channels = reader.ident_hdr.audio_channels;
        let rate = reader.ident_hdr.audio_sample_rate;
        Ok(Self {
            reader,
            channels,
            rate,
            total_samples: frames.map_or(u64::MAX, |frames| {
                frames.saturating_mul(u64::from(channels))
            }),
            written: 0,
        })
    }

    pub fn channels(&self) -> u8 {
        self.channels
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    /// `JOrbisAudioStream.readChunk`: the samples of the next packet, or none at the end of the
    /// stream, kept to what the stream holds.
    fn read_chunk(&mut self, out: &mut Vec<i16>) -> Result<bool, DecodeError> {
        let Some(packet) = self
            .reader
            .read_dec_packet_generic::<InterleavedSamples<f32>>()
            .map_err(|error| DecodeError(format!("{error:?}")))?
        else {
            return Ok(false);
        };
        let room = self.total_samples.saturating_sub(self.written);
        let take = usize::try_from(room)
            .unwrap_or(usize::MAX)
            .min(packet.samples.len());
        self.written += take as u64;
        out.extend(packet.samples[..take].iter().map(|sample| to_i16(*sample)));
        Ok(true)
    }

    /// `FloatSampleSource.read`: packets are read until at least `expected_bytes` bytes of samples
    /// are in hand; an empty result is the end of the stream.
    pub fn read(&mut self, expected_bytes: usize) -> Result<Vec<i16>, DecodeError> {
        let mut out = Vec::new();
        while out.len() * 2 < expected_bytes && self.read_chunk(&mut out)? {}
        Ok(out)
    }

    /// `FloatSampleSource.readAll`.
    pub fn read_all(&mut self) -> Result<Vec<i16>, DecodeError> {
        let mut out = Vec::new();
        while self.read_chunk(&mut out)? {}
        Ok(out)
    }
}

/// A stream that starts over when it ends (`LoopingAudioStream`).
pub struct LoopingOggStream {
    bytes: Vec<u8>,
    stream: OggStream,
}

impl LoopingOggStream {
    pub fn new(bytes: Vec<u8>) -> Result<Self, DecodeError> {
        let stream = OggStream::new(bytes.clone())?;
        Ok(Self { bytes, stream })
    }

    pub fn channels(&self) -> u8 {
        self.stream.channels()
    }

    pub fn rate(&self) -> u32 {
        self.stream.rate()
    }

    /// `LoopingAudioStream.read`: a read that finds the end reads again from the start.
    pub fn read(&mut self, expected_bytes: usize) -> Result<Vec<i16>, DecodeError> {
        let result = self.stream.read(expected_bytes)?;
        if !result.is_empty() {
            return Ok(result);
        }
        self.stream = OggStream::new(self.bytes.clone())?;
        self.stream.read(expected_bytes)
    }
}

/// `JOrbisAudioStream.readAll` on one file's bytes.
pub fn decode_ogg(bytes: &[u8]) -> Result<Pcm, DecodeError> {
    let mut stream = OggStream::new(bytes.to_vec())?;
    let samples = stream.read_all()?;
    Ok(Pcm {
        channels: stream.channels(),
        rate: stream.rate(),
        samples,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_rounded_towards_zero_after_the_half_step_and_clamped() {
        assert_eq!(to_i16(0.0), 0);
        assert_eq!(to_i16(1.0), 32767);
        assert_eq!(to_i16(-1.0), -32768);
        assert_eq!(to_i16(2.5), 32767);
        assert_eq!(to_i16(-2.5), -32768);
        assert_eq!(to_i16(f32::NAN), 0);
        // A tenth of the way up: 3276.75 - 0.5 truncates to 3276.
        assert_eq!(to_i16(0.1), 3276);
    }

    #[test]
    fn the_stream_holds_what_its_last_page_says_when_that_page_ends_it() {
        let mut page = b"OggS".to_vec();
        page.extend([0, 4]);
        page.extend(1234_u64.to_le_bytes());
        page.extend([0; 12]);
        assert_eq!(frames_in_stream(&page), Some(1234));
        // A last page that does not end the stream says nothing of its length.
        page[5] = 0;
        assert_eq!(frames_in_stream(&page), None);
        assert_eq!(frames_in_stream(b"nothing"), None);
    }
}
