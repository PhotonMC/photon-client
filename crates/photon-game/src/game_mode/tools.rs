//! The client's part of `Item.useOn` for the tools that change a block: the server changes it,
//! the client only decides whether the tool applies and plays the sound the server leaves out
//! for the one using it.

use super::context::Context;
use super::mining::{block, is_air};
use super::{Aim, UseResult};
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `HoeItem.TILLABLES`: whether the hoe works on the block (by name), which of them need the
/// face not to be the bottom one and air above.
fn tillable(name: &str) -> Option<bool> {
    match name {
        "minecraft:grass_block"
        | "minecraft:dirt_path"
        | "minecraft:dirt"
        | "minecraft:coarse_dirt" => Some(true),
        "minecraft:rooted_dirt" => Some(false),
        _ => None,
    }
}

/// `HoeItem.useOn` for the client: a tillable block with its predicate met sounds the hoe and
/// succeeds; any other passes.
pub(super) fn hoe_use_on(ctx: &mut Context<'_>, aim: Aim) -> UseResult {
    let Aim::Block { x, y, z, face, .. } = aim else {
        return UseResult::Pass;
    };
    let Some(needs_air) = tillable(block(ctx.state_at((x, y, z))).name) else {
        return UseResult::Pass;
    };
    // `HoeItem.onlyIfAirAbove`: the face is not down, and the block above is air.
    if needs_air && (face == super::mining::DIRECTION_DOWN || !is_air(ctx.state_at((x, y + 1, z))))
    {
        return UseResult::Pass;
    }
    // `Level.playSound(player, pos, ...)`: at the middle of the block, loud and at normal pitch.
    ctx.world.push_sound(LocalSound::at(
        "minecraft:item.hoe.till",
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5),
    ));
    UseResult::Success { swing: true }
}

#[cfg(test)]
mod tests {
    use super::tillable;

    #[test]
    fn the_hoe_tills_five_blocks() {
        for name in [
            "minecraft:grass_block",
            "minecraft:dirt_path",
            "minecraft:dirt",
            "minecraft:coarse_dirt",
        ] {
            assert_eq!(tillable(name), Some(true), "{name}");
        }
        assert_eq!(tillable("minecraft:rooted_dirt"), Some(false));
        assert_eq!(tillable("minecraft:stone"), None);
    }
}
