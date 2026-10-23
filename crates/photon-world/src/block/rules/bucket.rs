//! What a bucket does to the block it meets: `canBeReplaced(fluid)`, `LiquidBlockContainer`
//! and `BucketPickup`.

use super::*;
use photon_data::block::classes::BlockInterface;

fn air() -> BlockStateId {
    registry::default_state_named("minecraft:air")
}

/// `BlockBehaviour.canBeReplaced(state, fluid)`: a replaceable block, or one that is not solid;
/// the end portal and the end gateway refuse.
pub fn replaceable_by_fluid(state: BlockStateId) -> bool {
    let info = crate::block::state_or_air(state);
    let block = info.block();
    if block.class.is(BlockClass::EndPortalBlock) || block.class.is(BlockClass::EndGatewayBlock) {
        return false;
    }
    block.replaceable || !info.solid
}

/// `LiquidBlockContainer.canPlaceLiquid(.., WATER)`; `None` for a block that is no container.
/// `creative` is `Player.isCreative`, which a barrier asks.
pub fn water_container(state: BlockStateId, creative: bool) -> Option<bool> {
    let class = block(state)?.class;
    if !class.implements(BlockInterface::LiquidBlockContainer)
        && !class.implements(BlockInterface::SimpleWaterloggedBlock)
    {
        return None;
    }
    let plants = [
        BlockClass::KelpBlock,
        BlockClass::KelpPlantBlock,
        BlockClass::SeagrassBlock,
        BlockClass::TallSeagrassBlock,
    ];
    Some(if plants.iter().any(|plant| class.is(*plant)) {
        false
    } else if class.is(BlockClass::BarrierBlock) {
        creative
    } else if class.is(BlockClass::SlabBlock) {
        property(state, "type") != Some("double")
    } else {
        true
    })
}

/// What `placeLiquid` of water does on the client, as the state it sets and the flags: the
/// default waterlogging is the server's alone, but a candle and a campfire set their own state.
pub fn client_waterlogging(state: BlockStateId) -> Option<(BlockStateId, u32)> {
    let class = block(state)?.class;
    if property(state, "waterlogged") != Some("false") {
        return None;
    }
    let wet = with_property(state, "waterlogged", "true")?;
    if class.is(BlockClass::CandleBlock) {
        // A lit candle goes out with `setLit` (flags 11); an unlit one just fills.
        return Some(match property(state, "lit") {
            Some("true") => (with_property(wet, "lit", "false")?, 11),
            _ => (wet, 3),
        });
    }
    if class.is(BlockClass::CampfireBlock) {
        return Some((with_property(wet, "lit", "false")?, 3));
    }
    None
}

/// What `BucketPickup.pickupBlock` does to a block and what it hands over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pickup {
    /// The filled bucket's item.
    pub bucket: &'static str,
    /// `getPickupSound`.
    pub sound: &'static str,
    /// The state the block takes, set with `flags`.
    pub after: BlockStateId,
    pub flags: u32,
}

/// `BucketPickup.pickupBlock` of the block, `None` when it is no pickup or has nothing to give.
pub fn pickup(state: BlockStateId) -> Option<Pickup> {
    let block = block(state)?;
    let water = |after, flags| Pickup {
        bucket: "minecraft:water_bucket",
        sound: "minecraft:item.bucket.fill",
        after,
        flags,
    };
    if block.class.is(BlockClass::LiquidBlock) {
        // Only a source gives a bucket.
        if property(state, "level") != Some("0") {
            return None;
        }
        return Some(if block.name == "minecraft:lava" {
            Pickup {
                bucket: "minecraft:lava_bucket",
                sound: "minecraft:item.bucket.fill_lava",
                after: air(),
                flags: 11,
            }
        } else {
            water(air(), 11)
        });
    }
    if block.class.is(BlockClass::BubbleColumnBlock) {
        return Some(water(air(), 11));
    }
    if block.class.is(BlockClass::PowderSnowBlock) {
        return Some(Pickup {
            bucket: "minecraft:powder_snow_bucket",
            sound: "minecraft:item.bucket.fill_powder_snow",
            after: air(),
            flags: 11,
        });
    }
    if block
        .class
        .implements(BlockInterface::SimpleWaterloggedBlock)
        && property(state, "waterlogged") == Some("true")
    {
        return Some(water(with_property(state, "waterlogged", "false")?, 3));
    }
    None
}
