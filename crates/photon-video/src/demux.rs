//! Reading the AV1 packets out of a WebM file.

use webm_iterable::WebmIterator;
use webm_iterable::matroska_spec::{MatroskaSpec, SimpleBlock};

use crate::VideoError;

/// One compressed frame of the video track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub data: Vec<u8>,
    /// When it is shown, in milliseconds from the start.
    pub pts_ms: u64,
}

const MAX_PACKETS: usize = 20_000;

/// The video track's packets of a WebM file, in file order.
pub fn demux(file: &[u8]) -> Result<Vec<Packet>, VideoError> {
    let mut source = file;
    let mut scale_ns: u64 = 1_000_000;
    let mut cluster_ms: u64 = 0;
    let mut track_number: Option<u64> = None;
    let mut av1_track: Option<u64> = None;
    let mut packets = Vec::new();
    for tag in WebmIterator::new(&mut source, &[]) {
        let tag = tag.map_err(|_| VideoError::Container("not a WebM file"))?;
        match &tag {
            MatroskaSpec::TimestampScale(value) => scale_ns = (*value).max(1),
            MatroskaSpec::Timestamp(value) => cluster_ms = value * scale_ns / 1_000_000,
            MatroskaSpec::TrackNumber(number) => track_number = Some(*number),
            MatroskaSpec::CodecID(codec) if codec == "V_AV1" => av1_track = track_number,
            MatroskaSpec::SimpleBlock(_) => {
                if packets.len() >= MAX_PACKETS {
                    return Err(VideoError::Container("too many frames"));
                }
                let times = (cluster_ms, scale_ns);
                block_packets(&tag, av1_track, times, &mut packets)?;
            }
            _ => {}
        }
    }
    if av1_track.is_none() {
        return Err(VideoError::Container("no AV1 video track"));
    }
    Ok(packets)
}

/// The packets of one block of the video track, added to `packets`; other tracks add none.
fn block_packets(
    tag: &MatroskaSpec,
    track: Option<u64>,
    (cluster_ms, scale_ns): (u64, u64),
    packets: &mut Vec<Packet>,
) -> Result<(), VideoError> {
    let block = SimpleBlock::try_from(tag).map_err(|_| VideoError::Container("bad block"))?;
    if Some(block.track) != track {
        return Ok(());
    }
    let nanos = cluster_ms as i64 * 1_000_000 + i64::from(block.timestamp) * scale_ns as i64;
    let pts_ms = nanos.max(0) as u64 / 1_000_000;
    let frames = block
        .read_frame_data()
        .map_err(|_| VideoError::Container("bad lacing"))?;
    packets.extend(frames.into_iter().map(|frame| Packet {
        data: frame.data.to_vec(),
        pts_ms,
    }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &[u8] = include_bytes!("../tests/data/tiny.webm");

    #[test]
    fn a_clip_gives_its_packets_in_time_order() {
        let packets = demux(TINY).unwrap();
        assert_eq!(packets.len(), 10);
        assert!(
            packets
                .windows(2)
                .all(|pair| pair[0].pts_ms < pair[1].pts_ms)
        );
        assert_eq!(packets[0].pts_ms, 0);
        assert!(packets[9].pts_ms >= 800);
    }

    #[test]
    fn something_that_is_not_webm_is_refused() {
        assert!(demux(b"not a video at all").is_err());
        assert!(demux(&[]).is_err());
    }
}
