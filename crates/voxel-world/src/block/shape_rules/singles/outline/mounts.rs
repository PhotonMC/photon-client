use super::*;

/// `getShape` of blocks fixed to a wall, a ceiling or a post: signs, banners, heads, ladders, torches, chains and hanging things; `None` for any other class.
#[expect(clippy::too_many_lines, reason = "one arm per class")]
pub(super) fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
        C::SignBlock | C::BannerBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 16.0)).clone(),
        C::CopperGolemStatueBlock => cached!(VoxelShape, column_xz(10.0, 0.0, 14.0)).clone(),
        C::SkullBlock => {
            if block.name.contains("piglin") {
                cached!(VoxelShape, column_xz(10.0, 0.0, 8.0)).clone()
            } else {
                cached!(VoxelShape, column_xz(8.0, 0.0, 8.0)).clone()
            }
        }
        C::WallSkullBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z_xy(8.0, 8.0, 16.0))
            ),
        ),
        C::PiglinWallSkullBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z_sized(10.0, 8.0, 8.0, 16.0))
            ),
        ),
        C::RodBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 3],
                Shapes::rotate_all_axis(&block_cube(4.0, 4.0, 16.0))
            );
            arm(rings, facing(block, offset, "facing").axis()).clone()
        }
        C::ChainBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 3],
                Shapes::rotate_all_axis(&block_cube(3.0, 3.0, 16.0))
            );
            let axis = match prop(block, offset, "axis") {
                Some("x") => Axis::X,
                Some("y") => Axis::Y,
                _ => Axis::Z,
            };
            arm(rings, axis).clone()
        }
        C::ShelfBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&Shapes::or_many(
                    &cube(0.0, 12.0, 11.0, 16.0, 16.0, 13.0),
                    &[
                        cube(0.0, 0.0, 13.0, 16.0, 16.0, 16.0),
                        cube(0.0, 0.0, 11.0, 16.0, 4.0, 13.0)
                    ],
                ))
            ),
        ),
        C::CeilingHangingSignBlock => {
            let ring = cached!(
                Horizontal,
                Shapes::rotate_horizontal(&column(14.0, 2.0, 0.0, 10.0))
            );
            // `RotationSegment.convertToSegment`: four segments to a quarter turn.
            let segment = number(block, offset, "rotation");
            [
                Direction::South,
                Direction::West,
                Direction::North,
                Direction::East,
            ]
            .into_iter()
            .find(|direction| direction.data2d() as usize * 4 == segment)
            .map_or_else(
                || cached!(VoxelShape, column_xz(10.0, 0.0, 16.0)).clone(),
                |direction| arm(ring, direction).clone(),
            )
        }
        C::WallHangingSignBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 2],
                Shapes::rotate_horizontal_axis(&Shapes::or(
                    &column(16.0, 4.0, 14.0, 16.0),
                    &column(14.0, 2.0, 0.0, 10.0)
                ))
            );
            by_horizontal_axis(block, offset, rings)
        }
        C::WallSignBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z(16.0, 4.5, 12.5, 14.0, 16.0))
            ),
        ),
        C::WallBannerBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z(16.0, 0.0, 12.5, 14.0, 16.0))
            ),
        ),
        C::BaseCoralWallFanBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z_sized(16.0, 8.0, 5.0, 16.0))
            ),
        ),
        C::LadderBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z_xy(16.0, 13.0, 16.0))
            ),
        ),
        C::WallTorchBlock | C::RedstoneWallTorchBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z(5.0, 3.0, 13.0, 11.0, 16.0))
            ),
        ),
        C::TripWireHookBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z(6.0, 0.0, 10.0, 10.0, 16.0))
            ),
        ),
        C::AttachedStemBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(&box_z(4.0, 0.0, 10.0, 0.0, 10.0))
            ),
        ),
        C::BigDripleafStemBlock => by_facing(
            block,
            offset,
            cached!(
                Horizontal,
                Shapes::rotate_horizontal(
                    &column_xz(6.0, 0.0, 16.0).move_by(0.0, 0.0, 0.25).optimize()
                )
            ),
        ),
        C::LanternBlock => {
            let standing = cached!(
                VoxelShape,
                Shapes::or(&column_xz(4.0, 7.0, 9.0), &column_xz(6.0, 0.0, 7.0))
            );
            if flag(block, offset, "hanging") {
                cached!(VoxelShape, standing.move_by(0.0, 0.0625, 0.0).optimize()).clone()
            } else {
                standing.clone()
            }
        }
        C::BaseTorchBlock => cached!(VoxelShape, column_xz(4.0, 0.0, 10.0)).clone(),
        C::BellBlock => bell(block, offset),
        C::LecternBlock => by_facing(
            block,
            offset,
            cached!(Horizontal, {
                Shapes::rotate_horizontal(&Shapes::or_many(
                    &box_z(16.0, 10.0, 14.0, 1.0, 5.333333),
                    &[
                        box_z(16.0, 12.0, 16.0, 5.333333, 9.666667),
                        box_z(16.0, 14.0, 18.0, 9.666667, 14.0),
                        lectern_base(),
                    ],
                ))
            }),
        ),
        _ => return None,
    })
}
