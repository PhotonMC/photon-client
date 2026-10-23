//! What `InstrumentItem` reads of an instrument: the sound, how long it is used and how far it
//! carries, from the data pack by the registry id the server synchronised.

use crate::registries::Registries;
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

/// `InstrumentItem.getInstrument`: the stack's `instrument` component, else the item's own (the
/// goat horn's is the ponder call). A direct instrument in the component is not read.
pub fn of_stack<'a>(
    stack: &ClientItemStackState,
    item_name: &str,
    instruments: &'a [Option<Instrument>],
) -> Option<&'a Instrument> {
    let id = photon_data::item::component_types::INSTRUMENT;
    match stack.patched(id) {
        Some(None) => None,
        Some(Some(_)) => {
            let values =
                crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
            let holder = usize::try_from(values.holder_id(id)?).ok()?;
            instruments.get(holder)?.as_ref()
        }
        None if item_name == "minecraft:goat_horn" => instruments
            .iter()
            .flatten()
            .find(|instrument| instrument.name == "minecraft:ponder_goat_horn"),
        None => None,
    }
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
            of_stack(stack, "minecraft:goat_horn", &instruments).map(|found| found.name.as_str())
        };
        for call in ["sing", "seek", "feel", "admire", "call", "yearn", "dream"] {
            let id = format!("minecraft:{call}_goat_horn");
            let named: &'static str = Box::leak(id.clone().into_boxed_str());
            let components: &'static [Component] =
                Box::leak(Box::new([Component::Instrument(named)]));
            let entry = CreativeStack::of("minecraft:goat_horn").with(components);
            let stack = crate::menus::creative::table_stack(&entry, &registries);
            assert_eq!(name(&stack), Some(id.as_str()));
        }
        let plain = crate::menus::creative::table_stack(
            &CreativeStack::of("minecraft:goat_horn"),
            &registries,
        );
        assert_eq!(name(&plain), Some("minecraft:ponder_goat_horn"));
    }
}
