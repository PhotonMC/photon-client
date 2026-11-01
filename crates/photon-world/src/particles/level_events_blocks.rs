//! The level events that carry a block state: 2001 breaks a block, 3008 finishes brushing one
//! (`LevelEventHandler.levelEvent`).

use photon_core::binary::BlockPos;
use photon_core::identifier::Identifier;
use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;

use super::super::LevelEvent;
use super::super::local_player::{LocalSound, LocalSoundSource};
use super::engine::ParticleEngine;
use super::level_events::{LevelEventContext, local_sound};

/// `BrushableBlock.getBrushCompletedSound`: the sound a brushable block makes when it is done, the
/// one it brushes with (`kind` of sand or gravel). Another block has none.
fn brush_completed_sound(state: BlockStateId) -> Option<&'static str> {
    let block = crate::block::state_or_air(state).block().index();
    let kind = if block == photon_data::block::ids::SUSPICIOUS_SAND {
        "sand"
    } else if block == photon_data::block::ids::SUSPICIOUS_GRAVEL {
        "gravel"
    } else {
        return None;
    };
    let name = format!(
        "{}:item.brush.brushing.{kind}",
        Identifier::DEFAULT_NAMESPACE
    );
    photon_data::sound_event::id(&name).and_then(photon_data::sound_event::name)
}

impl ParticleEngine {
    /// A block broken (2001) or brushed through (3008): the sound it plays, if any, then the
    /// crumbs about the block.
    pub(super) fn block_state_event<F: FnMut() -> RandomSource>(
        &mut self,
        context: &mut LevelEventContext<'_, F>,
        (event, pos): (i32, (i32, i32, i32)),
        data: i32,
    ) -> Option<LocalSound> {
        let sound = if event == 2001 {
            LevelEvent::Level { event, pos, data }
                .break_sound()
                .inspect(|_| {
                    // `playSound`'s seed.
                    context.level_random.next_long();
                })
        } else {
            brush_completed_sound(BlockStateId(data)).map(|name| {
                local_sound(
                    name,
                    LocalSoundSource::Players,
                    (1.0, 1.0),
                    pos,
                    context.level_random,
                )
            })
        };
        let at = BlockPos::new(pos.0, pos.1, pos.2);
        self.add_destroy_block_effect(
            context.world,
            (at, BlockStateId(data)),
            &mut context.particle_random,
        );
        sound
    }
}
