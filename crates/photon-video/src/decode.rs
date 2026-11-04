//! The AV1 decoder (`rav1d`, called through its dav1d-shaped interface) and the pictures it
//! gives back.

use std::mem::MaybeUninit;
use std::ptr::NonNull;

use rav1d::include::dav1d::data::Dav1dData;
use rav1d::include::dav1d::dav1d::{Dav1dContext, Dav1dSettings};
use rav1d::include::dav1d::headers::DAV1D_PIXEL_LAYOUT_I420;
use rav1d::include::dav1d::picture::Dav1dPicture;
use rav1d::src::lib::{
    dav1d_close, dav1d_data_create, dav1d_default_settings, dav1d_get_picture, dav1d_open,
    dav1d_picture_unref, dav1d_send_data,
};

use crate::VideoError;
use crate::convert::yuv420_to_rgba;

/// The largest picture taken, per side.
const MAX_SIDE: i32 = 1920;
/// `-EAGAIN`: the decoder wants its pictures taken before it takes more data.
const AGAIN: i32 = -11;

/// One decoded picture.
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub struct Decoder {
    context: Option<Dav1dContext>,
}

impl Decoder {
    pub fn new(threads: i32) -> Result<Self, VideoError> {
        let mut settings = MaybeUninit::<Dav1dSettings>::zeroed();
        let mut context = None;
        // SAFETY: `settings` is zeroed storage the call fills in, `context` is a place for the
        // handle, and both outlive the call.
        let opened = unsafe {
            dav1d_default_settings(NonNull::from(&mut settings).cast());
            let mut filled = settings.assume_init();
            filled.n_threads = threads;
            filled.frame_size_limit = (MAX_SIDE * MAX_SIDE) as u32;
            dav1d_open(NonNull::new(&mut context), NonNull::new(&mut filled))
        };
        if opened.0 != 0 || context.is_none() {
            return Err(VideoError::Codec("decoder did not open"));
        }
        Ok(Self { context })
    }

    /// The pictures that one packet completes.
    pub fn decode(&mut self, packet: &[u8]) -> Result<Vec<Picture>, VideoError> {
        let data = MaybeUninit::<Dav1dData>::zeroed();
        // SAFETY: `data` is zeroed storage `dav1d_data_create` fills; the buffer it returns holds
        // `packet.len()` bytes, which are copied in before the decoder reads them.
        let mut data = unsafe {
            let mut data = data.assume_init();
            let buffer = dav1d_data_create(NonNull::new(&mut data), packet.len());
            if buffer.is_null() {
                return Err(VideoError::Codec("out of memory"));
            }
            std::ptr::copy_nonoverlapping(packet.as_ptr(), buffer, packet.len());
            data
        };
        let mut pictures = Vec::new();
        loop {
            // SAFETY: the context is open, and `data` is the packet made above.
            let sent = unsafe { dav1d_send_data(self.context, NonNull::new(&mut data)) };
            self.drain(&mut pictures)?;
            match sent.0 {
                0 => return Ok(pictures),
                AGAIN => {}
                _ => return Err(VideoError::Codec("bad data")),
            }
        }
    }

    /// The pictures still held back once every packet has been given.
    pub fn finish(&mut self) -> Result<Vec<Picture>, VideoError> {
        let mut pictures = Vec::new();
        self.drain(&mut pictures)?;
        Ok(pictures)
    }

    fn drain(&mut self, pictures: &mut Vec<Picture>) -> Result<(), VideoError> {
        loop {
            let picture = MaybeUninit::<Dav1dPicture>::zeroed();
            // SAFETY: zeroed storage is what the call expects; on success it holds a picture that
            // is read and then released exactly once.
            let taken = unsafe {
                let mut zeroed = picture.assume_init();
                let status = dav1d_get_picture(self.context, NonNull::new(&mut zeroed));
                (status.0 == 0).then_some(zeroed)
            };
            let Some(mut picture) = taken else {
                return Ok(());
            };
            let converted = convert(&picture);
            // SAFETY: the picture was filled by `dav1d_get_picture` above and is released once.
            unsafe { dav1d_picture_unref(NonNull::new(&mut picture)) };
            pictures.push(converted?);
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: the context was opened by `dav1d_open` and is closed once.
        unsafe { dav1d_close(NonNull::new(&mut self.context)) };
    }
}

fn convert(picture: &Dav1dPicture) -> Result<Picture, VideoError> {
    let (width, height) = (picture.p.w, picture.p.h);
    if picture.p.layout != DAV1D_PIXEL_LAYOUT_I420 || picture.p.bpc != 8 {
        return Err(VideoError::Codec("only 8-bit 4:2:0 video is played"));
    }
    if !(1..=MAX_SIDE).contains(&width) || !(1..=MAX_SIDE).contains(&height) {
        return Err(VideoError::Codec("picture size"));
    }
    let (width, height) = (width as usize, height as usize);
    let (y_stride, c_stride) = (picture.stride[0] as usize, picture.stride[1] as usize);
    let (Some(y), Some(u), Some(v)) = (picture.data[0], picture.data[1], picture.data[2]) else {
        return Err(VideoError::Codec("missing plane"));
    };
    let chroma_rows = height.div_ceil(2);
    // SAFETY: the decoder owns planes of `stride * rows` bytes for a picture of this size; they
    // stay valid until the picture is released, which happens after this copy.
    let rgba = unsafe {
        yuv420_to_rgba(
            (
                std::slice::from_raw_parts(y.as_ptr().cast::<u8>(), y_stride * height),
                y_stride,
            ),
            (
                std::slice::from_raw_parts(u.as_ptr().cast::<u8>(), c_stride * chroma_rows),
                std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), c_stride * chroma_rows),
                c_stride,
            ),
            (width, height),
        )
    };
    Ok(Picture {
        width: width as u32,
        height: height as u32,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demux::demux;

    const TINY: &[u8] = include_bytes!("../tests/data/tiny.webm");

    #[test]
    fn a_clip_decodes_to_all_its_pictures() {
        let packets = demux(TINY).unwrap();
        let mut decoder = Decoder::new(1).unwrap();
        let mut pictures = Vec::new();
        for packet in &packets {
            pictures.extend(decoder.decode(&packet.data).unwrap());
        }
        pictures.extend(decoder.finish().unwrap());
        assert_eq!(pictures.len(), packets.len());
        assert_eq!((pictures[0].width, pictures[0].height), (64, 48));
        assert_eq!(pictures[0].rgba.len(), 64 * 48 * 4);
        // A test pattern is not one flat colour.
        assert!(
            pictures[0]
                .rgba
                .chunks(4)
                .any(|px| px[..3] != pictures[0].rgba[..3])
        );
    }

    #[test]
    fn garbage_is_an_error_not_a_crash() {
        let mut decoder = Decoder::new(1).unwrap();
        let _ = decoder.decode(&[1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
