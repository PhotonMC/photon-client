//! Links the imports of `photon_api::IMPORTS` for the capabilities a mod was granted. The table
//! decides which functions exist and which capability each needs; this file only supplies the
//! closure behind each name, and a name the table has without one is an error at load.

use photon_api::{FS_MOD_DATA, GAME_TIME, HUD_TEXT, IMPORTS, Import, PLAYER_STATE, SOUND_PLAY};
use wasmtime::StoreContextMut;
use wasmtime::component::Linker;

use crate::data::DataError;
use crate::host::{Caps, Effect, MAX_TEXT, ModState};

type Cx<'a> = StoreContextMut<'a, ModState>;

/// `text` cut at a character boundary to at most [`MAX_TEXT`] bytes.
fn clipped(mut text: String) -> String {
    if text.len() > MAX_TEXT {
        let mut end = MAX_TEXT;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    text
}

impl Caps {
    /// Whether the capability named `capability` in `photon_api` is granted.
    pub fn grants(&self, capability: &str) -> bool {
        match capability {
            photon_api::EVENTS_TICK => self.tick,
            HUD_TEXT => self.hud_text,
            SOUND_PLAY => self.sound,
            GAME_TIME => self.game_time,
            PLAYER_STATE => self.player,
            FS_MOD_DATA => self.mod_data,
            _ => false,
        }
    }
}

/// Defines every import of the table whose capability `caps` grants. A capability that is not
/// granted is not defined, so a mod that imports it fails to load.
pub(crate) fn link(linker: &mut Linker<ModState>, caps: Caps) -> wasmtime::Result<()> {
    for import in IMPORTS.iter().filter(|i| caps.grants(i.capability)) {
        bind(linker, import)?;
    }
    Ok(())
}

/// Defines one import with its closure.
pub(crate) fn bind(linker: &mut Linker<ModState>, import: &Import) -> wasmtime::Result<()> {
    match import.capability {
        HUD_TEXT => bind_hud(linker, import.name),
        SOUND_PLAY => bind_sound(linker, import.name),
        GAME_TIME => bind_time(linker, import.name),
        PLAYER_STATE => bind_player(linker, import.name),
        FS_MOD_DATA => bind_data(linker, import.name),
        other => Err(wasmtime::format_err!("no imports for capability `{other}`")),
    }
}

fn missing(name: &str) -> wasmtime::Error {
    wasmtime::format_err!("the table declares `{name}` but the host has no function for it")
}

fn bind_hud(linker: &mut Linker<ModState>, name: &str) -> wasmtime::Result<()> {
    match name {
        "hud-text" => linker.root().func_wrap(
            name,
            |mut cx: Cx<'_>, (x, y, colour, text): (i32, i32, u32, String)| {
                let text = clipped(text);
                cx.data_mut().push(Effect::HudText { x, y, colour, text });
                Ok(())
            },
        ),
        _ => Err(missing(name)),
    }
}

fn bind_sound(linker: &mut Linker<ModState>, name: &str) -> wasmtime::Result<()> {
    match name {
        "play-sound" => linker.root().func_wrap(
            name,
            |mut cx: Cx<'_>, (name, volume, pitch): (String, f32, f32)| {
                let name = clipped(name);
                let (volume, pitch) = (volume.clamp(0.0, 1.0), pitch.clamp(0.5, 2.0));
                cx.data_mut().push(Effect::Sound {
                    name,
                    volume,
                    pitch,
                });
                Ok(())
            },
        ),
        _ => Err(missing(name)),
    }
}

fn bind_time(linker: &mut Linker<ModState>, name: &str) -> wasmtime::Result<()> {
    let mut root = linker.root();
    match name {
        "game-time" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().world.game_time,))),
        "day-time" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().world.day_time,))),
        _ => Err(missing(name)),
    }
}

/// The reads of `player.state`: each returns the value the tick started with.
fn bind_player(linker: &mut Linker<ModState>, name: &str) -> wasmtime::Result<()> {
    let mut root = linker.root();
    match name {
        "player-x" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.x,))),
        "player-y" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.y,))),
        "player-z" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.z,))),
        "player-health" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.health,))),
        "player-yaw" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.yaw,))),
        "player-pitch" => root.func_wrap(name, |cx: Cx<'_>, ()| Ok((cx.data().player.pitch,))),
        _ => Err(missing(name)),
    }
}

/// `data-get`, `data-set` and `data-delete` of `fs.mod_data`. A key that is not valid reads
/// nothing; `data-set` answers 0 or a [`DataError`] code.
fn bind_data(linker: &mut Linker<ModState>, name: &str) -> wasmtime::Result<()> {
    let mut root = linker.root();
    match name {
        "data-get" => root.func_wrap(name, |cx: Cx<'_>, (key,): (String,)| {
            Ok((cx.data().data.as_ref().and_then(|data| data.get(&key)),))
        }),
        "data-set" => root.func_wrap(name, |cx: Cx<'_>, (key, value): (String, Vec<u8>)| {
            let code = match &cx.data().data {
                Some(data) => data.set(&key, &value).err().map_or(0, |e| e as u32),
                None => DataError::Unavailable as u32,
            };
            Ok((code,))
        }),
        "data-delete" => root.func_wrap(name, |cx: Cx<'_>, (key,): (String,)| {
            let deleted = cx
                .data()
                .data
                .as_ref()
                .is_some_and(|data| data.delete(&key));
            Ok((deleted,))
        }),
        _ => Err(missing(name)),
    }
}

#[cfg(test)]
mod tests;
