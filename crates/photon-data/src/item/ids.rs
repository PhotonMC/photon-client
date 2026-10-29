//! Items the code compares by id, resolved while compiling like [`super::registry::named`]: a
//! name the registry lacks stops the build. Add one when code needs it, not the whole registry.

use photon_core::ids::ItemId;

/// `NAME = name` defines `pub const NAME` as the id of `minecraft:name`. The prefix is joined
/// here so the lint counts a name only where code spells it out.
macro_rules! items {
    ($($id:ident = $name:ident),* $(,)?) => {
        $(pub const $id: ItemId = super::registry::named(concat!("minecraft:", stringify!($name)));)*
    };
}

items! {
    BUCKET = bucket,
    DRAGON_HEAD = dragon_head,
    LAVA_BUCKET = lava_bucket,
    MILK_BUCKET = milk_bucket,
    SULFUR_CUBE_BUCKET = sulfur_cube_bucket,
    TRIDENT = trident,
}
