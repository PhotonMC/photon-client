use super::*;

pub(super) fn scaffolding() -> &'static Scaffolding {
    cached!(Scaffolding, {
        let merged = |ring: Horizontal| {
            ring.iter()
                .fold(Shapes::empty(), |shape, (_, part)| Shapes::or(&shape, part))
        };
        let legs = merged(Shapes::rotate_horizontal(&cube(
            0.0, 0.0, 0.0, 2.0, 16.0, 2.0,
        )));
        let stable = Shapes::or(&column_xz(16.0, 14.0, 16.0), &legs);
        let edges = merged(Shapes::rotate_horizontal(&box_z(16.0, 0.0, 2.0, 0.0, 2.0)));
        Scaffolding {
            unstable: Shapes::or_many(&stable, &[column_xz(16.0, 0.0, 2.0), edges]),
            bottom: column_xz(16.0, 0.0, 2.0),
            stable,
        }
    })
}

/// `ScaffoldingBlock.getCollisionShape`, for whoever asks: `feet` is the height of its feet
/// (`None` with no entity asking, which counts as above) and `descending` whether it sneaks.
pub fn scaffolding_collision(
    (distance, bottom): (i32, bool),
    feet: Option<(f64, i32)>,
    descending: bool,
) -> VoxelShape {
    // `EntityCollisionContext.isAbove`: the feet over the top of `shape`, which is `top` blocks
    // above the block's own base.
    let above =
        |top: i32| feet.is_none_or(|(y, base)| y > f64::from(base + top) - f64::from(1.0e-5_f32));
    let shapes = scaffolding();
    if above(1) && !descending {
        shapes.stable.clone()
    } else if distance != 0 && bottom && above(0) {
        shapes.bottom.clone()
    } else {
        Shapes::empty()
    }
}

pub(super) fn lectern_base() -> VoxelShape {
    Shapes::or(&column_xz(16.0, 0.0, 2.0), &column_xz(8.0, 2.0, 14.0))
}

pub(super) fn composter(level: usize) -> VoxelShape {
    let min_y = (1 + level as i32 * 2).clamp(2, 16);
    Shapes::join(
        &Shapes::block(),
        &column_xz(12.0, f64::from(min_y), 16.0),
        BooleanOp::OnlyFirst,
    )
}

pub(super) fn leaf(block: &Block, offset: u32) -> VoxelShape {
    match prop(block, offset, "tilt") {
        Some("full") => Shapes::empty(),
        Some("partial") => cached!(VoxelShape, column_xz(16.0, 11.0, 13.0)).clone(),
        _ => cached!(VoxelShape, column_xz(16.0, 11.0, 15.0)).clone(),
    }
}

pub(super) fn bell(block: &Block, offset: u32) -> VoxelShape {
    struct Bell {
        ceiling: VoxelShape,
        floor: [(Axis, VoxelShape); 2],
        double_wall: [(Axis, VoxelShape); 2],
        single_wall: Horizontal,
    }
    let bell = cached!(Bell, {
        let body = Shapes::or(&column_xz(6.0, 6.0, 13.0), &column_xz(8.0, 4.0, 6.0));
        Bell {
            ceiling: Shapes::or(&body, &column_xz(2.0, 13.0, 16.0)),
            floor: Shapes::rotate_horizontal_axis(&block_cube(16.0, 16.0, 8.0)),
            double_wall: Shapes::rotate_horizontal_axis(&Shapes::or(
                &body,
                &column(2.0, 16.0, 13.0, 15.0),
            )),
            single_wall: Shapes::rotate_horizontal(&Shapes::or(
                &body,
                &box_z(2.0, 13.0, 15.0, 0.0, 13.0),
            )),
        }
    });
    let facing = facing(block, offset, "facing");
    match prop(block, offset, "attachment") {
        Some("floor") => arm(&bell.floor, facing.axis()).clone(),
        Some("single_wall") => arm(&bell.single_wall, facing).clone(),
        Some("double_wall") => arm(&bell.double_wall, facing.axis()).clone(),
        _ => bell.ceiling.clone(),
    }
}

