//! The sheep's part of a click on the client: `Sheep.mobInteract` and `DyeItem.interactLivingEntity`.

use super::*;

impl Context<'_> {
    /// `Sheep.mobInteract` on the client: shears only consume the click (the shearing is the
    /// server's); anything else is `Animal.mobInteract` with the sheep food.
    pub(super) fn interact_sheep(&mut self, id: EntityId, hand: u8) -> EntityUse {
        let shears = self
            .held(hand)
            .is_some_and(|(item, _)| crate::item::id_of("minecraft:shears") == Some(item));
        if shears {
            EntityUse::Success { swing: false }
        } else {
            self.feed_animal(id, hand, "minecraft:sheep_food")
        }
    }

    /// `DyeItem.interactLivingEntity` on the client: a dye of another colour on a sheep that
    /// still has its wool plays the dye sound and swings; the server dyes it and takes the item.
    pub(super) fn dye_on_sheep(&mut self, id: EntityId, hand: u8) -> bool {
        let Some(entity) = self.world.entities().at(id) else {
            return false;
        };
        let is_sheep = self.world.entity_type_name(id) == Some("minecraft:sheep");
        let alive = entity.health().is_none_or(|health| health > 0.0);
        if !(is_sheep && alive && !entity.is_sheared()) {
            return false;
        }
        let (current, at) = (entity.sheep_dye_id(), entity.position());
        let Some(dye) = self.held_dye(hand) else {
            return false;
        };
        if dye == current {
            return false;
        }
        self.world
            .push_sound(photon_world::local_player::LocalSound::at(
                "minecraft:item.dye.use",
                photon_world::local_player::LocalSoundSource::Players,
                (1.0, 1.0),
                (at.0, at.1, at.2),
            ));
        true
    }

    /// `DyeColor.getId` of the held dye: the stack has the `dye` component, and the vanilla
    /// items carrying it are the sixteen `<colour>_dye`.
    pub(super) fn held_dye(&self, hand: u8) -> Option<u8> {
        let stack = self
            .world
            .local_player()
            .and_then(|state| state.inventory_slot(self.hand_slot(hand)))?;
        let item = stack.item_registry_id();
        if stack.item_count() <= 0
            || !crate::item::values::Values::new(item, stack.patch_bytes())
                .has(photon_data::item::component_types::DYE)
        {
            return None;
        }
        let name = crate::item::definition(item)?.name;
        let colour = name.strip_prefix("minecraft:")?.strip_suffix("_dye")?;
        let id = crate::item::default_components::DYE_NAMES
            .iter()
            .position(|dye| *dye == colour)?;
        u8::try_from(id).ok()
    }
}
