use super::*;

/// `getShape` of solid and container blocks with a shape of their own: chests, beds, anvils, cauldrons, hoppers, portals, slabs; `None` for any other class.
pub(super) fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
        C::StonecutterBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 9.0)).clone(),
        C::SlabBlock => match prop(block, offset, "type") {
            Some("top") => cached!(VoxelShape, column_xz(16.0, 8.0, 16.0)).clone(),
            Some("bottom") => cached!(VoxelShape, column_xz(16.0, 0.0, 8.0)).clone(),
            _ => Shapes::block(),
        },
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
        C::GrindstoneBlock => {
            let rings = cached!([(photon_core::shape::AttachFace, Horizontal); 3], {
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
        C::BedBlock | C::StrawBedBlock | C::ShelfMushroomBlock => {
            grown_furniture(class, block, offset)
        }
        C::CampfireBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 7.0)).clone(),
        C::ShulkerBoxBlock => Shapes::block(),
        C::AirBlock
        | C::LiquidBlock
        | C::LightBlock
        | C::MovingPistonBlock
        | C::BubbleColumnBlock => Shapes::empty(),
        C::DecoratedPotBlock | C::DragonEggBlock => {
            cached!(VoxelShape, column_xz(14.0, 0.0, 16.0)).clone()
        }
        C::CandleCakeBlock => cached!(
            VoxelShape,
            Shapes::or(&column_xz(2.0, 8.0, 14.0), &column_xz(14.0, 0.0, 8.0))
        )
        .clone(),
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
        C::CakeBlock => cube(
            1.0 + number(block, offset, "bites") as f64 * 2.0,
            0.0,
            1.0,
            15.0,
            8.0,
            15.0,
        ),
        C::ConduitBlock | C::StructureVoidBlock => {
            cached!(VoxelShape, block_cube(6.0, 6.0, 6.0)).clone()
        }
        C::HeavyCoreBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 8.0)).clone(),
        C::EnchantingTableBlock => cached!(VoxelShape, column_xz(16.0, 0.0, 12.0)).clone(),
        C::EndPortalBlock => cached!(VoxelShape, column_xz(16.0, 6.0, 12.0)).clone(),
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
        C::ComposterBlock => {
            let level = number(block, offset, "level").min(7);
            composter(level)
        }
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
        _ => return None,
    })
}

/// `getShape` of the straw bed and the shelf mushroom, which the newest version added.
fn grown_furniture(class: BlockClass, block: &Block, offset: u32) -> VoxelShape {
    match class {
        BlockClass::BedBlock => bed(block, offset),
        BlockClass::StrawBedBlock => straw_bed(block, offset),
        _ => shelf_mushroom(block, offset),
    }
}

fn bed(block: &Block, offset: u32) -> VoxelShape {
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

fn straw_bed(block: &Block, offset: u32) -> VoxelShape {
    let rings = cached!([Horizontal; 2], {
        let base = column(16.0, 16.0, 0.0, 4.0);
        let pillow = cube(0.0, 0.0, 0.0, 16.0, 5.0, 8.0);
        [
            Shapes::rotate_horizontal(&base),
            Shapes::rotate_horizontal(&Shapes::or(&base, &pillow)),
        ]
    });
    let facing = facing(block, offset, "facing");
    // `getConnectedDirection(state).getOpposite()`.
    let head = prop(block, offset, "part") == Some("head");
    let direction = if head { facing } else { facing.opposite() };
    arm(&rings[usize::from(head)], direction).clone()
}

fn shelf_mushroom(block: &Block, offset: u32) -> VoxelShape {
    let rings = cached!([Horizontal; 2], {
        let grown = |i: f64| {
            let cap = column(10.0 + i * 4.0, 7.0 + i * 3.0, 1.0 + i, 3.0 + i * 2.0)
                .move_by(0.0, (8.0 - i * 2.0) / 16.0, -(i * 1.5 - 4.5) / 16.0)
                .optimize();
            let stem = column(6.0 + i * 2.0, 4.0 + i * 2.0, 0.0, 1.0 + i)
                .move_by(0.0, (8.0 - i * 2.0) / 16.0, -(i - 6.0) / 16.0)
                .optimize();
            Shapes::rotate_horizontal(&Shapes::or(&cap, &stem))
        };
        [grown(0.0), grown(1.0)]
    });
    let age = number(block, offset, "age");
    arm(&rings[age.clamp(0, 1)], facing(block, offset, "facing")).clone()
}
