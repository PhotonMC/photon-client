use super::*;
use photon_core::ids::BlockStateId;

/// The cache of the version being played, worked out on its first use: some of a state's values
/// differ by version, so each version has its own table.
pub fn states() -> &'static States {
    static STATES: [OnceLock<States>; photon_core::version::Version::ALL.len()] =
        [const { OnceLock::new() }; photon_core::version::Version::ALL.len()];
    STATES[photon_core::versions::behaviour::table_slot()].get_or_init(build)
}

/// A state's values; `None` outside the registry.
pub fn state(state: BlockStateId) -> Option<&'static StateInfo> {
    states().get(state)
}

/// A state's values, a number outside the registry reading as air
/// (`Block.stateById`).
pub fn state_or_air(state: BlockStateId) -> &'static StateInfo {
    let states = states();
    states.get(state).unwrap_or(&states.states[0])
}

pub(super) fn build() -> States {
    let mut faces = Faces::default();
    let mut facts = ShapeFacts::default();
    let mut pool = ShapePool::default();
    let work: Vec<(usize, &'static Block, u32)> = BLOCKS
        .iter()
        .enumerate()
        .flat_map(|(index, block)| {
            (0..block.state_count()).map(move |offset| (index, block, offset))
        })
        .collect();
    // The shapes each class works out do not depend on one another, so they
    // are made side by side; numbering them and everything that is worked
    // out from them follows in state order.
    let own_shapes: Vec<(VoxelShape, VoxelShape)> = work
        .par_iter()
        .map(|&(_, block, offset)| {
            (
                shape_rules::outline(block, offset),
                shape_rules::collision(block, offset),
            )
        })
        .collect();
    let states = work
        .into_iter()
        .zip(own_shapes)
        .map(|(state, shapes)| compute(state, shapes, &mut faces, &mut facts, &mut pool))
        .collect();
    let faces = faces.shapes;
    let count = faces.len();
    States {
        states,
        shapes: pool.shapes,
        faces,
        joins: PairCache::new(count, |shape, occluder| {
            Shapes::join_is_not_empty(shape, occluder, BooleanOp::OnlyFirst)
        }),
        covers: PairCache::new(count, Shapes::face_shape_occludes),
    }
}

/// `Block.isShapeFullBlock`.
pub fn is_shape_full_block(shape: &VoxelShape) -> bool {
    !Shapes::join_is_not_empty(&Shapes::block(), shape, BooleanOp::NotSame)
}

pub(super) fn horizontal_axis(block: &Block, offset: u32) -> Axis {
    match block.value(offset, &HORIZONTAL_FACING) {
        Some("east" | "west") => Axis::X,
        _ => Axis::Z,
    }
}

/// Signs are left out of the `blocks_motion_in_heightmap` tag.
fn is_sign(block: &Block) -> bool {
    matches!(
        block.class,
        BlockClass::StandingSignBlock | BlockClass::WallSignBlock
    )
}

pub(super) fn compute(
    (index, block, offset): (usize, &'static Block, u32),
    (outline, collision): (VoxelShape, VoxelShape),
    faces: &mut Faces,
    facts: &mut ShapeFacts,
    pool: &mut ShapePool,
) -> StateInfo {
    let fluid = fluid_state(block, offset);
    let outline_place = pool.intern(outline);
    let collision_place = pool.intern(collision);
    let outline = pool.shapes[usize::from(outline_place)].clone();
    let collision = pool.shapes[usize::from(collision_place)].clone();
    let occlusion = occlusion_shape(block, offset, &outline, &collision);
    let occlusion_place = pool.intern(occlusion.clone());
    let solid_render = facts.of(&occlusion).full;
    let face_numbers = facts.faces(&occlusion, faces);
    let propagates_skylight_down =
        propagates_skylight_down(block, offset, facts.of(&outline).full, fluid);
    let collision_full_block = collision_full_block(block, facts.of(&collision).full);
    let solid = legacy_solid(block, &collision);
    let (blocks_motion, in_motion_heightmap) = motion_facts(solid, index, is_sign(block));
    let support = support_shape(block, offset, &outline, &collision);
    let support_facts = facts.of(&support);
    let sturdy_faces = support_facts.full_faces;
    let fallback = blocks_motion && collision_full_block;
    let suffocating = block.suffocating.value(offset).unwrap_or(fallback);
    StateInfo {
        block: u16::try_from(index).expect("fewer than 65536 blocks"),
        offset: u16::try_from(offset).expect("fewer than 65536 states a block"),
        fluid,
        faces: face_numbers,
        solid_render,
        propagates_skylight_down,
        light_dampening: light_dampening(block, solid_render, propagates_skylight_down),
        light_emission: block.light_emission.value(offset),
        emissive_rendering: block.emissive_rendering.value(offset).unwrap_or(false),
        shade_brightness: shade_brightness(block, offset, collision_full_block),
        collision_full_block,
        use_shape_for_light_occlusion: use_shape_for_light_occlusion(block, offset),
        can_occlude: block.can_occlude,
        blocks_motion,
        in_motion_heightmap,
        solid,
        sturdy_faces,
        center_faces: support_facts.center_faces,
        rigid_faces: support_facts.rigid_faces,
        suffocating,
        view_blocking: view_blocking_rule(block)
            .value(offset)
            .unwrap_or(suffocating),
        near_plane_view_blocking: has_near_plane_rule(block),
        redstone_conductor: block
            .redstone_conductor
            .value(offset)
            .unwrap_or(collision_full_block),
        outline: outline_place,
        collision: collision_place,
        occlusion: occlusion_place,
    }
}

/// `canOcclude ? getOcclusionShape(state) : Shapes.empty()`.
pub(super) fn occlusion_shape(
    block: &Block,
    offset: u32,
    outline: &VoxelShape,
    collision: &VoxelShape,
) -> VoxelShape {
    if !block.can_occlude {
        return Shapes::empty();
    }
    for class in block.class.lineage() {
        match class {
            BlockClass::FenceBlock => {
                return shapes::fence_occlusion(|direction| {
                    let property = match direction {
                        Direction::North => &NORTH,
                        Direction::East => &EAST,
                        Direction::South => &SOUTH,
                        _ => &WEST,
                    };
                    block.flag(offset, property)
                });
            }
            BlockClass::FenceGateBlock => {
                return shapes::fence_gate_occlusion(
                    horizontal_axis(block, offset),
                    block.flag(offset, &IN_WALL),
                );
            }
            // Their `SHAPE_COLLISION`, which is also their collision shape.
            BlockClass::LecternBlock | BlockClass::SculkShriekerBlock => return collision.clone(),
            _ => {}
        }
    }
    outline.clone()
}

/// `propagatesSkylightDown`.
pub(super) fn propagates_skylight_down(
    block: &Block,
    offset: u32,
    full_outline: bool,
    fluid: FluidState,
) -> bool {
    for class in block.class.lineage() {
        match class {
            BlockClass::BambooStalkBlock
            | BlockClass::HangingMossBlock
            | BlockClass::MossyCarpetBlock
            | BlockClass::TransparentBlock
            | BlockClass::VineBlock => return true,
            BlockClass::BarrierBlock
            | BlockClass::GlowLichenBlock
            | BlockClass::LightBlock
            | BlockClass::VegetationBlock => return fluid.is_empty(),
            BlockClass::CrossCollisionBlock | BlockClass::WallBlock => {
                return !block.flag(offset, &photon_data::block::properties::WATERLOGGED);
            }
            BlockClass::LiquidBlock
            | BlockClass::PipeBlock
            | BlockClass::ShulkerBoxBlock
            | BlockClass::TintedGlassBlock => return false,
            _ => {}
        }
    }
    !full_outline && fluid.is_empty()
}

/// `getLightDampening`.
pub(super) fn light_dampening(
    block: &Block,
    solid_render: bool,
    propagates_skylight_down: bool,
) -> u8 {
    for class in block.class.lineage() {
        match class {
            BlockClass::LeavesBlock => return 1,
            BlockClass::TintedGlassBlock => return 15,
            _ => {}
        }
    }
    if solid_render {
        15
    } else {
        u8::from(!propagates_skylight_down)
    }
}

/// `getShadeBrightness`.
pub(super) fn shade_brightness(block: &Block, offset: u32, collision_full_block: bool) -> f32 {
    for class in block.class.lineage() {
        match class {
            BlockClass::BarrierBlock
            | BlockClass::LightBlock
            | BlockClass::StructureVoidBlock
            | BlockClass::TransparentBlock => return 1.0,
            BlockClass::MudBlock | BlockClass::SoulSandBlock => return 0.2,
            BlockClass::SnowLayerBlock => {
                return if block.value(offset, &LAYERS) == Some("8") {
                    0.2
                } else {
                    1.0
                };
            }
            _ => {}
        }
    }
    if collision_full_block { 0.2 } else { 1.0 }
}

/// `useShapeForLightOcclusion`.
pub(super) fn use_shape_for_light_occlusion(block: &Block, offset: u32) -> bool {
    for class in block.class.lineage() {
        match class {
            BlockClass::DaylightDetectorBlock
            | BlockClass::PathBlock
            | BlockClass::EnchantingTableBlock
            | BlockClass::EndPortalFrameBlock
            | BlockClass::FarmlandBlock
            | BlockClass::LecternBlock
            | BlockClass::SculkSensorBlock
            | BlockClass::SculkShriekerBlock
            | BlockClass::ShelfBlock
            | BlockClass::SnowLayerBlock
            | BlockClass::StairBlock
            | BlockClass::StonecutterBlock
            | BlockClass::PistonHeadBlock => return true,
            BlockClass::SlabBlock => return block.value(offset, &SLAB_TYPE) != Some("double"),
            BlockClass::PistonBaseBlock => return block.flag(offset, &EXTENDED),
            _ => {}
        }
    }
    false
}

/// `isCollisionShapeFullBlock`.
pub(super) fn collision_full_block(block: &Block, full_collision: bool) -> bool {
    for class in block.class.lineage() {
        if matches!(
            class,
            BlockClass::BambooStalkBlock | BlockClass::SpeleothemBlock
        ) {
            return false;
        }
    }
    full_collision
}

/// `BlockStateBase.calculateSolid`: the deprecated "solid" that
/// `blocksMotion` reads. A block with a dynamic shape has no cache and so
/// never counts as solid unless forced.
pub(super) fn legacy_solid(block: &Block, collision: &VoxelShape) -> bool {
    if block.force_solid_on && forced_solid_in_version(block) {
        return true;
    }
    if block.force_solid_off || block.dynamic_shape {
        return false;
    }
    collision
        .bounds()
        .is_some_and(|bounds| bounds.size() >= 0.729_166_666_666_666_6 || bounds.y_size() >= 1.0)
}

/// `getBlockSupportShape` with no block entity at the origin.
pub(super) fn support_shape(
    block: &Block,
    offset: u32,
    outline: &VoxelShape,
    collision: &VoxelShape,
) -> VoxelShape {
    for class in block.class.lineage() {
        match class {
            BlockClass::CeilingHangingSignBlock
            | BlockClass::WallHangingSignBlock
            | BlockClass::SnowLayerBlock => return outline.clone(),
            BlockClass::ChorusFlowerBlock => return shapes::chorus_flower_support(),
            BlockClass::FenceGateBlock => {
                return if block.flag(offset, &OPEN) {
                    Shapes::empty()
                } else {
                    shapes::fence_gate_support(horizontal_axis(block, offset))
                };
            }
            BlockClass::LeavesBlock => return Shapes::empty(),
            BlockClass::MudBlock | BlockClass::SoulSandBlock | BlockClass::ShulkerBoxBlock => {
                return Shapes::block();
            }
            _ => {}
        }
    }
    collision.clone()
}

/// The block's view-blocking rule in the active version: 26.3's near plane rule replaced what
/// 26.2 had.
fn view_blocking_rule(
    block: &photon_data::block::definition::Block,
) -> photon_data::block::definition::Rule {
    if photon_core::versions::rules()
        .world
        .blocks_view_at_near_plane
    {
        block.view_blocking
    } else if block.view_blocking == photon_data::block::definition::Rule::NearPlane {
        block.view_blocking_before_26_3
    } else {
        block.view_blocking
    }
}

/// Whether the view-blocking rule is the near plane one, which only 26.3 has.
fn has_near_plane_rule(block: &photon_data::block::definition::Block) -> bool {
    photon_core::versions::rules()
        .world
        .blocks_view_at_near_plane
        && block.view_blocking == photon_data::block::definition::Rule::NearPlane
}

/// `blocksMotion` and whether the motion blocking heightmap counts the state. 26.2: cobweb and
/// bamboo sapling do not block motion, and the heightmap counts what does. 26.3: every solid
/// state blocks motion, and the heightmap leaves out signs and those two.
fn motion_facts(solid: bool, index: usize, sign: bool) -> (bool, bool) {
    let rules = photon_core::versions::rules();
    let web = index == id::COBWEB || index == id::BAMBOO_SAPLING;
    let blocks_motion = solid && (rules.world.blocks_motion_is_solidity || !web);
    let listed_out = rules.world.motion_heightmap_has_own_list && (sign || web);
    (blocks_motion, blocks_motion && !listed_out)
}

/// Whether the block's forced solidity holds in the active version: the resin brick wall's is
/// 26.3's.
fn forced_solid_in_version(block: &Block) -> bool {
    photon_core::versions::rules()
        .world
        .resin_brick_wall_is_forced_solid
        || !block.name.ends_with(":resin_brick_wall")
}
