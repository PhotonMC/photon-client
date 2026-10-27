//! What `InstrumentItem` reads of an instrument: the sound, how long it is used and how far it
//! carries, from the data pack by the registry id the server synchronised.

use std::borrow::Cow;

use crate::registries::Registries;
use photon_core::wire::read_varint;
use photon_world::ClientItemStackState;

/// One instrument of the registry.
#[derive(Debug, Clone, PartialEq)]
pub struct Instrument {
    pub name: String,
    /// The sound event, as the sound table names it.
    pub sound: &'static str,
    /// `Mth.floor(useDuration * 20)`.
    pub use_ticks: i32,
    /// `range / 16`: the volume of the sound.
    pub volume: f32,
}

/// The instruments by registry id; one the data pack does not describe is `None`.
pub fn instruments_of(registries: &Registries) -> Vec<Option<Instrument>> {
    registries
        .instruments
        .iter()
        .map(|entry| {
            let sound = photon_data::sound_event::name(photon_data::sound_event::id(
                entry.sound_event.as_deref()?,
            )?)?;
            Some(Instrument {
                name: entry.name.clone(),
                sound,
                use_ticks: (entry.use_duration? * 20.0).floor() as i32,
                volume: entry.range? / 16.0,
            })
        })
        .collect()
}

/// `InstrumentItem.getInstrument`: the stack's `instrument` component (a registry entry, or one
/// written out in full), else the item's own (the goat horn's is the ponder call).
pub fn of_stack<'a>(
    stack: &ClientItemStackState,
    item_name: &str,
    instruments: &'a [Option<Instrument>],
) -> Option<Cow<'a, Instrument>> {
    let id = photon_data::item::component_types::INSTRUMENT;
    match stack.patched(id) {
        Some(None) => None,
        Some(Some(_)) => {
            let values =
                crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
            let mut data = values.raw(id)?;
            match read_varint(&mut data).ok()? {
                0 => direct(&mut data).map(Cow::Owned),
                holder => instruments
                    .get(usize::try_from(holder - 1).ok()?)?
                    .as_ref()
                    .map(Cow::Borrowed),
            }
        }
        None if item_name == "minecraft:goat_horn" => instruments
            .iter()
            .flatten()
            .find(|instrument| instrument.name == "minecraft:ponder_goat_horn")
            .map(Cow::Borrowed),
        None => None,
    }
}

/// `Instrument.DIRECT_STREAM_CODEC`: the sound, the use duration and the range (the description
/// is not read).
fn direct(data: &mut &[u8]) -> Option<Instrument> {
    let sound = match read_varint(data).ok()? {
        0 => {
            let name = read_text(data)?;
            // `SoundEvent.DIRECT_STREAM_CODEC`'s fixed range, which the volume does not read.
            if take(data, 1)?[0] != 0 {
                take(data, 4)?;
            }
            sound_name(&name)
        }
        holder => photon_data::sound_event::name(holder - 1)?,
    };
    let use_duration = read_float(data)?;
    let range = read_float(data)?;
    Some(Instrument {
        name: String::new(),
        sound,
        use_ticks: (use_duration * 20.0).floor() as i32,
        volume: range / 16.0,
    })
}

fn take<'a>(data: &mut &'a [u8], count: usize) -> Option<&'a [u8]> {
    let (taken, rest) = (data.get(..count)?, data.get(count..)?);
    *data = rest;
    Some(taken)
}

fn read_float(data: &mut &[u8]) -> Option<f32> {
    Some(f32::from_be_bytes(take(data, 4)?.try_into().ok()?))
}

fn read_text(data: &mut &[u8]) -> Option<String> {
    let length = usize::try_from(read_varint(data).ok()?).ok()?;
    String::from_utf8(take(data, length)?.to_vec()).ok()
}

/// A sound event's name as the sound table holds it; a name the table lacks (a pack's own) is
/// kept once for the rest of the run.
fn sound_name(name: &str) -> &'static str {
    let name = if name.contains(':') {
        name.to_owned()
    } else {
        format!("minecraft:{name}")
    };
    if let Some(known) =
        photon_data::sound_event::id(&name).and_then(photon_data::sound_event::name)
    {
        return known;
    }
    static NAMES: std::sync::Mutex<std::collections::BTreeSet<&'static str>> =
        std::sync::Mutex::new(std::collections::BTreeSet::new());
    let mut names = NAMES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(kept) = names.get(name.as_str()) {
        return kept;
    }
    let kept: &'static str = Box::leak(name.into_boxed_str());
    names.insert(kept);
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_data::item::component::Component;
    use photon_data::item::creative::CreativeStack;

    /// The creative tab's horns are the stacks the oracle compares byte for byte with the
    /// reference's, so the patch bytes here are the reference's wire format.
    #[test]
    fn a_horn_names_its_instrument_by_its_patch_or_else_the_ponder_call() {
        let registries = crate::registries::for_tests();
        let instruments = instruments_of(&registries);
        if instruments.is_empty() {
            // No data pack next to PHOTON_ASSETS: nothing to resolve against.
            return;
        }
        let name = |stack: &ClientItemStackState| {
            of_stack(stack, "minecraft:goat_horn", &instruments).map(|found| found.name.clone())
        };
        for call in ["sing", "seek", "feel", "admire", "call", "yearn", "dream"] {
            let id = format!("minecraft:{call}_goat_horn");
            let named: &'static str = Box::leak(id.clone().into_boxed_str());
            let components: &'static [Component] =
                Box::leak(Box::new([Component::Instrument(named)]));
            let entry = CreativeStack::of("minecraft:goat_horn").with(components);
            let stack = crate::menus::creative::table_stack(&entry, &registries);
            assert_eq!(name(&stack), Some(id.clone()));
        }
        let plain = crate::menus::creative::table_stack(
            &CreativeStack::of("minecraft:goat_horn"),
            &registries,
        );
        assert_eq!(name(&plain).as_deref(), Some("minecraft:ponder_goat_horn"));
    }
}
