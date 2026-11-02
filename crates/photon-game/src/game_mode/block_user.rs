//! The user of a block by hand.

use super::{Aim, ClientPlayerGameMode, Context, MAIN_HAND};
use photon_world::block::rules;

/// Who uses a block by hand, and where the click landed.
pub(super) fn user_of(ctx: &Context<'_>, aim: Aim) -> rules::User {
    let Aim::Block {
        x,
        y,
        z,
        face,
        location,
        ..
    } = aim
    else {
        unreachable!("a block is used from an aim at a block");
    };
    rules::User {
        facing: photon_core::direction::Direction::from_y_rot(f64::from(ctx.player.rotation().0)),
        may_build: ctx.mode != ClientPlayerGameMode::Adventure,
        face: photon_core::direction::Direction::from_3d_data_value(i32::from(face)),
        click_y: location[1] - f64::from(y),
        location,
        can_eat: ctx.player.abilities().invulnerable()
            || ctx
                .world
                .local_player()
                .is_some_and(|state| state.food() < 20),
        main_hand_empty: ctx.held(MAIN_HAND).is_none(),
        block_entity: ctx.world.block_entity_at(x, y, z).is_some(),
    }
}
