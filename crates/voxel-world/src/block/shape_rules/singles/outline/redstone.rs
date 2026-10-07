use super::*;

/// `getShape` of redstone parts, rails, fire and the other thin shapes that connect to their neighbours; `None` for any other class.
#[expect(clippy::too_many_lines, reason = "one arm per class")]
pub(super) fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
        C::SculkSensorBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone(),
        C::FireBlock => {
            let shapes = cached!(All, Shapes::rotate_all(&box_z_xy(16.0, 0.0, 1.0)));
            let shape = faces(block, offset).fold(Shapes::empty(), |shape, direction| {
                Shapes::or(&shape, arm(shapes, direction))
            });
            if shape.is_empty() {
                cached!(VoxelShape, column_xz(16.0, 0.0, 1.0)).clone()
            } else {
                shape
            }
        }
        C::BaseFireBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 1.0)).clone(),
        C::MultifaceBlock | C::VineBlock => {
            let shapes = cached!(All, Shapes::rotate_all(&box_z_xy(16.0, 0.0, 1.0)));
            let shape = faces(block, offset)
                .filter(|direction| class == C::MultifaceBlock || *direction != Direction::Down)
                .fold(Shapes::empty(), |shape, direction| {
                    Shapes::or(&shape, arm(shapes, direction))
                });
            if shape.is_empty() {
                Shapes::block()
            } else {
                shape
            }
        }
        C::PipeBlock => {
            let core = cached!(VoxelShape, block_cube(10.0, 10.0, 10.0));
            let shapes = cached!(All, Shapes::rotate_all(&box_z_xy(10.0, 0.0, 8.0)));
            faces(block, offset).fold(core.clone(), |shape, direction| {
                Shapes::or(arm(shapes, direction), &shape)
            })
        }
        C::RedStoneWireBlock => redstone_wire(block, offset),
        C::FenceGateBlock => {
            let rings = cached!([[(Axis, VoxelShape); 2]; 2], {
                let free = Shapes::rotate_horizontal_axis(&block_cube(16.0, 16.0, 4.0));
                let wall = free.clone().map(|(axis, shape)| {
                    (
                        axis,
                        Shapes::join(&shape, &column_xz(16.0, 13.0, 16.0), BooleanOp::OnlyFirst),
                    )
                });
                [free, wall]
            });
            let in_wall = flag(block, offset, "in_wall");
            by_horizontal_axis(block, offset, &rings[usize::from(in_wall)])
        }
        C::ButtonBlock => {
            struct Button {
                rings: [(voxel_core::shape::AttachFace, Horizontal); 3],
                pressed: VoxelShape,
                unpressed: VoxelShape,
            }
            let button = cached!(
                Button,
                Button {
                    rings: Shapes::rotate_attach_face(&box_z_sized(6.0, 4.0, 8.0, 16.0)),
                    pressed: block_cube(14.0, 14.0, 14.0),
                    unpressed: block_cube(12.0, 12.0, 12.0),
                }
            );
            let face = attach_face(block, offset, &button.rings);
            let press = if flag(block, offset, "powered") {
                &button.pressed
            } else {
                &button.unpressed
            };
            Shapes::join(&face, press, BooleanOp::OnlyFirst)
        }
        C::LeverBlock => {
            let rings = cached!(
                [(voxel_core::shape::AttachFace, Horizontal); 3],
                Shapes::rotate_attach_face(&box_z_sized(6.0, 8.0, 10.0, 16.0))
            );
            attach_face(block, offset, rings)
        }
        C::BaseRailBlock => {
            let slope =
                prop(block, offset, "shape").is_some_and(|shape| shape.starts_with("ascending"));
            if slope {
                cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone()
            } else {
                cached!(VoxelShape, column_xz(16.0, 0.0, 2.0)).clone()
            }
        }
        C::DiodeBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 2.0)).clone(),
        C::BasePressurePlateBlock => plate(block, offset),
        C::TripWireBlock => {
            if flag(block, offset, "attached") {
                cached!(VoxelShape, column_xz(16.0, 1.0, 2.5)).clone()
            } else {
                cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone()
            }
        }
        C::PistonBaseBlock => {
            if flag(block, offset, "extended") {
                let shapes = cached!(All, Shapes::rotate_all(&box_z_xy(16.0, 4.0, 16.0)));
                arm(shapes, facing(block, offset, "facing")).clone()
            } else {
                Shapes::block()
            }
        }
        C::PistonHeadBlock => {
            struct Head {
                short: All,
                long: All,
            }
            let head = cached!(Head, {
                let platform = box_z_xy(16.0, 0.0, 4.0);
                Head {
                    short: Shapes::rotate_all(&Shapes::or(&platform, &box_z_xy(4.0, 4.0, 16.0))),
                    long: Shapes::rotate_all(&Shapes::or(&platform, &box_z_xy(4.0, 4.0, 20.0))),
                }
            });
            let shapes = if flag(block, offset, "short") {
                &head.short
            } else {
                &head.long
            };
            arm(shapes, facing(block, offset, "facing")).clone()
        }
        C::DaylightDetectorBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 6.0)).clone(),
        _ => return None,
    })
}
