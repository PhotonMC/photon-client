use super::*;
use photon_core::ids::BlockStateId;

/// What a block state holds for movement, looked up once.
#[derive(Clone, Copy)]
pub(crate) struct StateInfo {
    pub(crate) name: &'static str,
    pub(crate) friction: f32,
    pub(crate) speed_factor: f32,
    pub(crate) jump_factor: f32,
    pub(crate) bounce: f32,
    pub(crate) flags: u16,
    pub(crate) fluid: Fluid,
    pub(super) own_height: f32,
    pub(super) falling: bool,
}

/// A living body as the blocks it is inside act on it.
pub(crate) struct LivingEffects<'a> {
    pub(crate) delta: &'a mut Vec3,
    pub(crate) fall_distance: &'a mut f64,
    pub(crate) stuck: &'a mut Vec3,
    pub(crate) position: Vec3,
    pub(crate) on_ground: bool,
    pub(crate) width: f32,
    /// The block the body's feet are in.
    pub(crate) block_pos: BlockPos,
}

impl StateInfo {
    /// A state against the server's tags; a number outside the registry
    /// reads as air.
    pub(super) fn of(state_id: Option<BlockStateId>, tags: &crate::block::tags::BlockTags) -> Self {
        let state = crate::block::state_or_air(
            state_id.unwrap_or(crate::constants::VANILLA_AIR_BLOCK_STATE_ID),
        );
        let block = state.block();
        let index = usize::from(state.block);
        let tagged = [
            ("minecraft:climbable", flag::CLIMBABLE),
            ("minecraft:fences", flag::FENCE),
            ("minecraft:walls", flag::WALL),
            ("minecraft:can_glide_through", flag::CAN_GLIDE_THROUGH),
            ("minecraft:suppresses_bounce", flag::SUPPRESSES_BOUNCE),
        ];
        let classes = [
            (BlockClass::FenceGateBlock, flag::FENCE_GATE),
            (BlockClass::TrapDoorBlock, flag::TRAPDOOR),
            (BlockClass::LadderBlock, flag::LADDER),
        ];
        let flags = tagged
            .iter()
            .filter(|(tag, _)| tags.contains(index, tag))
            .map(|(_, bit)| bit)
            .chain(
                classes
                    .iter()
                    .filter(|(class, _)| block.class.is(*class))
                    .map(|(_, bit)| bit),
            )
            .fold(0, |flags, bit| flags | bit);
        Self {
            name: block.name,
            friction: block.friction,
            speed_factor: block.speed_factor,
            jump_factor: block.jump_factor,
            bounce: block.bounce_restitution,
            flags,
            fluid: state.fluid.fluid,
            own_height: state.fluid.own_height(),
            falling: state.fluid.falling,
        }
    }

    pub(super) fn is_air(&self) -> bool {
        matches!(
            self.name,
            "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
        )
    }

    pub(crate) fn is_water(&self) -> bool {
        matches!(self.fluid, Fluid::Water | Fluid::FlowingWater)
    }

    pub(crate) fn is_lava(&self) -> bool {
        matches!(self.fluid, Fluid::Lava | Fluid::FlowingLava)
    }
}
