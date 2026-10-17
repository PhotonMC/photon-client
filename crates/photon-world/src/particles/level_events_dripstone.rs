//! Level event 1504: the drip a pointed dripstone's tip lets go of
//! (`PointedDripstoneBlock.spawnDripParticle`).

use photon_core::ids::BlockStateId;
use photon_core::random::RandomSource;
use photon_data::block::blocks::id::MUD;
use photon_data::block::classes::BlockClass;
use photon_data::block::definition::Fluid;

use super::engine::ParticleEngine;
use super::level_events::LevelEventContext;
use crate::ClientWorld;

/// How far the root of a stalactite is searched for (`MAX_SEARCH_LENGTH_WHEN_CHECKING_DRIP_TYPE`).
const MAX_SEARCH_LENGTH: i32 = 11;

/// What the dimension says about dripstone: `WATER_EVAPORATES` and `DEFAULT_DRIPSTONE_PARTICLE`
/// (lava rather than water). The client does not read a dimension type's attributes yet, so they
/// are the ones the vanilla dimensions set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DripstoneAttributes {
    pub water_evaporates: bool,
    pub lava_by_default: bool,
}

impl DripstoneAttributes {
    /// The attributes of a dimension type by its registry name.
    pub fn of_dimension_type(name: &str) -> Self {
        let nether = name == "minecraft:the_nether";
        Self {
            water_evaporates: nether,
            lava_by_default: nether,
        }
    }
}

/// A block state's property value by name; `None` when the block has no such property.
fn property_of(state: &crate::block::StateInfo, name: &str) -> Option<&'static str> {
    let block = state.block();
    let property = block.properties.iter().find(|own| own.name == name)?;
    block.value(u32::from(state.offset), property)
}

/// `isSpeleothemWithDirection` for the tag `speleothems` (the pointed dripstone and the sulfur
/// spike): a speleothem whose tip points `direction`.
fn is_speleothem_pointing(state: &crate::block::StateInfo, direction: &str) -> bool {
    state.block().class.is(BlockClass::SpeleothemBlock)
        && property_of(state, "vertical_direction") == Some(direction)
}

/// `PointedDripstoneBlock.getFluidAboveStalactite` as the kind of fluid it finds: `None` where
/// the block is no stalactite or has no root within reach.
fn fluid_above_stalactite(
    world: &ClientWorld,
    pos: (i32, i32, i32),
    attributes: DripstoneAttributes,
) -> Option<Fluid> {
    let at = |x, y, z| crate::block::state_or_air(world.block_state_or_air(x, y, z));
    let tip = at(pos.0, pos.1, pos.2);
    if !is_speleothem_pointing(tip, "down") {
        return None;
    }
    // `findRootBlock`: up the column to its first block that is another block.
    let bounds = world.dimension();
    let mut root = pos.1;
    let mut found = false;
    for _ in 1..MAX_SEARCH_LENGTH {
        root += 1;
        let state = at(pos.0, root, pos.2);
        if state.block != tip.block {
            found = true;
            break;
        }
        let outside = root < bounds.min_y || root >= bounds.min_y + bounds.height;
        if outside || property_of(state, "vertical_direction") != Some("down") {
            break;
        }
    }
    if !found {
        return None;
    }
    let above = at(pos.0, root + 1, pos.2);
    if usize::from(above.block) == MUD && !attributes.water_evaporates {
        Some(Fluid::Water)
    } else {
        Some(above.fluid.fluid)
    }
}

/// `canFillCauldron`: still water or still lava.
fn can_fill_cauldron(fluid: Fluid) -> bool {
    matches!(fluid, Fluid::Lava | Fluid::Water)
}

/// `PointedDripstoneBlock.isFreeHangingStalactite`: the tip of a stalactite not under water.
fn is_free_hanging_stalactite(state: &crate::block::StateInfo) -> bool {
    is_speleothem_pointing(state, "down")
        && property_of(state, "thickness") == Some("tip")
        && property_of(state, "waterlogged") != Some("true")
}

impl ParticleEngine {
    /// `PointedDripstoneBlock.spawnDripParticle`: the drip of the fluid above the stalactite,
    /// or the dimension's own where there is none.
    pub(super) fn dripstone_drip<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        context: &mut LevelEventContext<'_, F>,
    ) {
        if let Some(fluid) = fluid_above_stalactite(context.world, pos, context.dripstone) {
            self.spawn_dripstone_drip(pos, fluid, context);
        }
    }

    /// `PointedDripstoneBlock.animateTick`: now and then a drip falls from the tip of a stalactite
    /// that hangs free; of fluid that could fill a cauldron it falls more often.
    pub(super) fn dripstone_animate_tick<F: FnMut() -> RandomSource>(
        &mut self,
        state: BlockStateId,
        pos: (i32, i32, i32),
        random: &mut RandomSource,
        context: &mut LevelEventContext<'_, F>,
    ) {
        if !is_free_hanging_stalactite(crate::block::state_or_air(state)) {
            return;
        }
        let value = random.next_float();
        if value > 0.12 {
            return;
        }
        let Some(fluid) = fluid_above_stalactite(context.world, pos, context.dripstone) else {
            return;
        };
        if value < 0.02 || can_fill_cauldron(fluid) {
            self.spawn_dripstone_drip(pos, fluid, context);
        }
    }

    /// The private `spawnDripParticle`: a drop of the fluid's kind at the tip.
    fn spawn_dripstone_drip<F: FnMut() -> RandomSource>(
        &mut self,
        pos: (i32, i32, i32),
        fluid: Fluid,
        context: &mut LevelEventContext<'_, F>,
    ) {
        let attributes = context.dripstone;
        let lava = match fluid {
            Fluid::Empty => attributes.lava_by_default,
            other => matches!(other, Fluid::Lava | Fluid::FlowingLava),
        };
        let name = if lava {
            "dripping_dripstone_lava"
        } else {
            "dripping_dripstone_water"
        };
        let state =
            crate::block::state_or_air(context.world.block_state_or_air(pos.0, pos.1, pos.2));
        let offset = crate::block::offset(state.block(), [pos.0, pos.1, pos.2]);
        // The tip's shape starts five sixteenths up the block; the drop hangs a sixteenth below.
        let at = (
            f64::from(pos.0) + 0.5 + offset[0],
            f64::from(pos.1) + 5.0 / 16.0 - 0.0625,
            f64::from(pos.2) + 0.5 + offset[2],
        );
        self.level_add(context, name, at, (0.0, 0.0, 0.0));
    }
}
