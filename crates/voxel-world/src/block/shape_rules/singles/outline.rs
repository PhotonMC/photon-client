use super::*;

/// `getShape` of a class that has one shape it always gives, or a few it
/// picks from by the state; `None` for a class this does not cover.
#[expect(clippy::too_many_lines, reason = "one arm per class")]
#[expect(clippy::cognitive_complexity, reason = "one arm per class")]
pub fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
        C::SculkSensorBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone(),
        C::StonecutterBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 9.0)).clone(),
        C::SlabBlock => match prop(block, offset, "type") {
            Some("top") => cached!(VoxelShape, column_xz(16.0, 8.0, 16.0)).clone(),
            Some("bottom") => cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone(),
            _ => Shapes::block(),
        },
        C::SignBlock | C::BannerBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 16.0)).clone(),
        C::CopperGolemStatueBlock => cached!(VoxelShape, column_xz(10.0, 0.0, 14.0)).clone(),
        C::ChestBlock | C::EnderChestBlock => match prop(block, offset, "type") {
            Some(kind @ ("left" | "right")) => {
                let ring = cached!(
                    Horizontal,
                    Shapes::rotate_horizontal(&box_z(14.0, 0.0, 14.0, 0.0, 15.0))
                );
                let facing = facing(block, offset, "facing");
                let connected = if kind == "left" {
                    facing.clockwise_y()
                } else {
                    facing.counter_clockwise_y()
                }
                .expect("a horizontal facing turns about y");
                arm(ring, connected).clone()
            }
            _ => cached!(VoxelShape, column_xz(14.0, 0.0, 14.0)).clone(),
        },
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
        C::GrindstoneBlock => {
            let rings = cached!([(voxel_core::shape::AttachFace, Horizontal); 3], {
                let left = Shapes::or(
                    &cube(2.0, 6.0, 7.0, 4.0, 10.0, 16.0),
                    &cube(2.0, 5.0, 3.0, 4.0, 11.0, 9.0),
                );
                let right = Shapes::rotate(&left, ShapeRotation::InvertX);
                let north = Shapes::or_many(&box_z(8.0, 2.0, 14.0, 0.0, 12.0), &[left, right]);
                Shapes::rotate_attach_face(&north)
            });
            attach_face(block, offset, rings)
        }
        C::CandleBlock => match number(block, offset, "candles") {
            1 => cached!(VoxelShape, column_xz(2.0, 0.0, 6.0)).clone(),
            2 => cached!(VoxelShape, cube(5.0, 0.0, 6.0, 11.0, 6.0, 9.0)).clone(),
            3 => cached!(VoxelShape, cube(5.0, 0.0, 6.0, 10.0, 6.0, 11.0)).clone(),
            _ => cached!(VoxelShape, cube(5.0, 0.0, 5.0, 11.0, 6.0, 10.0)).clone(),
        },
        C::BedBlock => {
            let ring = cached!(Horizontal, {
                let north_west = cube(0.0, 0.0, 0.0, 3.0, 3.0, 3.0);
                let north_east = Shapes::rotate(&north_west, ShapeRotation::Rot90YNeg);
                Shapes::rotate_horizontal(&Shapes::or_many(
                    &column(16.0, 16.0, 3.0, 9.0),
                    &[north_west, north_east],
                ))
            });
            let facing = facing(block, offset, "facing");
            // `getConnectedDirection(state).getOpposite()`.
            let direction = if prop(block, offset, "part") == Some("head") {
                facing
            } else {
                facing.opposite()
            };
            arm(ring, direction).clone()
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
        C::CampfireBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 7.0)).clone(),
        C::BasePressurePlateBlock => plate(block, offset),
        C::TripWireBlock => {
            if flag(block, offset, "attached") {
                cached!(VoxelShape, column_xz(16.0, 1.0, 2.5)).clone()
            } else {
                cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone()
            }
        }
        C::ShulkerBoxBlock => Shapes::block(),
        C::AirBlock
        | C::LiquidBlock
        | C::LightBlock
        | C::MovingPistonBlock
        | C::BubbleColumnBlock => Shapes::empty(),
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
        C::BaseCoralFanBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 4.0)).clone(),
        C::FlowerBlock => cached!(VoxelShape, column_xz(6.0, 0.0, 10.0)).clone(),
        C::CarpetBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 1.0)).clone(),
        C::SaplingBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 12.0)).clone(),
        C::CactusBlock => cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone(),
        C::SugarCaneBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 16.0)).clone(),
        C::StemBlock => column_xz(2.0, 0.0, 2.0 + number(block, offset, "age") as f64 * 2.0),
        C::CropBlock => column_xz(16.0, 0.0, 2.0 + number(block, offset, "age") as f64 * 2.0),
        C::CarrotBlock | C::PotatoBlock => {
            column_xz(16.0, 0.0, 2.0 + number(block, offset, "age") as f64)
        }
        C::BeetrootBlock => column_xz(16.0, 0.0, 2.0 + number(block, offset, "age") as f64 * 2.0),
        C::TorchflowerCropBlock => {
            column_xz(6.0, 0.0, 6.0 + number(block, offset, "age") as f64 * 4.0)
        }
        C::NetherWartBlock => column_xz(16.0, 0.0, 5.0 + number(block, offset, "age") as f64 * 3.0),
        C::SmallDripleafBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 13.0)).clone(),
        C::DecoratedPotBlock | C::DragonEggBlock => {
            cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone()
        }
        C::TurtleEggBlock => {
            if number(block, offset, "eggs") == 1 {
                cached!(VoxelShape, cube(3.0, 0.0, 3.0, 12.0, 7.0, 12.0)).clone()
            } else {
                cached!(VoxelShape, column_xz(14.0, 0.0, 7.0)).clone()
            }
        }
        C::BambooStalkBlock => {
            if prop(block, offset, "leaves") == Some("large") {
                cached!(VoxelShape, column_xz(10.0, 0.0, 16.0)).clone()
            } else {
                cached!(VoxelShape, column_xz(6.0, 0.0, 16.0)).clone()
            }
        }
        C::BambooSaplingBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 12.0)).clone(),
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
        C::FlowerPotBlock | C::MushroomBlock => {
            cached!(VoxelShape, column_xz(6.0, 0.0, 6.0)).clone()
        }
        C::BaseCoralPlantBlock | C::CoralPlantBlock => {
            cached!(VoxelShape, column_xz(12.0, 0.0, 15.0)).clone()
        }
        C::CandleCakeBlock => cached!(
            VoxelShape,
            Shapes::or(&column_xz(2.0, 8.0, 14.0), &column_xz(14.0, 0.0, 8.0))
        )
        .clone(),
        C::DaylightDetectorBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 6.0)).clone(),
        C::DriedGhastBlock => cached!(VoxelShape, column(10.0, 10.0, 0.0, 10.0)).clone(),
        C::AnvilBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 2],
                Shapes::rotate_horizontal_axis(&Shapes::or_many(
                    &column_xz(12.0, 0.0, 4.0),
                    &[
                        column(8.0, 10.0, 4.0, 5.0),
                        column(4.0, 8.0, 5.0, 10.0),
                        column(10.0, 16.0, 10.0, 16.0)
                    ],
                ))
            );
            by_horizontal_axis(block, offset, rings)
        }
        C::CocoaBlock => {
            let rings = cached!(
                [Horizontal; 3],
                [0, 1, 2].map(|i| {
                    let i = f64::from(i);
                    Shapes::rotate_horizontal(
                        &column_xz(4.0 + i * 2.0, 7.0 - i * 2.0, 12.0)
                            .move_by(0.0, 0.0, (i - 5.0) / 16.0)
                            .optimize(),
                    )
                })
            );
            by_facing(block, offset, &rings[number(block, offset, "age")])
        }
        C::NetherPortalBlock => {
            let rings = cached!(
                [(Axis, VoxelShape); 2],
                Shapes::rotate_horizontal_axis(&column(4.0, 16.0, 0.0, 16.0))
            );
            let axis = if prop(block, offset, "axis") == Some("x") {
                Axis::X
            } else {
                Axis::Z
            };
            arm(rings, axis).clone()
        }
        C::SnowLayerBlock => column_xz(16.0, 0.0, number(block, offset, "layers") as f64 * 2.0),
        C::BrewingStandBlock => cached!(
            VoxelShape,
            Shapes::or(&column_xz(2.0, 2.0, 14.0), &column_xz(14.0, 0.0, 2.0))
        )
        .clone(),
        C::AbstractCauldronBlock => cached!(
            VoxelShape,
            Shapes::join(
                &Shapes::block(),
                &Shapes::or_many(
                    &column(16.0, 8.0, 0.0, 3.0),
                    &[
                        column(8.0, 16.0, 0.0, 3.0),
                        column_xz(12.0, 0.0, 3.0),
                        column_xz(12.0, 4.0, 16.0)
                    ],
                ),
                BooleanOp::OnlyFirst,
            )
        )
        .clone(),
        C::EndPortalFrameBlock => {
            if flag(block, offset, "eye") {
                cached!(
                    VoxelShape,
                    Shapes::or(&column_xz(16.0, 0.0, 13.0), &column_xz(8.0, 13.0, 16.0))
                )
                .clone()
            } else {
                cached!(VoxelShape, column_xz(16.0, 0.0, 13.0)).clone()
            }
        }
        C::SeaPickleBlock => match number(block, offset, "pickles") {
            2 => column_xz(10.0, 0.0, 6.0),
            3 => column_xz(12.0, 0.0, 6.0),
            4 => column_xz(12.0, 0.0, 7.0),
            _ => column_xz(4.0, 0.0, 6.0),
        },
        C::CakeBlock => cube(
            1.0 + number(block, offset, "bites") as f64 * 2.0,
            0.0,
            1.0,
            15.0,
            8.0,
            15.0,
        ),
        C::BaseTorchBlock => cached!(VoxelShape, column_xz(4.0, 0.0, 10.0)).clone(),
        C::SweetBerryBushBlock => match number(block, offset, "age") {
            0 => cached!(VoxelShape, column_xz(10.0, 0.0, 8.0)).clone(),
            3 => Shapes::block(),
            _ => cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone(),
        },
        C::SnifferEggBlock => cached!(VoxelShape, column(14.0, 12.0, 0.0, 16.0)).clone(),
        C::TallGrassBlock | C::NetherRootsBlock | C::DryVegetationBlock => {
            cached!(VoxelShape, column_xz(12.0, 0.0, 13.0)).clone()
        }
        C::TallSeagrassBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 16.0)).clone(),
        C::ConduitBlock | C::StructureVoidBlock => {
            cached!(VoxelShape, block_cube(6.0, 6.0, 6.0)).clone()
        }
        C::NetherFungusBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 9.0)).clone(),
        C::AzaleaBlock => cached!(
            VoxelShape,
            Shapes::or(&column_xz(16.0, 8.0, 16.0), &column_xz(4.0, 0.0, 8.0))
        )
        .clone(),
        C::HangingRootsBlock => cached!(VoxelShape, column_xz(12.0, 10.0, 16.0)).clone(),
        C::HeavyCoreBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 8.0)).clone(),
        C::HangingMossBlock => {
            if flag(block, offset, "tip") {
                cached!(VoxelShape, column_xz(14.0, 2.0, 16.0)).clone()
            } else {
                cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone()
            }
        }
        C::BushBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 13.0)).clone(),
        C::ShortDryGrassBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 10.0)).clone(),
        C::TallDryGrassBlock => cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone(),
        C::SeagrassBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 12.0)).clone(),
        C::CactusFlowerBlock => cached!(VoxelShape, column_xz(14.0, 0.0, 12.0)).clone(),
        C::LilyPadBlock => cached!(VoxelShape, column_xz(14.0, 0.0, 1.5)).clone(),
        C::FrogspawnBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 1.5)).clone(),
        C::EnchantingTableBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 12.0)).clone(),
        C::EndPortalBlock => cached!(VoxelShape, column_xz(16.0, 6.0, 12.0)).clone(),
        C::DirtPathBlock | C::FarmlandBlock => {
            cached!(VoxelShape, column_xz(16.0, 0.0, 15.0)).clone()
        }
        C::NetherSproutsBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 3.0)).clone(),
        C::SporeBlossomBlock => cached!(VoxelShape, column_xz(12.0, 13.0, 16.0)).clone(),
        C::FlowerBedBlock => segments(block, offset, 3.0, "flower_amount"),
        C::LeafLitterBlock => segments(block, offset, 1.0, "segment_amount"),
        C::HopperBlock => {
            struct Hopper {
                spoutless: VoxelShape,
                spouts: All,
            }
            let hopper = cached!(Hopper, {
                let inside = column_xz(12.0, 11.0, 16.0);
                let outline = Shapes::or(&column_xz(16.0, 10.0, 16.0), &column_xz(8.0, 4.0, 10.0));
                Hopper {
                    spoutless: Shapes::join(&outline, &inside, BooleanOp::OnlyFirst),
                    spouts: Shapes::rotate_all_with_center(
                        &box_z(4.0, 4.0, 8.0, 0.0, 8.0),
                        Vec3::new(8.0 * 0.0625, 6.0 * 0.0625, 8.0 * 0.0625),
                    ),
                }
            });
            let spout = Shapes::join(
                arm(&hopper.spouts, facing(block, offset, "facing")),
                &Shapes::block(),
                BooleanOp::And,
            );
            Shapes::or(&hopper.spoutless, &spout)
        }
        C::PitcherCropBlock => {
            const PLANT_HEIGHTS: [i32; 5] = [0, 9, 11, 22, 26];
            let age = number(block, offset, "age");
            let height = (if age == 0 { 4 } else { 6 }) + PLANT_HEIGHTS[age];
            let width = if age == 0 { 6.0 } else { 10.0 };
            if prop(block, offset, "half") == Some("lower") {
                column_xz(width, -1.0, f64::from(16.min(-1 + height)))
            } else {
                column_xz(width, 0.0, f64::from(0.max(-1 + height - 16)))
            }
        }
        C::ComposterBlock => {
            let level = number(block, offset, "level").min(7);
            composter(level)
        }
        C::BigDripleafBlock => {
            let stems = cached!(
                Horizontal,
                Shapes::rotate_horizontal(
                    &column_xz(6.0, 0.0, 13.0).move_by(0.0, 0.0, 0.25).optimize()
                )
            );
            Shapes::or(
                &leaf(block, offset),
                arm(stems, facing(block, offset, "facing")),
            )
        }
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
        C::MossyCarpetBlock => mossy_carpet(block, offset),
        C::GrowingPlantBlock => match block.class {
            C::KelpBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 9.0)).clone(),
            C::KelpPlantBlock => Shapes::block(),
            C::WeepingVinesBlock => cached!(VoxelShape, column_xz(8.0, 9.0, 16.0)).clone(),
            C::TwistingVinesBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 15.0)).clone(),
            C::TwistingVinesPlantBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 16.0)).clone(),
            // `CaveVines.SHAPE`, which the weeping vines' plant repeats.
            C::WeepingVinesPlantBlock | C::CaveVinesBlock | C::CaveVinesPlantBlock => {
                cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone()
            }
            _ => return None,
        },
        C::AmethystClusterBlock => {
            // `new AmethystClusterBlock(height, width, ..)` of each block.
            let rings = cached!(
                [All; 4],
                [(7.0, 10.0), (5.0, 10.0), (4.0, 10.0), (3.0, 8.0)].map(|(height, width)| {
                    Shapes::rotate_all(&box_z_xy(width, 16.0 - height, 16.0))
                })
            );
            let ring = match block.name {
                "minecraft:amethyst_cluster" => &rings[0],
                "minecraft:large_amethyst_bud" => &rings[1],
                "minecraft:medium_amethyst_bud" => &rings[2],
                "minecraft:small_amethyst_bud" => &rings[3],
                _ => return None,
            };
            arm(ring, facing(block, offset, "facing")).clone()
        }
        C::SpeleothemBlock => {
            let thickness = prop(block, offset, "thickness").unwrap_or("base");
            match thickness {
                "tip_merge" => column_xz(6.0, 0.0, 16.0),
                "tip" if prop(block, offset, "vertical_direction") == Some("down") => {
                    column_xz(6.0, 5.0, 16.0)
                }
                "tip" => column_xz(6.0, 0.0, 11.0),
                "frustum" => column_xz(8.0, 0.0, 16.0),
                "middle" => column_xz(10.0, 0.0, 16.0),
                _ => column_xz(12.0, 0.0, 16.0),
            }
        }
        C::ScaffoldingBlock => {
            let scaffolding = scaffolding();
            if flag(block, offset, "bottom") {
                scaffolding.unstable.clone()
            } else {
                scaffolding.stable.clone()
            }
        }
        C::MangrovePropaguleBlock => {
            const MIN_Y: [f64; 5] = [13.0, 10.0, 7.0, 3.0, 0.0];
            let age = if flag(block, offset, "hanging") {
                number(block, offset, "age")
            } else {
                4
            };
            column_xz(2.0, MIN_Y[age], 16.0)
        }
        _ => return None,
    })
}
