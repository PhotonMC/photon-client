//! The durability and size of a stack, as the item models read them.

use photon_core::wire::{read_varint, take};
use photon_game::menus::container::{
    bundle_fullness, damage, is_damaged, max_damage, max_stack_size,
};
use photon_render::item_model::{CustomModelData, StackNumbers};
use photon_world::ClientItemStackState;

/// What the `damaged`, `damage` and `count` item model properties read of a stack.
pub fn stack_numbers(stack: &ClientItemStackState) -> StackNumbers {
    StackNumbers {
        damaged: is_damaged(stack),
        damage: damage(stack),
        max_damage: max_damage(stack),
        count: stack.item_count(),
        max_count: max_stack_size(stack),
        fullness: bundle_fullness(stack).to_bits(),
    }
}

/// A stack's `custom_model_data` (floats, flags, strings, colours: each a counted list), if it
/// carries one and the value reads through.
pub fn stack_custom_model_data(stack: &ClientItemStackState) -> Option<Box<CustomModelData>> {
    let values =
        photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    let mut input = values.raw(photon_data::item::component_types::CUSTOM_MODEL_DATA)?;
    let floats = read_list(&mut input, |input| take::<4>(input).map(u32::from_be_bytes))?;
    let flags = read_list(&mut input, |input| take::<1>(input).map(|[byte]| byte != 0))?;
    let strings = read_list(&mut input, read_string)?;
    let colors = read_list(&mut input, |input| take::<4>(input).map(i32::from_be_bytes))?;
    Some(Box::new(CustomModelData {
        floats,
        flags,
        strings,
        colors,
    }))
}

/// A varint count and that many entries.
fn read_list<T>(input: &mut &[u8], read: impl Fn(&mut &[u8]) -> Option<T>) -> Option<Vec<T>> {
    let count = usize::try_from(read_varint(input).ok()?).ok()?;
    (0..count).map(|_| read(input)).collect()
}

/// A varint byte length and that many bytes of text.
fn read_string(input: &mut &[u8]) -> Option<String> {
    let length = usize::try_from(read_varint(input).ok()?).ok()?;
    let (text, rest) = input.split_at_checked(length)?;
    *input = rest;
    Some(String::from_utf8_lossy(text).into_owned())
}
