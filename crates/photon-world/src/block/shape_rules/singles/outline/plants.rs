use super::*;

/// `getShape` of plants, crops and the things that grow, hang or lie on the ground; `None` for any other class.
pub(super) fn outline(class: BlockClass, block: &Block, offset: u32) -> Option<VoxelShape> {
    use BlockClass as C;
    Some(match class {
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
        C::FlowerPotBlock | C::MushroomBlock => {
            cached!(VoxelShape, column_xz(6.0, 0.0, 6.0)).clone()
        }
        C::BaseCoralPlantBlock | C::CoralPlantBlock => {
            cached!(VoxelShape, column_xz(12.0, 0.0, 15.0)).clone()
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
        C::SnowLayerBlock => column_xz(16.0, 0.0, number(block, offset, "layers") as f64 * 2.0),
        C::SeaPickleBlock => match number(block, offset, "pickles") {
            2 => column_xz(10.0, 0.0, 6.0),
            3 => column_xz(12.0, 0.0, 6.0),
            4 => column_xz(12.0, 0.0, 7.0),
            _ => column_xz(4.0, 0.0, 6.0),
        },
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
        C::NetherFungusBlock => cached!(VoxelShape, column_xz(8.0, 0.0, 9.0)).clone(),
        C::AzaleaBlock => cached!(
            VoxelShape,
            Shapes::or(&column_xz(16.0, 8.0, 16.0), &column_xz(4.0, 0.0, 8.0))
        )
        .clone(),
        C::HangingRootsBlock => cached!(VoxelShape, column_xz(12.0, 10.0, 16.0)).clone(),
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
        C::DirtPathBlock | C::FarmlandBlock => {
            cached!(VoxelShape, column_xz(16.0, 0.0, 15.0)).clone()
        }
        C::NetherSproutsBlock => cached!(VoxelShape, column_xz(12.0, 0.0, 3.0)).clone(),
        C::SporeBlossomBlock => cached!(VoxelShape, column_xz(12.0, 13.0, 16.0)).clone(),
        C::FlowerBedBlock => segments(block, offset, 3.0, "flower_amount"),
        C::LeafLitterBlock => segments(block, offset, 1.0, "segment_amount"),
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
