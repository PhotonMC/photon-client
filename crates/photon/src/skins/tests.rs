use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use photon_render::DecodedImage;

use super::*;

fn blank(side: u32) -> DecodedImage {
    DecodedImage {
        width: side,
        height: side,
        rgba: vec![0; (side * side * 4) as usize],
    }
}

/// Entity textures with the default skins and the places.
fn textures() -> EntityTextures {
    let mut named: Vec<_> = photon_render::entity::player_renderer::skin_textures()
        .map(|name| (name.to_owned(), blank(64)))
        .collect();
    named.extend((0..SLOT_COUNT).map(|index| (slot_name(index), blank(64))));
    EntityTextures::build(named).expect("textures")
}

fn property(value: &str) -> ProfileProperty {
    ProfileProperty {
        name: "textures".into(),
        value: value.into(),
        signature: None,
    }
}

/// A skin named by the property's value, filled with the first byte of the value.
fn loader(property: &ProfileProperty, _: &KeyCell, _: &Path) -> Option<Loaded> {
    if property.value == "broken" {
        return None;
    }
    let fill = property.value.as_bytes()[0];
    Some(Loaded {
        hash: property.value.clone(),
        slim: property.value.starts_with('s'),
        secure: property.signature.is_some(),
        rgba: Arc::new(vec![fill; 64 * 64 * 4]),
    })
}

fn who(id: u64, properties: &[ProfileProperty], require_secure: bool) -> Who<'_> {
    Who {
        id: Uuid::new(0, id),
        properties,
        require_secure,
    }
}

/// Frames until the player's skin has loaded: the rectangle it ends up with.
fn settle(skins: &mut Skins, atlas: &mut EntityTextures, who: &Who) -> (TextureRect, bool) {
    let started = Instant::now();
    loop {
        skins.begin_frame();
        let found = skins.resolve(atlas, who).expect("a skin");
        if skins.rects.contains(&found.0) || started.elapsed() > Duration::from_secs(5) {
            return found;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[test]
fn a_player_wears_the_default_skin_without_textures_and_while_loading() {
    let mut atlas = textures();
    let mut skins = Skins::with_loader(&atlas, PathBuf::new(), loader);
    let default = photon_render::entity::player_renderer::default_skin(Uuid::new(0, 7));
    let expected = (atlas.rect(default.texture).unwrap(), default.slim);
    skins.begin_frame();
    assert_eq!(
        skins.resolve(&mut atlas, &who(7, &[], false)),
        Some(expected)
    );
    let properties = [property("wide")];
    // The first frame only asks for the load.
    assert_eq!(
        skins.resolve(&mut atlas, &who(7, &properties, false)),
        Some(expected)
    );
}

#[test]
fn a_loaded_skin_is_written_to_its_place() {
    let mut atlas = textures();
    let mut skins = Skins::with_loader(&atlas, PathBuf::new(), loader);
    let properties = [property("slim")];
    let (rect, slim) = settle(&mut skins, &mut atlas, &who(1, &properties, false));
    assert!(slim);
    assert!(!skins.take_dirty().is_empty());
    let (width, height) = (atlas.image.width as f32, atlas.image.height as f32);
    let (x, y) = (
        (rect[0] * width).round() as usize,
        (rect[1] * height).round() as usize,
    );
    let at = |atlas: &EntityTextures, x: usize, y: usize| {
        atlas.image.rgba[(y * atlas.image.width as usize + x) * 4]
    };
    assert_eq!(at(&atlas, x, y), b's');
    assert_eq!(at(&atlas, x + 63, y + 63), b's');
    // Another skin, in another place, leaves it alone.
    let others = [property("wide")];
    let (other, slim) = settle(&mut skins, &mut atlas, &who(2, &others, false));
    assert!(!slim);
    assert_ne!(rect, other);
    assert_eq!(at(&atlas, x, y), b's');
}

#[test]
fn players_with_one_skin_share_a_place() {
    let mut atlas = textures();
    let mut skins = Skins::with_loader(&atlas, PathBuf::new(), loader);
    let properties = [property("same")];
    let first = settle(&mut skins, &mut atlas, &who(1, &properties, false));
    let second = settle(&mut skins, &mut atlas, &who(2, &properties, false));
    assert_eq!(first.0, second.0);
}

#[test]
fn another_player_needs_a_signed_property_but_the_local_player_does_not() {
    let mut atlas = textures();
    let mut skins = Skins::with_loader(&atlas, PathBuf::new(), loader);
    let properties = [property("unsigned")];
    let own = settle(&mut skins, &mut atlas, &who(1, &properties, false));
    assert!(skins.rects.contains(&own.0));
    // Loaded, but not shown to another player.
    for _ in 0..50 {
        skins.begin_frame();
        std::thread::sleep(Duration::from_millis(2));
    }
    let shown = skins
        .resolve(&mut atlas, &who(3, &properties, true))
        .unwrap();
    let default = photon_render::entity::player_renderer::default_skin(Uuid::new(0, 3));
    assert_eq!(shown.0, atlas.rect(default.texture).unwrap());
}

#[test]
fn a_skin_that_fails_leaves_the_default_skin() {
    let mut atlas = textures();
    let mut skins = Skins::with_loader(&atlas, PathBuf::new(), loader);
    let properties = [property("broken")];
    let default = photon_render::entity::player_renderer::default_skin(Uuid::new(0, 4));
    for _ in 0..50 {
        skins.begin_frame();
        let found = skins
            .resolve(&mut atlas, &who(4, &properties, false))
            .unwrap();
        assert_eq!(found.0, atlas.rect(default.texture).unwrap());
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(skins.take_dirty().is_empty());
}
