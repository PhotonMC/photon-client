//! `PotentSulfurBlock.animateTick`: bubbles that rise through the water over a sulfur block that is
//! not dry, and now and again its gas's sound.

use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;
use photon_data::block::registry::Block;

use super::super::local_player::LocalSound;
use super::super::world::SulfurState;
use super::animate::ambient;
use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;

type Pos = (i32, i32, i32);

impl ParticleEngine {
    /// `PotentSulfurBlock.animateTick`; false for another block.
    pub(super) fn sulfur_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        (state, block): (BlockStateId, &'static Block),
        pos: Pos,
        random: &mut RandomSource,
        (context, sounds): (&mut LevelEventContext<'_, F>, &mut Vec<LocalSound>),
    ) -> bool {
        if block.index() != photon_data::block::ids::POTENT_SULFUR {
            return false;
        }
        let above = (pos.0, pos.1 + 1, pos.2);
        if SulfurState::of(state) == SulfurState::Dry || !context.world.is_water_source_at(above) {
            return true;
        }
        for _ in 0..2 {
            let at = (
                f64::from(pos.0) + f64::from(random.next_float()),
                f64::from(pos.1 + 1) + f64::from(random.next_float()),
                f64::from(pos.2) + f64::from(random.next_float()),
            );
            self.level_add_flags(context, "sulfur_bubbles", at, (false, true));
        }
        if random.next_int_bound(10).unwrap_or(1) == 0 {
            let at = (f64::from(pos.0), f64::from(pos.1), f64::from(pos.2));
            sounds.push(ambient(
                "minecraft:block.potent_sulfur.noxious_gas",
                (1.0, 1.0),
                at,
                context,
            ));
        }
        true
    }
}