pub(super) fn mossy_carpet(block: &Block, offset: u32) -> VoxelShape {
    struct Moss {
        low: Horizontal,
        tall: All,
    }
    let moss = cached!(
        Moss,
        Moss {
            low: Shapes::rotate_horizontal(&box_z(16.0, 0.0, 10.0, 0.0, 1.0)),
            tall: Shapes::rotate_all(&box_z_xy(16.0, 0.0, 1.0)),
        }
    );
    let mut shape = if flag(block, offset, "bottom") {
        arm(&moss.tall, Direction::Down).clone()
    } else {
        Shapes::empty()
    };
    for (direction, name) in [
        (Direction::North, "north"),
        (Direction::East, "east"),
        (Direction::South, "south"),
        (Direction::West, "west"),
    ] {
        match prop(block, offset, name) {
            Some("low") => shape = Shapes::or(&shape, arm(&moss.low, direction)),
            Some("tall") => shape = Shapes::or(&shape, arm(&moss.tall, direction)),
            _ => {}
        }
    }
    if shape.is_empty() {
        Shapes::block()
    } else {
        shape
    }
}

/// A floor head's column: the piglin's is wider.
pub(super) fn skull(block: &Block, height: f64) -> VoxelShape {
    let width = if block.name.contains("piglin") {
        10.0
    } else {
        8.0
    };
    column_xz(width, 0.0, height)
}

/// `getCollisionShape` of a class that gives a fixed one; `None` for a
/// class this does not cover.
pub fn collision(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
        C::SkullBlock => skull(block, 8.0),
        C::CactusBlock | C::HoneyBlock => cached!(VoxelShape, column_xz(14.0, 0.0, 15.0)).clone(),
        C::SoulSandBlock | C::MudBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 14.0)).clone(),
        C::LecternBlock => lectern_base(),
        C::SculkShriekerBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone(),
        C::SnowLayerBlock => column_xz(
            16.0,
            0.0,
            (number(block, offset, "layers") as f64 - 1.0) * 2.0,
        ),
        C::LiquidBlock | C::MovingPistonBlock | C::PowderSnowBlock => Shapes::empty(),
        C::ComposterBlock => composter(0),
        C::BambooStalkBlock => cached!(VoxelShape, column_xz(3.0, 0.0, 16.0)).clone(),
        C::WallHangingSignBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 2],
                Shapes::rotate_horizontal_axis(&column(16.0, 4.0, 14.0, 16.0))
            );
            by_horizontal_axis(block, offset, rings)
        }
        C::FenceGateBlock => {
            if flag(block, offset, "open") {
                Shapes::empty()
            } else {
                let rings = cached!(
                    [(Axis, VoxelShape); 2],
                    Shapes::rotate_horizontal_axis(&column(16.0, 4.0, 0.0, 24.0))
                );
                by_horizontal_axis(block, offset, rings)
            }
        }
        C::BigDripleafBlock => leaf(block, offset),
        // With no entity asking, the walk is above it and not descending.
        C::ScaffoldingBlock => scaffolding().stable.clone(),
        C::GrindstoneBlock | C::BellBlock => outline(class, block, offset)?,
        C::PitcherCropBlock => {
            if prop(block, offset, "half") == Some("lower") {
                if number(block, offset, "age") == 0 {
                    cached!(VoxelShape, column_xz(6.0, -1.0, 3.0)).clone()
                } else {
                    cached!(VoxelShape, column_xz(10.0, -1.0, 5.0)).clone()
                }
            } else {
                Shapes::empty()
            }
        }
        C::MossyCarpetBlock => {
            if flag(block, offset, "bottom") {
                let default = (block.default_state_id().0 - block.first_state_id().0) as u32;
                mossy_carpet(block, default)
            } else {
                Shapes::empty()
            }
        }
        _ => return None,
    })
}

/// `getInteractionShape`: the cauldrons' inside, the composter and scaffolding's whole block and
/// the hopper's inside with its spout; empty for every other block.
pub fn interaction(block: &Block, offset: u32) -> VoxelShape {
    use photon_data::block::classes::BlockClass as C;
    let class = block.class;
    if class.is(C::AbstractCauldronBlock) {
        cached!(VoxelShape, column_xz(12.0, 4.0, 16.0)).clone()
    } else if class.is(C::ComposterBlock) || class.is(C::ScaffoldingBlock) {
        Shapes::block()
    } else if class.is(C::HopperBlock) {
        let inside = column_xz(12.0, 11.0, 16.0);
        match facing(block, offset, "facing") {
            Direction::Down => inside,
            side => {
                let spout = Shapes::rotate_horizontal(&box_z(4.0, 8.0, 10.0, 0.0, 4.0));
                Shapes::or(&inside, arm(&spout, side))
            }
        }
    } else {
        Shapes::empty()
    }
}
