//! What a game brings up for its sound: the engine, the music and the ambience.

use std::path::Path;

use photon_game::options::Options;
use photon_net::connection::PlaySession;

use crate::music::{Carried, GameMusic};

/// The engine and music manager the menus handed over (or new ones), with the music of the
/// level's dimension and biomes and its ambient sounds.
pub(super) fn load(
    carried: Option<Carried>,
    session: &PlaySession,
    (data_root, root): (&Path, &Path),
    options: &Options,
) -> (
    crate::sound::SoundSlot,
    GameMusic,
    crate::ambient::GameAmbient,
) {
    let (mut sound, manager) = Carried::split(carried);
    // `Minecraft.setLevel`: a level starts in silence.
    if let Some(system) = sound.as_mut().and_then(|slot| slot.as_mut()) {
        system.reset_level();
    }
    let dimension = session.dimension_type().unwrap_or("minecraft:overworld");
    let biomes = session.registry_names("minecraft:worldgen/biome");
    (
        sound.unwrap_or_else(|| crate::sound::SoundSlot::open(root, options)),
        GameMusic::load(data_root, dimension, biomes, manager),
        crate::ambient::GameAmbient::load(data_root, dimension, biomes),
    )
}
