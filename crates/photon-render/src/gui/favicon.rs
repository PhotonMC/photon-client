//! A server's icon in the GUI atlas (`FaviconTexture` and the list entry's `uploadServerIcon`).

use crate::gui::atlas::GuiAtlas;
use crate::texture::png::{DecodedImage, decode_rgba};
use photon_core::sha1::hash_unencoded_chars;

/// `FaviconTexture.forServer`: the texture a server of `address` has its icon registered as.
pub fn texture_name(address: &str) -> String {
    format!("servers/{}/icon", hash_unencoded_chars(address))
}

/// `NativeImage.read` and the size check of `FaviconTexture.upload`: the image of a 64 by 64 PNG.
fn decode_icon(bytes: &[u8]) -> Option<DecodedImage> {
    decode_rgba(bytes)
        .ok()
        .filter(|image| image.width == 64 && image.height == 64)
}

/// What a draw of a list entry made of its server's icon.
#[derive(Debug, PartialEq, Eq)]
pub struct IconSync {
    /// The texture the entry draws the icon from; none draws the unknown server.
    pub texture: Option<String>,
    /// Whether the atlas image changed and is to be sent to the GPU again.
    pub atlas_changed: bool,
    /// Whether the icon could not be used, which the entry answers by forgetting it.
    pub rejected: bool,
}

/// The entry's check of `bytes`, the server's icon, against `last`, the ones it showed: none when
/// they are the same, otherwise the icon uploaded or the texture cleared.
pub fn sync_icon(
    atlas: &mut GuiAtlas,
    atlas_image: &mut DecodedImage,
    address: &str,
    bytes: Option<&[u8]>,
    last: &mut Option<Vec<u8>>,
) -> Option<IconSync> {
    if bytes == last.as_deref() {
        return None;
    }
    let name = texture_name(address);
    let icon = bytes.map(decode_icon);
    let image = icon.as_ref().and_then(Option::as_ref);
    let rejected = bytes.is_some() && image.is_none();
    // An icon that fails to upload leaves the texture as it was: the entry's own is empty unless
    // it showed an icon before, which the cleared bytes then take out.
    let placed = if rejected && last.is_none() {
        false
    } else {
        atlas.set_favicon(atlas_image, &name, image)
    };
    *last = if rejected {
        None
    } else {
        bytes.map(<[u8]>::to_vec)
    };
    Some(IconSync {
        texture: (placed && image.is_some()).then_some(name),
        atlas_changed: placed,
        rejected,
    })
}

/// The texture a pack's icon is registered as (`PackSelectionScreen.loadPackIcon`).
pub fn pack_texture_name(id: &str) -> String {
    let clean: String = id
        .chars()
        .map(|ch| {
            if matches!(ch, '_' | '-' | 'a'..='z' | '0'..='9' | '/' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    format!("pack/{clean}/{}/icon", hash_unencoded_chars(id))
}

/// The image of a pack icon of any size, as the 64 by 64 slots of the atlas take it: each slot
/// pixel is the average of the source pixels it covers. The reference keeps the full size and
/// samples that.
fn fit_icon(image: &DecodedImage) -> Option<DecodedImage> {
    let (width, height) = (image.width as usize, image.height as usize);
    if width == 0 || height == 0 || image.rgba.len() < width * height * 4 {
        return None;
    }
    let size = 64_usize;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        let rows = covered(y, size, height);
        for x in 0..size {
            rgba.extend_from_slice(&average(image, covered(x, size, width), rows.clone()));
        }
    }
    Some(DecodedImage {
        width: size as u32,
        height: size as u32,
        rgba,
    })
}

/// The source pixels of a line of `source` that slot pixel `index` of `slots` covers.
fn covered(index: usize, slots: usize, source: usize) -> std::ops::Range<usize> {
    let from = index * source / slots;
    let to = ((index + 1) * source).div_ceil(slots);
    from..to.max(from + 1).min(source)
}

/// The mean colour of a block of the image.
fn average(
    image: &DecodedImage,
    columns: std::ops::Range<usize>,
    rows: std::ops::Range<usize>,
) -> [u8; 4] {
    let width = image.width as usize;
    let mut sum = [0_u32; 4];
    let count = (columns.len() * rows.len()).max(1) as u32;
    for row in rows {
        for column in columns.clone() {
            let from = (row * width + column) * 4;
            for (total, byte) in sum.iter_mut().zip(&image.rgba[from..from + 4]) {
                *total += u32::from(*byte);
            }
        }
    }
    sum.map(|total| (total / count) as u8)
}

/// A pack's `pack.png` put into the atlas: the texture to draw it from, or none when the image
/// cannot be read or there is no slot left. `atlas_changed` says the GPU copy is to be sent again.
pub fn upload_pack_icon(
    atlas: &mut GuiAtlas,
    atlas_image: &mut DecodedImage,
    id: &str,
    bytes: &[u8],
) -> (Option<String>, bool) {
    let Some(icon) = decode_rgba(bytes).ok().and_then(|image| fit_icon(&image)) else {
        return (None, false);
    };
    let name = pack_texture_name(id);
    let placed = atlas.set_favicon(atlas_image, &name, Some(&icon));
    (placed.then_some(name), placed)
}

#[cfg(test)]
mod pack_icon_tests {
    use super::*;

    fn image(size: u32, pixel: impl Fn(u32, u32) -> u8) -> DecodedImage {
        let mut rgba = Vec::new();
        for y in 0..size {
            for x in 0..size {
                rgba.extend_from_slice(&[pixel(x, y), 0, 0, 255]);
            }
        }
        DecodedImage {
            width: size,
            height: size,
            rgba,
        }
    }

    #[test]
    fn a_large_icon_is_averaged_into_the_slot() {
        // 128 wide: two source pixels per slot pixel, alternately 0 and 100.
        let fitted = fit_icon(&image(128, |x, _| if x % 2 == 0 { 0 } else { 100 })).unwrap();
        assert_eq!((fitted.width, fitted.height), (64, 64));
        assert_eq!(fitted.rgba[0], 50);
        assert_eq!(fitted.rgba[3], 255);
    }

    #[test]
    fn an_icon_of_the_slot_size_is_kept() {
        let fitted = fit_icon(&image(64, |x, y| (x + y) as u8)).unwrap();
        assert_eq!(fitted.rgba[(3 * 64 + 5) * 4], 8);
    }

    #[test]
    fn a_pack_texture_is_named_by_its_id() {
        let name = pack_texture_name("file/My Pack.zip");
        assert!(name.starts_with("pack/file/my_pack.zip/") || name.starts_with("pack/file/"));
        assert!(name.ends_with("/icon"));
    }
}
