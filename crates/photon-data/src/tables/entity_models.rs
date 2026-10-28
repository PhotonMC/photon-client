//! Every model layer the reference builds, in order of id.
//! The tree of each is as `LayerDefinitions` assembles it: poses, cubes and children
//! by name.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

#![allow(
    clippy::approx_constant,
    reason = "the reference writes 1.5708 and 3.1416 for its angles"
)]

use crate::entity_model::{Face, ModelCube, ModelLayer, ModelPart, PartPose};

#[rustfmt::skip]
pub static LAYERS: [ModelLayer; 358] = [
    ModelLayer::of("minecraft:allay#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 23.5, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -4.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-1.5, 0.0, -1.0], [3.0, 4.0, 2.0]), ModelCube::of([0.0, 16.0], [-1.5, 0.0, -1.0], [3.0, 5.0, 2.0]).grow([-0.2, -0.2, -0.2])], &[
                ModelPart::of("left_arm", PartPose::offset([1.75, 0.5, 0.0]), &[ModelCube::of([23.0, 6.0], [-0.25, -0.5, -1.0], [1.0, 4.0, 2.0]).grow([-0.01, -0.01, -0.01])], &[]),
                ModelPart::of("left_wing", PartPose::offset([0.5, 0.0, 0.6]), &[ModelCube::of([16.0, 14.0], [0.0, 1.0, 0.0], [0.0, 5.0, 8.0])], &[]),
                ModelPart::of("right_arm", PartPose::offset([-1.75, 0.5, 0.0]), &[ModelCube::of([23.0, 0.0], [-0.75, -0.5, -1.0], [1.0, 4.0, 2.0]).grow([-0.01, -0.01, -0.01])], &[]),
                ModelPart::of("right_wing", PartPose::offset([-0.5, 0.0, 0.6]), &[ModelCube::of([16.0, 14.0], [0.0, 1.0, 0.0], [0.0, 5.0, 8.0])], &[]),
            ]),
            ModelPart::of("head", PartPose::offset([0.0, -3.99, 0.0]), &[ModelCube::of([0.0, 0.0], [-2.5, -5.0, -2.5], [5.0, 5.0, 5.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:armadillo#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 21.0, 4.0]), &[ModelCube::of([0.0, 20.0], [-4.0, -7.0, -10.0], [8.0, 8.0, 12.0]).grow([0.3, 0.3, 0.3]), ModelCube::of([0.0, 40.0], [-4.0, -7.0, -10.0], [8.0, 8.0, 12.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -2.0, -11.0]), &[], &[
                ModelPart::of("head_cube", PartPose::rotation([-0.3927, 0.0, 0.0]), &[ModelCube::of([43.0, 15.0], [-1.5, -1.0, -1.0], [3.0, 5.0, 2.0])], &[]),
                ModelPart::of("left_ear", PartPose::offset([1.0, -2.0, 0.0]), &[], &[
                    ModelPart::of("left_ear_cube", PartPose::offset_and_rotation([0.5, 1.0, -0.6], [0.1886, 0.3864, 0.0718]), &[ModelCube::of([47.0, 10.0], [0.0, -3.0, 0.0], [2.0, 5.0, 0.0])], &[]),
                ]),
                ModelPart::of("right_ear", PartPose::offset([-1.0, -1.0, 0.0]), &[], &[
                    ModelPart::of("right_ear_cube", PartPose::offset_and_rotation([-0.5, 0.0, -0.6], [0.1886, -0.3864, -0.0718]), &[ModelCube::of([43.0, 10.0], [-2.0, -3.0, 0.0], [2.0, 5.0, 0.0])], &[]),
                ]),
            ]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -3.0, 1.0], [0.5061, 0.0, 0.0]), &[ModelCube::of([44.0, 53.0], [-0.5, -0.0865, 0.0933], [1.0, 6.0, 1.0])], &[]),
        ]),
        ModelPart::of("cube", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-5.0, -10.0, -6.0], [10.0, 10.0, 10.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 21.0, -4.0]), &[ModelCube::of([42.0, 43.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 21.0, 4.0]), &[ModelCube::of([42.0, 31.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 21.0, -4.0]), &[ModelCube::of([51.0, 43.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 21.0, 4.0]), &[ModelCube::of([51.0, 31.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:armadillo_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.0, 0.5]), &[ModelCube::of([0.0, 0.0], [-2.5, -2.0, -3.5], [5.0, 4.0, 7.0]).grow([0.3, 0.3, 0.3]), ModelCube::of([0.0, 11.0], [-2.5, -2.0, -3.0], [5.0, 4.0, 6.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, 0.0, -3.2]), &[], &[
                ModelPart::of("head_cube", PartPose::rotation([0.7417649, 0.0, 0.0]), &[ModelCube::of([20.0, 17.0], [-1.0, -2.0, -4.0], [2.0, 2.0, 4.0])], &[
                    ModelPart::of("left_ear", PartPose::offset_and_rotation([1.0, -2.0, -0.3], [-0.4363, 0.1134, -0.0524]), &[ModelCube::of([28.0, 8.0], [-0.2, -2.0, 0.0], [2.0, 3.0, 0.0])], &[]),
                    ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.0, -2.0, -0.3], [-0.4363, -0.1134, 0.0524]), &[ModelCube::of([28.0, 8.0], [-1.8, -2.0, 0.0], [2.0, 3.0, 0.0]).mirror()], &[]),
                ]),
            ]),
            ModelPart::of("tail", PartPose::offset([0.0, 0.0, 3.4]), &[], &[
                ModelPart::of("right_ear_cube", PartPose::offset_and_rotation([0.0, 1.5, 1.0], [-1.0472, 0.0, 0.0]), &[ModelCube::of([22.0, 11.0], [-0.5, -0.5, -2.0], [1.0, 1.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("cube", PartPose::offset([0.0, 20.7, 0.5]), &[ModelCube::of([0.0, 25.0], [-3.0, -3.0, -3.0], [6.0, 6.0, 6.0]).grow([0.3, 0.3, 0.3])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([-1.5, 22.0, -1.5]), &[ModelCube::of([24.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.5, 22.0, 2.5]), &[ModelCube::of([20.0, 27.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([1.5, 22.0, -1.5]), &[ModelCube::of([20.0, 23.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.5, 22.0, 2.5]), &[ModelCube::of([20.0, 27.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0]).mirror()], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 1.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 11.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 11.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 1.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 11.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 11.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 1.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 11.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 11.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 1.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 11.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 11.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base_plate", PartPose::offset([0.0, 12.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-6.0, 11.0, -6.0], [12.0, 1.0, 12.0])], &[]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([0.0, 26.0], [-6.0, 0.0, -1.5], [12.0, 3.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 1.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -7.0, -1.0], [2.0, 7.0, 2.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 16.0], [0.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_body_stick", PartPose::ZERO, &[ModelCube::of([48.0, 16.0], [1.0, 3.0, -1.0], [2.0, 7.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-2.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_body_stick", PartPose::ZERO, &[ModelCube::of([16.0, 0.0], [-3.0, 3.0, -1.0], [2.0, 7.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([8.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ModelPart::of("shoulder_stick", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-4.0, 10.0, -1.0], [8.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand_small#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("head", PartPose::new([0.0, 12.75, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::new([2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("left_leg", PartPose::new([0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::new([-2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("right_leg", PartPose::new([-0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand_small#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::new([0.0, 12.75, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::new([2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::new([0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("right_arm", PartPose::new([-2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::new([-0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand_small#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("head", PartPose::new([0.0, 12.75, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::new([2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("left_leg", PartPose::new([0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("right_arm", PartPose::new([-2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("right_leg", PartPose::new([-0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand_small#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::new([0.0, 12.75, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::new([2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("left_leg", PartPose::new([0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::new([-2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[]),
        ModelPart::of("right_leg", PartPose::new([-0.95, 17.5, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:armor_stand_small#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base_plate", PartPose::new([0.0, 18.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 32.0], [-6.0, 11.0, -6.0], [12.0, 1.0, 12.0])], &[]),
        ModelPart::of("body", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 26.0], [-6.0, 0.0, -1.5], [12.0, 3.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::new([0.0, 12.75, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[ModelCube::of([0.0, 0.0], [-1.0, -7.0, -1.0], [2.0, 7.0, 2.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::new([2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([32.0, 16.0], [0.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_body_stick", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([48.0, 16.0], [1.0, 3.0, -1.0], [2.0, 7.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::new([0.95, 18.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([40.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::new([-2.5, 13.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([24.0, 0.0], [-2.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_body_stick", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([16.0, 0.0], [-3.0, 3.0, -1.0], [2.0, 7.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::new([-0.95, 18.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([8.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ModelPart::of("shoulder_stick", PartPose::new([0.0, 12.0, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[ModelCube::of([0.0, 48.0], [-4.0, 10.0, -1.0], [8.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:arrow#main", 32, 32, ModelPart::of("root", PartPose::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.9, 0.9, 0.9]), &[], &[
        ModelPart::of("back", PartPose::new([-11.0, 0.0, 0.0], [0.7853982, 0.0, 0.0], [0.8, 0.8, 0.8]), &[ModelCube::of([0.0, 0.0], [0.0, -2.5, -2.5], [0.0, 5.0, 5.0])], &[]),
        ModelPart::of("cross_1", PartPose::rotation([0.7853982, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-12.0, -2.0, 0.0], [16.0, 4.0, 0.0]).uv_scale([1.0, 0.8])], &[]),
        ModelPart::of("cross_2", PartPose::rotation([2.3561945, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-12.0, -2.0, 0.0], [16.0, 4.0, 0.0]).uv_scale([1.0, 0.8])], &[]),
    ])),
    ModelLayer::of("minecraft:axolotl#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 19.5, 5.0]), &[ModelCube::of([0.0, 11.0], [-4.0, -2.0, -9.0], [8.0, 4.0, 10.0]), ModelCube::of([2.0, 17.0], [0.0, -3.0, -8.0], [0.0, 5.0, 9.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, 0.0, -9.0]), &[ModelCube::of([0.0, 1.0], [-4.0, -3.0, -5.0], [8.0, 5.0, 5.0]).grow([0.001, 0.001, 0.001])], &[
                ModelPart::of("left_gills", PartPose::offset([-4.0, 0.0, -1.0]), &[ModelCube::of([0.0, 40.0], [-3.0, -5.0, 0.0], [3.0, 7.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
                ModelPart::of("right_gills", PartPose::offset([4.0, 0.0, -1.0]), &[ModelCube::of([11.0, 40.0], [0.0, -5.0, 0.0], [3.0, 7.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
                ModelPart::of("top_gills", PartPose::offset([0.0, -3.0, -1.0]), &[ModelCube::of([3.0, 37.0], [-4.0, -3.0, 0.0], [8.0, 3.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
            ]),
            ModelPart::of("left_front_leg", PartPose::offset([3.5, 1.0, -8.0]), &[ModelCube::of([2.0, 13.0], [-1.0, 0.0, 0.0], [3.0, 5.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
            ModelPart::of("left_hind_leg", PartPose::offset([3.5, 1.0, -1.0]), &[ModelCube::of([2.0, 13.0], [-1.0, 0.0, 0.0], [3.0, 5.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
            ModelPart::of("right_front_leg", PartPose::offset([-3.5, 1.0, -8.0]), &[ModelCube::of([2.0, 13.0], [-2.0, 0.0, 0.0], [3.0, 5.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
            ModelPart::of("right_hind_leg", PartPose::offset([-3.5, 1.0, -1.0]), &[ModelCube::of([2.0, 13.0], [-2.0, 0.0, 0.0], [3.0, 5.0, 0.0]).grow([0.001, 0.001, 0.001])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, 0.0, 1.0]), &[ModelCube::of([2.0, 19.0], [0.0, -3.0, 0.0], [0.0, 5.0, 12.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:axolotl_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -1.25, 1.75]), &[ModelCube::of([0.0, 0.0], [-2.0, -0.75, -2.75], [4.0, 2.0, 6.0]), ModelCube::of([0.0, 12.0], [0.0, -1.75, -2.75], [0.0, 3.0, 5.0])], &[
                ModelPart::of("head", PartPose::offset([0.0, 0.25, -2.75]), &[ModelCube::of([0.0, 8.0], [-3.0, -2.0, -4.0], [6.0, 3.0, 4.0])], &[
                    ModelPart::of("left_gills", PartPose::offset([3.0, -0.5, -2.0]), &[ModelCube::of([20.0, 8.0], [0.0, -3.5, 0.0], [3.0, 5.0, 0.0])], &[]),
                    ModelPart::of("right_gills", PartPose::offset([-3.0, -0.5, -2.0]), &[ModelCube::of([20.0, 3.0], [-3.0, -3.5, 0.0], [3.0, 5.0, 0.0])], &[]),
                    ModelPart::of("top_gills", PartPose::offset([0.0, -2.0, -2.0]), &[ModelCube::of([20.0, 0.0], [-3.0, -3.0, 0.0], [6.0, 3.0, 0.0])], &[]),
                ]),
                ModelPart::of("left_front_leg", PartPose::offset([2.0, 0.25, -1.25]), &[ModelCube::of([20.0, 13.0], [0.0, 0.0, -0.5], [3.0, 0.0, 1.0])], &[]),
                ModelPart::of("left_hind_leg", PartPose::offset([2.0, 0.25, 1.75]), &[ModelCube::of([20.0, 14.0], [0.0, 0.0, -0.5], [3.0, 0.0, 1.0])], &[]),
                ModelPart::of("right_front_leg", PartPose::offset([-2.0, 0.25, -1.25]), &[ModelCube::of([20.0, 16.0], [-3.0, 0.0, -0.5], [3.0, 0.0, 1.0])], &[]),
                ModelPart::of("right_hind_leg", PartPose::offset_and_rotation([-2.0, 0.25, 1.75], [0.0, 1.5708, 1.5708]), &[], &[
                    ModelPart::of("right_leg_r1", PartPose::rotation([-1.5708, 0.0, 1.5708]), &[ModelCube::of([20.0, 14.0], [0.0, 0.0, -0.5], [3.0, 0.0, 1.0])], &[]),
                ]),
                ModelPart::of("tail", PartPose::offset([0.0, -0.25, 3.25]), &[ModelCube::of([10.0, 9.0], [0.0, -1.5, -1.0], [0.0, 3.0, 8.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:bat#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.5, 0.0, -1.0], [3.0, 5.0, 2.0])], &[
            ModelPart::of("feet", PartPose::offset([0.0, 5.0, 0.0]), &[ModelCube::of([16.0, 16.0], [-1.5, 0.0, 0.0], [3.0, 2.0, 0.0])], &[]),
            ModelPart::of("left_wing", PartPose::offset([1.5, 0.0, 0.0]), &[ModelCube::of([12.0, 7.0], [0.0, -2.0, 0.0], [2.0, 7.0, 0.0])], &[
                ModelPart::of("left_wing_tip", PartPose::offset([2.0, 0.0, 0.0]), &[ModelCube::of([16.0, 8.0], [0.0, -2.0, 0.0], [6.0, 8.0, 0.0])], &[]),
            ]),
            ModelPart::of("right_wing", PartPose::offset([-1.5, 0.0, 0.0]), &[ModelCube::of([12.0, 0.0], [-2.0, -2.0, 0.0], [2.0, 7.0, 0.0])], &[
                ModelPart::of("right_wing_tip", PartPose::offset([-2.0, 0.0, 0.0]), &[ModelCube::of([16.0, 0.0], [-6.0, -2.0, 0.0], [6.0, 8.0, 0.0])], &[]),
            ]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 17.0, 0.0]), &[ModelCube::of([0.0, 7.0], [-2.0, -3.0, -1.0], [4.0, 3.0, 2.0])], &[
            ModelPart::of("left_ear", PartPose::offset([1.1, -3.0, 0.0]), &[ModelCube::of([8.0, 15.0], [-0.1, -3.0, 0.0], [3.0, 5.0, 0.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset([-1.5, -2.0, 0.0]), &[ModelCube::of([1.0, 15.0], [-2.5, -4.0, 0.0], [3.0, 5.0, 0.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:bee#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 19.0, 0.0]), &[], &[
            ModelPart::of("back_legs", PartPose::offset([1.5, 3.0, 2.0]), &[ModelCube::of([26.0, 5.0], [-5.0, 0.0, 0.0], [7.0, 2.0, 0.0])], &[]),
            ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-3.5, -4.0, -5.0], [7.0, 7.0, 10.0])], &[
                ModelPart::of("left_antenna", PartPose::offset([0.0, -2.0, -5.0]), &[ModelCube::of([2.0, 0.0], [1.5, -2.0, -3.0], [1.0, 2.0, 3.0])], &[]),
                ModelPart::of("right_antenna", PartPose::offset([0.0, -2.0, -5.0]), &[ModelCube::of([2.0, 3.0], [-2.5, -2.0, -3.0], [1.0, 2.0, 3.0])], &[]),
                ModelPart::of("stinger", PartPose::ZERO, &[ModelCube::of([26.0, 7.0], [0.0, -1.0, 5.0], [0.0, 1.0, 2.0])], &[]),
            ]),
            ModelPart::of("front_legs", PartPose::offset([1.5, 3.0, -2.0]), &[ModelCube::of([26.0, 1.0], [-5.0, 0.0, 0.0], [7.0, 2.0, 0.0])], &[]),
            ModelPart::of("left_wing", PartPose::offset_and_rotation([1.5, -4.0, -3.0], [0.0, 0.2618, 0.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, 0.0], [9.0, 0.0, 6.0]).grow([0.001, 0.001, 0.001]).mirror()], &[]),
            ModelPart::of("middle_legs", PartPose::offset([1.5, 3.0, 0.0]), &[ModelCube::of([26.0, 3.0], [-5.0, 0.0, 0.0], [7.0, 2.0, 0.0])], &[]),
            ModelPart::of("right_wing", PartPose::offset_and_rotation([-1.5, -4.0, -3.0], [0.0, -0.2618, 0.0]), &[ModelCube::of([0.0, 18.0], [-9.0, 0.0, 0.0], [9.0, 0.0, 6.0]).grow([0.001, 0.001, 0.001])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:bee_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 19.6667, -1.8567]), &[ModelCube::of([6.0, 12.0], [1.0, -1.6667, -2.1633], [1.0, 2.0, 2.0]), ModelCube::of([0.0, 12.0], [-2.0, -1.6667, -2.1933], [1.0, 2.0, 2.0])], &[
            ModelPart::of("back_legs", PartPose::offset([0.0, 3.3333, 3.8567]), &[ModelCube::of([13.0, 2.0], [-1.5, 0.0, 0.0], [3.0, 1.0, 0.0])], &[]),
            ModelPart::of("body", PartPose::offset([0.0, 1.3333, 2.3567]), &[ModelCube::of([0.0, 0.0], [-2.0, -2.0, -2.5], [4.0, 4.0, 5.0])], &[
                ModelPart::of("stinger", PartPose::offset([0.0, 0.5, 2.5]), &[ModelCube::of([13.0, 2.0], [0.0, -0.5, 0.0], [0.0, 1.0, 1.0])], &[]),
            ]),
            ModelPart::of("front_legs", PartPose::offset([0.0, 3.3333, 1.8567]), &[ModelCube::of([13.0, 0.0], [-1.5, 0.0, 0.0], [3.0, 1.0, 0.0])], &[]),
            ModelPart::of("left_wing", PartPose::offset_and_rotation([1.0, -0.6667, 0.8567], [0.2182, -0.3491, 0.0]), &[ModelCube::of([-3.0, 9.0], [0.0, 0.0, 0.0], [3.0, 0.0, 3.0]).mirror()], &[]),
            ModelPart::of("middle_legs", PartPose::offset([0.0, 3.3333, 2.8567]), &[ModelCube::of([13.0, 1.0], [-1.5, 0.0, 0.0], [3.0, 1.0, 0.0])], &[]),
            ModelPart::of("right_wing", PartPose::offset_and_rotation([-1.0, -0.6667, 0.8567], [0.2182, 0.3491, 0.0]), &[ModelCube::of([3.0, 9.0], [-3.0, 0.0, 0.0], [3.0, 0.0, 3.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:bee_stinger#main", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cross_1", PartPose::rotation([0.7853982, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -0.5, 0.0], [2.0, 1.0, 0.0])], &[]),
        ModelPart::of("cross_2", PartPose::rotation([2.3561945, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -0.5, 0.0], [2.0, 1.0, 0.0])], &[]),
    ])),
    ModelLayer::of("minecraft:bell#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bell_body", PartPose::offset([8.0, 12.0, 8.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -6.0, -3.0], [6.0, 7.0, 6.0])], &[
            ModelPart::of("bell_base", PartPose::offset([-8.0, -12.0, -8.0]), &[ModelCube::of([0.0, 13.0], [4.0, 4.0, 4.0], [8.0, 2.0, 8.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:blaze#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("part0", PartPose::offset([9.0, -1.0, 0.0]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part1", PartPose::offset([1.1021821e-15, -1.1224087, 9.0]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part10", PartPose::offset([-4.455076, 11.34671, -2.269867]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part11", PartPose::offset([2.269867, 10.614307, -4.455076]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part2", PartPose::offset([-9.0, -1.4596672, 1.1021821e-15]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part3", PartPose::offset([0.0, -1.9292085, -9.0]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part4", PartPose::offset([4.9497476, 1.5839192, 4.9497476]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part5", PartPose::offset([-4.9497476, 1.1989107, 4.9497476]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part6", PartPose::offset([-4.9497476, 1.0100093, -4.9497476]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part7", PartPose::offset([4.9497476, 1.0635324, -4.9497476]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part8", PartPose::offset([4.455076, 11.960163, 2.269867]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("part9", PartPose::offset([-2.269867, 11.893008, 4.455076]), &[ModelCube::of([0.0, 16.0], [0.0, 0.0, 0.0], [2.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat#water_patch", 0, 0, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("water_patch", PartPose::offset_and_rotation([0.0, -3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/acacia#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/bamboo#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, -2.1, 1.0], [1.5708, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -11.0, -4.0], [28.0, 20.0, 4.0]), ModelCube::of([0.0, 0.0], [-14.0, -9.0, -8.0], [28.0, 16.0, 4.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -4.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([0.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([0.0, 24.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -4.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([40.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([40.0, 24.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/birch#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/cherry#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/dark_oak#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/jungle#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/mangrove#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/oak#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/pale_oak#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:boat/spruce#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("mushrooms", PartPose::ZERO, &[], &[
                ModelPart::of("brown_mushroom_1", PartPose::offset_and_rotation([-3.0, -8.0, -3.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([50.0, 22.0], [-3.0, -3.0, 0.0], [6.0, 4.0, 0.0])], &[]),
                ModelPart::of("brown_mushroom_2", PartPose::offset_and_rotation([-3.0, -8.0, -3.0], [0.0, 2.3561945, 0.0]), &[ModelCube::of([50.0, 22.0], [-3.0, -3.0, 0.0], [6.0, 4.0, 0.0])], &[]),
                ModelPart::of("brown_mushroom_3", PartPose::offset_and_rotation([-2.0, -1.0, 4.0], [-1.5707964, 0.0, 0.7853982]), &[ModelCube::of([50.0, 28.0], [-3.0, -4.0, 0.0], [6.0, 4.0, 0.0])], &[]),
                ModelPart::of("brown_mushroom_4", PartPose::offset_and_rotation([-2.0, -1.0, 4.0], [-1.5707964, 0.0, 2.3561945]), &[ModelCube::of([50.0, 28.0], [-3.0, -4.0, 0.0], [6.0, 4.0, 0.0])], &[]),
                ModelPart::of("red_mushroom_1", PartPose::offset_and_rotation([3.0, -8.0, 3.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([50.0, 16.0], [-3.0, -3.0, 0.0], [6.0, 4.0, 0.0])], &[]),
                ModelPart::of("red_mushroom_2", PartPose::offset_and_rotation([3.0, -8.0, 3.0], [0.0, 2.3561945, 0.0]), &[ModelCube::of([50.0, 16.0], [-3.0, -3.0, 0.0], [6.0, 4.0, 0.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:bogged#outer", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.2, 0.2, 0.2])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.7, 0.7, 0.7])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.2, 0.2, 0.2]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.2, 0.2, 0.2]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.2, 0.2, 0.2])], &[]),
    ])),
    ModelLayer::of("minecraft:book#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("flip_page1", PartPose::ZERO, &[ModelCube::of([24.0, 10.0], [0.0, -4.0, 0.0], [5.0, 8.0, 0.005])], &[]),
        ModelPart::of("flip_page2", PartPose::ZERO, &[ModelCube::of([24.0, 10.0], [0.0, -4.0, 0.0], [5.0, 8.0, 0.005])], &[]),
        ModelPart::of("left_lid", PartPose::offset([0.0, 0.0, -1.0]), &[ModelCube::of([0.0, 0.0], [-6.0, -5.0, -0.005], [6.0, 10.0, 0.005])], &[]),
        ModelPart::of("left_pages", PartPose::ZERO, &[ModelCube::of([0.0, 10.0], [0.0, -4.0, -0.99], [5.0, 8.0, 1.0])], &[]),
        ModelPart::of("right_lid", PartPose::offset([0.0, 0.0, 1.0]), &[ModelCube::of([16.0, 0.0], [0.0, -5.0, -0.005], [6.0, 10.0, 0.005])], &[]),
        ModelPart::of("right_pages", PartPose::ZERO, &[ModelCube::of([12.0, 10.0], [0.0, -4.0, -0.01], [5.0, 8.0, 1.0])], &[]),
        ModelPart::of("seam", PartPose::rotation([0.0, 1.5707964, 0.0]), &[ModelCube::of([12.0, 0.0], [-1.0, -5.0, 0.0], [2.0, 10.0, 0.005])], &[]),
    ])),
    ModelLayer::of("minecraft:breeze#eyes", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("head", PartPose::offset([0.0, 4.0, 0.0]), &[], &[
                ModelPart::of("eyes", PartPose::ZERO, &[ModelCube::of([4.0, 24.0], [-5.0, -5.0, -4.2], [10.0, 3.0, 4.0]), ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
            ]),
            ModelPart::of("rods", PartPose::offset([0.0, 8.0, 0.0]), &[], &[
                ModelPart::of("rod_1", PartPose::offset_and_rotation([2.5981, -3.0, 1.5], [-2.7489, -1.0472, 3.1416]), &[], &[]),
                ModelPart::of("rod_2", PartPose::offset_and_rotation([-2.5981, -3.0, 1.5], [-2.7489, 1.0472, 3.1416]), &[], &[]),
                ModelPart::of("rod_3", PartPose::offset_and_rotation([0.0, -3.0, -3.0], [0.3927, 0.0, 0.0]), &[], &[]),
            ]),
        ]),
        ModelPart::of("wind_body", PartPose::ZERO, &[], &[
            ModelPart::of("wind_bottom", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
                ModelPart::of("wind_mid", PartPose::offset([0.0, -7.0, 0.0]), &[], &[
                    ModelPart::of("wind_top", PartPose::offset([0.0, -6.0, 0.0]), &[], &[]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:breeze#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("head", PartPose::offset([0.0, 4.0, 0.0]), &[ModelCube::of([4.0, 24.0], [-5.0, -5.0, -4.2], [10.0, 3.0, 4.0]), ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
                ModelPart::of("eyes", PartPose::ZERO, &[ModelCube::of([4.0, 24.0], [-5.0, -5.0, -4.2], [10.0, 3.0, 4.0]), ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
            ]),
            ModelPart::of("rods", PartPose::offset([0.0, 8.0, 0.0]), &[], &[
                ModelPart::of("rod_1", PartPose::offset_and_rotation([2.5981, -3.0, 1.5], [-2.7489, -1.0472, 3.1416]), &[ModelCube::of([0.0, 17.0], [-1.0, 0.0, -3.0], [2.0, 8.0, 2.0])], &[]),
                ModelPart::of("rod_2", PartPose::offset_and_rotation([-2.5981, -3.0, 1.5], [-2.7489, 1.0472, 3.1416]), &[ModelCube::of([0.0, 17.0], [-1.0, 0.0, -3.0], [2.0, 8.0, 2.0])], &[]),
                ModelPart::of("rod_3", PartPose::offset_and_rotation([0.0, -3.0, -3.0], [0.3927, 0.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-1.0, 0.0, -3.0], [2.0, 8.0, 2.0])], &[]),
            ]),
        ]),
        ModelPart::of("wind_body", PartPose::ZERO, &[], &[
            ModelPart::of("wind_bottom", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
                ModelPart::of("wind_mid", PartPose::offset([0.0, -7.0, 0.0]), &[], &[
                    ModelPart::of("wind_top", PartPose::offset([0.0, -6.0, 0.0]), &[], &[]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:breeze#wind", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("head", PartPose::offset([0.0, 4.0, 0.0]), &[], &[
                ModelPart::of("eyes", PartPose::ZERO, &[], &[]),
            ]),
            ModelPart::of("rods", PartPose::offset([0.0, 8.0, 0.0]), &[], &[
                ModelPart::of("rod_1", PartPose::offset_and_rotation([2.5981, -3.0, 1.5], [-2.7489, -1.0472, 3.1416]), &[], &[]),
                ModelPart::of("rod_2", PartPose::offset_and_rotation([-2.5981, -3.0, 1.5], [-2.7489, 1.0472, 3.1416]), &[], &[]),
                ModelPart::of("rod_3", PartPose::offset_and_rotation([0.0, -3.0, -3.0], [0.3927, 0.0, 0.0]), &[], &[]),
            ]),
        ]),
        ModelPart::of("wind_body", PartPose::ZERO, &[], &[
            ModelPart::of("wind_bottom", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([1.0, 83.0], [-2.5, -7.0, -2.5], [5.0, 7.0, 5.0])], &[
                ModelPart::of("wind_mid", PartPose::offset([0.0, -7.0, 0.0]), &[ModelCube::of([74.0, 28.0], [-6.0, -6.0, -6.0], [12.0, 6.0, 12.0]), ModelCube::of([78.0, 32.0], [-4.0, -6.0, -4.0], [8.0, 6.0, 8.0]), ModelCube::of([49.0, 71.0], [-2.5, -6.0, -2.5], [5.0, 6.0, 5.0])], &[
                    ModelPart::of("wind_top", PartPose::offset([0.0, -6.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-9.0, -8.0, -9.0], [18.0, 8.0, 18.0]), ModelCube::of([6.0, 6.0], [-6.0, -8.0, -6.0], [12.0, 8.0, 12.0]), ModelCube::of([105.0, 57.0], [-2.5, -8.0, -2.5], [5.0, 8.0, 5.0])], &[]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:camel#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 4.0, 9.5]), &[ModelCube::of([0.0, 25.0], [-7.5, -12.0, -23.5], [15.0, 12.0, 27.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -3.0, -19.5]), &[ModelCube::of([60.0, 24.0], [-3.5, -7.0, -15.0], [7.0, 8.0, 19.0]), ModelCube::of([21.0, 0.0], [-3.5, -21.0, -15.0], [7.0, 14.0, 7.0]), ModelCube::of([50.0, 0.0], [-2.5, -21.0, -21.0], [5.0, 5.0, 6.0])], &[
                ModelPart::of("left_ear", PartPose::offset([2.5, -21.0, -9.5]), &[ModelCube::of([45.0, 0.0], [-0.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-2.5, -21.0, -9.5]), &[ModelCube::of([67.0, 0.0], [-2.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
            ]),
            ModelPart::of("hump", PartPose::offset([0.0, -12.0, -10.0]), &[ModelCube::of([74.0, 0.0], [-4.5, -5.0, -5.5], [9.0, 5.0, 11.0])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, -9.0, 3.5]), &[ModelCube::of([122.0, 0.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 0.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 0.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.9, 1.0, 9.5]), &[ModelCube::of([58.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 26.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.9, 1.0, 9.5]), &[ModelCube::of([94.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:camel#saddle", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 4.0, 9.5]), &[ModelCube::of([0.0, 25.0], [-7.5, -12.0, -23.5], [15.0, 12.0, 27.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -3.0, -19.5]), &[ModelCube::of([60.0, 24.0], [-3.5, -7.0, -15.0], [7.0, 8.0, 19.0]), ModelCube::of([21.0, 0.0], [-3.5, -21.0, -15.0], [7.0, 14.0, 7.0]), ModelCube::of([50.0, 0.0], [-2.5, -21.0, -21.0], [5.0, 5.0, 6.0])], &[
                ModelPart::of("bridle", PartPose::ZERO, &[ModelCube::of([60.0, 87.0], [-3.5, -7.0, -15.0], [7.0, 8.0, 19.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([21.0, 64.0], [-3.5, -21.0, -15.0], [7.0, 14.0, 7.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([50.0, 64.0], [-2.5, -21.0, -21.0], [5.0, 5.0, 6.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([74.0, 70.0], [2.5, -19.0, -18.0], [1.0, 2.0, 2.0]), ModelCube::of([74.0, 70.0], [-3.5, -19.0, -18.0], [1.0, 2.0, 2.0]).mirror()], &[]),
                ModelPart::of("left_ear", PartPose::offset([2.5, -21.0, -9.5]), &[ModelCube::of([45.0, 0.0], [-0.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
                ModelPart::of("reins", PartPose::ZERO, &[ModelCube::of([98.0, 42.0], [3.51, -18.0, -17.0], [0.0, 7.0, 15.0]), ModelCube::of([84.0, 57.0], [-3.5, -18.0, -2.0], [7.0, 7.0, 0.0]), ModelCube::of([98.0, 42.0], [-3.51, -18.0, -17.0], [0.0, 7.0, 15.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-2.5, -21.0, -9.5]), &[ModelCube::of([67.0, 0.0], [-2.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
            ]),
            ModelPart::of("hump", PartPose::offset([0.0, -12.0, -10.0]), &[ModelCube::of([74.0, 0.0], [-4.5, -5.0, -5.5], [9.0, 5.0, 11.0])], &[]),
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([74.0, 64.0], [-4.5, -17.0, -15.5], [9.0, 5.0, 11.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([92.0, 114.0], [-3.5, -20.0, -15.5], [7.0, 3.0, 11.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([0.0, 89.0], [-7.5, -12.0, -23.5], [15.0, 12.0, 27.0]).grow([0.05, 0.05, 0.05])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, -9.0, 3.5]), &[ModelCube::of([122.0, 0.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 0.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 0.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.9, 1.0, 9.5]), &[ModelCube::of([58.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 26.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.9, 1.0, 9.5]), &[ModelCube::of([94.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:camel_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 7.0, 0.0]), &[ModelCube::of([0.0, 14.0], [-4.5, -4.0, -8.0], [9.0, 8.0, 16.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, 1.0, -7.5]), &[ModelCube::of([20.0, 0.0], [-2.5, -3.0, -7.5], [5.0, 5.0, 7.0]), ModelCube::of([0.0, 0.0], [-2.5, -12.0, -7.5], [5.0, 9.0, 5.0]), ModelCube::of([0.0, 14.0], [-2.5, -12.0, -10.5], [5.0, 4.0, 3.0])], &[
                ModelPart::of("left_ear", PartPose::offset([2.5, -11.0, -4.0]), &[ModelCube::of([47.0, 0.0], [0.0, -0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-2.5, -11.0, -4.0]), &[ModelCube::of([37.0, 0.0], [-3.0, -0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
            ]),
            ModelPart::of("tail", PartPose::offset([0.0, -1.5, 8.05]), &[ModelCube::of([50.0, 38.0], [-1.5, -0.5, 0.0], [3.0, 9.0, 0.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 11.5, -5.5]), &[ModelCube::of([48.0, 14.0], [-1.5, -0.5, -1.5], [3.0, 13.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 11.5, 5.5]), &[ModelCube::of([12.0, 38.0], [-1.5, -0.5, -1.5], [3.0, 13.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 11.5, -5.5]), &[ModelCube::of([36.0, 14.0], [-1.5, -0.5, -1.5], [3.0, 13.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 11.5, 5.5]), &[ModelCube::of([0.0, 38.0], [-1.5, -0.5, -1.5], [3.0, 13.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:camel_husk#saddle", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 4.0, 9.5]), &[ModelCube::of([0.0, 25.0], [-7.5, -12.0, -23.5], [15.0, 12.0, 27.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -3.0, -19.5]), &[ModelCube::of([60.0, 24.0], [-3.5, -7.0, -15.0], [7.0, 8.0, 19.0]), ModelCube::of([21.0, 0.0], [-3.5, -21.0, -15.0], [7.0, 14.0, 7.0]), ModelCube::of([50.0, 0.0], [-2.5, -21.0, -21.0], [5.0, 5.0, 6.0])], &[
                ModelPart::of("bridle", PartPose::ZERO, &[ModelCube::of([60.0, 87.0], [-3.5, -7.0, -15.0], [7.0, 8.0, 19.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([21.0, 64.0], [-3.5, -21.0, -15.0], [7.0, 14.0, 7.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([50.0, 64.0], [-2.5, -21.0, -21.0], [5.0, 5.0, 6.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([74.0, 70.0], [2.5, -19.0, -18.0], [1.0, 2.0, 2.0]), ModelCube::of([74.0, 70.0], [-3.5, -19.0, -18.0], [1.0, 2.0, 2.0]).mirror()], &[]),
                ModelPart::of("left_ear", PartPose::offset([2.5, -21.0, -9.5]), &[ModelCube::of([45.0, 0.0], [-0.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
                ModelPart::of("reins", PartPose::ZERO, &[ModelCube::of([98.0, 42.0], [3.51, -18.0, -17.0], [0.0, 7.0, 15.0]), ModelCube::of([84.0, 57.0], [-3.5, -18.0, -2.0], [7.0, 7.0, 0.0]), ModelCube::of([98.0, 42.0], [-3.51, -18.0, -17.0], [0.0, 7.0, 15.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-2.5, -21.0, -9.5]), &[ModelCube::of([67.0, 0.0], [-2.5, 0.5, -1.0], [3.0, 1.0, 2.0])], &[]),
            ]),
            ModelPart::of("hump", PartPose::offset([0.0, -12.0, -10.0]), &[ModelCube::of([74.0, 0.0], [-4.5, -5.0, -5.5], [9.0, 5.0, 11.0])], &[]),
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([74.0, 64.0], [-4.5, -17.0, -15.5], [9.0, 5.0, 11.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([92.0, 114.0], [-3.5, -20.0, -15.5], [7.0, 3.0, 11.0]).grow([0.05, 0.05, 0.05]), ModelCube::of([0.0, 89.0], [-7.5, -12.0, -23.5], [15.0, 12.0, 27.0]).grow([0.05, 0.05, 0.05])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, -9.0, 3.5]), &[ModelCube::of([122.0, 0.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 0.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 0.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.9, 1.0, 9.5]), &[ModelCube::of([58.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.9, 1.0, -10.5]), &[ModelCube::of([0.0, 26.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.9, 1.0, 9.5]), &[ModelCube::of([94.0, 16.0], [-2.5, 2.0, -2.5], [5.0, 21.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cat#collar", 64, 32, ModelPart::of("root", PartPose::new([0.0, 4.8032, 0.0], [0.0, 0.0, 0.0], [0.8, 0.8, 0.8]), &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 12.0, -10.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([20.0, 0.0], [-2.0, 3.0, -8.0], [4.0, 16.0, 6.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -9.0]), &[ModelCube::of([0.0, 0.0], [-2.5, -2.0, -3.0], [5.0, 4.0, 5.0]).grow([0.01, 0.01, 0.01]), ModelCube::of([0.0, 24.0], [-1.5, -0.001, -4.0], [3.0, 2.0, 2.0]).grow([0.01, 0.01, 0.01]), ModelCube::of([0.0, 10.0], [-2.0, -3.0, 0.0], [1.0, 1.0, 2.0]).grow([0.01, 0.01, 0.01]), ModelCube::of([6.0, 10.0], [1.0, -3.0, 0.0], [1.0, 1.0, 2.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 15.0, 8.0], [0.9, 0.0, 0.0]), &[ModelCube::of([0.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0]).grow([0.01, 0.01, 0.01])], &[]),
        ModelPart::of("tail2", PartPose::offset([0.0, 20.0, 14.0]), &[ModelCube::of([4.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0]).grow([-0.02, -0.02, -0.02])], &[]),
    ])),
    ModelLayer::of("minecraft:cat#main", 64, 32, ModelPart::of("root", PartPose::new([0.0, 4.8032, 0.0], [0.0, 0.0, 0.0], [0.8, 0.8, 0.8]), &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 12.0, -10.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([20.0, 0.0], [-2.0, 3.0, -8.0], [4.0, 16.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -9.0]), &[ModelCube::of([0.0, 0.0], [-2.5, -2.0, -3.0], [5.0, 4.0, 5.0]), ModelCube::of([0.0, 24.0], [-1.5, -0.001, -4.0], [3.0, 2.0, 2.0]), ModelCube::of([0.0, 10.0], [-2.0, -3.0, 0.0], [1.0, 1.0, 2.0]), ModelCube::of([6.0, 10.0], [1.0, -3.0, 0.0], [1.0, 1.0, 2.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 15.0, 8.0], [0.9, 0.0, 0.0]), &[ModelCube::of([0.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0])], &[]),
        ModelPart::of("tail2", PartPose::offset([0.0, 20.0, 14.0]), &[ModelCube::of([4.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0]).grow([-0.02, -0.02, -0.02])], &[]),
    ])),
    ModelLayer::of("minecraft:cat_baby#collar", 32, 32, ModelPart::of("root", PartPose::new([0.0, -0.24015978, 0.0], [0.0, 0.0, 0.0], [1.01, 1.01, 1.01]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.5, 0.5]), &[ModelCube::of([0.0, 8.0], [-2.0, -1.5, -3.5], [4.0, 3.0, 7.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -3.125]), &[ModelCube::of([0.0, 0.0], [-2.5, -3.0, -2.875], [5.0, 4.0, 4.0]), ModelCube::of([18.0, 0.0], [-2.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([24.0, 0.0], [1.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([18.0, 3.0], [-1.5, -1.0, -3.875], [3.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.0, 22.0, -1.5]), &[ModelCube::of([18.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.0, 22.0, 2.5]), &[ModelCube::of([18.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.0, 22.0, -1.5]), &[ModelCube::of([12.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.0, 22.0, 2.5]), &[ModelCube::of([12.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 19.107, 3.9151], [-0.567232, 0.0, 0.0]), &[ModelCube::of([0.0, 18.0], [-0.5, -0.107, 0.0849], [1.0, 1.0, 5.0])], &[]),
        ModelPart::of("tail2", PartPose::ZERO, &[], &[]),
    ])),
    ModelLayer::of("minecraft:cat_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.5, 0.5]), &[ModelCube::of([0.0, 8.0], [-2.0, -1.5, -3.5], [4.0, 3.0, 7.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -3.125]), &[ModelCube::of([0.0, 0.0], [-2.5, -3.0, -2.875], [5.0, 4.0, 4.0]), ModelCube::of([18.0, 0.0], [-2.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([24.0, 0.0], [1.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([18.0, 3.0], [-1.5, -1.0, -3.875], [3.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.0, 22.0, -1.5]), &[ModelCube::of([18.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.0, 22.0, 2.5]), &[ModelCube::of([18.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.0, 22.0, -1.5]), &[ModelCube::of([12.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.0, 22.0, 2.5]), &[ModelCube::of([12.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 19.107, 3.9151], [-0.567232, 0.0, 0.0]), &[ModelCube::of([0.0, 18.0], [-0.5, -0.107, 0.0849], [1.0, 1.0, 5.0])], &[]),
        ModelPart::of("tail2", PartPose::ZERO, &[], &[]),
    ])),
    ModelLayer::of("minecraft:cave_spider#main", 64, 32, ModelPart::of("root", PartPose::new([0.0, 7.2048006, 0.0], [0.0, 0.0, 0.0], [0.7, 0.7, 0.7]), &[], &[
        ModelPart::of("body0", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.0, -3.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("body1", PartPose::offset([0.0, 15.0, 9.0]), &[ModelCube::of([0.0, 12.0], [-5.0, -4.0, -6.0], [10.0, 8.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -3.0]), &[ModelCube::of([32.0, 4.0], [-4.0, -4.0, -8.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset_and_rotation([4.0, 15.0, -1.0], [0.0, 0.7853982, 0.7853982]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset_and_rotation([4.0, 15.0, 2.0], [0.0, -0.7853982, 0.7853982]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_middle_front_leg", PartPose::offset_and_rotation([4.0, 15.0, 0.0], [0.0, 0.3926991, 0.58119464]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_middle_hind_leg", PartPose::offset_and_rotation([4.0, 15.0, 1.0], [0.0, -0.3926991, 0.58119464]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset_and_rotation([-4.0, 15.0, -1.0], [0.0, -0.7853982, -0.7853982]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset_and_rotation([-4.0, 15.0, 2.0], [0.0, 0.7853982, -0.7853982]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_middle_front_leg", PartPose::offset_and_rotation([-4.0, 15.0, 0.0], [0.0, -0.3926991, -0.58119464]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_middle_hind_leg", PartPose::offset_and_rotation([-4.0, 15.0, 1.0], [0.0, 0.3926991, -0.58119464]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::ZERO, &[ModelCube::of([0.0, 19.0], [1.0, 0.0, 1.0], [14.0, 10.0, 14.0])], &[]),
        ModelPart::of("lid", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [1.0, 0.0, 0.0], [14.0, 5.0, 14.0])], &[]),
        ModelPart::of("lock", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [7.0, -2.0, 14.0], [2.0, 4.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/acacia#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/bamboo#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, -2.1, 1.0], [1.5708, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -11.0, -4.0], [28.0, 20.0, 4.0]), ModelCube::of([0.0, 0.0], [-14.0, -9.0, -8.0], [28.0, 16.0, 4.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -10.1, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -14.1, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -11.1, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -4.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([0.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([0.0, 24.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -4.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([40.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([40.0, 24.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/birch#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/cherry#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/dark_oak#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/jungle#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/mangrove#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/oak#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/pale_oak#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_boat/spruce#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([-15.0, 4.0, 4.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 19.0], [-13.0, -7.0, -1.0], [18.0, 6.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 3.0, 1.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-14.0, -9.0, -3.0], [28.0, 16.0, 3.0])], &[]),
        ModelPart::of("chest_bottom", PartPose::offset_and_rotation([-2.0, -5.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [12.0, 8.0, 12.0])], &[]),
        ModelPart::of("chest_lid", PartPose::offset_and_rotation([-2.0, -9.0, -6.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("chest_lock", PartPose::offset_and_rotation([-1.0, -6.0, -1.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 59.0], [0.0, 0.0, 0.0], [2.0, 4.0, 1.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([15.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 27.0], [-8.0, -7.0, -1.0], [16.0, 6.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset([0.0, 4.0, 9.0]), &[ModelCube::of([0.0, 43.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("left_paddle", PartPose::offset_and_rotation([3.0, -5.0, 9.0], [0.0, 0.0, 0.19634955]), &[ModelCube::of([62.0, 0.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 0.0], [-1.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([0.0, 4.0, -9.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 35.0], [-14.0, -7.0, -1.0], [28.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_paddle", PartPose::offset_and_rotation([3.0, -5.0, -9.0], [0.0, 3.1415927, 0.19634955]), &[ModelCube::of([62.0, 20.0], [-1.0, 0.0, -5.0], [2.0, 2.0, 18.0]), ModelCube::of([62.0, 20.0], [0.001, -3.0, 8.0], [1.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chest_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chicken#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 16.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 9.0], [-3.0, -4.0, -3.0], [6.0, 8.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -6.0, -2.0], [4.0, 6.0, 3.0])], &[
            ModelPart::of("beak", PartPose::ZERO, &[ModelCube::of([14.0, 0.0], [-2.0, -4.0, -4.0], [4.0, 2.0, 2.0])], &[]),
            ModelPart::of("red_thing", PartPose::ZERO, &[ModelCube::of([14.0, 4.0], [-1.0, -2.0, -3.0], [2.0, 2.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 19.0, 1.0]), &[ModelCube::of([26.0, 0.0], [-1.0, 0.0, -3.0], [3.0, 5.0, 3.0])], &[]),
        ModelPart::of("left_wing", PartPose::offset([4.0, 13.0, 0.0]), &[ModelCube::of([24.0, 13.0], [-1.0, 0.0, -3.0], [1.0, 4.0, 6.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 19.0, 1.0]), &[ModelCube::of([26.0, 0.0], [-1.0, 0.0, -3.0], [3.0, 5.0, 3.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset([-4.0, 13.0, 0.0]), &[ModelCube::of([24.0, 13.0], [0.0, 0.0, -3.0], [1.0, 4.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:chicken_baby#main", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.25, -1.25]), &[ModelCube::of([0.0, 0.0], [-2.0, -2.25, -0.75], [4.0, 4.0, 4.0]), ModelCube::of([10.0, 8.0], [-1.0, -0.25, -1.75], [2.0, 1.0, 1.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 22.0, 0.5]), &[ModelCube::of([2.0, 2.0], [-0.5, 0.0, 0.0], [1.0, 2.0, 0.0]), ModelCube::of([0.0, 1.0], [-0.5, 2.0, -1.0], [1.0, 0.0, 1.0])], &[]),
        ModelPart::of("left_wing", PartPose::offset([-2.0, 20.0, 0.0]), &[ModelCube::of([4.0, 8.0], [-1.0, 0.0, -1.0], [1.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 22.0, 0.5]), &[ModelCube::of([0.0, 2.0], [-0.5, 0.0, 0.0], [1.0, 2.0, 0.0]), ModelCube::of([0.0, 0.0], [-0.5, 2.0, -1.0], [1.0, 0.0, 1.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset([2.0, 20.0, 0.0]), &[ModelCube::of([6.0, 8.0], [0.0, 0.0, -1.0], [1.0, 0.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cod#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -2.0, 0.0], [2.0, 4.0, 7.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([11.0, 0.0], [-1.0, -2.0, -3.0], [2.0, 4.0, 3.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.0, 23.0, 0.0], [0.0, 0.0, 0.7853982]), &[ModelCube::of([22.0, 4.0], [0.0, 0.0, -1.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("nose", PartPose::offset([0.0, 22.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -2.0, -1.0], [2.0, 3.0, 1.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.0, 23.0, 0.0], [0.0, 0.0, -0.7853982]), &[ModelCube::of([22.0, 1.0], [-2.0, 0.0, -1.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("tail_fin", PartPose::offset([0.0, 22.0, 7.0]), &[ModelCube::of([22.0, 3.0], [0.0, -2.0, 0.0], [0.0, 4.0, 4.0])], &[]),
        ModelPart::of("top_fin", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([20.0, -6.0], [0.0, -1.0, -1.0], [0.0, 1.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cold_chicken#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 16.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 9.0], [-3.0, -4.0, -3.0], [6.0, 8.0, 6.0]), ModelCube::of([38.0, 9.0], [0.0, 3.0, -1.0], [0.0, 3.0, 5.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -6.0, -2.0], [4.0, 6.0, 3.0]), ModelCube::of([44.0, 0.0], [-3.0, -7.0, -2.015], [6.0, 3.0, 4.0])], &[
            ModelPart::of("beak", PartPose::ZERO, &[ModelCube::of([14.0, 0.0], [-2.0, -4.0, -4.0], [4.0, 2.0, 2.0])], &[]),
            ModelPart::of("red_thing", PartPose::ZERO, &[ModelCube::of([14.0, 4.0], [-1.0, -2.0, -3.0], [2.0, 2.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 19.0, 1.0]), &[ModelCube::of([26.0, 0.0], [-1.0, 0.0, -3.0], [3.0, 5.0, 3.0])], &[]),
        ModelPart::of("left_wing", PartPose::offset([4.0, 13.0, 0.0]), &[ModelCube::of([24.0, 13.0], [-1.0, 0.0, -3.0], [1.0, 4.0, 6.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 19.0, 1.0]), &[ModelCube::of([26.0, 0.0], [-1.0, 0.0, -3.0], [3.0, 5.0, 3.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset([-4.0, 13.0, 0.0]), &[ModelCube::of([24.0, 13.0], [0.0, 0.0, -3.0], [1.0, 4.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cold_cow#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([20.0, 32.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([18.0, 4.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]), ModelCube::of([52.0, 0.0], [-2.0, 2.0, -8.0], [4.0, 6.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -6.0], [8.0, 8.0, 6.0]), ModelCube::of([9.0, 33.0], [-3.0, 1.0, -7.0], [6.0, 3.0, 1.0])], &[
            ModelPart::of("left_horn", PartPose::offset_and_rotation([5.5, -2.5, -5.0], [1.5708, 0.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-1.5, -3.0, -0.5], [2.0, 6.0, 2.0])], &[]),
            ModelPart::of("right_horn", PartPose::offset_and_rotation([-4.5, -2.5, -3.5], [1.5708, 0.0, 0.0]), &[ModelCube::of([0.0, 40.0], [-1.5, -4.5, -0.5], [2.0, 6.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cold_cow_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([3.0, 19.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -7.0, -1.0], [8.0, 6.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 13.569, -5.1667]), &[ModelCube::of([0.0, 18.0], [-3.0, -4.569, -4.8333], [6.0, 6.0, 5.0]), ModelCube::of([8.0, 29.0], [3.0, -5.569, -3.8333], [1.0, 2.0, 1.0]), ModelCube::of([4.0, 29.0], [-4.0, -5.569, -3.8333], [1.0, 2.0, 1.0]).mirror(), ModelCube::of([12.0, 29.0], [-2.0, -1.569, -5.8333], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -3.5]), &[ModelCube::of([34.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 3.5]), &[ModelCube::of([34.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -3.5]), &[ModelCube::of([22.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 3.5]), &[ModelCube::of([22.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cold_pig#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 11.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-5.0, -10.0, -7.0], [10.0, 16.0, 8.0]), ModelCube::of([28.0, 32.0], [-5.0, -10.0, -7.0], [10.0, 16.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -8.0], [8.0, 8.0, 8.0]), ModelCube::of([16.0, 16.0], [-2.0, 0.0, -9.0], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:command_block_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:conduit#cage", 32, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("shell", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:conduit#eye", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("eye", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, 0.0], [8.0, 8.0, 0.0]).grow([0.01, 0.01, 0.01])], &[]),
    ])),
    ModelLayer::of("minecraft:conduit#shell", 32, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("shell", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-3.0, -3.0, -3.0], [6.0, 6.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:conduit#wind", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("wind", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:copper_golem#eyes", 64, 64, ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -5.0, 0.0]), &[], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0, 0.0]), &[], &[]),
            ModelPart::of("left_arm", PartPose::offset([4.0, -6.0, 0.0]), &[], &[]),
            ModelPart::of("right_arm", PartPose::offset([-4.0, -6.0, 0.0]), &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([0.0, -5.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([0.0, -5.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:copper_golem#main", 64, 64, ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -5.0, 0.0]), &[ModelCube::of([0.0, 15.0], [-4.0, -6.0, -3.0], [8.0, 6.0, 6.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -5.0, -5.0], [8.0, 5.0, 10.0]).grow([0.015, 0.015, 0.015]), ModelCube::of([56.0, 0.0], [-1.0, -2.0, -6.0], [2.0, 3.0, 2.0]), ModelCube::of([37.0, 8.0], [-1.0, -9.0, -1.0], [2.0, 4.0, 2.0]).grow([-0.015, -0.015, -0.015]), ModelCube::of([37.0, 0.0], [-2.0, -13.0, -2.0], [4.0, 4.0, 4.0]).grow([-0.015, -0.015, -0.015])], &[]),
            ModelPart::of("left_arm", PartPose::offset([4.0, -6.0, 0.0]), &[ModelCube::of([50.0, 16.0], [0.0, -1.0, -2.0], [3.0, 10.0, 4.0])], &[]),
            ModelPart::of("right_arm", PartPose::offset([-4.0, -6.0, 0.0]), &[ModelCube::of([36.0, 16.0], [-3.0, -1.0, -2.0], [3.0, 10.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([0.0, -5.0, 0.0]), &[ModelCube::of([16.0, 27.0], [0.0, 0.0, -2.0], [4.0, 5.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([0.0, -5.0, 0.0]), &[ModelCube::of([0.0, 27.0], [-4.0, 0.0, -2.0], [4.0, 5.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:copper_golem_running#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([-1.064, -5.0, 0.0]), &[], &[
            ModelPart::of("body_r1", PartPose::offset_and_rotation([1.1, 0.1, 0.7], [0.1204, -0.0064, -0.0779]), &[ModelCube::of([0.0, 15.0], [-4.02, -6.116, -3.5], [8.0, 6.0, 6.0])], &[]),
            ModelPart::of("head", PartPose::offset([0.7, -5.6, -1.8]), &[ModelCube::of([0.0, 0.0], [-4.0, -5.1, -5.0], [8.0, 5.0, 10.0]), ModelCube::of([56.0, 0.0], [-1.02, -2.1, -6.0], [2.0, 3.0, 2.0]), ModelCube::of([37.0, 8.0], [-1.02, -9.1, -1.0], [2.0, 4.0, 2.0]).grow([-0.015, -0.015, -0.015]), ModelCube::of([37.0, 0.0], [-2.0, -13.1, -2.0], [4.0, 4.0, 4.0]).grow([-0.015, -0.015, -0.015])], &[]),
            ModelPart::of("left_arm", PartPose::offset([4.0, -6.0, 0.0]), &[], &[
                ModelPart::of("left_arm_r1", PartPose::offset_and_rotation([0.732, 0.0, 0.0], [-0.8715, -0.0535, -0.0449]), &[ModelCube::of([50.0, 16.0], [0.032, -1.1, -2.0], [3.0, 10.0, 4.0])], &[]),
            ]),
            ModelPart::of("right_arm", PartPose::offset([-4.0, -6.0, 0.0]), &[], &[
                ModelPart::of("right_arm_r1", PartPose::offset_and_rotation([0.7, -0.248, -1.62], [1.0036, 0.0, 0.0]), &[ModelCube::of([36.0, 16.0], [-3.052, -1.11, -2.036], [3.0, 10.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([0.936, -5.0, 0.0]), &[], &[
            ModelPart::of("left_leg_r1", PartPose::offset_and_rotation([1.0, 0.0, 0.0], [0.7854, 0.0, 0.0]), &[ModelCube::of([16.0, 27.0], [-2.088, -0.1, -2.0], [4.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-3.064, -5.0, 0.0]), &[], &[
            ModelPart::of("right_leg_r1", PartPose::offset_and_rotation([1.048, 0.0, -0.9], [-0.8727, 0.0, 0.0]), &[ModelCube::of([0.0, 27.0], [-1.856, -0.1, -1.09], [4.0, 5.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:copper_golem_sitting#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -3.0, 2.325]), &[ModelCube::of([3.0, 19.0], [-3.0, -4.0, -4.525], [6.0, 1.0, 6.0]), ModelCube::of([0.0, 15.0], [-4.0, -3.0, -3.525], [8.0, 6.0, 6.0])], &[
            ModelPart::of("body_r1", PartPose::offset_and_rotation([0.0, -1.0, -4.325], [0.0, 0.0, -3.1416]), &[ModelCube::of([3.0, 18.0], [-4.0, -3.0, -2.2], [8.0, 6.0, 3.0])], &[]),
            ModelPart::of("head", PartPose::offset([0.0, -6.0, -0.2]), &[ModelCube::of([37.0, 8.0], [-1.0, -7.0, -3.3], [2.0, 4.0, 2.0]).grow([-0.015, -0.015, -0.015]), ModelCube::of([37.0, 0.0], [-2.0, -11.0, -4.3], [4.0, 4.0, 4.0]).grow([-0.015, -0.015, -0.015]), ModelCube::of([0.0, 0.0], [-4.0, -3.0, -7.325], [8.0, 5.0, 10.0]), ModelCube::of([56.0, 0.0], [-1.0, 0.0, -8.325], [2.0, 3.0, 2.0])], &[]),
            ModelPart::of("left_arm", PartPose::offset_and_rotation([4.0, -5.6, -1.7], [0.4363, 0.0, 0.0]), &[], &[
                ModelPart::of("left_arm_r1", PartPose::offset_and_rotation([0.0, -0.0015, -0.0808], [-1.0472, 0.0, 0.0]), &[ModelCube::of([50.0, 16.0], [0.075, -1.0443, -1.8997], [3.0, 10.0, 4.0])], &[]),
            ]),
            ModelPart::of("right_arm", PartPose::offset_and_rotation([-4.0, -5.6, -1.8], [0.4363, 0.0, 0.0]), &[], &[
                ModelPart::of("right_arm_r1", PartPose::offset_and_rotation([0.0, 0.0893, 0.1198], [-1.0472, 0.0, 0.0]), &[ModelCube::of([36.0, 16.0], [-3.075, -0.9733, -1.9966], [3.0, 10.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([2.0, -2.0, -2.075]), &[], &[
            ModelPart::of("left_leg_r1", PartPose::offset_and_rotation([0.05, -2.0, 1.075], [-1.5708, 0.0, 0.0]), &[ModelCube::of([16.0, 27.0], [-2.0, 0.975, 0.0], [4.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-2.1, -2.1, -2.075]), &[], &[
            ModelPart::of("right_leg_r1", PartPose::offset_and_rotation([0.05, -1.9, 1.075], [-1.5708, 0.0, 0.0]), &[ModelCube::of([0.0, 27.0], [-2.0, 0.975, 0.0], [4.0, 5.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:copper_golem_star#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -5.0, 0.0]), &[ModelCube::of([0.0, 15.0], [-4.0, -6.0, -3.0], [8.0, 6.0, 6.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -5.0, -5.0], [8.0, 5.0, 10.0]), ModelCube::of([56.0, 0.0], [-1.0, -2.0, -6.0], [2.0, 3.0, 2.0]), ModelCube::of([37.0, 8.0], [-1.0, -9.0, -1.0], [2.0, 4.0, 2.0]).grow([-0.015, -0.015, -0.015]), ModelCube::of([37.0, 0.0], [-2.0, -13.0, -2.0], [4.0, 4.0, 4.0]).grow([-0.015, -0.015, -0.015])], &[]),
            ModelPart::of("left_arm", PartPose::offset([4.0, -6.0, 0.0]), &[], &[
                ModelPart::of("left_arm_r1", PartPose::offset_and_rotation([-1.0, 1.0, 0.0], [0.0, 0.0, -1.9199]), &[ModelCube::of([50.0, 16.0], [-1.5, -5.0, -2.0], [3.0, 10.0, 4.0])], &[]),
            ]),
            ModelPart::of("right_arm", PartPose::offset([-4.0, -6.0, 0.0]), &[], &[
                ModelPart::of("rightItem", PartPose::offset([-1.0, 7.4, -1.0]), &[], &[]),
                ModelPart::of("right_arm_r1", PartPose::offset_and_rotation([1.0, 1.0, 0.0], [0.0, 0.0, 1.9199]), &[ModelCube::of([36.0, 16.0], [-1.5, -5.0, -2.0], [3.0, 10.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.0, -5.0, 0.0]), &[], &[
            ModelPart::of("left_leg_r1", PartPose::offset_and_rotation([1.65, 2.0, 0.0], [0.0, 0.0, -0.2618]), &[ModelCube::of([16.0, 27.0], [-2.0, -2.5, -2.0], [4.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-3.0, -5.0, 0.0]), &[], &[
            ModelPart::of("right_leg_r1", PartPose::offset_and_rotation([0.35, 2.0, 0.01], [0.0, 0.0, 0.2618]), &[ModelCube::of([0.0, 27.0], [-2.0, -2.5, -2.0], [4.0, 5.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:cow#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([18.0, 4.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]), ModelCube::of([52.0, 0.0], [-2.0, 2.0, -8.0], [4.0, 6.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -6.0], [8.0, 8.0, 6.0]), ModelCube::of([1.0, 33.0], [-3.0, 1.0, -7.0], [6.0, 3.0, 1.0]), ModelCube::of([22.0, 0.0], [-5.0, -5.0, -5.0], [1.0, 3.0, 1.0]), ModelCube::of([22.0, 0.0], [4.0, -5.0, -5.0], [1.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:cow_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([3.0, 19.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -7.0, -1.0], [8.0, 6.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 13.569, -5.1667]), &[ModelCube::of([0.0, 18.0], [-3.0, -4.569, -4.8333], [6.0, 6.0, 5.0]), ModelCube::of([8.0, 29.0], [3.0, -5.569, -3.8333], [1.0, 2.0, 1.0]), ModelCube::of([4.0, 29.0], [-4.0, -5.569, -3.8333], [1.0, 2.0, 1.0]).mirror(), ModelCube::of([12.0, 29.0], [-2.0, -1.569, -5.8333], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -3.5]), &[ModelCube::of([34.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 3.5]), &[ModelCube::of([34.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -3.5]), &[ModelCube::of([22.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 3.5]), &[ModelCube::of([22.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:creaking#eyes", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("left_leg", PartPose::offset([1.5, -16.0, 0.5]), &[], &[]),
            ModelPart::of("right_leg", PartPose::offset([-1.0, -17.5, 0.5]), &[], &[]),
            ModelPart::of("upper_body", PartPose::offset([-1.0, -19.0, 0.0]), &[], &[
                ModelPart::of("body", PartPose::offset([0.0, -7.0, 1.0]), &[], &[]),
                ModelPart::of("head", PartPose::offset([-3.0, -11.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -10.0, -3.0], [6.0, 10.0, 6.0]), ModelCube::of([28.0, 31.0], [-3.0, -13.0, -3.0], [6.0, 3.0, 6.0]), ModelCube::of([12.0, 40.0], [3.0, -13.0, 0.0], [9.0, 14.0, 0.0]), ModelCube::of([34.0, 12.0], [-12.0, -14.0, 0.0], [9.0, 14.0, 0.0])], &[]),
                ModelPart::of("left_arm", PartPose::offset([6.0, -9.0, 0.5]), &[], &[]),
                ModelPart::of("right_arm", PartPose::offset([-7.0, -9.5, 1.5]), &[], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:creaking#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("left_leg", PartPose::offset([1.5, -16.0, 0.5]), &[ModelCube::of([42.0, 40.0], [-1.5, 0.0, -1.5], [3.0, 16.0, 3.0]), ModelCube::of([45.0, 55.0], [-1.5, 15.7, -4.5], [5.0, 0.0, 9.0])], &[]),
            ModelPart::of("right_leg", PartPose::offset([-1.0, -17.5, 0.5]), &[ModelCube::of([0.0, 34.0], [-3.0, -1.5, -1.5], [3.0, 19.0, 3.0]), ModelCube::of([45.0, 46.0], [-5.0, 17.2, -4.5], [5.0, 0.0, 9.0]), ModelCube::of([12.0, 34.0], [-3.0, -4.5, -1.5], [3.0, 3.0, 3.0])], &[]),
            ModelPart::of("upper_body", PartPose::offset([-1.0, -19.0, 0.0]), &[], &[
                ModelPart::of("body", PartPose::offset([0.0, -7.0, 1.0]), &[ModelCube::of([0.0, 16.0], [0.0, -3.0, -3.0], [6.0, 13.0, 5.0]), ModelCube::of([24.0, 0.0], [-6.0, -4.0, -3.0], [6.0, 7.0, 5.0])], &[]),
                ModelPart::of("head", PartPose::offset([-3.0, -11.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -10.0, -3.0], [6.0, 10.0, 6.0]), ModelCube::of([28.0, 31.0], [-3.0, -13.0, -3.0], [6.0, 3.0, 6.0]), ModelCube::of([12.0, 40.0], [3.0, -13.0, 0.0], [9.0, 14.0, 0.0]), ModelCube::of([34.0, 12.0], [-12.0, -14.0, 0.0], [9.0, 14.0, 0.0])], &[]),
                ModelPart::of("left_arm", PartPose::offset([6.0, -9.0, 0.5]), &[ModelCube::of([30.0, 40.0], [0.0, -1.0, -1.5], [3.0, 16.0, 3.0]), ModelCube::of([52.0, 12.0], [0.0, -5.0, -1.5], [3.0, 4.0, 3.0]), ModelCube::of([52.0, 19.0], [0.0, 15.0, -1.5], [3.0, 4.0, 3.0])], &[]),
                ModelPart::of("right_arm", PartPose::offset([-7.0, -9.5, 1.5]), &[ModelCube::of([22.0, 13.0], [-2.0, -1.5, -1.5], [3.0, 21.0, 3.0]), ModelCube::of([46.0, 0.0], [-2.0, 19.5, -1.5], [3.0, 4.0, 3.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:creeper#armor", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 6.0, 0.0]), &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([2.0, 2.0, 2.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 6.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 18.0, -4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 18.0, 4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 18.0, -4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 18.0, 4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([2.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:creeper#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 6.0, 0.0]), &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 6.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 18.0, -4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 18.0, 4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 18.0, -4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 18.0, 4.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:creeper_head#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:decorated_pot_base#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::offset([1.0, 0.0, 1.0]), &[ModelCube::of([-14.0, 13.0], [0.0, 0.0, 0.0], [14.0, 0.0, 14.0])], &[]),
        ModelPart::of("neck", PartPose::offset_and_rotation([0.0, 37.0, 16.0], [3.1415927, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [4.0, 17.0, 4.0], [8.0, 3.0, 8.0]).grow([-0.1, -0.1, -0.1]), ModelCube::of([0.0, 5.0], [5.0, 20.0, 5.0], [6.0, 1.0, 6.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("top", PartPose::offset([1.0, 16.0, 1.0]), &[ModelCube::of([-14.0, 13.0], [0.0, 0.0, 0.0], [14.0, 0.0, 14.0])], &[]),
    ])),
    ModelLayer::of("minecraft:decorated_pot_sides#main", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([15.0, 16.0, 1.0], [0.0, 0.0, 3.1415927]), &[ModelCube::of([1.0, 0.0], [0.0, 0.0, 0.0], [14.0, 16.0, 0.0]).faces(&[Face::North])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([1.0, 16.0, 15.0], [3.1415927, 0.0, 0.0]), &[ModelCube::of([1.0, 0.0], [0.0, 0.0, 0.0], [14.0, 16.0, 0.0]).faces(&[Face::North])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([1.0, 16.0, 1.0], [0.0, -1.5707964, 3.1415927]), &[ModelCube::of([1.0, 0.0], [0.0, 0.0, 0.0], [14.0, 16.0, 0.0]).faces(&[Face::North])], &[]),
        ModelPart::of("right", PartPose::offset_and_rotation([15.0, 16.0, 15.0], [0.0, 1.5707964, 3.1415927]), &[ModelCube::of([1.0, 0.0], [0.0, 0.0, 0.0], [14.0, 16.0, 0.0]).faces(&[Face::North])], &[]),
    ])),
    ModelLayer::of("minecraft:dolphin#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, -5.0]), &[ModelCube::of([22.0, 0.0], [-4.0, -7.0, 0.0], [8.0, 7.0, 13.0])], &[
            ModelPart::of("back_fin", PartPose::rotation([1.0471976, 0.0, 0.0]), &[ModelCube::of([51.0, 0.0], [-0.5, 0.0, 8.0], [1.0, 4.0, 5.0])], &[]),
            ModelPart::of("head", PartPose::offset([0.0, -4.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -3.0, -3.0], [8.0, 7.0, 6.0])], &[
                ModelPart::of("nose", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-1.0, 2.0, -7.0], [2.0, 2.0, 4.0])], &[]),
            ]),
            ModelPart::of("left_fin", PartPose::offset_and_rotation([2.0, -2.0, 4.0], [1.0471976, 0.0, 2.0943952]), &[ModelCube::of([48.0, 20.0], [-0.5, -4.0, 0.0], [1.0, 4.0, 7.0]).mirror()], &[]),
            ModelPart::of("right_fin", PartPose::offset_and_rotation([-2.0, -2.0, 4.0], [1.0471976, 0.0, -2.0943952]), &[ModelCube::of([48.0, 20.0], [-0.5, -4.0, 0.0], [1.0, 4.0, 7.0])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -2.5, 11.0], [-0.10471976, 0.0, 0.0]), &[ModelCube::of([0.0, 19.0], [-2.0, -2.5, 0.0], [4.0, 5.0, 11.0])], &[
                ModelPart::of("tail_fin", PartPose::offset([0.0, 0.0, 9.0]), &[ModelCube::of([19.0, 20.0], [-5.0, -0.5, 0.0], [10.0, 1.0, 6.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:dolphin_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 21.5, 0.0]), &[ModelCube::of([20.0, 0.0], [-3.0, -2.5, -4.0], [6.0, 5.0, 8.0])], &[
            ModelPart::of("back_fin", PartPose::offset_and_rotation([0.0, -1.0, -2.7], [0.8727, 0.0, 0.0]), &[ModelCube::of([42.0, 0.0], [-0.5, -1.0, 1.0], [1.0, 3.0, 4.0])], &[]),
            ModelPart::of("head", PartPose::offset([0.0, 1.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.5, -4.0], [6.0, 5.0, 4.0])], &[
                ModelPart::of("nose", PartPose::offset([0.0, 0.5, -4.0]), &[ModelCube::of([0.0, 9.0], [-1.0, -1.0, -2.0], [2.0, 2.0, 2.0])], &[]),
            ]),
            ModelPart::of("left_fin", PartPose::offset_and_rotation([1.8, 0.85, -2.6], [0.8727, 0.0, 1.7017]), &[ModelCube::of([34.0, 18.0], [-0.5, -1.5, -0.5], [1.0, 3.0, 6.0])], &[]),
            ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.8, 0.85, -2.6], [0.8727, 0.0, -1.7017]), &[ModelCube::of([48.0, 18.0], [-0.5, -1.5, -0.5], [1.0, 3.0, 6.0]).mirror()], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, 1.0, 4.0]), &[ModelCube::of([0.0, 13.0], [-2.0, -1.5, 0.0], [4.0, 3.0, 7.0])], &[
                ModelPart::of("tail_fin", PartPose::offset([0.0, 0.0, 6.0]), &[ModelCube::of([22.0, 13.0], [-4.0, -0.5, -1.0], [8.0, 1.0, 4.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:donkey#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 3.12208, 0.0], [0.0, 0.0, 0.0], [0.87, 0.87, 0.87]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("left_chest", PartPose::offset_and_rotation([6.0, -8.0, 0.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset_and_rotation([-6.0, -8.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([1.25, -10.0, 4.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.25, -10.0, 4.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:donkey#saddle", 64, 64, ModelPart::of("root", PartPose::new([0.0, 3.12208, 0.0], [0.0, 0.0, 0.0], [0.87, 0.87, 0.87]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("left_chest", PartPose::offset_and_rotation([6.0, -8.0, 0.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset_and_rotation([-6.0, -8.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-5.0, -8.0, -9.0], [10.0, 9.0, 9.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([1.25, -10.0, 4.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.25, -10.0, 4.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
            ]),
            ModelPart::of("head_saddle", PartPose::ZERO, &[ModelCube::of([1.0, 1.0], [-3.0, -11.0, -1.9], [6.0, 5.0, 6.0]).grow([0.22, 0.22, 0.22])], &[]),
            ModelPart::of("left_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("left_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [2.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("mouth_saddle_wrap", PartPose::ZERO, &[ModelCube::of([19.0, 0.0], [-2.0, -11.0, -4.0], [4.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("right_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [-3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("right_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [-3.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:donkey_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([1.0, 14.0, 0.0]), &[ModelCube::of([0.0, 13.0], [-5.0, -3.0, -7.0], [8.0, 6.0, 14.0])], &[
            ModelPart::of("head_parts", PartPose::offset([0.0, -3.0, -5.0]), &[], &[
                ModelPart::of("head", PartPose::offset([0.0, -5.0, -3.0]), &[], &[
                    ModelPart::of("head_r1", PartPose::offset_and_rotation([0.0, -1.0, 1.0], [0.3927, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -3.6, -8.4], [6.0, 4.0, 9.0])], &[]),
                    ModelPart::of("left_ear", PartPose::offset_and_rotation([2.0, -3.5, -1.0], [0.48, 0.0, 0.48]), &[ModelCube::of([0.0, 0.0], [-2.0, -6.5, -0.3], [2.0, 7.0, 1.0])], &[]),
                    ModelPart::of("right_ear", PartPose::offset_and_rotation([-2.0, -3.5, -1.0], [0.48, 0.0, -0.48]), &[ModelCube::of([22.0, 0.0], [-2.0, -6.5, -0.3], [2.0, 7.0, 1.0]).mirror()], &[]),
                ]),
                ModelPart::of("neck_r1", PartPose::rotation([0.3927, 0.0, 0.0]), &[ModelCube::of([30.0, 9.0], [-3.0, -6.0, -3.0], [4.0, 8.0, 4.0])], &[]),
            ]),
            ModelPart::of("left_chest", PartPose::offset([-1.0, 10.0, 0.0]), &[], &[]),
            ModelPart::of("left_front_leg", PartPose::offset([2.4, 3.5, -5.3]), &[ModelCube::of([12.0, 33.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("left_hind_leg", PartPose::offset([2.25, 3.5, 5.25]), &[ModelCube::of([12.0, 44.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset([-1.0, 10.0, 0.0]), &[], &[]),
            ModelPart::of("right_front_leg", PartPose::offset([-2.4, 3.5, -5.4]), &[ModelCube::of([0.0, 33.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_hind_leg", PartPose::offset([-2.4, 3.5, 5.4]), &[ModelCube::of([0.0, 44.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, -1.5, 6.5]), &[], &[
                ModelPart::of("tail_r1", PartPose::rotation([-0.7418, 0.0, 0.0]), &[ModelCube::of([24.0, 33.0], [-2.5, -1.0, -0.5], [3.0, 3.0, 8.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:double_chest_left#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::ZERO, &[ModelCube::of([0.0, 19.0], [0.0, 0.0, 1.0], [15.0, 10.0, 14.0]).faces(&[Face::Down, Face::East, Face::North, Face::South, Face::Up])], &[]),
        ModelPart::of("lid", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [15.0, 5.0, 14.0]).faces(&[Face::Down, Face::East, Face::North, Face::South, Face::Up])], &[]),
        ModelPart::of("lock", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [0.0, -2.0, 14.0], [1.0, 4.0, 1.0]).faces(&[Face::Down, Face::East, Face::North, Face::South, Face::Up])], &[]),
    ])),
    ModelLayer::of("minecraft:double_chest_right#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bottom", PartPose::ZERO, &[ModelCube::of([0.0, 19.0], [1.0, 0.0, 1.0], [15.0, 10.0, 14.0]).faces(&[Face::Down, Face::North, Face::South, Face::Up, Face::West])], &[]),
        ModelPart::of("lid", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [1.0, 0.0, 0.0], [15.0, 5.0, 14.0]).faces(&[Face::Down, Face::North, Face::South, Face::Up, Face::West])], &[]),
        ModelPart::of("lock", PartPose::offset([0.0, 9.0, 1.0]), &[ModelCube::of([0.0, 0.0], [15.0, -2.0, 14.0], [1.0, 4.0, 1.0]).faces(&[Face::Down, Face::North, Face::South, Face::Up, Face::West])], &[]),
    ])),
    ModelLayer::of("minecraft:dragon_skull#main", 256, 256, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::new([0.0, -5.99, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[ModelCube::of([176.0, 44.0], [-6.0, -1.0, -24.0], [12.0, 5.0, 16.0]), ModelCube::of([112.0, 30.0], [-8.0, -8.0, -10.0], [16.0, 16.0, 16.0]), ModelCube::of([0.0, 0.0], [-5.0, -12.0, -4.0], [2.0, 4.0, 6.0]).mirror(), ModelCube::of([112.0, 0.0], [-5.0, -3.0, -22.0], [2.0, 2.0, 4.0]).mirror(), ModelCube::of([0.0, 0.0], [3.0, -12.0, -4.0], [2.0, 4.0, 6.0]), ModelCube::of([112.0, 0.0], [3.0, -3.0, -22.0], [2.0, 2.0, 4.0])], &[
            ModelPart::of("jaw", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([176.0, 65.0], [-6.0, 0.0, -16.0], [12.0, 4.0, 16.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:drowned#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:drowned#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:drowned#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned#outer", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.75, 0.75, 0.75])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([-0.1, 0.5, 0.3])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([-0.2, 0.20000002, 0.20000002])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.5, 0.0]), &[ModelCube::of([16.0, 16.0], [-2.0, -2.5, -1.0], [4.0, 5.0, 2.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.25, 0.0]), &[ModelCube::of([3.0, 3.0], [-3.0, -6.25, -3.0], [6.0, 6.0, 6.0]), ModelCube::of([35.0, 3.0], [-3.0, -6.15, -3.0], [6.0, 6.0, 6.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([28.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 20.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([36.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 20.0, 0.0]), &[ModelCube::of([8.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:drowned_baby#outer", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.5, 0.0]), &[ModelCube::of([16.0, 16.0], [-2.0, -2.5, -1.0], [4.0, 5.0, 2.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.25, 0.0]), &[ModelCube::of([3.0, 3.0], [-3.0, -6.25, -3.0], [6.0, 6.0, 6.0]), ModelCube::of([35.0, 3.0], [-3.0, -6.15, -3.0], [6.0, 6.0, 6.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([28.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 20.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([36.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 20.0, 0.0]), &[ModelCube::of([8.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0]).grow([0.25, 0.25, 0.25])], &[]),
    ])),
    ModelLayer::of("minecraft:elder_guardian#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -32.4216, 0.0], [0.0, 0.0, 0.0], [2.35, 2.35, 2.35]), &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-6.0, 10.0, -8.0], [12.0, 12.0, 16.0]), ModelCube::of([0.0, 28.0], [-8.0, 10.0, -6.0], [2.0, 12.0, 12.0]), ModelCube::of([0.0, 28.0], [6.0, 10.0, -6.0], [2.0, 12.0, 12.0]).mirror(), ModelCube::of([16.0, 40.0], [-6.0, 8.0, -6.0], [12.0, 2.0, 12.0]), ModelCube::of([16.0, 40.0], [-6.0, 22.0, -6.0], [12.0, 2.0, 12.0])], &[
            ModelPart::of("eye", PartPose::offset([0.0, 0.0, -8.25]), &[ModelCube::of([8.0, 0.0], [-1.0, 15.0, 0.0], [2.0, 2.0, 1.0])], &[]),
            ModelPart::of("spike0", PartPose::offset_and_rotation([0.0, 7.92, 8.08], [5.4977875, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike1", PartPose::offset_and_rotation([0.0, 7.9567738, -8.043226], [0.7853982, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike10", PartPose::offset_and_rotation([7.932871, 23.93287, 0.0], [0.0, 0.0, 2.3561945]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike11", PartPose::offset_and_rotation([-8.000353, 24.000353, 0.0], [0.0, 0.0, 3.926991]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike2", PartPose::offset_and_rotation([7.9667134, 8.033287, 0.0], [0.0, 0.0, 0.7853982]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike3", PartPose::offset_and_rotation([-7.9208007, 8.079199, 0.0], [0.0, 0.0, 5.4977875]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike4", PartPose::offset_and_rotation([-7.9477057, 16.0, -7.9477057], [1.5707964, 0.7853982, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike5", PartPose::offset_and_rotation([8.022686, 16.0, -8.022686], [1.5707964, 5.4977875, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike6", PartPose::offset_and_rotation([8.076813, 16.0, 8.076813], [1.5707964, 3.926991, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike7", PartPose::offset_and_rotation([-8.060315, 16.0, 8.060315], [1.5707964, 2.3561945, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike8", PartPose::offset_and_rotation([0.0, 23.988361, 7.9883604], [3.926991, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike9", PartPose::offset_and_rotation([0.0, 23.92711, -7.9271107], [2.3561945, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("tail0", PartPose::ZERO, &[ModelCube::of([40.0, 0.0], [-2.0, 14.0, 7.0], [4.0, 4.0, 8.0])], &[
                ModelPart::of("tail1", PartPose::offset([-1.5, 0.5, 14.0]), &[ModelCube::of([0.0, 54.0], [0.0, 14.0, 0.0], [3.0, 3.0, 7.0])], &[
                    ModelPart::of("tail2", PartPose::offset([0.5, 0.5, 6.0]), &[ModelCube::of([41.0, 32.0], [0.0, 14.0, 0.0], [2.0, 2.0, 6.0]), ModelCube::of([25.0, 19.0], [1.0, 10.5, 3.0], [1.0, 9.0, 9.0])], &[]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:elytra#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("left_wing", PartPose::offset_and_rotation([5.0, 0.0, 0.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([22.0, 0.0], [-10.0, 0.0, 0.0], [10.0, 20.0, 2.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset_and_rotation([-5.0, 0.0, 0.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([22.0, 0.0], [0.0, 0.0, 0.0], [10.0, 20.0, 2.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
    ])),
    ModelLayer::of("minecraft:elytra_baby#main", 64, 32, ModelPart::of("root", PartPose::new([0.0, 12.008, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[
        ModelPart::of("left_wing", PartPose::offset_and_rotation([5.0, 0.0, 0.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([22.0, 0.0], [-10.0, 0.0, 0.0], [10.0, 20.0, 2.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset_and_rotation([-5.0, 0.0, 0.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([22.0, 0.0], [0.0, 0.0, 0.0], [10.0, 20.0, 2.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
    ])),
    ModelLayer::of("minecraft:end_crystal#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base", PartPose::ZERO, &[ModelCube::of([0.0, 16.0], [-6.0, 0.0, -6.0], [12.0, 4.0, 12.0])], &[]),
        ModelPart::of("outer_glass", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("inner_glass", PartPose::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.875, 0.875, 0.875]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[
                ModelPart::of("cube", PartPose::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.765625, 0.765625, 0.765625]), &[ModelCube::of([32.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:ender_dragon#main", 256, 256, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 3.0, 8.0]), &[ModelCube::of([0.0, 0.0], [-12.0, 1.0, -16.0], [24.0, 24.0, 64.0]), ModelCube::of([220.0, 53.0], [-1.0, -5.0, -10.0], [2.0, 6.0, 12.0]), ModelCube::of([220.0, 53.0], [-1.0, -5.0, 10.0], [2.0, 6.0, 12.0]), ModelCube::of([220.0, 53.0], [-1.0, -5.0, 30.0], [2.0, 6.0, 12.0])], &[
            ModelPart::of("left_front_leg", PartPose::offset_and_rotation([12.0, 17.0, -6.0], [1.3, 0.0, 0.0]), &[ModelCube::of([112.0, 104.0], [-4.0, -4.0, -4.0], [8.0, 24.0, 8.0])], &[
                ModelPart::of("left_front_leg_tip", PartPose::offset_and_rotation([0.0, 20.0, -1.0], [-0.5, 0.0, 0.0]), &[ModelCube::of([226.0, 138.0], [-3.0, -1.0, -3.0], [6.0, 24.0, 6.0])], &[
                    ModelPart::of("left_front_foot", PartPose::offset_and_rotation([0.0, 23.0, 0.0], [0.75, 0.0, 0.0]), &[ModelCube::of([144.0, 104.0], [-4.0, 0.0, -12.0], [8.0, 4.0, 16.0])], &[]),
                ]),
            ]),
            ModelPart::of("left_hind_leg", PartPose::offset_and_rotation([16.0, 13.0, 34.0], [1.0, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -4.0, -8.0], [16.0, 32.0, 16.0])], &[
                ModelPart::of("left_hind_leg_tip", PartPose::offset_and_rotation([0.0, 32.0, -4.0], [0.5, 0.0, 0.0]), &[ModelCube::of([196.0, 0.0], [-6.0, -2.0, 0.0], [12.0, 32.0, 12.0])], &[
                    ModelPart::of("left_hind_foot", PartPose::offset_and_rotation([0.0, 31.0, 4.0], [0.75, 0.0, 0.0]), &[ModelCube::of([112.0, 0.0], [-9.0, 0.0, -20.0], [18.0, 6.0, 24.0])], &[]),
                ]),
            ]),
            ModelPart::of("left_wing", PartPose::offset([12.0, 2.0, -6.0]), &[ModelCube::of([112.0, 88.0], [0.0, -4.0, -4.0], [56.0, 8.0, 8.0]).mirror(), ModelCube::of([-56.0, 88.0], [0.0, 0.0, 2.0], [56.0, 0.0, 56.0]).mirror()], &[
                ModelPart::of("left_wing_tip", PartPose::offset([56.0, 0.0, 0.0]), &[ModelCube::of([112.0, 136.0], [0.0, -2.0, -2.0], [56.0, 4.0, 4.0]).mirror(), ModelCube::of([-56.0, 144.0], [0.0, 0.0, 2.0], [56.0, 0.0, 56.0]).mirror()], &[]),
            ]),
            ModelPart::of("right_front_leg", PartPose::offset_and_rotation([-12.0, 17.0, -6.0], [1.3, 0.0, 0.0]), &[ModelCube::of([112.0, 104.0], [-4.0, -4.0, -4.0], [8.0, 24.0, 8.0])], &[
                ModelPart::of("right_front_leg_tip", PartPose::offset_and_rotation([0.0, 20.0, -1.0], [-0.5, 0.0, 0.0]), &[ModelCube::of([226.0, 138.0], [-3.0, -1.0, -3.0], [6.0, 24.0, 6.0])], &[
                    ModelPart::of("right_front_foot", PartPose::offset_and_rotation([0.0, 23.0, 0.0], [0.75, 0.0, 0.0]), &[ModelCube::of([144.0, 104.0], [-4.0, 0.0, -12.0], [8.0, 4.0, 16.0])], &[]),
                ]),
            ]),
            ModelPart::of("right_hind_leg", PartPose::offset_and_rotation([-16.0, 13.0, 34.0], [1.0, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -4.0, -8.0], [16.0, 32.0, 16.0])], &[
                ModelPart::of("right_hind_leg_tip", PartPose::offset_and_rotation([0.0, 32.0, -4.0], [0.5, 0.0, 0.0]), &[ModelCube::of([196.0, 0.0], [-6.0, -2.0, 0.0], [12.0, 32.0, 12.0])], &[
                    ModelPart::of("right_hind_foot", PartPose::offset_and_rotation([0.0, 31.0, 4.0], [0.75, 0.0, 0.0]), &[ModelCube::of([112.0, 0.0], [-9.0, 0.0, -20.0], [18.0, 6.0, 24.0])], &[]),
                ]),
            ]),
            ModelPart::of("right_wing", PartPose::offset([-12.0, 2.0, -6.0]), &[ModelCube::of([112.0, 88.0], [-56.0, -4.0, -4.0], [56.0, 8.0, 8.0]), ModelCube::of([-56.0, 88.0], [-56.0, 0.0, 2.0], [56.0, 0.0, 56.0])], &[
                ModelPart::of("right_wing_tip", PartPose::offset([-56.0, 0.0, 0.0]), &[ModelCube::of([112.0, 136.0], [-56.0, -2.0, -2.0], [56.0, 4.0, 4.0]), ModelCube::of([-56.0, 144.0], [-56.0, 0.0, 2.0], [56.0, 0.0, 56.0])], &[]),
            ]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -62.0]), &[ModelCube::of([176.0, 44.0], [-6.0, -1.0, -24.0], [12.0, 5.0, 16.0]), ModelCube::of([112.0, 30.0], [-8.0, -8.0, -10.0], [16.0, 16.0, 16.0]), ModelCube::of([0.0, 0.0], [-5.0, -12.0, -4.0], [2.0, 4.0, 6.0]).mirror(), ModelCube::of([112.0, 0.0], [-5.0, -3.0, -22.0], [2.0, 2.0, 4.0]).mirror(), ModelCube::of([0.0, 0.0], [3.0, -12.0, -4.0], [2.0, 4.0, 6.0]).mirror(), ModelCube::of([112.0, 0.0], [3.0, -3.0, -22.0], [2.0, 2.0, 4.0]).mirror()], &[
            ModelPart::of("jaw", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([176.0, 65.0], [-6.0, 0.0, -16.0], [12.0, 4.0, 16.0])], &[]),
        ]),
        ModelPart::of("neck0", PartPose::offset([0.0, 20.0, -12.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("neck1", PartPose::offset([0.0, 20.0, -22.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("neck2", PartPose::offset([0.0, 20.0, -32.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("neck3", PartPose::offset([0.0, 20.0, -42.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("neck4", PartPose::offset([0.0, 20.0, -52.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail0", PartPose::offset([0.0, 10.0, 60.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail1", PartPose::offset([0.0, 10.0, 70.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail10", PartPose::offset([0.0, 10.0, 160.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail11", PartPose::offset([0.0, 10.0, 170.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail2", PartPose::offset([0.0, 10.0, 80.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail3", PartPose::offset([0.0, 10.0, 90.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail4", PartPose::offset([0.0, 10.0, 100.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail5", PartPose::offset([0.0, 10.0, 110.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail6", PartPose::offset([0.0, 10.0, 120.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail7", PartPose::offset([0.0, 10.0, 130.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail8", PartPose::offset([0.0, 10.0, 140.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
        ModelPart::of("tail9", PartPose::offset([0.0, 10.0, 150.0]), &[ModelCube::of([192.0, 104.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0]), ModelCube::of([48.0, 0.0], [-1.0, -9.0, -3.0], [2.0, 4.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:enderman#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -14.0, 0.0]), &[ModelCube::of([32.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([0.0, 16.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([-0.5, -0.5, -0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, -12.0, 0.0]), &[ModelCube::of([56.0, 0.0], [-1.0, -2.0, -1.0], [2.0, 30.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, -5.0, 0.0]), &[ModelCube::of([56.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 30.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, -12.0, 0.0]), &[ModelCube::of([56.0, 0.0], [-1.0, -2.0, -1.0], [2.0, 30.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, -5.0, 0.0]), &[ModelCube::of([56.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 30.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:endermite#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("segment0", PartPose::offset([0.0, 21.0, -3.5]), &[ModelCube::of([0.0, 0.0], [-2.0, 0.0, -1.0], [4.0, 3.0, 2.0])], &[]),
        ModelPart::of("segment1", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([0.0, 5.0], [-3.0, 0.0, -2.5], [6.0, 4.0, 5.0])], &[]),
        ModelPart::of("segment2", PartPose::offset([0.0, 21.0, 3.0]), &[ModelCube::of([0.0, 14.0], [-1.5, 0.0, -0.5], [3.0, 3.0, 1.0])], &[]),
        ModelPart::of("segment3", PartPose::offset([0.0, 22.0, 4.0]), &[ModelCube::of([0.0, 18.0], [-0.5, 0.0, -0.5], [1.0, 2.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:evoker#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[
            ModelPart::of("left_shoulder", PartPose::ZERO, &[ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror()], &[]),
        ]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 12.0, 8.0]).grow([0.45, 0.45, 0.45])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:evoker_fangs#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base", PartPose::offset([-5.0, 24.0, -5.0]), &[ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [10.0, 12.0, 10.0])], &[
            ModelPart::of("lower_jaw", PartPose::offset_and_rotation([3.5, 0.0, 9.0], [0.0, 3.1415927, 4.2411504]), &[ModelCube::of([40.0, 0.0], [0.0, 0.0, 0.0], [4.0, 14.0, 8.0])], &[]),
            ModelPart::of("upper_jaw", PartPose::offset_and_rotation([6.5, 0.0, 1.0], [0.0, 0.0, 2.042035]), &[ModelCube::of([40.0, 0.0], [0.0, 0.0, 0.0], [4.0, 14.0, 8.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:fox#main", 48, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 16.0, -6.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([24.0, 15.0], [-3.0, 3.999, -3.5], [6.0, 11.0, 6.0])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([-4.0, 15.0, -1.0], [-0.05235988, 0.0, 0.0]), &[ModelCube::of([30.0, 0.0], [2.0, 0.0, -1.0], [4.0, 9.0, 5.0])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset([-1.0, 16.5, -3.0]), &[ModelCube::of([1.0, 5.0], [-3.0, -2.0, -5.0], [8.0, 6.0, 6.0])], &[
            ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([15.0, 1.0], [3.0, -4.0, -4.0], [2.0, 2.0, 1.0])], &[]),
            ModelPart::of("nose", PartPose::ZERO, &[ModelCube::of([6.0, 18.0], [-1.0, 2.01, -8.0], [4.0, 2.0, 3.0])], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([8.0, 1.0], [-3.0, -4.0, -4.0], [2.0, 2.0, 1.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([-1.0, 17.5, 0.0]), &[ModelCube::of([4.0, 24.0], [2.0, 0.5, -1.0], [2.0, 6.0, 2.0]).grow([0.001, 0.001, 0.001])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([-1.0, 17.5, 7.0]), &[ModelCube::of([4.0, 24.0], [2.0, 0.5, -1.0], [2.0, 6.0, 2.0]).grow([0.001, 0.001, 0.001])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-5.0, 17.5, 0.0]), &[ModelCube::of([13.0, 24.0], [2.0, 0.5, -1.0], [2.0, 6.0, 2.0]).grow([0.001, 0.001, 0.001])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-5.0, 17.5, 7.0]), &[ModelCube::of([13.0, 24.0], [2.0, 0.5, -1.0], [2.0, 6.0, 2.0]).grow([0.001, 0.001, 0.001])], &[]),
    ])),
    ModelLayer::of("minecraft:fox_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.0, 2.0]), &[ModelCube::of([0.0, 10.0], [-2.5, -2.0, -3.0], [5.0, 4.0, 6.0])], &[
            ModelPart::of("tail", PartPose::offset([0.0, -0.5, 3.0]), &[ModelCube::of([0.0, 20.0], [-1.5, -1.48, -1.0], [3.0, 3.0, 6.0])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 18.125, 0.125]), &[ModelCube::of([0.0, 0.0], [-3.0, -2.125, -5.125], [6.0, 5.0, 5.0]), ModelCube::of([18.0, 20.0], [-1.0, 0.875, -7.125], [2.0, 2.0, 2.0]), ModelCube::of([22.0, 8.0], [-3.0, -4.125, -4.125], [2.0, 2.0, 1.0]), ModelCube::of([22.0, 11.0], [1.0, -4.125, -4.125], [2.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.5, 22.0, 0.0]), &[ModelCube::of([22.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.5, 22.0, 4.0]), &[ModelCube::of([22.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.5, 22.0, 0.0]), &[ModelCube::of([22.0, 4.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.5, 22.0, 4.0]), &[ModelCube::of([22.0, 4.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:frog#main", 48, 48, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -2.0, 4.0]), &[ModelCube::of([3.0, 1.0], [-3.5, -2.0, -8.0], [7.0, 3.0, 9.0]), ModelCube::of([23.0, 22.0], [-3.5, -1.0, -8.0], [7.0, 0.0, 9.0])], &[
                ModelPart::of("croaking_body", PartPose::offset([0.0, -1.0, -5.0]), &[ModelCube::of([26.0, 5.0], [-3.5, -0.1, -2.9], [7.0, 2.0, 3.0]).grow([-0.1, -0.1, -0.1])], &[]),
                ModelPart::of("head", PartPose::offset([0.0, -2.0, -1.0]), &[ModelCube::of([23.0, 13.0], [-3.5, -1.0, -7.0], [7.0, 0.0, 9.0]), ModelCube::of([0.0, 13.0], [-3.5, -2.0, -7.0], [7.0, 3.0, 9.0])], &[
                    ModelPart::of("eyes", PartPose::offset([-0.5, 0.0, 2.0]), &[], &[
                        ModelPart::of("left_eye", PartPose::offset([2.5, -3.0, -6.5]), &[ModelCube::of([0.0, 5.0], [-1.5, -1.0, -1.5], [3.0, 2.0, 3.0])], &[]),
                        ModelPart::of("right_eye", PartPose::offset([-1.5, -3.0, -6.5]), &[ModelCube::of([0.0, 0.0], [-1.5, -1.0, -1.5], [3.0, 2.0, 3.0])], &[]),
                    ]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([4.0, -1.0, -6.5]), &[ModelCube::of([0.0, 32.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 3.0])], &[
                    ModelPart::of("left_hand", PartPose::offset([0.0, 3.0, -1.0]), &[ModelCube::of([18.0, 40.0], [-4.0, 0.01, -4.0], [8.0, 0.0, 8.0])], &[]),
                ]),
                ModelPart::of("right_arm", PartPose::offset([-4.0, -1.0, -6.5]), &[ModelCube::of([0.0, 38.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 3.0])], &[
                    ModelPart::of("right_hand", PartPose::offset([0.0, 3.0, 0.0]), &[ModelCube::of([2.0, 40.0], [-4.0, 0.01, -5.0], [8.0, 0.0, 8.0])], &[]),
                ]),
                ModelPart::of("tongue", PartPose::offset([0.0, -1.01, 1.0]), &[ModelCube::of([17.0, 13.0], [-2.0, 0.0, -7.1], [4.0, 0.0, 7.0])], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([3.5, -3.0, 4.0]), &[ModelCube::of([14.0, 25.0], [-1.0, 0.0, -2.0], [3.0, 3.0, 4.0])], &[
                ModelPart::of("left_foot", PartPose::offset([2.0, 3.0, 0.0]), &[ModelCube::of([2.0, 32.0], [-4.0, 0.01, -4.0], [8.0, 0.0, 8.0])], &[]),
            ]),
            ModelPart::of("right_leg", PartPose::offset([-3.5, -3.0, 4.0]), &[ModelCube::of([0.0, 25.0], [-2.0, 0.0, -2.0], [3.0, 3.0, 4.0])], &[
                ModelPart::of("right_foot", PartPose::offset([-2.0, 3.0, 0.0]), &[ModelCube::of([18.0, 32.0], [-4.0, 0.01, -4.0], [8.0, 0.0, 8.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:furnace_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:ghast#main", 64, 32, ModelPart::of("root", PartPose::new([0.0, -84.056, 0.0], [0.0, 0.0, 0.0], [4.5, 4.5, 4.5]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.6, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0])], &[]),
        ModelPart::of("tentacle0", PartPose::offset([-3.75, 24.6, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("tentacle1", PartPose::offset([1.25, 24.6, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 13.0, 2.0])], &[]),
        ModelPart::of("tentacle2", PartPose::offset([6.25, 24.6, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 9.0, 2.0])], &[]),
        ModelPart::of("tentacle3", PartPose::offset([-6.25, 24.6, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ModelPart::of("tentacle4", PartPose::offset([-1.25, 24.6, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ModelPart::of("tentacle5", PartPose::offset([3.75, 24.6, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 10.0, 2.0])], &[]),
        ModelPart::of("tentacle6", PartPose::offset([-3.75, 24.6, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("tentacle7", PartPose::offset([1.25, 24.6, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 9.0, 2.0])], &[]),
        ModelPart::of("tentacle8", PartPose::offset([6.25, 24.6, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:giant#boots", 64, 32, ModelPart::of("root", PartPose::new([0.0, -120.08, 0.0], [0.0, 0.0, 0.0], [6.0, 6.0, 6.0]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:giant#chestplate", 64, 32, ModelPart::of("root", PartPose::new([0.0, -120.08, 0.0], [0.0, 0.0, 0.0], [6.0, 6.0, 6.0]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:giant#helmet", 64, 32, ModelPart::of("root", PartPose::new([0.0, -120.08, 0.0], [0.0, 0.0, 0.0], [6.0, 6.0, 6.0]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:giant#leggings", 64, 32, ModelPart::of("root", PartPose::new([0.0, -120.08, 0.0], [0.0, 0.0, 0.0], [6.0, 6.0, 6.0]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:giant#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -120.08, 0.0], [0.0, 0.0, 0.0], [6.0, 6.0, 6.0]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:glow_squid#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 8.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-6.0, -8.0, -6.0], [12.0, 16.0, 12.0]).grow([0.02, 0.02, 0.02])], &[]),
        ModelPart::of("tentacle0", PartPose::offset_and_rotation([5.0, 15.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle1", PartPose::offset_and_rotation([3.535534, 15.0, 3.535534], [0.0, 0.7853982, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle2", PartPose::offset([3.061617e-16, 15.0, 5.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle3", PartPose::offset_and_rotation([-3.535534, 15.0, 3.535534], [0.0, -0.7853982, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle4", PartPose::offset_and_rotation([-5.0, 15.0, 6.123234e-16], [0.0, -1.5707964, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle5", PartPose::offset_and_rotation([-3.535534, 15.0, -3.535534], [0.0, -2.3561945, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle6", PartPose::offset_and_rotation([-9.18485e-16, 15.0, -5.0], [0.0, -3.1415927, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle7", PartPose::offset_and_rotation([3.535534, 15.0, -3.535534], [0.0, -3.9269907, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:glow_squid_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 13.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -5.0, -4.0], [8.0, 10.0, 8.0])], &[]),
        ModelPart::of("tentacle0", PartPose::offset_and_rotation([3.0, 18.5, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle1", PartPose::offset_and_rotation([2.1213202, 18.5, 2.1213202], [0.0, 0.7853982, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle2", PartPose::offset([1.8369703e-16, 18.5, 3.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle3", PartPose::offset_and_rotation([-2.1213202, 18.5, 2.1213202], [0.0, -0.7853982, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle4", PartPose::offset_and_rotation([-3.0, 18.5, 3.6739406e-16], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle5", PartPose::offset_and_rotation([-2.1213202, 18.5, -2.1213202], [0.0, -2.3561945, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle6", PartPose::offset_and_rotation([-5.5109107e-16, 18.5, -3.0], [0.0, -3.1415927, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle7", PartPose::offset_and_rotation([2.1213202, 18.5, -2.1213202], [0.0, -3.9269907, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:goat#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([1.0, 1.0], [-4.0, -17.0, -7.0], [9.0, 11.0, 16.0]), ModelCube::of([0.0, 28.0], [-5.0, -18.0, -8.0], [11.0, 14.0, 11.0])], &[]),
        ModelPart::of("head", PartPose::offset([1.0, 14.0, 0.0]), &[ModelCube::of([2.0, 61.0], [-6.0, -11.0, -10.0], [3.0, 2.0, 1.0]), ModelCube::of([2.0, 61.0], [2.0, -11.0, -10.0], [3.0, 2.0, 1.0]).mirror(), ModelCube::of([23.0, 52.0], [-0.5, -3.0, -14.0], [0.0, 7.0, 5.0]).mirror()], &[
            ModelPart::of("left_horn", PartPose::ZERO, &[ModelCube::of([12.0, 55.0], [-0.01, -16.0, -10.0], [2.0, 7.0, 2.0])], &[]),
            ModelPart::of("nose", PartPose::offset_and_rotation([0.0, -8.0, -8.0], [0.9599, 0.0, 0.0]), &[ModelCube::of([34.0, 46.0], [-3.0, -4.0, -8.0], [5.0, 7.0, 10.0])], &[]),
            ModelPart::of("right_horn", PartPose::ZERO, &[ModelCube::of([12.0, 55.0], [-2.99, -16.0, -10.0], [2.0, 7.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([1.0, 14.0, -6.0]), &[ModelCube::of([49.0, 2.0], [0.0, 0.0, 0.0], [3.0, 10.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.0, 14.0, 4.0]), &[ModelCube::of([36.0, 29.0], [0.0, 4.0, 0.0], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 14.0, -6.0]), &[ModelCube::of([35.0, 2.0], [0.0, 0.0, 0.0], [3.0, 10.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 14.0, 4.0]), &[ModelCube::of([49.0, 29.0], [0.0, 4.0, 0.0], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:goat_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.8, 0.0]), &[ModelCube::of([0.0, 10.0], [-3.0, -2.3, -4.5], [6.0, 5.0, 9.0]), ModelCube::of([0.0, 24.0], [-2.5, -2.2, -4.0], [5.0, 4.0, 8.0])], &[]),
        ModelPart::of("head", PartPose::offset_and_rotation([0.0, 15.5, -3.0], [0.4363, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -3.8126, -5.1548], [4.0, 4.0, 6.0])], &[
            ModelPart::of("HeadMain", PartPose::offset([0.0, -1.3126, -1.1548]), &[ModelCube::of([0.0, 0.0], [-2.0, -2.5, -4.0], [4.0, 4.0, 6.0])], &[]),
            ModelPart::of("left_ear", PartPose::offset_and_rotation([1.7, -2.3126, 0.1452], [0.0, 0.5236, 0.0]), &[ModelCube::of([0.0, 12.0], [0.0, -0.5, -0.5], [2.0, 1.0, 1.0])], &[]),
            ModelPart::of("left_horn", PartPose::offset_and_rotation([-1.5, -1.5, -1.0], [-0.3926991, 0.0, 0.0]), &[ModelCube::of([24.0, 0.0], [2.0, -4.5, 0.0], [1.0, 2.0, 1.0]).mirror()], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.7, -2.3126, 0.1452], [0.0, -0.5236, 0.0]), &[ModelCube::of([0.0, 12.0], [-2.0, -0.5, -0.5], [2.0, 1.0, 1.0]).mirror()], &[]),
            ModelPart::of("right_horn", PartPose::offset_and_rotation([-1.5, -1.5, -1.0], [-0.3926991, 0.0, 0.0]), &[ModelCube::of([24.0, 0.0], [0.0, -4.5, 0.0], [1.0, 2.0, 1.0]).mirror()], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([1.5, 19.5, -2.0]), &[ModelCube::of([29.0, 5.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.5, 19.5, 3.0]), &[ModelCube::of([29.0, 12.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.5, 19.5, -2.0]), &[ModelCube::of([21.0, 5.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.5, 19.5, 3.0]), &[ModelCube::of([21.0, 12.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:guardian#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-6.0, 10.0, -8.0], [12.0, 12.0, 16.0]), ModelCube::of([0.0, 28.0], [-8.0, 10.0, -6.0], [2.0, 12.0, 12.0]), ModelCube::of([0.0, 28.0], [6.0, 10.0, -6.0], [2.0, 12.0, 12.0]).mirror(), ModelCube::of([16.0, 40.0], [-6.0, 8.0, -6.0], [12.0, 2.0, 12.0]), ModelCube::of([16.0, 40.0], [-6.0, 22.0, -6.0], [12.0, 2.0, 12.0])], &[
            ModelPart::of("eye", PartPose::offset([0.0, 0.0, -8.25]), &[ModelCube::of([8.0, 0.0], [-1.0, 15.0, 0.0], [2.0, 2.0, 1.0])], &[]),
            ModelPart::of("spike0", PartPose::offset_and_rotation([0.0, 7.92, 8.08], [5.4977875, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike1", PartPose::offset_and_rotation([0.0, 7.9567738, -8.043226], [0.7853982, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike10", PartPose::offset_and_rotation([7.932871, 23.93287, 0.0], [0.0, 0.0, 2.3561945]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike11", PartPose::offset_and_rotation([-8.000353, 24.000353, 0.0], [0.0, 0.0, 3.926991]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike2", PartPose::offset_and_rotation([7.9667134, 8.033287, 0.0], [0.0, 0.0, 0.7853982]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike3", PartPose::offset_and_rotation([-7.9208007, 8.079199, 0.0], [0.0, 0.0, 5.4977875]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike4", PartPose::offset_and_rotation([-7.9477057, 16.0, -7.9477057], [1.5707964, 0.7853982, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike5", PartPose::offset_and_rotation([8.022686, 16.0, -8.022686], [1.5707964, 5.4977875, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike6", PartPose::offset_and_rotation([8.076813, 16.0, 8.076813], [1.5707964, 3.926991, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike7", PartPose::offset_and_rotation([-8.060315, 16.0, 8.060315], [1.5707964, 2.3561945, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike8", PartPose::offset_and_rotation([0.0, 23.988361, 7.9883604], [3.926991, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("spike9", PartPose::offset_and_rotation([0.0, 23.92711, -7.9271107], [2.3561945, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -4.5, -1.0], [2.0, 9.0, 2.0])], &[]),
            ModelPart::of("tail0", PartPose::ZERO, &[ModelCube::of([40.0, 0.0], [-2.0, 14.0, 7.0], [4.0, 4.0, 8.0])], &[
                ModelPart::of("tail1", PartPose::offset([-1.5, 0.5, 14.0]), &[ModelCube::of([0.0, 54.0], [0.0, 14.0, 0.0], [3.0, 3.0, 7.0])], &[
                    ModelPart::of("tail2", PartPose::offset([0.5, 0.5, 6.0]), &[ModelCube::of([41.0, 32.0], [0.0, 14.0, 0.0], [2.0, 2.0, 6.0]), ModelCube::of([25.0, 19.0], [1.0, 10.5, 3.0], [1.0, 9.0, 9.0])], &[]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:happy_ghast#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -72.048004, 0.0], [0.0, 0.0, 0.0], [4.0, 4.0, 4.0]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0])], &[
            ModelPart::of("tentacle0", PartPose::offset([-3.75, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle1", PartPose::offset([1.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0])], &[]),
            ModelPart::of("tentacle2", PartPose::offset([6.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
            ModelPart::of("tentacle3", PartPose::offset([-6.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle4", PartPose::offset([-1.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle5", PartPose::offset([3.75, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0])], &[]),
            ModelPart::of("tentacle6", PartPose::offset([-3.75, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
            ModelPart::of("tentacle7", PartPose::offset([1.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
            ModelPart::of("tentacle8", PartPose::offset([6.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:happy_ghast_baby#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.2008, 0.0], [0.0, 0.0, 0.0], [0.95, 0.95, 0.95]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0])], &[
            ModelPart::of("inner_body", PartPose::offset([0.0, 8.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-8.0, -16.0, -8.0], [16.0, 16.0, 16.0]).grow([-0.5, -0.5, -0.5])], &[]),
            ModelPart::of("tentacle0", PartPose::offset([-3.75, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle1", PartPose::offset([1.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0])], &[]),
            ModelPart::of("tentacle2", PartPose::offset([6.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
            ModelPart::of("tentacle3", PartPose::offset([-6.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle4", PartPose::offset([-1.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
            ModelPart::of("tentacle5", PartPose::offset([3.75, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0])], &[]),
            ModelPart::of("tentacle6", PartPose::offset([-3.75, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
            ModelPart::of("tentacle7", PartPose::offset([1.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
            ModelPart::of("tentacle8", PartPose::offset([6.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:happy_ghast_baby_harness#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 18.597391, 0.0], [0.0, 0.0, 0.0], [0.225625, 0.225625, 0.225625]), &[], &[
        ModelPart::of("goggles", PartPose::offset([0.0, 14.0, -5.5]), &[ModelCube::of([0.0, 32.0], [-8.0, -2.5, -2.5], [16.0, 5.0, 5.0]).grow([0.15, 0.15, 0.15])], &[]),
        ModelPart::of("harness", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -16.0, -8.0], [16.0, 16.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:happy_ghast_baby_ropes#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.2008, 0.0], [0.0, 0.0, 0.0], [0.95, 0.95, 0.95]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0]).grow([0.2, 0.2, 0.2])], &[
            ModelPart::of("inner_body", PartPose::offset([0.0, 8.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-8.0, -16.0, -8.0], [16.0, 16.0, 16.0]).grow([-0.3, -0.3, -0.3])], &[]),
            ModelPart::of("tentacle0", PartPose::offset([-3.75, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle1", PartPose::offset([1.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle2", PartPose::offset([6.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle3", PartPose::offset([-6.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle4", PartPose::offset([-1.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle5", PartPose::offset([3.75, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle6", PartPose::offset([-3.75, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle7", PartPose::offset([1.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle8", PartPose::offset([6.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:happy_ghast_harness#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -72.048004, 0.0], [0.0, 0.0, 0.0], [4.0, 4.0, 4.0]), &[], &[
        ModelPart::of("goggles", PartPose::offset([0.0, 14.0, -5.5]), &[ModelCube::of([0.0, 32.0], [-8.0, -2.5, -2.5], [16.0, 5.0, 5.0]).grow([0.15, 0.15, 0.15])], &[]),
        ModelPart::of("harness", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -16.0, -8.0], [16.0, 16.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:happy_ghast_ropes#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -72.048004, 0.0], [0.0, 0.0, 0.0], [4.0, 4.0, 4.0]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0]).grow([0.2, 0.2, 0.2])], &[
            ModelPart::of("tentacle0", PartPose::offset([-3.75, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle1", PartPose::offset([1.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle2", PartPose::offset([6.25, 7.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle3", PartPose::offset([-6.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle4", PartPose::offset([-1.25, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle5", PartPose::offset([3.75, 7.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 7.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle6", PartPose::offset([-3.75, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle7", PartPose::offset([1.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("tentacle8", PartPose::offset([6.25, 7.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:hoglin#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 7.0, 0.0]), &[ModelCube::of([1.0, 1.0], [-8.0, -7.0, -13.0], [16.0, 14.0, 26.0])], &[
            ModelPart::of("mane", PartPose::offset([0.0, -14.0, -7.0]), &[ModelCube::of([90.0, 33.0], [0.0, 0.0, -9.0], [0.0, 10.0, 19.0]).grow([0.001, 0.001, 0.001])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset_and_rotation([0.0, 2.0, -12.0], [0.87266463, 0.0, 0.0]), &[ModelCube::of([61.0, 1.0], [-7.0, -3.0, -19.0], [14.0, 6.0, 19.0])], &[
            ModelPart::of("left_ear", PartPose::offset_and_rotation([6.0, -2.0, -3.0], [0.0, 0.0, 0.6981317]), &[ModelCube::of([1.0, 6.0], [0.0, -1.0, -2.0], [6.0, 1.0, 4.0])], &[]),
            ModelPart::of("left_horn", PartPose::offset([7.0, 2.0, -12.0]), &[ModelCube::of([1.0, 13.0], [-1.0, -11.0, -1.0], [2.0, 11.0, 2.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-6.0, -2.0, -3.0], [0.0, 0.0, -0.6981317]), &[ModelCube::of([1.0, 1.0], [-6.0, -1.0, -2.0], [6.0, 1.0, 4.0])], &[]),
            ModelPart::of("right_horn", PartPose::offset([-7.0, 2.0, -12.0]), &[ModelCube::of([10.0, 13.0], [-1.0, -11.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 10.0, -8.5]), &[ModelCube::of([41.0, 42.0], [-3.0, 0.0, -3.0], [6.0, 14.0, 6.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([5.0, 13.0, 10.0]), &[ModelCube::of([0.0, 45.0], [-2.5, 0.0, -2.5], [5.0, 11.0, 5.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 10.0, -8.5]), &[ModelCube::of([66.0, 42.0], [-3.0, 0.0, -3.0], [6.0, 14.0, 6.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-5.0, 13.0, 10.0]), &[ModelCube::of([21.0, 45.0], [-2.5, 0.0, -2.5], [5.0, 11.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:hoglin_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-4.0, -14.0, -7.0], [8.0, 8.0, 14.0]).grow([0.02, 0.02, 0.02]), ModelCube::of([24.0, 39.0], [0.0, -18.0, -8.0], [0.0, 6.0, 11.0]).grow([0.02, 0.02, 0.02])], &[]),
        ModelPart::of("head", PartPose::offset_and_rotation([0.0, 13.0, -7.0], [0.8727, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-5.0, -2.2605, -10.547], [10.0, 4.0, 12.0]), ModelCube::of([44.0, 29.0], [-7.0, -4.0981, -8.4879], [2.0, 5.0, 2.0]), ModelCube::of([52.0, 29.0], [5.0, -4.0981, -8.4879], [2.0, 5.0, 2.0])], &[
            ModelPart::of("left_ear", PartPose::offset_and_rotation([5.0, -1.0, -1.5], [0.0, 0.0, 0.8727]), &[ModelCube::of([32.0, 0.0], [-0.9, -0.5, -2.0], [6.0, 1.0, 4.0]).mirror()], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-5.0, -1.0, -1.5], [0.0, 0.0, -0.8727]), &[ModelCube::of([32.0, 5.0], [-5.1, -0.5, -2.0], [6.0, 1.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -4.5]), &[ModelCube::of([12.0, 38.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 4.5]), &[ModelCube::of([12.0, 47.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -4.5]), &[ModelCube::of([0.0, 38.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 4.5]), &[ModelCube::of([0.0, 47.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:hopper_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:horse#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -2.4016006, 0.0], [0.0, 0.0, 0.0], [1.1, 1.1, 1.1]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:horse#saddle", 64, 64, ModelPart::of("root", PartPose::new([0.0, -2.4016006, 0.0], [0.0, 0.0, 0.0], [1.1, 1.1, 1.1]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-5.0, -8.0, -9.0], [10.0, 9.0, 9.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("head_saddle", PartPose::ZERO, &[ModelCube::of([1.0, 1.0], [-3.0, -11.0, -1.9], [6.0, 5.0, 6.0]).grow([0.22, 0.22, 0.22])], &[]),
            ModelPart::of("left_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("left_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [2.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("mouth_saddle_wrap", PartPose::ZERO, &[ModelCube::of([19.0, 0.0], [-2.0, -11.0, -4.0], [4.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("right_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [-3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("right_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [-3.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:horse_armor#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -2.4016006, 0.0], [0.0, 0.0, 0.0], [1.1, 1.1, 1.1]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0]).grow([0.1, 0.1, 0.1])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0]).grow([0.1, 0.1, 0.1])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0]).grow([0.1, 0.1, 0.1])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
    ])),
    ModelLayer::of("minecraft:horse_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 12.5, 0.0]), &[ModelCube::of([0.0, 13.0], [-4.0, -3.5, -7.0], [8.0, 7.0, 14.0])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -1.0, 7.0], [-0.7418, 0.0, 0.0]), &[ModelCube::of([24.0, 34.0], [-1.5, -1.5, -1.0], [3.0, 3.0, 8.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 10.0, -6.0], [0.6109, 0.0, 0.0]), &[ModelCube::of([30.0, 0.0], [-2.0, -6.0, -2.0], [4.0, 8.0, 4.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0516, -0.2951]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.9484, -6.705], [6.0, 4.0, 9.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([2.0, -4.2484, 1.9451], [0.0, 0.0, 0.2618]), &[ModelCube::of([0.0, 4.0], [-1.0, -2.5, -0.8], [2.0, 3.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-2.0, -4.2484, 1.645], [0.0, 0.0, -0.2618]), &[ModelCube::of([0.0, 0.0], [-1.0, -2.5, -0.5], [2.0, 3.0, 1.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([2.4, 16.0, -5.4]), &[ModelCube::of([12.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.4, 16.0, 5.4]), &[ModelCube::of([12.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.4, 16.0, -5.4]), &[ModelCube::of([0.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.4, 16.0, 5.4]), &[ModelCube::of([0.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:husk#boots", 64, 32, ModelPart::of("root", PartPose::new([0.0, -1.501, 0.0], [0.0, 0.0, 0.0], [1.0625, 1.0625, 1.0625]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:husk#chestplate", 64, 32, ModelPart::of("root", PartPose::new([0.0, -1.501, 0.0], [0.0, 0.0, 0.0], [1.0625, 1.0625, 1.0625]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:husk#helmet", 64, 32, ModelPart::of("root", PartPose::new([0.0, -1.501, 0.0], [0.0, 0.0, 0.0], [1.0625, 1.0625, 1.0625]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:husk#leggings", 64, 32, ModelPart::of("root", PartPose::new([0.0, -1.501, 0.0], [0.0, 0.0, 0.0], [1.0625, 1.0625, 1.0625]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:husk#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, -1.501, 0.0], [0.0, 0.0, 0.0], [1.0625, 1.0625, 1.0625]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:husk_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:husk_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:husk_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([-0.1, 0.5, 0.3])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:husk_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([-0.2, 0.20000002, 0.20000002])], &[]),
    ])),
    ModelLayer::of("minecraft:husk_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.5, 0.0]), &[ModelCube::of([16.0, 16.0], [-2.0, -2.5, -1.0], [4.0, 5.0, 2.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.25, 0.0]), &[ModelCube::of([3.0, 3.0], [-3.0, -6.25, -3.0], [6.0, 6.0, 6.0]), ModelCube::of([35.0, 3.0], [-3.0, -6.15, -3.0], [6.0, 6.0, 6.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([28.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 20.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([36.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 20.0, 0.0]), &[ModelCube::of([8.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:illusioner#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[
            ModelPart::of("left_shoulder", PartPose::ZERO, &[ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror()], &[]),
        ]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 12.0, 8.0]).grow([0.45, 0.45, 0.45])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:iron_golem#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, -7.0, 0.0]), &[ModelCube::of([0.0, 40.0], [-9.0, -2.0, -6.0], [18.0, 12.0, 11.0]), ModelCube::of([0.0, 70.0], [-4.5, 10.0, -3.0], [9.0, 5.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, -7.0, -2.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -12.0, -5.5], [8.0, 10.0, 8.0]), ModelCube::of([24.0, 0.0], [-1.0, -5.0, -7.5], [2.0, 4.0, 2.0])], &[]),
        ModelPart::of("left_arm", PartPose::offset([0.0, -7.0, 0.0]), &[ModelCube::of([60.0, 58.0], [9.0, -2.5, -3.0], [4.0, 30.0, 6.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([5.0, 11.0, 0.0]), &[ModelCube::of([60.0, 0.0], [-3.5, -3.0, -3.0], [6.0, 16.0, 5.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([0.0, -7.0, 0.0]), &[ModelCube::of([60.0, 21.0], [-13.0, -2.5, -3.0], [4.0, 30.0, 6.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-4.0, 11.0, 0.0]), &[ModelCube::of([37.0, 0.0], [-3.5, -3.0, -3.0], [6.0, 16.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:leash_knot#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("knot", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-3.0, -8.0, -3.0], [6.0, 8.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:llama#decor", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([29.0, 0.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 7.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -14.0, -10.0], [4.0, 4.0, 9.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([0.0, 14.0], [-4.0, -16.0, -6.0], [8.0, 18.0, 6.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([17.0, 0.0], [-4.0, -19.0, -4.0], [3.0, 3.0, 2.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([17.0, 0.0], [1.0, -19.0, -4.0], [3.0, 3.0, 2.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
    ])),
    ModelLayer::of("minecraft:llama#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([29.0, 0.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 7.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -14.0, -10.0], [4.0, 4.0, 9.0]), ModelCube::of([0.0, 14.0], [-4.0, -16.0, -6.0], [8.0, 18.0, 6.0]), ModelCube::of([17.0, 0.0], [-4.0, -19.0, -4.0], [3.0, 3.0, 2.0]), ModelCube::of([17.0, 0.0], [1.0, -19.0, -4.0], [3.0, 3.0, 2.0])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:llama_baby#decor", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 14.0, 2.5]), &[ModelCube::of([0.0, 15.0], [-4.0, -3.0, -8.5], [8.0, 6.0, 13.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -9.0, -4.0], [6.0, 11.0, 4.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([0.0, 15.0], [-1.5, -7.0, -7.0], [3.0, 3.0, 3.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([20.0, 4.0], [0.5, -11.0, -3.0], [2.0, 2.0, 2.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([20.0, 0.0], [-2.5, -11.0, -3.0], [2.0, 2.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 16.5, -3.5]), &[ModelCube::of([12.0, 34.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 16.5, 4.5]), &[ModelCube::of([12.0, 45.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 16.5, -3.5]), &[ModelCube::of([0.0, 34.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 16.5, 4.5]), &[ModelCube::of([0.0, 45.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
    ])),
    ModelLayer::of("minecraft:llama_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 14.0, 2.5]), &[ModelCube::of([0.0, 15.0], [-4.0, -3.0, -8.5], [8.0, 6.0, 13.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -9.0, -4.0], [6.0, 11.0, 4.0]), ModelCube::of([0.0, 15.0], [-1.5, -7.0, -7.0], [3.0, 3.0, 3.0]), ModelCube::of([20.0, 4.0], [0.5, -11.0, -3.0], [2.0, 2.0, 2.0]), ModelCube::of([20.0, 0.0], [-2.5, -11.0, -3.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 16.5, -3.5]), &[ModelCube::of([12.0, 34.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 16.5, 4.5]), &[ModelCube::of([12.0, 45.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 16.5, -3.5]), &[ModelCube::of([0.0, 34.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 16.5, 4.5]), &[ModelCube::of([0.0, 45.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:llama_spit#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("main", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, 0.0, 0.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [0.0, -4.0, 0.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [0.0, 0.0, -4.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [0.0, 2.0, 0.0], [2.0, 2.0, 2.0]), ModelCube::of([0.0, 0.0], [0.0, 0.0, 2.0], [2.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:magma_cube#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube0", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, 16.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube1", PartPose::ZERO, &[ModelCube::of([0.0, 9.0], [-4.0, 17.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube2", PartPose::ZERO, &[ModelCube::of([0.0, 18.0], [-4.0, 18.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube3", PartPose::ZERO, &[ModelCube::of([0.0, 27.0], [-4.0, 19.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube4", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, 20.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube5", PartPose::ZERO, &[ModelCube::of([32.0, 9.0], [-4.0, 21.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube6", PartPose::ZERO, &[ModelCube::of([32.0, 18.0], [-4.0, 22.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("cube7", PartPose::ZERO, &[ModelCube::of([32.0, 27.0], [-4.0, 23.0, -4.0], [8.0, 1.0, 8.0])], &[]),
        ModelPart::of("inside_cube", PartPose::ZERO, &[ModelCube::of([24.0, 40.0], [-2.0, 18.0, -2.0], [4.0, 4.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:mooshroom#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([18.0, 4.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]), ModelCube::of([52.0, 0.0], [-2.0, 2.0, -8.0], [4.0, 6.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -6.0], [8.0, 8.0, 6.0]), ModelCube::of([1.0, 33.0], [-3.0, 1.0, -7.0], [6.0, 3.0, 1.0]), ModelCube::of([22.0, 0.0], [-5.0, -5.0, -5.0], [1.0, 3.0, 1.0]), ModelCube::of([22.0, 0.0], [4.0, -5.0, -5.0], [1.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:mooshroom_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([3.0, 19.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -7.0, -1.0], [8.0, 6.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 13.569, -5.1667]), &[ModelCube::of([0.0, 18.0], [-3.0, -4.569, -4.8333], [6.0, 6.0, 5.0]), ModelCube::of([8.0, 29.0], [3.0, -5.569, -3.8333], [1.0, 2.0, 1.0]), ModelCube::of([4.0, 29.0], [-4.0, -5.569, -3.8333], [1.0, 2.0, 1.0]).mirror(), ModelCube::of([12.0, 29.0], [-2.0, -1.569, -5.8333], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -3.5]), &[ModelCube::of([34.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 3.5]), &[ModelCube::of([34.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -3.5]), &[ModelCube::of([22.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 3.5]), &[ModelCube::of([22.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:mule#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.9212797, 0.0], [0.0, 0.0, 0.0], [0.92, 0.92, 0.92]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("left_chest", PartPose::offset_and_rotation([6.0, -8.0, 0.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset_and_rotation([-6.0, -8.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([1.25, -10.0, 4.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.25, -10.0, 4.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:mule#saddle", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.9212797, 0.0], [0.0, 0.0, 0.0], [0.92, 0.92, 0.92]), &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("left_chest", PartPose::offset_and_rotation([6.0, -8.0, 0.0], [0.0, -1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset_and_rotation([-6.0, -8.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([26.0, 21.0], [-4.0, 0.0, -2.0], [8.0, 8.0, 3.0])], &[]),
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-5.0, -8.0, -9.0], [10.0, 9.0, 9.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([1.25, -10.0, 4.0], [0.2617994, 0.0, 0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-1.25, -10.0, 4.0], [0.2617994, 0.0, -0.2617994]), &[ModelCube::of([0.0, 12.0], [-1.0, -7.0, 0.0], [2.0, 7.0, 1.0])], &[]),
            ]),
            ModelPart::of("head_saddle", PartPose::ZERO, &[ModelCube::of([1.0, 1.0], [-3.0, -11.0, -1.9], [6.0, 5.0, 6.0]).grow([0.22, 0.22, 0.22])], &[]),
            ModelPart::of("left_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("left_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [2.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("mouth_saddle_wrap", PartPose::ZERO, &[ModelCube::of([19.0, 0.0], [-2.0, -11.0, -4.0], [4.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("right_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [-3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("right_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [-3.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:mule_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([1.0, 14.0, 0.0]), &[ModelCube::of([0.0, 13.0], [-5.0, -3.0, -7.0], [8.0, 6.0, 14.0])], &[
            ModelPart::of("head_parts", PartPose::offset([0.0, -3.0, -5.0]), &[], &[
                ModelPart::of("head", PartPose::offset([0.0, -5.0, -3.0]), &[], &[
                    ModelPart::of("head_r1", PartPose::offset_and_rotation([0.0, -1.0, 1.0], [0.3927, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -3.6, -8.4], [6.0, 4.0, 9.0])], &[]),
                    ModelPart::of("left_ear", PartPose::offset_and_rotation([2.0, -3.5, -1.0], [0.48, 0.0, 0.48]), &[ModelCube::of([0.0, 0.0], [-2.0, -6.5, -0.3], [2.0, 7.0, 1.0])], &[]),
                    ModelPart::of("right_ear", PartPose::offset_and_rotation([-2.0, -3.5, -1.0], [0.48, 0.0, -0.48]), &[ModelCube::of([22.0, 0.0], [-2.0, -6.5, -0.3], [2.0, 7.0, 1.0]).mirror()], &[]),
                ]),
                ModelPart::of("neck_r1", PartPose::rotation([0.3927, 0.0, 0.0]), &[ModelCube::of([30.0, 9.0], [-3.0, -6.0, -3.0], [4.0, 8.0, 4.0])], &[]),
            ]),
            ModelPart::of("left_chest", PartPose::offset([-1.0, 10.0, 0.0]), &[], &[]),
            ModelPart::of("left_front_leg", PartPose::offset([2.4, 3.5, -5.3]), &[ModelCube::of([12.0, 33.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("left_hind_leg", PartPose::offset([2.25, 3.5, 5.25]), &[ModelCube::of([12.0, 44.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_chest", PartPose::offset([-1.0, 10.0, 0.0]), &[], &[]),
            ModelPart::of("right_front_leg", PartPose::offset([-2.4, 3.5, -5.4]), &[ModelCube::of([0.0, 33.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("right_hind_leg", PartPose::offset([-2.4, 3.5, 5.4]), &[ModelCube::of([0.0, 44.0], [-2.5, -1.5, -1.5], [3.0, 8.0, 3.0])], &[]),
            ModelPart::of("tail", PartPose::offset([0.0, -1.5, 6.5]), &[], &[
                ModelPart::of("tail_r1", PartPose::rotation([-0.7418, 0.0, 0.0]), &[ModelCube::of([24.0, 33.0], [-2.5, -1.0, -0.5], [3.0, 3.0, 8.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:nautilus#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 29.0, -6.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -8.5, 12.3]), &[ModelCube::of([0.0, 54.0], [-5.0, -4.51, -3.0], [10.0, 8.0, 14.0]), ModelCube::of([0.0, 76.0], [-5.0, -4.51, 7.0], [10.0, 8.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -0.51, 7.5]), &[ModelCube::of([54.0, 70.0], [-3.0, -2.0, -0.5], [6.0, 4.0, 4.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, 1.49, 7.0]), &[ModelCube::of([54.0, 62.0], [-5.0, -1.98, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.51, 7.0]), &[ModelCube::of([54.0, 54.0], [-5.0, -2.0, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([0.0, -13.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -10.0, -7.0], [14.0, 10.0, 16.0]), ModelCube::of([0.0, 26.0], [-7.0, 0.0, -7.0], [14.0, 8.0, 20.0]), ModelCube::of([48.0, 26.0], [-7.0, 0.0, 6.0], [14.0, 8.0, 0.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:nautilus#saddle", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 29.0, -6.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -8.5, 12.3]), &[ModelCube::of([0.0, 54.0], [-5.0, -4.51, -3.0], [10.0, 8.0, 14.0]), ModelCube::of([0.0, 76.0], [-5.0, -4.51, 7.0], [10.0, 8.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -0.51, 7.5]), &[ModelCube::of([54.0, 70.0], [-3.0, -2.0, -0.5], [6.0, 4.0, 4.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, 1.49, 7.0]), &[ModelCube::of([54.0, 62.0], [-5.0, -1.98, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.51, 7.0]), &[ModelCube::of([54.0, 54.0], [-5.0, -2.0, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([0.0, -13.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -10.0, -7.0], [14.0, 10.0, 16.0]).grow([0.2, 0.2, 0.2])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:nautilus_armor#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 29.0, -6.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -8.5, 12.3]), &[ModelCube::of([0.0, 54.0], [-5.0, -4.51, -3.0], [10.0, 8.0, 14.0]), ModelCube::of([0.0, 76.0], [-5.0, -4.51, 7.0], [10.0, 8.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -0.51, 7.5]), &[ModelCube::of([54.0, 70.0], [-3.0, -2.0, -0.5], [6.0, 4.0, 4.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, 1.49, 7.0]), &[ModelCube::of([54.0, 62.0], [-5.0, -1.98, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.51, 7.0]), &[ModelCube::of([54.0, 54.0], [-5.0, -2.0, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([0.0, -13.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -10.0, -7.0], [14.0, 10.0, 16.0]).grow([0.01, 0.01, 0.01]), ModelCube::of([0.0, 26.0], [-7.0, 0.0, -7.0], [14.0, 8.0, 20.0]).grow([0.01, 0.01, 0.01]), ModelCube::of([48.0, 26.0], [-7.0, 0.0, 6.0], [14.0, 8.0, 0.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:nautilus_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([-0.5, 28.0, -0.5]), &[], &[
            ModelPart::of("body", PartPose::offset([0.5, -5.0, 3.0]), &[ModelCube::of([0.0, 24.0], [-2.5, -3.01, -1.0], [5.0, 4.0, 7.0]), ModelCube::of([0.0, 35.0], [-2.5, -3.01, 4.1], [5.0, 4.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -1.01, 4.9]), &[ModelCube::of([24.0, 32.0], [-1.5, -1.0, -1.0], [3.0, 2.0, 2.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, -0.01, 3.9]), &[ModelCube::of([24.0, 28.0], [-2.5, -1.0, 0.0], [5.0, 2.0, 2.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.01, 3.9]), &[ModelCube::of([24.0, 24.0], [-2.5, -1.0, 0.0], [5.0, 2.0, 2.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([3.0, -8.0, -2.0]), &[ModelCube::of([0.0, 0.0], [-6.0, -4.0, -1.0], [7.0, 4.0, 7.0]), ModelCube::of([0.0, 11.0], [-6.0, 0.0, -1.0], [7.0, 4.0, 9.0]), ModelCube::of([23.0, 11.0], [-6.0, 0.0, 5.0], [7.0, 4.0, 0.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:ocelot#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 12.0, -10.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([20.0, 0.0], [-2.0, 3.0, -8.0], [4.0, 16.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -9.0]), &[ModelCube::of([0.0, 0.0], [-2.5, -2.0, -3.0], [5.0, 4.0, 5.0]), ModelCube::of([0.0, 24.0], [-1.5, -0.001, -4.0], [3.0, 2.0, 2.0]), ModelCube::of([0.0, 10.0], [-2.0, -3.0, 0.0], [1.0, 1.0, 2.0]), ModelCube::of([6.0, 10.0], [1.0, -3.0, 0.0], [1.0, 1.0, 2.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.2, 14.1, -5.0]), &[ModelCube::of([40.0, 0.0], [-1.0, 0.0, 0.0], [2.0, 10.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.1, 18.0, 5.0]), &[ModelCube::of([8.0, 13.0], [-1.0, 0.0, 1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 15.0, 8.0], [0.9, 0.0, 0.0]), &[ModelCube::of([0.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0])], &[]),
        ModelPart::of("tail2", PartPose::offset([0.0, 20.0, 14.0]), &[ModelCube::of([4.0, 15.0], [-0.5, 0.0, 0.0], [1.0, 8.0, 1.0]).grow([-0.02, -0.02, -0.02])], &[]),
    ])),
    ModelLayer::of("minecraft:ocelot_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 20.5, 0.5]), &[ModelCube::of([0.0, 8.0], [-2.0, -1.5, -3.5], [4.0, 3.0, 7.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -3.125]), &[ModelCube::of([0.0, 0.0], [-2.5, -3.0, -2.875], [5.0, 4.0, 4.0]), ModelCube::of([18.0, 0.0], [-2.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([24.0, 0.0], [1.0, -4.0, -0.875], [1.0, 1.0, 2.0]), ModelCube::of([18.0, 3.0], [-1.5, -1.0, -3.875], [3.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([1.0, 22.0, -1.5]), &[ModelCube::of([18.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.0, 22.0, 2.5]), &[ModelCube::of([18.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.0, 22.0, -1.5]), &[ModelCube::of([12.0, 18.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.0, 22.0, 2.5]), &[ModelCube::of([12.0, 22.0], [-0.5, 0.0, -1.0], [1.0, 2.0, 2.0])], &[]),
        ModelPart::of("tail1", PartPose::offset_and_rotation([0.0, 19.107, 3.9151], [-0.567232, 0.0, 0.0]), &[ModelCube::of([0.0, 18.0], [-0.5, -0.107, 0.0849], [1.0, 1.0, 5.0])], &[]),
        ModelPart::of("tail2", PartPose::ZERO, &[], &[]),
    ])),
    ModelLayer::of("minecraft:panda#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 10.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 25.0], [-9.5, -13.0, -6.5], [19.0, 26.0, 13.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 11.5, -17.0]), &[ModelCube::of([0.0, 6.0], [-6.5, -5.0, -4.0], [13.0, 10.0, 9.0]), ModelCube::of([45.0, 16.0], [-3.5, 0.0, -6.0], [7.0, 5.0, 2.0]), ModelCube::of([52.0, 25.0], [3.5, -8.0, -1.0], [5.0, 4.0, 1.0]), ModelCube::of([52.0, 25.0], [-8.5, -8.0, -1.0], [5.0, 4.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([5.5, 15.0, -9.0]), &[ModelCube::of([40.0, 0.0], [-3.0, 0.0, -3.0], [6.0, 9.0, 6.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([5.5, 15.0, 9.0]), &[ModelCube::of([40.0, 0.0], [-3.0, 0.0, -3.0], [6.0, 9.0, 6.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-5.5, 15.0, -9.0]), &[ModelCube::of([40.0, 0.0], [-3.0, 0.0, -3.0], [6.0, 9.0, 6.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-5.5, 15.0, 9.0]), &[ModelCube::of([40.0, 0.0], [-3.0, 0.0, -3.0], [6.0, 9.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:panda_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.5, 2.5]), &[ModelCube::of([0.0, 11.0], [-4.5, -3.5, -5.5], [9.0, 7.0, 11.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 19.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-3.5, -3.0, -5.0], [7.0, 6.0, 5.0]), ModelCube::of([24.0, 6.0], [-2.0, 1.0, -6.0], [4.0, 2.0, 1.0]), ModelCube::of([24.0, 0.0], [-4.5, -4.0, -3.5], [3.0, 3.0, 1.0]), ModelCube::of([33.0, 0.0], [1.5, -4.0, -3.5], [3.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 22.0, -1.5]), &[ModelCube::of([12.0, 29.0], [-1.5, 0.0, -1.5], [3.0, 2.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 22.0, 6.5]), &[ModelCube::of([12.0, 34.0], [-1.5, 0.0, -1.5], [3.0, 2.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 22.0, -1.5]), &[ModelCube::of([0.0, 29.0], [-1.5, 0.0, -1.5], [3.0, 2.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 22.0, 6.5]), &[ModelCube::of([0.0, 34.0], [-1.5, 0.0, -1.5], [3.0, 2.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:parched#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:parched#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:parched#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:parched#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:parched#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]), ModelCube::of([28.0, 0.0], [-4.0, 10.0, -2.0], [8.0, 1.0, 4.0]), ModelCube::of([16.0, 48.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.025, 0.025, 0.025])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]), ModelCube::of([0.0, 32.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.2, 0.2, 0.2])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.5, 2.0, 0.0]), &[ModelCube::of([56.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]), ModelCube::of([40.0, 48.0], [-1.45, -2.025, -1.5], [3.0, 12.0, 3.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]), ModelCube::of([4.0, 49.0], [-1.5, 0.0, -1.5], [3.0, 12.0, 3.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.5, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]), ModelCube::of([42.0, 33.0], [-1.55, -2.025, -1.5], [3.0, 12.0, 3.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]), ModelCube::of([0.0, 49.0], [-1.5, -0.0, -1.5], [3.0, 12.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:parched#outer", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.75, 0.75, 0.75])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
    ])),
    ModelLayer::of("minecraft:parrot#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 16.5, -3.0], [0.4937, 0.0, 0.0]), &[ModelCube::of([2.0, 8.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.69, -2.76]), &[ModelCube::of([2.0, 2.0], [-1.0, -1.5, -1.0], [2.0, 3.0, 2.0])], &[
            ModelPart::of("beak1", PartPose::offset([0.0, -0.5, -1.5]), &[ModelCube::of([11.0, 7.0], [-0.5, -1.0, -0.5], [1.0, 2.0, 1.0])], &[]),
            ModelPart::of("beak2", PartPose::offset([0.0, -1.75, -2.45]), &[ModelCube::of([16.0, 7.0], [-0.5, 0.0, -0.5], [1.0, 2.0, 1.0])], &[]),
            ModelPart::of("feather", PartPose::offset_and_rotation([0.0, -2.15, 0.15], [-0.2214, 0.0, 0.0]), &[ModelCube::of([2.0, 18.0], [0.0, -4.0, -2.0], [0.0, 5.0, 4.0])], &[]),
            ModelPart::of("head2", PartPose::offset([0.0, -2.0, -1.0]), &[ModelCube::of([10.0, 0.0], [-1.0, -0.5, -2.0], [2.0, 1.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset_and_rotation([1.0, 22.0, -1.05], [-0.0299, 0.0, 0.0]), &[ModelCube::of([14.0, 18.0], [-0.5, 0.0, -0.5], [1.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_wing", PartPose::offset_and_rotation([1.5, 16.94, -2.76], [-0.6981, -3.1415927, 0.0]), &[ModelCube::of([19.0, 8.0], [-0.5, 0.0, -1.5], [1.0, 5.0, 3.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset_and_rotation([-1.0, 22.0, -1.05], [-0.0299, 0.0, 0.0]), &[ModelCube::of([14.0, 18.0], [-0.5, 0.0, -0.5], [1.0, 2.0, 1.0])], &[]),
        ModelPart::of("right_wing", PartPose::offset_and_rotation([-1.5, 16.94, -2.76], [-0.6981, -3.1415927, 0.0]), &[ModelCube::of([19.0, 8.0], [-0.5, 0.0, -1.5], [1.0, 5.0, 3.0])], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([0.0, 21.07, 1.16], [1.015, 0.0, 0.0]), &[ModelCube::of([22.0, 1.0], [-1.5, -1.0, -1.0], [3.0, 4.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:phantom#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::rotation([-0.1, 0.0, 0.0]), &[ModelCube::of([0.0, 8.0], [-3.0, -2.0, -8.0], [5.0, 3.0, 9.0])], &[
            ModelPart::of("head", PartPose::offset_and_rotation([0.0, 1.0, -7.0], [0.2, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -2.0, -5.0], [7.0, 3.0, 5.0])], &[]),
            ModelPart::of("left_wing_base", PartPose::offset_and_rotation([2.0, -2.0, -8.0], [0.0, 0.0, 0.1]), &[ModelCube::of([23.0, 12.0], [0.0, 0.0, 0.0], [6.0, 2.0, 9.0])], &[
                ModelPart::of("left_wing_tip", PartPose::offset_and_rotation([6.0, 0.0, 0.0], [0.0, 0.0, 0.1]), &[ModelCube::of([16.0, 24.0], [0.0, 0.0, 0.0], [13.0, 1.0, 9.0])], &[]),
            ]),
            ModelPart::of("right_wing_base", PartPose::offset_and_rotation([-3.0, -2.0, -8.0], [0.0, 0.0, -0.1]), &[ModelCube::of([23.0, 12.0], [-6.0, 0.0, 0.0], [6.0, 2.0, 9.0]).mirror()], &[
                ModelPart::of("right_wing_tip", PartPose::offset_and_rotation([-6.0, 0.0, 0.0], [0.0, 0.0, -0.1]), &[ModelCube::of([16.0, 24.0], [-13.0, 0.0, 0.0], [13.0, 1.0, 9.0]).mirror()], &[]),
            ]),
            ModelPart::of("tail_base", PartPose::offset([0.0, -2.0, 1.0]), &[ModelCube::of([3.0, 20.0], [-2.0, 0.0, 0.0], [3.0, 2.0, 6.0])], &[
                ModelPart::of("tail_tip", PartPose::offset([0.0, 0.5, 6.0]), &[ModelCube::of([4.0, 29.0], [-1.0, 0.0, 0.0], [1.0, 1.0, 6.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:pig#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 11.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-5.0, -10.0, -7.0], [10.0, 16.0, 8.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -8.0], [8.0, 8.0, 8.0]), ModelCube::of([16.0, 16.0], [-2.0, 0.0, -9.0], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:pig#saddle", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 11.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-5.0, -10.0, -7.0], [10.0, 16.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -8.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([16.0, 16.0], [-2.0, 0.0, -9.0], [4.0, 3.0, 1.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 18.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 18.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
    ])),
    ModelLayer::of("minecraft:pig_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 19.0, 0.5]), &[ModelCube::of([0.0, 0.0], [-3.5, -3.0, -4.5], [7.0, 6.0, 9.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 19.0, -2.0]), &[ModelCube::of([0.0, 15.0], [-3.5, -5.0, -5.0], [7.0, 6.0, 6.0]).grow([0.025, 0.025, 0.025]), ModelCube::of([6.0, 27.0], [-1.5, -1.975, -6.0], [3.0, 2.0, 1.0]).grow([0.015, 0.015, 0.015])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 22.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 22.0, 4.0]), &[ModelCube::of([0.0, 4.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 22.0, -3.0]), &[ModelCube::of([23.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 22.0, 4.0]), &[ModelCube::of([23.0, 4.0], [-1.0, 0.0, -1.0], [2.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:piglin#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02]).mirror()], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.52, 1.52, 1.52])], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([16.0, 32.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-5.0, -8.0, -4.0], [10.0, 8.0, 8.0]), ModelCube::of([31.0, 1.0], [-2.0, -4.0, -5.0], [4.0, 4.0, 1.0]), ModelCube::of([2.0, 4.0], [2.0, -2.0, -5.0], [1.0, 2.0, 1.0]), ModelCube::of([2.0, 0.0], [-3.0, -2.0, -5.0], [1.0, 2.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset_and_rotation([4.5, -6.0, 0.0], [0.0, 0.0, -0.5235988]), &[ModelCube::of([51.0, 6.0], [0.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-4.5, -6.0, 0.0], [0.0, 0.0, 0.5235988]), &[ModelCube::of([39.0, 6.0], [-1.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[ModelCube::of([48.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[ModelCube::of([40.0, 32.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([0.7, 0.7, 0.7]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:piglin_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:piglin_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([0.7, 0.7, 0.7])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:piglin_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([0.59999996, 0.59999996, 0.59999996])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([0.59999996, 0.59999996, 0.59999996])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([0.59999996, 0.59999996, 0.59999996])], &[]),
    ])),
    ModelLayer::of("minecraft:piglin_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, -0.5]), &[ModelCube::of([0.0, 13.0], [-3.0, -3.0, -1.0], [6.0, 5.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([21.0, 30.0], [-1.5, -3.0, -4.5], [3.0, 3.0, 1.0]), ModelCube::of([0.0, 0.0], [-4.5, -6.0, -3.5], [9.0, 6.0, 7.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset([4.2, -4.0, 0.0]), &[], &[
                ModelPart::of("left_ear_r1", PartPose::offset_and_rotation([1.0, 1.75, 0.0], [0.0, 0.0, -0.6109]), &[ModelCube::of([0.0, 21.0], [-0.5, -3.0, -2.0], [1.0, 6.0, 4.0])], &[]),
            ]),
            ModelPart::of("right_ear", PartPose::offset([-4.2, -4.0, 0.0]), &[], &[
                ModelPart::of("right_ear_r1", PartPose::offset_and_rotation([-1.0, 1.75, 0.0], [0.0, 0.0, 0.6109]), &[ModelCube::of([18.0, 13.0], [-0.5, -3.0, -2.0], [1.0, 6.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[ModelCube::of([28.0, 13.0], [-1.0, 0.0, -1.5], [2.0, 5.0, 3.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.0]), &[ModelCube::of([10.0, 23.0], [-1.5, 0.0, -1.5], [3.0, 4.0, 3.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[ModelCube::of([10.0, 30.0], [-1.0, 0.0, -1.5], [2.0, 5.0, 3.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.0]), &[ModelCube::of([22.0, 23.0], [-1.5, 0.0, -1.5], [3.0, 4.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:piglin_brute#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_brute#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02]).mirror()], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_brute#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.52, 1.52, 1.52])], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_brute#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_brute#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([16.0, 32.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-5.0, -8.0, -4.0], [10.0, 8.0, 8.0]), ModelCube::of([31.0, 1.0], [-2.0, -4.0, -5.0], [4.0, 4.0, 1.0]), ModelCube::of([2.0, 4.0], [2.0, -2.0, -5.0], [1.0, 2.0, 1.0]), ModelCube::of([2.0, 0.0], [-3.0, -2.0, -5.0], [1.0, 2.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset_and_rotation([4.5, -6.0, 0.0], [0.0, 0.0, -0.5235988]), &[ModelCube::of([51.0, 6.0], [0.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-4.5, -6.0, 0.0], [0.0, 0.0, 0.5235988]), &[ModelCube::of([39.0, 6.0], [-1.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[ModelCube::of([48.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[ModelCube::of([40.0, 32.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:piglin_head#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-5.0, -8.0, -4.0], [10.0, 8.0, 8.0]), ModelCube::of([31.0, 1.0], [-2.0, -4.0, -5.0], [4.0, 4.0, 1.0]), ModelCube::of([2.0, 4.0], [2.0, -2.0, -5.0], [1.0, 2.0, 1.0]), ModelCube::of([2.0, 0.0], [-3.0, -2.0, -5.0], [1.0, 2.0, 1.0])], &[
            ModelPart::of("left_ear", PartPose::offset_and_rotation([4.5, -6.0, 0.0], [0.0, 0.0, -0.5235988]), &[ModelCube::of([51.0, 6.0], [0.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-4.5, -6.0, 0.0], [0.0, 0.0, 0.5235988]), &[ModelCube::of([39.0, 6.0], [-1.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:pillager#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[
            ModelPart::of("left_shoulder", PartPose::ZERO, &[ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror()], &[]),
        ]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 12.0, 8.0]).grow([0.45, 0.45, 0.45])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:player#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#cape", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("cape", PartPose::offset_and_rotation([0.0, 0.0, 2.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-5.0, 0.0, -1.0], [10.0, 16.0, 1.0]).uv_scale([1.0, 0.5])], &[]),
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#ears", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset([-6.0, -6.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-3.0, -6.0, -1.0], [6.0, 6.0, 1.0]).grow([1.0, 1.0, 1.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset([6.0, -6.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-3.0, -6.0, -1.0], [6.0, 6.0, 1.0]).grow([1.0, 1.0, 1.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([16.0, 32.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[ModelCube::of([48.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[ModelCube::of([40.0, 32.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_head#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_slim#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_slim#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_slim#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_slim#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:player_slim#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([16.0, 32.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [3.0, 12.0, 4.0])], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[ModelCube::of([48.0, 48.0], [-1.0, -2.0, -2.0], [3.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-2.0, -2.0, -2.0], [3.0, 12.0, 4.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[ModelCube::of([40.0, 32.0], [-2.0, -2.0, -2.0], [3.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:polar_bear#main", 128, 64, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([-2.0, 9.0, 12.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 19.0], [-5.0, -13.0, -7.0], [14.0, 14.0, 11.0]), ModelCube::of([39.0, 0.0], [-4.0, -25.0, -7.0], [12.0, 12.0, 10.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 10.0, -16.0]), &[ModelCube::of([0.0, 0.0], [-3.5, -3.0, -3.0], [7.0, 7.0, 7.0]), ModelCube::of([0.0, 44.0], [-2.5, 1.0, -6.0], [5.0, 3.0, 3.0]), ModelCube::of([26.0, 0.0], [-4.5, -4.0, -1.0], [2.0, 2.0, 1.0]), ModelCube::of([26.0, 0.0], [2.5, -4.0, -1.0], [2.0, 2.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.5, 14.0, -8.0]), &[ModelCube::of([50.0, 40.0], [-2.0, 0.0, -2.0], [4.0, 10.0, 6.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.5, 14.0, 6.0]), &[ModelCube::of([50.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 10.0, 8.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.5, 14.0, -8.0]), &[ModelCube::of([50.0, 40.0], [-2.0, 0.0, -2.0], [4.0, 10.0, 6.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.5, 14.0, 6.0]), &[ModelCube::of([50.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 10.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:polar_bear_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.5, 0.0]), &[ModelCube::of([0.0, 9.0], [-4.0, -3.5, -6.0], [8.0, 7.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 18.625, -5.75]), &[ModelCube::of([0.0, 0.0], [-3.0, -2.625, -4.25], [6.0, 5.0, 4.0]), ModelCube::of([20.0, 3.0], [-2.0, 0.375, -6.25], [4.0, 2.0, 2.0]), ModelCube::of([20.0, 0.0], [-4.0, -3.625, -2.75], [2.0, 2.0, 1.0]), ModelCube::of([26.0, 0.0], [2.0, -3.625, -2.75], [2.0, 2.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 21.5, -4.5]), &[ModelCube::of([12.0, 28.0], [-1.5, -0.5, -1.5], [3.0, 3.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 21.5, 4.5]), &[ModelCube::of([12.0, 34.0], [-1.5, -0.5, -1.5], [3.0, 3.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 21.5, -4.5]), &[ModelCube::of([0.0, 28.0], [-1.5, -0.5, -1.5], [3.0, 3.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 21.5, 4.5]), &[ModelCube::of([0.0, 34.0], [-1.5, -0.5, -1.5], [3.0, 3.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:pufferfish_big#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("bottom_back_fin", PartPose::offset_and_rotation([0.0, 22.0, 4.0], [0.7853982, 0.0, 0.0]), &[ModelCube::of([15.0, 20.0], [-4.0, 0.0, 0.0], [8.0, 1.0, 0.0])], &[]),
        ModelPart::of("bottom_front_fin", PartPose::offset_and_rotation([0.0, 22.0, -4.0], [-0.7853982, 0.0, 0.0]), &[ModelCube::of([15.0, 20.0], [-4.0, 0.0, 0.0], [8.0, 1.0, 0.0])], &[]),
        ModelPart::of("bottom_middle_fin", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([15.0, 20.0], [-4.0, 0.0, 0.0], [8.0, 1.0, 0.0])], &[]),
        ModelPart::of("left_back_fin", PartPose::offset_and_rotation([4.0, 22.0, 4.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([9.0, 17.0], [0.0, -8.0, 0.0], [1.0, 8.0, 0.0])], &[]),
        ModelPart::of("left_blue_fin", PartPose::offset([4.0, 15.0, -2.0]), &[ModelCube::of([24.0, 3.0], [0.0, 0.0, -1.0], [2.0, 1.0, 2.0])], &[]),
        ModelPart::of("left_front_fin", PartPose::offset_and_rotation([4.0, 22.0, -4.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([1.0, 17.0], [0.0, -8.0, 0.0], [1.0, 8.0, 0.0])], &[]),
        ModelPart::of("right_back_fin", PartPose::offset_and_rotation([-4.0, 22.0, 4.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([9.0, 17.0], [-1.0, -8.0, 0.0], [1.0, 8.0, 0.0])], &[]),
        ModelPart::of("right_blue_fin", PartPose::offset([-4.0, 15.0, -2.0]), &[ModelCube::of([24.0, 0.0], [-2.0, 0.0, -1.0], [2.0, 1.0, 2.0])], &[]),
        ModelPart::of("right_front_fin", PartPose::offset_and_rotation([-4.0, 22.0, -4.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([5.0, 17.0], [-1.0, -8.0, 0.0], [1.0, 8.0, 0.0])], &[]),
        ModelPart::of("top_back_fin", PartPose::offset_and_rotation([0.0, 14.0, 4.0], [-0.7853982, 0.0, 0.0]), &[ModelCube::of([23.0, 18.0], [-4.0, -1.0, 0.0], [8.0, 1.0, 0.0])], &[]),
        ModelPart::of("top_front_fin", PartPose::offset_and_rotation([0.0, 14.0, -4.0], [0.7853982, 0.0, 0.0]), &[ModelCube::of([15.0, 17.0], [-4.0, -1.0, 0.0], [8.0, 1.0, 0.0])], &[]),
        ModelPart::of("top_middle_fin", PartPose::offset([0.0, 14.0, 0.0]), &[ModelCube::of([14.0, 16.0], [-4.0, -1.0, 0.0], [8.0, 1.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:pufferfish_medium#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([12.0, 22.0], [-2.5, -5.0, -2.5], [5.0, 5.0, 5.0])], &[]),
        ModelPart::of("bottom_back_fin", PartPose::offset_and_rotation([-2.5, 22.0, 2.5], [0.7853982, 0.0, 0.0]), &[ModelCube::of([18.0, 20.0], [0.0, 0.0, 0.0], [5.0, 1.0, 0.0])], &[]),
        ModelPart::of("bottom_front_fin", PartPose::offset_and_rotation([0.0, 22.0, -2.5], [-0.7853982, 0.0, 0.0]), &[ModelCube::of([17.0, 19.0], [-2.5, 0.0, 0.0], [5.0, 1.0, 1.0])], &[]),
        ModelPart::of("left_back_fin", PartPose::offset_and_rotation([2.5, 22.0, 2.5], [0.0, -0.7853982, 0.0]), &[ModelCube::of([1.0, 17.0], [0.0, -5.0, 0.0], [1.0, 5.0, 0.0])], &[]),
        ModelPart::of("left_blue_fin", PartPose::offset([2.5, 18.0, -1.5]), &[ModelCube::of([24.0, 3.0], [0.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("left_front_fin", PartPose::offset_and_rotation([2.5, 22.0, -2.5], [0.0, 0.7853982, 0.0]), &[ModelCube::of([1.0, 17.0], [0.0, -5.0, 0.0], [1.0, 5.0, 0.0])], &[]),
        ModelPart::of("right_back_fin", PartPose::offset_and_rotation([-2.5, 22.0, 2.5], [0.0, 0.7853982, 0.0]), &[ModelCube::of([9.0, 17.0], [-1.0, -5.0, 0.0], [1.0, 5.0, 0.0])], &[]),
        ModelPart::of("right_blue_fin", PartPose::offset([-2.5, 18.0, -1.5]), &[ModelCube::of([24.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_front_fin", PartPose::offset_and_rotation([-2.5, 22.0, -2.5], [0.0, -0.7853982, 0.0]), &[ModelCube::of([5.0, 17.0], [-1.0, -5.0, 0.0], [1.0, 5.0, 0.0])], &[]),
        ModelPart::of("top_back_fin", PartPose::offset_and_rotation([0.0, 17.0, 2.5], [-0.7853982, 0.0, 0.0]), &[ModelCube::of([11.0, 17.0], [-2.5, -1.0, 0.0], [5.0, 1.0, 0.0])], &[]),
        ModelPart::of("top_front_fin", PartPose::offset_and_rotation([0.0, 17.0, -2.5], [0.7853982, 0.0, 0.0]), &[ModelCube::of([19.0, 17.0], [-2.5, -1.0, 0.0], [5.0, 1.0, 0.0])], &[]),
    ])),
    ModelLayer::of("minecraft:pufferfish_small#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back_fin", PartPose::offset([0.0, 22.0, 1.5]), &[ModelCube::of([-3.0, 0.0], [-1.5, 0.0, 0.0], [3.0, 0.0, 3.0])], &[]),
        ModelPart::of("body", PartPose::offset([0.0, 23.0, 0.0]), &[ModelCube::of([0.0, 27.0], [-1.5, -2.0, -1.5], [3.0, 2.0, 3.0])], &[]),
        ModelPart::of("left_eye", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([28.0, 6.0], [0.5, 0.0, -1.5], [1.0, 1.0, 1.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset([1.5, 22.0, -1.5]), &[ModelCube::of([25.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_eye", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([24.0, 6.0], [-1.5, 0.0, -1.5], [1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset([-1.5, 22.0, -1.5]), &[ModelCube::of([25.0, 0.0], [-1.0, 0.0, 0.0], [1.0, 0.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:rabbit#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("backlegs", PartPose::offset([0.0, 23.0, 4.0]), &[], &[
            ModelPart::of("left_hind_leg", PartPose::offset([3.0, 0.5, 0.0]), &[], &[
                ModelPart::of("left_haunch", PartPose::offset_and_rotation([0.0, -0.5, 0.0], [0.0, -0.3927, 0.0]), &[ModelCube::of([36.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 1.0, 6.0])], &[]),
            ]),
            ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 0.5, 0.0]), &[], &[
                ModelPart::of("right_haunch", PartPose::offset_and_rotation([0.0, -0.5, 0.0], [0.0, 0.3927, 0.0]), &[ModelCube::of([20.0, 24.0], [-1.0, 0.0, -5.0], [2.0, 1.0, 6.0])], &[]),
            ]),
        ]),
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 23.0, 4.0], [-0.3927, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -6.0, -9.0], [8.0, 6.0, 10.0])], &[
            ModelPart::of("frontlegs", PartPose::offset([0.0, -1.5349, -6.3108]), &[], &[
                ModelPart::of("left_front_leg", PartPose::offset_and_rotation([2.0, 1.9239, 0.4827], [0.3927, 0.0, 0.0]), &[ModelCube::of([44.0, 18.0], [-1.0, -1.0, -1.0], [2.0, 4.0, 2.0])], &[]),
                ModelPart::of("right_front_leg", PartPose::offset_and_rotation([-2.0, 1.9239, 0.3827], [0.3927, 0.0, 0.0]), &[ModelCube::of([36.0, 18.0], [-0.9, -1.0, -0.9], [2.0, 4.0, 2.0])], &[]),
            ]),
            ModelPart::of("head", PartPose::offset_and_rotation([0.0, -5.2929, -8.1213], [0.3927, 0.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.5, -3.0, -4.0], [5.0, 5.0, 5.0])], &[
                ModelPart::of("left_ear", PartPose::offset([1.5, -3.7071, -0.8787]), &[ModelCube::of([32.0, 0.0], [-1.0, -4.2929, -0.1213], [2.0, 5.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-1.5, -3.7071, -0.8787]), &[ModelCube::of([26.0, 0.0], [-1.0, -4.2929, -0.1213], [2.0, 5.0, 1.0])], &[]),
            ]),
            ModelPart::of("tail", PartPose::offset([0.0, -4.9916, 0.0125]), &[ModelCube::of([20.0, 16.0], [-2.0, -3.0084, -1.0125], [4.0, 4.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:rabbit_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("backlegs", PartPose::offset([0.0, 23.0, 2.0]), &[], &[
            ModelPart::of("left_hind_leg", PartPose::offset_and_rotation([1.5, 0.5, 0.5], [0.0, 3.1416, 0.0]), &[], &[
                ModelPart::of("left_haunch", PartPose::offset_and_rotation([1.0, 0.0, 0.5], [0.0, -0.7854, 0.0]), &[ModelCube::of([10.0, 17.0], [-2.0, -0.5, 0.0], [2.0, 1.0, 3.0])], &[]),
            ]),
            ModelPart::of("right_hind_leg", PartPose::offset_and_rotation([-1.5, 0.5, 0.5], [0.0, 3.1416, 0.0]), &[], &[
                ModelPart::of("right_haunch", PartPose::offset_and_rotation([0.5, 0.0, -0.9], [0.0, 0.7854, 0.0]), &[ModelCube::of([0.0, 17.0], [-2.0, -0.5, 0.0], [2.0, 1.0, 3.0])], &[]),
            ]),
        ]),
        ModelPart::of("body", PartPose::offset([0.0, 23.0, 1.6]), &[], &[
            ModelPart::of("body_r1", PartPose::offset_and_rotation([0.0, -2.0, -1.6], [-0.5236, 0.0, 0.0]), &[ModelCube::of([0.0, 8.0], [-2.0, -2.0, -3.0], [4.0, 3.0, 6.0])], &[]),
            ModelPart::of("frontlegs", PartPose::offset([0.0, -2.5, -2.6]), &[], &[
                ModelPart::of("left_front_leg", PartPose::offset_and_rotation([1.0, 1.0, -0.5], [0.3927, 0.0, 0.0]), &[], &[
                    ModelPart::of("left_front_leg_r1", PartPose::offset_and_rotation([0.0, 1.0, 0.0], [-0.3927, 0.0, 0.0]), &[ModelCube::of([18.0, 8.0], [-0.5, -1.5, -0.5], [1.0, 3.0, 1.0])], &[]),
                ]),
                ModelPart::of("right_front_leg", PartPose::offset_and_rotation([-1.0, 1.0, -0.5], [0.3927, 0.0, 0.0]), &[], &[
                    ModelPart::of("right_front_leg_r1", PartPose::offset_and_rotation([0.0, 1.0, 0.0], [-0.3927, 0.0, 0.0]), &[ModelCube::of([14.0, 8.0], [-0.5, -1.5, -0.5], [1.0, 3.0, 1.0])], &[]),
                ]),
            ]),
            ModelPart::of("head", PartPose::offset([0.0, -5.0, -2.6]), &[ModelCube::of([0.0, 0.0], [-2.5, -3.0, -3.0], [5.0, 4.0, 4.0])], &[
                ModelPart::of("left_ear", PartPose::offset([1.5, -3.5, -0.5]), &[ModelCube::of([24.0, 0.0], [-1.0, -3.5, -0.5], [2.0, 4.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset([-1.5, -3.5, -0.5]), &[ModelCube::of([18.0, 0.0], [-1.0, -3.5, -0.5], [2.0, 4.0, 1.0])], &[]),
            ]),
            ModelPart::of("tail", PartPose::offset([0.0, -2.2, 2.0]), &[], &[
                ModelPart::of("tail_r1", PartPose::offset_and_rotation([-0.1, 0.0, 0.0], [-0.5236, 0.0, 0.0]), &[ModelCube::of([0.0, 21.0], [-1.4, -2.0268, -1.0177], [3.0, 3.0, 3.0])], &[]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:ravager#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 1.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 55.0], [-7.0, -10.0, -7.0], [14.0, 16.0, 20.0]), ModelCube::of([0.0, 91.0], [-6.0, 6.0, -7.0], [12.0, 13.0, 18.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([8.0, -13.0, -5.0]), &[ModelCube::of([64.0, 0.0], [-4.0, 0.0, -4.0], [8.0, 37.0, 8.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([8.0, -13.0, 18.0]), &[ModelCube::of([96.0, 0.0], [-4.0, 0.0, -4.0], [8.0, 37.0, 8.0]).mirror()], &[]),
        ModelPart::of("neck", PartPose::offset([0.0, -7.0, 5.5]), &[ModelCube::of([68.0, 73.0], [-5.0, -1.0, -18.0], [10.0, 10.0, 18.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, 16.0, -17.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -20.0, -14.0], [16.0, 20.0, 16.0]), ModelCube::of([0.0, 0.0], [-2.0, -6.0, -18.0], [4.0, 8.0, 4.0])], &[
                ModelPart::of("left_horn", PartPose::offset_and_rotation([8.0, -14.0, -8.0], [1.0995574, 0.0, 0.0]), &[ModelCube::of([74.0, 55.0], [0.0, -14.0, -2.0], [2.0, 14.0, 4.0]).mirror()], &[]),
                ModelPart::of("mouth", PartPose::offset([0.0, -2.0, 2.0]), &[ModelCube::of([0.0, 36.0], [-8.0, 0.0, -16.0], [16.0, 3.0, 16.0])], &[]),
                ModelPart::of("right_horn", PartPose::offset_and_rotation([-10.0, -14.0, -8.0], [1.0995574, 0.0, 0.0]), &[ModelCube::of([74.0, 55.0], [0.0, -14.0, -2.0], [2.0, 14.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("right_front_leg", PartPose::offset([-8.0, -13.0, -5.0]), &[ModelCube::of([64.0, 0.0], [-4.0, 0.0, -4.0], [8.0, 37.0, 8.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-8.0, -13.0, 18.0]), &[ModelCube::of([96.0, 0.0], [-4.0, 0.0, -4.0], [8.0, 37.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:salmon#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body_back", PartPose::offset([0.0, 20.0, 0.8000002]), &[ModelCube::of([0.0, 13.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("back_fin", PartPose::offset([0.0, 0.0, 8.0]), &[ModelCube::of([20.0, 10.0], [0.0, -2.5, 0.0], [0.0, 5.0, 6.0])], &[]),
            ModelPart::of("top_back_fin", PartPose::offset([0.0, -4.5, -1.0]), &[ModelCube::of([0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 2.0, 4.0])], &[]),
        ]),
        ModelPart::of("body_front", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([0.0, 0.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("top_front_fin", PartPose::offset([0.0, -4.5, 5.0]), &[ModelCube::of([2.0, 1.0], [0.0, 0.0, 0.0], [0.0, 2.0, 3.0])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([22.0, 0.0], [-1.0, -2.0, -3.0], [2.0, 4.0, 3.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.5, 21.5, -7.2], [0.0, 0.0, 0.7853982]), &[ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.5, 21.5, -7.2], [0.0, 0.0, -0.7853982]), &[ModelCube::of([-4.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:salmon_large#main", 32, 32, ModelPart::of("root", PartPose::new([0.0, -12.008, 0.0], [0.0, 0.0, 0.0], [1.5, 1.5, 1.5]), &[], &[
        ModelPart::of("body_back", PartPose::offset([0.0, 20.0, 0.8000002]), &[ModelCube::of([0.0, 13.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("back_fin", PartPose::offset([0.0, 0.0, 8.0]), &[ModelCube::of([20.0, 10.0], [0.0, -2.5, 0.0], [0.0, 5.0, 6.0])], &[]),
            ModelPart::of("top_back_fin", PartPose::offset([0.0, -4.5, -1.0]), &[ModelCube::of([0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 2.0, 4.0])], &[]),
        ]),
        ModelPart::of("body_front", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([0.0, 0.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("top_front_fin", PartPose::offset([0.0, -4.5, 5.0]), &[ModelCube::of([2.0, 1.0], [0.0, 0.0, 0.0], [0.0, 2.0, 3.0])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([22.0, 0.0], [-1.0, -2.0, -3.0], [2.0, 4.0, 3.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.5, 21.5, -7.2], [0.0, 0.0, 0.7853982]), &[ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.5, 21.5, -7.2], [0.0, 0.0, -0.7853982]), &[ModelCube::of([-4.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:salmon_small#main", 32, 32, ModelPart::of("root", PartPose::new([0.0, 12.008, 0.0], [0.0, 0.0, 0.0], [0.5, 0.5, 0.5]), &[], &[
        ModelPart::of("body_back", PartPose::offset([0.0, 20.0, 0.8000002]), &[ModelCube::of([0.0, 13.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("back_fin", PartPose::offset([0.0, 0.0, 8.0]), &[ModelCube::of([20.0, 10.0], [0.0, -2.5, 0.0], [0.0, 5.0, 6.0])], &[]),
            ModelPart::of("top_back_fin", PartPose::offset([0.0, -4.5, -1.0]), &[ModelCube::of([0.0, 2.0], [0.0, 0.0, 0.0], [0.0, 2.0, 4.0])], &[]),
        ]),
        ModelPart::of("body_front", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([0.0, 0.0], [-1.5, -2.5, 0.0], [3.0, 5.0, 8.0])], &[
            ModelPart::of("top_front_fin", PartPose::offset([0.0, -4.5, 5.0]), &[ModelCube::of([2.0, 1.0], [0.0, 0.0, 0.0], [0.0, 2.0, 3.0])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset([0.0, 20.0, -7.2]), &[ModelCube::of([22.0, 0.0], [-1.0, -2.0, -3.0], [2.0, 4.0, 3.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.5, 21.5, -7.2], [0.0, 0.0, 0.7853982]), &[ModelCube::of([0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.5, 21.5, -7.2], [0.0, 0.0, -0.7853982]), &[ModelCube::of([-4.0, 0.0], [-2.0, 0.0, 0.0], [2.0, 0.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sheep#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-4.0, -10.0, -7.0], [8.0, 16.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 6.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -4.0, -6.0], [6.0, 6.0, 8.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
    ])),
    ModelLayer::of("minecraft:sheep#wool", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-4.0, -10.0, -7.0], [8.0, 16.0, 6.0]).grow([1.75, 1.75, 1.75])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 6.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -4.0, -4.0], [6.0, 6.0, 6.0]).grow([0.6, 0.6, 0.6])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 6.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
    ])),
    ModelLayer::of("minecraft:sheep#wool_undercoat", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([28.0, 8.0], [-4.0, -10.0, -7.0], [8.0, 16.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 6.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -4.0, -6.0], [6.0, 6.0, 8.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
    ])),
    ModelLayer::of("minecraft:sheep_baby#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.0, 0.5]), &[ModelCube::of([0.0, 10.0], [-3.0, -2.0, -4.5], [6.0, 4.0, 9.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.5, -2.5]), &[ModelCube::of([0.0, 0.0], [-2.5, -4.5, -3.5], [5.0, 5.0, 5.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 19.0, -2.0]), &[ModelCube::of([24.0, 5.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 19.0, 3.0]), &[ModelCube::of([24.0, 12.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 19.0, -2.0]), &[ModelCube::of([8.0, 23.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 19.0, 3.0]), &[ModelCube::of([0.0, 23.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sheep_baby#wool", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.0, 0.5]), &[ModelCube::of([0.0, 10.0], [-3.0, -2.0, -4.5], [6.0, 4.0, 9.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.5, -2.5]), &[ModelCube::of([0.0, 0.0], [-2.5, -4.5, -3.5], [5.0, 5.0, 5.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 19.0, -2.0]), &[ModelCube::of([24.0, 5.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 19.0, 3.0]), &[ModelCube::of([24.0, 12.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 19.0, -2.0]), &[ModelCube::of([8.0, 23.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 19.0, 3.0]), &[ModelCube::of([0.0, 23.0], [-1.0, 0.0, -1.0], [2.0, 5.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:shield#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("handle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-1.0, -3.0, -1.0], [2.0, 6.0, 6.0])], &[]),
        ModelPart::of("plate", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-6.0, -11.0, -2.0], [12.0, 22.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:shulker#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 28.0], [-8.0, -8.0, -8.0], [16.0, 8.0, 16.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, 0.0]), &[ModelCube::of([0.0, 52.0], [-3.0, 0.0, -3.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("lid", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -16.0, -8.0], [16.0, 12.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:shulker_box#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("base", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 28.0], [-8.0, -8.0, -8.0], [16.0, 8.0, 16.0])], &[]),
        ModelPart::of("lid", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -16.0, -8.0], [16.0, 12.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:shulker_bullet#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("main", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -1.0], [8.0, 8.0, 2.0]), ModelCube::of([0.0, 10.0], [-1.0, -4.0, -4.0], [2.0, 8.0, 8.0]), ModelCube::of([20.0, 0.0], [-4.0, -1.0, -4.0], [8.0, 2.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:silverfish#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("layer0", PartPose::offset([0.0, 16.0, 1.0]), &[ModelCube::of([20.0, 0.0], [-5.0, 0.0, -1.5], [10.0, 8.0, 3.0])], &[]),
        ModelPart::of("layer1", PartPose::offset([0.0, 20.0, 7.0]), &[ModelCube::of([20.0, 11.0], [-3.0, 0.0, -1.5], [6.0, 4.0, 3.0])], &[]),
        ModelPart::of("layer2", PartPose::offset([0.0, 19.0, -1.5]), &[ModelCube::of([20.0, 18.0], [-3.0, 0.0, -1.5], [6.0, 5.0, 2.0])], &[]),
        ModelPart::of("segment0", PartPose::offset([0.0, 22.0, -3.5]), &[ModelCube::of([0.0, 0.0], [-1.5, 0.0, -1.0], [3.0, 2.0, 2.0])], &[]),
        ModelPart::of("segment1", PartPose::offset([0.0, 21.0, -1.5]), &[ModelCube::of([0.0, 4.0], [-2.0, 0.0, -1.0], [4.0, 3.0, 2.0])], &[]),
        ModelPart::of("segment2", PartPose::offset([0.0, 20.0, 1.0]), &[ModelCube::of([0.0, 9.0], [-3.0, 0.0, -1.5], [6.0, 4.0, 3.0])], &[]),
        ModelPart::of("segment3", PartPose::offset([0.0, 21.0, 4.0]), &[ModelCube::of([0.0, 16.0], [-1.5, 0.0, -1.5], [3.0, 3.0, 3.0])], &[]),
        ModelPart::of("segment4", PartPose::offset([0.0, 22.0, 7.0]), &[ModelCube::of([0.0, 22.0], [-1.0, 0.0, -1.5], [2.0, 2.0, 3.0])], &[]),
        ModelPart::of("segment5", PartPose::offset([0.0, 23.0, 9.5]), &[ModelCube::of([11.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 1.0, 2.0])], &[]),
        ModelPart::of("segment6", PartPose::offset([0.0, 23.0, 11.5]), &[ModelCube::of([13.0, 4.0], [-0.5, 0.0, -1.0], [1.0, 1.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton_horse#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton_horse#saddle", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-5.0, -8.0, -9.0], [10.0, 9.0, 9.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("head_saddle", PartPose::ZERO, &[ModelCube::of([1.0, 1.0], [-3.0, -11.0, -1.9], [6.0, 5.0, 6.0]).grow([0.22, 0.22, 0.22])], &[]),
            ModelPart::of("left_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("left_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [2.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("mouth_saddle_wrap", PartPose::ZERO, &[ModelCube::of([19.0, 0.0], [-2.0, -11.0, -4.0], [4.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("right_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [-3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("right_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [-3.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton_horse_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 12.5, 0.0]), &[ModelCube::of([0.0, 13.0], [-4.0, -3.5, -7.0], [8.0, 7.0, 14.0])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -1.0, 7.0], [-0.7418, 0.0, 0.0]), &[ModelCube::of([24.0, 34.0], [-1.5, -1.5, -1.0], [3.0, 3.0, 8.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 10.0, -6.0], [0.6109, 0.0, 0.0]), &[ModelCube::of([30.0, 0.0], [-2.0, -6.0, -2.0], [4.0, 8.0, 4.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0516, -0.2951]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.9484, -6.705], [6.0, 4.0, 9.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([2.0, -4.2484, 1.9451], [0.0, 0.0, 0.2618]), &[ModelCube::of([0.0, 4.0], [-1.0, -2.5, -0.8], [2.0, 3.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-2.0, -4.2484, 1.645], [0.0, 0.0, -0.2618]), &[ModelCube::of([0.0, 0.0], [-1.0, -2.5, -0.5], [2.0, 3.0, 1.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([2.4, 16.0, -5.4]), &[ModelCube::of([12.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.4, 16.0, 5.4]), &[ModelCube::of([12.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.4, 16.0, -5.4]), &[ModelCube::of([0.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.4, 16.0, 5.4]), &[ModelCube::of([0.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:skeleton_skull#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:slime#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 16.0], [-3.0, 17.0, -3.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("left_eye", PartPose::ZERO, &[ModelCube::of([32.0, 4.0], [1.25, 18.0, -3.5], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("mouth", PartPose::ZERO, &[ModelCube::of([32.0, 8.0], [0.0, 21.0, -3.5], [1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_eye", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-3.25, 18.0, -3.5], [2.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:slime#outer", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, 16.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sniffer#main", 192, 192, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 5.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([62.0, 68.0], [-12.5, -14.0, -20.0], [25.0, 29.0, 40.0]), ModelCube::of([62.0, 0.0], [-12.5, -14.0, -20.0], [25.0, 24.0, 40.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([87.0, 68.0], [-12.5, 12.0, -20.0], [25.0, 0.0, 40.0])], &[
                ModelPart::of("head", PartPose::offset([0.0, 6.5, -19.48]), &[ModelCube::of([8.0, 15.0], [-6.5, -7.5, -11.5], [13.0, 18.0, 11.0]), ModelCube::of([8.0, 4.0], [-6.5, 7.5, -11.5], [13.0, 0.0, 11.0])], &[
                    ModelPart::of("left_ear", PartPose::offset([6.51, -7.5, -4.51]), &[ModelCube::of([2.0, 0.0], [0.0, 0.0, -3.0], [1.0, 19.0, 7.0])], &[]),
                    ModelPart::of("lower_beak", PartPose::offset([0.0, 2.5, -12.5]), &[ModelCube::of([10.0, 57.0], [-6.5, -7.0, -8.0], [13.0, 12.0, 9.0])], &[]),
                    ModelPart::of("nose", PartPose::offset([0.0, -4.5, -11.5]), &[ModelCube::of([10.0, 45.0], [-6.5, -2.0, -9.0], [13.0, 2.0, 9.0])], &[]),
                    ModelPart::of("right_ear", PartPose::offset([-6.51, -7.5, -4.51]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -3.0], [1.0, 19.0, 7.0])], &[]),
                ]),
            ]),
            ModelPart::of("left_front_leg", PartPose::offset([7.5, 10.0, -15.0]), &[ModelCube::of([0.0, 87.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
            ModelPart::of("left_hind_leg", PartPose::offset([7.5, 10.0, 15.0]), &[ModelCube::of([0.0, 123.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
            ModelPart::of("left_mid_leg", PartPose::offset([7.5, 10.0, 0.0]), &[ModelCube::of([0.0, 105.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
            ModelPart::of("right_front_leg", PartPose::offset([-7.5, 10.0, -15.0]), &[ModelCube::of([32.0, 87.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
            ModelPart::of("right_hind_leg", PartPose::offset([-7.5, 10.0, 15.0]), &[ModelCube::of([32.0, 123.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
            ModelPart::of("right_mid_leg", PartPose::offset([-7.5, 10.0, 0.0]), &[ModelCube::of([32.0, 105.0], [-3.5, -1.0, -4.0], [7.0, 10.0, 8.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:sniffer_baby#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([6.0, -3.0, -9.5]), &[ModelCube::of([0.0, 35.0], [-13.0, -14.0, -0.5], [14.0, 14.0, 20.0]).grow([0.25, 0.25, 0.25]), ModelCube::of([0.0, 0.0], [-13.0, -14.0, -0.5], [14.0, 15.0, 20.0]), ModelCube::of([68.0, 0.0], [-13.0, 0.0, -0.5], [14.0, 0.0, 20.0])], &[
                ModelPart::of("head", PartPose::offset([-6.0, -4.75, 0.0]), &[ModelCube::of([68.0, 20.0], [-5.0, -4.25, -7.5], [10.0, 9.0, 9.0]), ModelCube::of([88.0, 20.0], [-5.0, 3.75, -7.5], [10.0, 0.0, 9.0])], &[
                    ModelPart::of("left_ear", PartPose::offset([5.0, -4.25, -1.5]), &[ModelCube::of([104.0, 38.0], [0.0, 0.0, -2.0], [1.0, 11.0, 3.0])], &[]),
                    ModelPart::of("lower_beak", PartPose::offset([0.0, 1.25, -9.5]), &[ModelCube::of([68.0, 38.0], [-5.0, -2.5, -2.0], [10.0, 5.0, 4.0])], &[]),
                    ModelPart::of("nose", PartPose::offset([0.0, -1.25, -9.5]), &[ModelCube::of([68.0, 47.0], [-5.0, -3.0, -2.0], [10.0, 3.0, 4.0])], &[]),
                    ModelPart::of("right_ear", PartPose::offset([-5.0, -4.25, -1.5]), &[ModelCube::of([96.0, 38.0], [-1.0, 0.0, -2.0], [1.0, 11.0, 3.0])], &[]),
                ]),
            ]),
            ModelPart::of("left_front_leg", PartPose::offset([4.0, -4.0, -7.0]), &[ModelCube::of([16.0, 69.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
            ModelPart::of("left_hind_leg", PartPose::offset([4.0, -4.0, 7.0]), &[ModelCube::of([16.0, 87.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
            ModelPart::of("left_mid_leg", PartPose::offset([4.0, -4.0, 0.0]), &[ModelCube::of([16.0, 78.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_front_leg", PartPose::offset([-4.0, -4.0, -7.0]), &[ModelCube::of([0.0, 69.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_hind_leg", PartPose::offset([-4.0, -4.0, 7.0]), &[ModelCube::of([0.0, 87.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_mid_leg", PartPose::offset([-4.0, -4.0, 0.0]), &[ModelCube::of([0.0, 78.0], [-2.0, -1.0, -2.0], [4.0, 5.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:snow_golem#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::offset([0.0, 4.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([-0.5, -0.5, -0.5])], &[]),
        ModelPart::of("left_arm", PartPose::offset_and_rotation([5.0, 6.0, 1.0], [0.0, 0.0, 1.0]), &[ModelCube::of([32.0, 0.0], [-1.0, 0.0, -1.0], [12.0, 2.0, 2.0]).grow([-0.5, -0.5, -0.5])], &[]),
        ModelPart::of("lower_body", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-6.0, -12.0, -6.0], [12.0, 12.0, 12.0]).grow([-0.5, -0.5, -0.5])], &[]),
        ModelPart::of("right_arm", PartPose::offset_and_rotation([-5.0, 6.0, -1.0], [0.0, 3.1415927, -1.0]), &[ModelCube::of([32.0, 0.0], [-1.0, 0.0, -1.0], [12.0, 2.0, 2.0]).grow([-0.5, -0.5, -0.5])], &[]),
        ModelPart::of("upper_body", PartPose::offset([0.0, 13.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-5.0, -10.0, -5.0], [10.0, 10.0, 10.0]).grow([-0.5, -0.5, -0.5])], &[]),
    ])),
    ModelLayer::of("minecraft:spawner_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:spider#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body0", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.0, -3.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("body1", PartPose::offset([0.0, 15.0, 9.0]), &[ModelCube::of([0.0, 12.0], [-5.0, -4.0, -6.0], [10.0, 8.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, -3.0]), &[ModelCube::of([32.0, 4.0], [-4.0, -4.0, -8.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset_and_rotation([4.0, 15.0, -1.0], [0.0, 0.7853982, 0.7853982]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset_and_rotation([4.0, 15.0, 2.0], [0.0, -0.7853982, 0.7853982]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_middle_front_leg", PartPose::offset_and_rotation([4.0, 15.0, 0.0], [0.0, 0.3926991, 0.58119464]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_middle_hind_leg", PartPose::offset_and_rotation([4.0, 15.0, 1.0], [0.0, -0.3926991, 0.58119464]), &[ModelCube::of([18.0, 0.0], [-1.0, -1.0, -1.0], [16.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset_and_rotation([-4.0, 15.0, -1.0], [0.0, -0.7853982, -0.7853982]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset_and_rotation([-4.0, 15.0, 2.0], [0.0, 0.7853982, -0.7853982]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_middle_front_leg", PartPose::offset_and_rotation([-4.0, 15.0, 0.0], [0.0, -0.3926991, -0.58119464]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_middle_hind_leg", PartPose::offset_and_rotation([-4.0, 15.0, 1.0], [0.0, 0.3926991, -0.58119464]), &[ModelCube::of([18.0, 0.0], [-15.0, -1.0, -1.0], [16.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:spin_attack#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("box0", PartPose::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.75, 0.75, 0.75]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.599999, -8.0], [16.0, 32.0, 16.0])], &[]),
        ModelPart::of("box1", PartPose::new([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.5, 1.5, 1.5]), &[ModelCube::of([0.0, 0.0], [-8.0, 0.0, -8.0], [16.0, 32.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:squid#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 8.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-6.0, -8.0, -6.0], [12.0, 16.0, 12.0]).grow([0.02, 0.02, 0.02])], &[]),
        ModelPart::of("tentacle0", PartPose::offset_and_rotation([5.0, 15.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle1", PartPose::offset_and_rotation([3.535534, 15.0, 3.535534], [0.0, 0.7853982, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle2", PartPose::offset([3.061617e-16, 15.0, 5.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle3", PartPose::offset_and_rotation([-3.535534, 15.0, 3.535534], [0.0, -0.7853982, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle4", PartPose::offset_and_rotation([-5.0, 15.0, 6.123234e-16], [0.0, -1.5707964, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle5", PartPose::offset_and_rotation([-3.535534, 15.0, -3.535534], [0.0, -2.3561945, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle6", PartPose::offset_and_rotation([-9.18485e-16, 15.0, -5.0], [0.0, -3.1415927, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
        ModelPart::of("tentacle7", PartPose::offset_and_rotation([3.535534, 15.0, -3.535534], [0.0, -3.9269907, 0.0]), &[ModelCube::of([48.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 18.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:squid_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 13.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -5.0, -4.0], [8.0, 10.0, 8.0])], &[]),
        ModelPart::of("tentacle0", PartPose::offset_and_rotation([3.0, 18.5, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle1", PartPose::offset_and_rotation([2.1213202, 18.5, 2.1213202], [0.0, 0.7853982, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle2", PartPose::offset([1.8369703e-16, 18.5, 3.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle3", PartPose::offset_and_rotation([-2.1213202, 18.5, 2.1213202], [0.0, -0.7853982, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle4", PartPose::offset_and_rotation([-3.0, 18.5, 3.6739406e-16], [0.0, -1.5707964, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle5", PartPose::offset_and_rotation([-2.1213202, 18.5, -2.1213202], [0.0, -2.3561945, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle6", PartPose::offset_and_rotation([-5.5109107e-16, 18.5, -3.0], [0.0, -3.1415927, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
        ModelPart::of("tentacle7", PartPose::offset_and_rotation([2.1213202, 18.5, -2.1213202], [0.0, -3.9269907, 0.0]), &[ModelCube::of([0.0, 18.0], [-1.0, -0.5, -1.0], [2.0, 6.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:standing_banner#flag", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("flag", PartPose::offset([0.0, -44.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-10.0, 0.0, -2.0], [20.0, 40.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:standing_banner#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bar", PartPose::ZERO, &[ModelCube::of([0.0, 42.0], [-10.0, -44.0, -1.0], [20.0, 2.0, 2.0])], &[]),
        ModelPart::of("pole", PartPose::ZERO, &[ModelCube::of([44.0, 0.0], [-1.0, -42.0, -1.0], [2.0, 42.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:stray#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:stray#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:stray#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:stray#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:stray#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:stray#outer", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.75, 0.75, 0.75])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
    ])),
    ModelLayer::of("minecraft:strider#main", 64, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 1.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -6.0, -8.0], [16.0, 14.0, 16.0])], &[
            ModelPart::of("left_bottom_bristle", PartPose::offset_and_rotation([8.0, 3.0, -8.0], [0.0, 0.0, 1.2217305]), &[ModelCube::of([16.0, 65.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("left_middle_bristle", PartPose::offset_and_rotation([8.0, -2.0, -8.0], [0.0, 0.0, 1.134464]), &[ModelCube::of([16.0, 49.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("left_top_bristle", PartPose::offset_and_rotation([8.0, -6.0, -8.0], [0.0, 0.0, 0.87266463]), &[ModelCube::of([16.0, 33.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("right_bottom_bristle", PartPose::offset_and_rotation([-8.0, 4.0, -8.0], [0.0, 0.0, -1.2217305]), &[ModelCube::of([16.0, 65.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
            ModelPart::of("right_middle_bristle", PartPose::offset_and_rotation([-8.0, -1.0, -8.0], [0.0, 0.0, -1.134464]), &[ModelCube::of([16.0, 49.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
            ModelPart::of("right_top_bristle", PartPose::offset_and_rotation([-8.0, -5.0, -8.0], [0.0, 0.0, -0.87266463]), &[ModelCube::of([16.0, 33.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([4.0, 8.0, 0.0]), &[ModelCube::of([0.0, 55.0], [-2.0, 0.0, -2.0], [4.0, 16.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-4.0, 8.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 16.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:strider#saddle", 64, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 1.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -6.0, -8.0], [16.0, 14.0, 16.0])], &[
            ModelPart::of("left_bottom_bristle", PartPose::offset_and_rotation([8.0, 3.0, -8.0], [0.0, 0.0, 1.2217305]), &[ModelCube::of([16.0, 65.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("left_middle_bristle", PartPose::offset_and_rotation([8.0, -2.0, -8.0], [0.0, 0.0, 1.134464]), &[ModelCube::of([16.0, 49.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("left_top_bristle", PartPose::offset_and_rotation([8.0, -6.0, -8.0], [0.0, 0.0, 0.87266463]), &[ModelCube::of([16.0, 33.0], [0.0, 0.0, 0.0], [12.0, 0.0, 16.0])], &[]),
            ModelPart::of("right_bottom_bristle", PartPose::offset_and_rotation([-8.0, 4.0, -8.0], [0.0, 0.0, -1.2217305]), &[ModelCube::of([16.0, 65.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
            ModelPart::of("right_middle_bristle", PartPose::offset_and_rotation([-8.0, -1.0, -8.0], [0.0, 0.0, -1.134464]), &[ModelCube::of([16.0, 49.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
            ModelPart::of("right_top_bristle", PartPose::offset_and_rotation([-8.0, -5.0, -8.0], [0.0, 0.0, -0.87266463]), &[ModelCube::of([16.0, 33.0], [-12.0, 0.0, 0.0], [12.0, 0.0, 16.0]).mirror()], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([4.0, 8.0, 0.0]), &[ModelCube::of([0.0, 55.0], [-2.0, 0.0, -2.0], [4.0, 16.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-4.0, 8.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 16.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:strider_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 16.75, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.5, -3.75, -4.0], [7.0, 7.0, 8.0])], &[
            ModelPart::of("bristle0", PartPose::offset([0.0, -4.25, 2.0]), &[ModelCube::of([0.0, 21.0], [-3.5, -2.5, 0.0], [7.0, 3.0, 0.0])], &[]),
            ModelPart::of("bristle1", PartPose::offset([0.0, -4.25, 0.0]), &[ModelCube::of([0.0, 18.0], [-3.5, -2.5, 0.0], [7.0, 3.0, 0.0])], &[]),
            ModelPart::of("bristle2", PartPose::offset([0.0, -4.25, -2.0]), &[ModelCube::of([0.0, 15.0], [-3.5, -2.5, 0.0], [7.0, 3.0, 0.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.0]), &[ModelCube::of([8.0, 24.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.0]), &[ModelCube::of([0.0, 24.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sulfur_cube#inner", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 36.0], [-8.0, -8.0, -8.0], [16.0, 16.0, 16.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sulfur_cube#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-9.0, -9.0, -9.0], [18.0, 18.0, 18.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sulfur_cube_small#inner", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 20.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:sulfur_cube_small#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("cube", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-5.0, -5.0, -5.0], [10.0, 10.0, 10.0])], &[]),
    ])),
    ModelLayer::of("minecraft:tadpole#main", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-1.5, -1.0, 0.0], [3.0, 2.0, 3.0])], &[]),
        ModelPart::of("tail", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 2.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:tnt_minecart#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("back", PartPose::offset_and_rotation([9.0, 4.0, 0.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("bottom", PartPose::offset_and_rotation([0.0, 4.0, 0.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-10.0, -8.0, -1.0], [20.0, 16.0, 2.0])], &[]),
        ModelPart::of("front", PartPose::offset_and_rotation([-9.0, 4.0, 0.0], [0.0, 4.712389, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("left", PartPose::offset_and_rotation([0.0, 4.0, -7.0], [0.0, 3.1415927, 0.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
        ModelPart::of("right", PartPose::offset([0.0, 4.0, 7.0]), &[ModelCube::of([0.0, 0.0], [-8.0, -9.0, -1.0], [16.0, 8.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:trader_llama#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([29.0, 0.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 7.0, -6.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -14.0, -10.0], [4.0, 4.0, 9.0]), ModelCube::of([0.0, 14.0], [-4.0, -16.0, -6.0], [8.0, 18.0, 6.0]), ModelCube::of([17.0, 0.0], [-4.0, -19.0, -4.0], [3.0, 3.0, 2.0]), ModelCube::of([17.0, 0.0], [1.0, -19.0, -4.0], [3.0, 3.0, 2.0])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 3.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-3.5, 10.0, -5.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.5, 10.0, 6.0]), &[ModelCube::of([29.0, 29.0], [-2.0, 0.0, -2.0], [4.0, 14.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:trader_llama_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 14.0, 2.5]), &[ModelCube::of([0.0, 15.0], [-4.0, -3.0, -8.5], [8.0, 6.0, 13.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 12.0, -4.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -9.0, -4.0], [6.0, 11.0, 4.0]), ModelCube::of([0.0, 15.0], [-1.5, -7.0, -7.0], [3.0, 3.0, 3.0]), ModelCube::of([20.0, 4.0], [0.5, -11.0, -3.0], [2.0, 2.0, 2.0]), ModelCube::of([20.0, 0.0], [-2.5, -11.0, -3.0], [2.0, 2.0, 2.0])], &[]),
        ModelPart::of("left_chest", PartPose::offset_and_rotation([5.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 41.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 16.5, -3.5]), &[ModelCube::of([12.0, 34.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 16.5, 4.5]), &[ModelCube::of([12.0, 45.0], [-1.6, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_chest", PartPose::offset_and_rotation([-8.5, 4.0, 3.0], [0.0, 1.5707964, 0.0]), &[ModelCube::of([45.0, 28.0], [-3.0, 0.0, 0.0], [8.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 16.5, -3.5]), &[ModelCube::of([0.0, 34.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 16.5, 4.5]), &[ModelCube::of([0.0, 45.0], [-1.4, -0.5, -1.5], [3.0, 8.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:trident#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("pole", PartPose::ZERO, &[ModelCube::of([0.0, 6.0], [-0.5, 2.0, -0.5], [1.0, 25.0, 1.0])], &[
            ModelPart::of("base", PartPose::ZERO, &[ModelCube::of([4.0, 0.0], [-1.5, 0.0, -0.5], [3.0, 2.0, 1.0])], &[]),
            ModelPart::of("left_spike", PartPose::ZERO, &[ModelCube::of([4.0, 3.0], [-2.5, -3.0, -0.5], [1.0, 4.0, 1.0])], &[]),
            ModelPart::of("middle_spike", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-0.5, -4.0, -0.5], [1.0, 4.0, 1.0])], &[]),
            ModelPart::of("right_spike", PartPose::ZERO, &[ModelCube::of([4.0, 3.0], [1.5, -3.0, -0.5], [1.0, 4.0, 1.0]).mirror()], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:tropical_fish_large#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 20.0], [-1.0, -3.0, -3.0], [2.0, 6.0, 6.0])], &[]),
        ModelPart::of("bottom_fin", PartPose::offset([0.0, 22.0, -3.0]), &[ModelCube::of([20.0, 21.0], [0.0, 0.0, 0.0], [0.0, 4.0, 6.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.0, 20.0, 0.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([2.0, 12.0], [0.0, 0.0, 0.0], [2.0, 2.0, 0.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.0, 20.0, 0.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([2.0, 16.0], [-2.0, 0.0, 0.0], [2.0, 2.0, 0.0])], &[]),
        ModelPart::of("tail", PartPose::offset([0.0, 19.0, 3.0]), &[ModelCube::of([21.0, 16.0], [0.0, -3.0, 0.0], [0.0, 6.0, 5.0])], &[]),
        ModelPart::of("top_fin", PartPose::offset([0.0, 16.0, -3.0]), &[ModelCube::of([20.0, 11.0], [0.0, -4.0, 0.0], [0.0, 4.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:tropical_fish_large#pattern", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 20.0], [-1.0, -3.0, -3.0], [2.0, 6.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("bottom_fin", PartPose::offset([0.0, 22.0, -3.0]), &[ModelCube::of([20.0, 21.0], [0.0, 0.0, 0.0], [0.0, 4.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.0, 20.0, 0.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([2.0, 12.0], [0.0, 0.0, 0.0], [2.0, 2.0, 0.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.0, 20.0, 0.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([2.0, 16.0], [-2.0, 0.0, 0.0], [2.0, 2.0, 0.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("tail", PartPose::offset([0.0, 19.0, 3.0]), &[ModelCube::of([21.0, 16.0], [0.0, -3.0, 0.0], [0.0, 6.0, 5.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("top_fin", PartPose::offset([0.0, 16.0, -3.0]), &[ModelCube::of([20.0, 11.0], [0.0, -4.0, 0.0], [0.0, 4.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
    ])),
    ModelLayer::of("minecraft:tropical_fish_small#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -1.5, -3.0], [2.0, 3.0, 6.0])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.0, 22.5, 0.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([2.0, 12.0], [0.0, -1.0, 0.0], [2.0, 2.0, 0.0])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.0, 22.5, 0.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([2.0, 16.0], [-2.0, -1.0, 0.0], [2.0, 2.0, 0.0])], &[]),
        ModelPart::of("tail", PartPose::offset([0.0, 22.0, 3.0]), &[ModelCube::of([22.0, -6.0], [0.0, -1.5, 0.0], [0.0, 3.0, 6.0])], &[]),
        ModelPart::of("top_fin", PartPose::offset([0.0, 20.5, -3.0]), &[ModelCube::of([10.0, -5.0], [0.0, -3.0, 0.0], [0.0, 3.0, 6.0])], &[]),
    ])),
    ModelLayer::of("minecraft:tropical_fish_small#pattern", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-1.0, -1.5, -3.0], [2.0, 3.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("left_fin", PartPose::offset_and_rotation([1.0, 22.5, 0.0], [0.0, -0.7853982, 0.0]), &[ModelCube::of([2.0, 12.0], [0.0, -1.0, 0.0], [2.0, 2.0, 0.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("right_fin", PartPose::offset_and_rotation([-1.0, 22.5, 0.0], [0.0, 0.7853982, 0.0]), &[ModelCube::of([2.0, 16.0], [-2.0, -1.0, 0.0], [2.0, 2.0, 0.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("tail", PartPose::offset([0.0, 22.0, 3.0]), &[ModelCube::of([22.0, -6.0], [0.0, -1.5, 0.0], [0.0, 3.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
        ModelPart::of("top_fin", PartPose::offset([0.0, 20.5, -3.0]), &[ModelCube::of([10.0, -5.0], [0.0, -3.0, 0.0], [0.0, 3.0, 6.0]).grow([0.008, 0.008, 0.008])], &[]),
    ])),
    ModelLayer::of("minecraft:turtle#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 11.0, -10.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([7.0, 37.0], [-9.5, 3.0, -10.0], [19.0, 20.0, 6.0]), ModelCube::of([31.0, 1.0], [-5.5, 3.0, -13.0], [11.0, 18.0, 3.0])], &[]),
        ModelPart::of("egg_belly", PartPose::offset_and_rotation([0.0, 11.0, -10.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([70.0, 33.0], [-4.5, 3.0, -14.0], [9.0, 18.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 19.0, -10.0]), &[ModelCube::of([3.0, 0.0], [-3.0, -1.0, -3.0], [6.0, 5.0, 6.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([5.0, 21.0, -4.0]), &[ModelCube::of([27.0, 24.0], [0.0, 0.0, -2.0], [13.0, 1.0, 5.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([3.5, 22.0, 11.0]), &[ModelCube::of([1.0, 12.0], [-2.0, 0.0, 0.0], [4.0, 1.0, 10.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-5.0, 21.0, -4.0]), &[ModelCube::of([27.0, 30.0], [-13.0, 0.0, -2.0], [13.0, 1.0, 5.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-3.5, 22.0, 11.0]), &[ModelCube::of([1.0, 23.0], [-2.0, 0.0, 0.0], [4.0, 1.0, 10.0])], &[]),
    ])),
    ModelLayer::of("minecraft:turtle_baby#main", 16, 16, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 22.9, 1.0]), &[ModelCube::of([0.0, 0.0], [-2.0, -1.0, -2.0], [4.0, 2.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 22.9, -1.0]), &[ModelCube::of([0.0, 6.0], [-1.5, -2.0, -3.0], [3.0, 3.0, 3.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.0, 23.9, -0.5]), &[ModelCube::of([8.0, 7.0], [0.0, 0.0, -0.5], [2.0, 0.0, 1.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.0, 23.9, 2.5]), &[ModelCube::of([-1.0, 1.0], [0.0, 0.0, -0.5], [2.0, 0.0, 1.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.0, 23.9, -0.5]), &[ModelCube::of([8.0, 6.0], [-2.0, 0.0, -0.5], [2.0, 0.0, 1.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.0, 23.9, 2.5]), &[ModelCube::of([-1.0, 0.0], [-2.0, 0.0, -0.5], [2.0, 0.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:undead_horse_armor#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0]).grow([0.1, 0.1, 0.1])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0]).grow([0.1, 0.1, 0.1])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0]).grow([0.1, 0.1, 0.1])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0]).grow([0.1, 0.1, 0.1])], &[]),
    ])),
    ModelLayer::of("minecraft:vex#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, -2.5, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([0.0, 10.0], [-1.5, 0.0, -1.0], [3.0, 4.0, 2.0]), ModelCube::of([0.0, 16.0], [-1.5, 1.0, -1.0], [3.0, 5.0, 2.0]).grow([-0.2, -0.2, -0.2])], &[
                ModelPart::of("left_arm", PartPose::offset([1.75, 0.25, 0.0]), &[ModelCube::of([23.0, 6.0], [-0.75, -0.5, -1.0], [2.0, 4.0, 2.0]).grow([-0.1, -0.1, -0.1])], &[]),
                ModelPart::of("left_wing", PartPose::offset([0.5, 1.0, 1.0]), &[ModelCube::of([16.0, 14.0], [0.0, 0.0, 0.0], [0.0, 5.0, 8.0]).mirror()], &[]),
                ModelPart::of("right_arm", PartPose::offset([-1.75, 0.25, 0.0]), &[ModelCube::of([23.0, 0.0], [-1.25, -0.5, -1.0], [2.0, 4.0, 2.0]).grow([-0.1, -0.1, -0.1])], &[]),
                ModelPart::of("right_wing", PartPose::offset([-0.5, 1.0, 1.0]), &[ModelCube::of([16.0, 14.0], [0.0, 0.0, 0.0], [0.0, 5.0, 8.0])], &[]),
            ]),
            ModelPart::of("head", PartPose::offset([0.0, 20.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-2.5, -5.0, -2.5], [5.0, 5.0, 5.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:villager#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror(), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0]).grow([0.51, 0.51, 0.51])], &[
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[ModelCube::of([30.0, 47.0], [-8.0, -8.0, -6.0], [16.0, 16.0, 1.0])], &[]),
            ]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:villager_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("arms", PartPose::offset([0.0, 17.5, 0.0]), &[], &[
            ModelPart::of("middlearm_r1", PartPose::offset_and_rotation([0.0, 0.9024, -1.8175], [-1.0472, 0.0, 0.0]), &[ModelCube::of([24.0, 17.0], [-2.0, -0.9924, -0.9825], [4.0, 2.0, 2.0])], &[]),
            ModelPart::of("right_hand", PartPose::offset_and_rotation([-3.0, 1.4025, -0.9599], [-1.0472, 0.0, 0.0]), &[ModelCube::of([36.0, 15.0], [-1.0, -2.4925, -1.8401], [2.0, 4.0, 2.0]), ModelCube::of([16.0, 15.0], [5.0, -2.4925, -1.8401], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("bb_main", PartPose::offset([0.5, 24.0, 0.0]), &[ModelCube::of([16.0, 21.0], [-2.5, -8.0, -1.5], [4.0, 6.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("body", PartPose::offset([0.0, 18.75, 0.0]), &[ModelCube::of([0.0, 15.0], [-2.0, -2.75, -1.5], [4.0, 5.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -3.5], [8.0, 8.0, 7.0])], &[
            ModelPart::of("hat", PartPose::offset([0.0, -4.0, 0.0]), &[ModelCube::of([0.0, 30.0], [-4.0, -4.0, -3.5], [8.0, 8.0, 7.0]).grow([0.3, 0.3, 0.3])], &[]),
            ModelPart::of("hat_rim", PartPose::offset([0.0, -4.5, 0.0]), &[ModelCube::of([0.0, 45.0], [-7.0, -0.5, -6.0], [14.0, 1.0, 12.0])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, -4.0]), &[ModelCube::of([23.0, 0.0], [-1.0, 0.0, -0.5], [2.0, 2.0, 1.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 21.5, 0.0]), &[ModelCube::of([0.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 21.5, 0.0]), &[ModelCube::of([8.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:villager_baby_no_hat#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("arms", PartPose::offset([0.0, 17.5, 0.0]), &[], &[
            ModelPart::of("middlearm_r1", PartPose::offset_and_rotation([0.0, 0.9024, -1.8175], [-1.0472, 0.0, 0.0]), &[ModelCube::of([24.0, 17.0], [-2.0, -0.9924, -0.9825], [4.0, 2.0, 2.0])], &[]),
            ModelPart::of("right_hand", PartPose::offset_and_rotation([-3.0, 1.4025, -0.9599], [-1.0472, 0.0, 0.0]), &[ModelCube::of([36.0, 15.0], [-1.0, -2.4925, -1.8401], [2.0, 4.0, 2.0]), ModelCube::of([16.0, 15.0], [5.0, -2.4925, -1.8401], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("bb_main", PartPose::offset([0.5, 24.0, 0.0]), &[ModelCube::of([16.0, 21.0], [-2.5, -8.0, -1.5], [4.0, 6.0, 3.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("body", PartPose::offset([0.0, 18.75, 0.0]), &[ModelCube::of([0.0, 15.0], [-2.0, -2.75, -1.5], [4.0, 5.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 16.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::offset([0.0, -4.0, 0.0]), &[], &[]),
            ModelPart::of("hat_rim", PartPose::offset([0.0, -4.5, 0.0]), &[], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, -4.0]), &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 21.5, 0.0]), &[ModelCube::of([0.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 21.5, 0.0]), &[ModelCube::of([8.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:villager_no_hat#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror(), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[], &[]),
            ]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:vindicator#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[
            ModelPart::of("left_shoulder", PartPose::ZERO, &[ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror()], &[]),
        ]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 12.0, 8.0]).grow([0.45, 0.45, 0.45])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 46.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wall_banner#flag", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("flag", PartPose::offset([0.0, -20.5, 10.5]), &[ModelCube::of([0.0, 0.0], [-10.0, 0.0, -2.0], [20.0, 40.0, 1.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wall_banner#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bar", PartPose::ZERO, &[ModelCube::of([0.0, 42.0], [-10.0, -20.5, 9.5], [20.0, 2.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wandering_trader#main", 64, 64, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror(), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0]).grow([0.51, 0.51, 0.51])], &[
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[ModelCube::of([30.0, 47.0], [-8.0, -8.0, -6.0], [16.0, 16.0, 1.0])], &[]),
            ]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:warden#bioluminescent", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -21.0, 0.0]), &[], &[
                ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-8.0, -16.0, -5.0], [16.0, 16.0, 10.0])], &[
                    ModelPart::of("left_tendril", PartPose::offset([8.0, -12.0, 0.0]), &[], &[]),
                    ModelPart::of("right_tendril", PartPose::offset([-8.0, -12.0, 0.0]), &[], &[]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([13.0, -13.0, 1.0]), &[ModelCube::of([0.0, 58.0], [-4.0, 0.0, -4.0], [8.0, 28.0, 8.0])], &[]),
                ModelPart::of("left_ribcage", PartPose::offset([7.0, -2.0, -4.0]), &[], &[]),
                ModelPart::of("right_arm", PartPose::offset([-13.0, -13.0, 1.0]), &[ModelCube::of([44.0, 50.0], [-4.0, 0.0, -4.0], [8.0, 28.0, 8.0])], &[]),
                ModelPart::of("right_ribcage", PartPose::offset([-7.0, -2.0, -4.0]), &[], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 76.0], [-2.9, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
            ModelPart::of("right_leg", PartPose::offset([-5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 48.0], [-3.1, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:warden#heart", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -21.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-9.0, -13.0, -4.0], [18.0, 21.0, 11.0])], &[
                ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[], &[
                    ModelPart::of("left_tendril", PartPose::offset([8.0, -12.0, 0.0]), &[], &[]),
                    ModelPart::of("right_tendril", PartPose::offset([-8.0, -12.0, 0.0]), &[], &[]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("left_ribcage", PartPose::offset([7.0, -2.0, -4.0]), &[], &[]),
                ModelPart::of("right_arm", PartPose::offset([-13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("right_ribcage", PartPose::offset([-7.0, -2.0, -4.0]), &[], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([5.9, -13.0, 0.0]), &[], &[]),
            ModelPart::of("right_leg", PartPose::offset([-5.9, -13.0, 0.0]), &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:warden#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -21.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-9.0, -13.0, -4.0], [18.0, 21.0, 11.0])], &[
                ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[ModelCube::of([0.0, 32.0], [-8.0, -16.0, -5.0], [16.0, 16.0, 10.0])], &[
                    ModelPart::of("left_tendril", PartPose::offset([8.0, -12.0, 0.0]), &[ModelCube::of([58.0, 0.0], [0.0, -13.0, 0.0], [16.0, 16.0, 0.0])], &[]),
                    ModelPart::of("right_tendril", PartPose::offset([-8.0, -12.0, 0.0]), &[ModelCube::of([52.0, 32.0], [-16.0, -13.0, 0.0], [16.0, 16.0, 0.0])], &[]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([13.0, -13.0, 1.0]), &[ModelCube::of([0.0, 58.0], [-4.0, 0.0, -4.0], [8.0, 28.0, 8.0])], &[]),
                ModelPart::of("left_ribcage", PartPose::offset([7.0, -2.0, -4.0]), &[ModelCube::of([90.0, 11.0], [-7.0, -11.0, -0.1], [9.0, 21.0, 0.0]).mirror()], &[]),
                ModelPart::of("right_arm", PartPose::offset([-13.0, -13.0, 1.0]), &[ModelCube::of([44.0, 50.0], [-4.0, 0.0, -4.0], [8.0, 28.0, 8.0])], &[]),
                ModelPart::of("right_ribcage", PartPose::offset([-7.0, -2.0, -4.0]), &[ModelCube::of([90.0, 11.0], [-2.0, -11.0, -0.1], [9.0, 21.0, 0.0])], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 76.0], [-2.9, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
            ModelPart::of("right_leg", PartPose::offset([-5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 48.0], [-3.1, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:warden#pulsating_spots", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -21.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-9.0, -13.0, -4.0], [18.0, 21.0, 11.0])], &[
                ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[], &[
                    ModelPart::of("left_tendril", PartPose::offset([8.0, -12.0, 0.0]), &[], &[]),
                    ModelPart::of("right_tendril", PartPose::offset([-8.0, -12.0, 0.0]), &[], &[]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("left_ribcage", PartPose::offset([7.0, -2.0, -4.0]), &[], &[]),
                ModelPart::of("right_arm", PartPose::offset([-13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("right_ribcage", PartPose::offset([-7.0, -2.0, -4.0]), &[], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 76.0], [-2.9, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
            ModelPart::of("right_leg", PartPose::offset([-5.9, -13.0, 0.0]), &[ModelCube::of([76.0, 48.0], [-3.1, 0.0, -3.0], [6.0, 13.0, 6.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:warden#tendrils", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::offset([0.0, 24.0, 0.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -21.0, 0.0]), &[], &[
                ModelPart::of("head", PartPose::offset([0.0, -13.0, 0.0]), &[], &[
                    ModelPart::of("left_tendril", PartPose::offset([8.0, -12.0, 0.0]), &[ModelCube::of([58.0, 0.0], [0.0, -13.0, 0.0], [16.0, 16.0, 0.0])], &[]),
                    ModelPart::of("right_tendril", PartPose::offset([-8.0, -12.0, 0.0]), &[ModelCube::of([52.0, 32.0], [-16.0, -13.0, 0.0], [16.0, 16.0, 0.0])], &[]),
                ]),
                ModelPart::of("left_arm", PartPose::offset([13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("left_ribcage", PartPose::offset([7.0, -2.0, -4.0]), &[], &[]),
                ModelPart::of("right_arm", PartPose::offset([-13.0, -13.0, 1.0]), &[], &[]),
                ModelPart::of("right_ribcage", PartPose::offset([-7.0, -2.0, -4.0]), &[], &[]),
            ]),
            ModelPart::of("left_leg", PartPose::offset([5.9, -13.0, 0.0]), &[], &[]),
            ModelPart::of("right_leg", PartPose::offset([-5.9, -13.0, 0.0]), &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:warm_cow#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 5.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([18.0, 4.0], [-6.0, -10.0, -7.0], [12.0, 18.0, 10.0]), ModelCube::of([52.0, 0.0], [-2.0, 2.0, -8.0], [4.0, 6.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 4.0, -8.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -6.0], [8.0, 8.0, 6.0]), ModelCube::of([1.0, 33.0], [-3.0, 1.0, -7.0], [6.0, 3.0, 1.0]), ModelCube::of([27.0, 0.0], [-8.0, -3.0, -5.0], [4.0, 2.0, 2.0]), ModelCube::of([39.0, 0.0], [-8.0, -5.0, -5.0], [2.0, 2.0, 2.0]), ModelCube::of([27.0, 0.0], [4.0, -3.0, -5.0], [4.0, 2.0, 2.0]).mirror(), ModelCube::of([39.0, 0.0], [6.0, -5.0, -5.0], [2.0, 2.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 12.0, -5.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 12.0, 7.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:warm_cow_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([3.0, 19.0, -5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -7.0, -1.0], [8.0, 6.0, 12.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 13.569, -5.1667]), &[ModelCube::of([0.0, 18.0], [-3.0, -4.569, -4.8333], [6.0, 6.0, 5.0]), ModelCube::of([8.0, 29.0], [3.0, -5.569, -3.8333], [1.0, 2.0, 1.0]), ModelCube::of([4.0, 29.0], [-4.0, -5.569, -3.8333], [1.0, 2.0, 1.0]).mirror(), ModelCube::of([12.0, 29.0], [-2.0, -1.569, -5.8333], [4.0, 3.0, 1.0])], &[]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -3.5]), &[ModelCube::of([34.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 3.5]), &[ModelCube::of([34.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -3.5]), &[ModelCube::of([22.0, 18.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 3.5]), &[ModelCube::of([22.0, 27.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wind_charge#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("bone", PartPose::ZERO, &[], &[
            ModelPart::of("wind", PartPose::rotation([0.0, -0.7854, 0.0]), &[ModelCube::of([15.0, 20.0], [-4.0, -1.0, -4.0], [8.0, 2.0, 8.0]), ModelCube::of([0.0, 9.0], [-3.0, -2.0, -3.0], [6.0, 4.0, 6.0])], &[]),
            ModelPart::of("wind_charge", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-2.0, -2.0, -2.0], [4.0, 4.0, 4.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:witch#main", 64, 128, ModelPart::of("root", PartPose::new([0.0, 1.501, 0.0], [0.0, 0.0, 0.0], [0.9375, 0.9375, 0.9375]), &[], &[
        ModelPart::of("arms", PartPose::offset_and_rotation([0.0, 3.0, -1.0], [-0.75, 0.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-8.0, -2.0, -2.0], [4.0, 8.0, 4.0]), ModelCube::of([44.0, 22.0], [4.0, -2.0, -2.0], [4.0, 8.0, 4.0]).mirror(), ModelCube::of([40.0, 38.0], [-4.0, 2.0, -2.0], [8.0, 4.0, 4.0])], &[]),
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0])], &[
            ModelPart::of("hat", PartPose::offset([-5.0, -10.03125, -5.0]), &[ModelCube::of([0.0, 64.0], [0.0, 0.0, 0.0], [10.0, 2.0, 10.0])], &[
                ModelPart::of("hat2", PartPose::offset_and_rotation([1.75, -4.0, 2.0], [-0.05235988, 0.0, 0.02617994]), &[ModelCube::of([0.0, 76.0], [0.0, 0.0, 0.0], [7.0, 4.0, 7.0])], &[
                    ModelPart::of("hat3", PartPose::offset_and_rotation([1.75, -4.0, 2.0], [-0.10471976, 0.0, 0.05235988]), &[ModelCube::of([0.0, 87.0], [0.0, 0.0, 0.0], [4.0, 4.0, 4.0])], &[
                        ModelPart::of("hat4", PartPose::offset_and_rotation([1.75, -2.0, 2.0], [-0.20943952, 0.0, 0.10471976]), &[ModelCube::of([0.0, 95.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]).grow([0.25, 0.25, 0.25])], &[]),
                    ]),
                ]),
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[ModelCube::of([30.0, 47.0], [-8.0, -8.0, -6.0], [16.0, 16.0, 1.0])], &[]),
            ]),
            ModelPart::of("nose", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([24.0, 0.0], [-1.0, -1.0, -6.0], [2.0, 4.0, 2.0])], &[
                ModelPart::of("mole", PartPose::offset([0.0, -2.0, 0.0]), &[ModelCube::of([0.0, 0.0], [0.0, 3.0, -6.75], [1.0, 1.0, 1.0]).grow([-0.25, -0.25, -0.25])], &[]),
            ]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wither#armor", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("center_head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("left_head", PartPose::offset([10.0, 4.0, 0.0]), &[ModelCube::of([32.0, 0.0], [-4.0, -4.0, -4.0], [6.0, 6.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("ribcage", PartPose::offset_and_rotation([-2.0, 6.9, -0.5], [0.20420352, 0.0, 0.0]), &[ModelCube::of([0.0, 22.0], [0.0, 0.0, 0.0], [3.0, 10.0, 3.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([24.0, 22.0], [-4.0, 1.5, 0.5], [11.0, 2.0, 2.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([24.0, 22.0], [-4.0, 4.0, 0.5], [11.0, 2.0, 2.0]).grow([0.5, 0.5, 0.5]), ModelCube::of([24.0, 22.0], [-4.0, 6.5, 0.5], [11.0, 2.0, 2.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("right_head", PartPose::offset([-8.0, 4.0, 0.0]), &[ModelCube::of([32.0, 0.0], [-4.0, -4.0, -4.0], [6.0, 6.0, 6.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("shoulders", PartPose::ZERO, &[ModelCube::of([0.0, 16.0], [-10.0, 3.9, -0.5], [20.0, 3.0, 3.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([-2.0, 16.692408, 1.5270092], [0.83252203, 0.0, 0.0]), &[ModelCube::of([12.0, 22.0], [0.0, 0.0, 0.0], [3.0, 6.0, 3.0]).grow([0.5, 0.5, 0.5])], &[]),
    ])),
    ModelLayer::of("minecraft:wither#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("center_head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -4.0, -4.0], [8.0, 8.0, 8.0])], &[]),
        ModelPart::of("left_head", PartPose::offset([10.0, 4.0, 0.0]), &[ModelCube::of([32.0, 0.0], [-4.0, -4.0, -4.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("ribcage", PartPose::offset_and_rotation([-2.0, 6.9, -0.5], [0.20420352, 0.0, 0.0]), &[ModelCube::of([0.0, 22.0], [0.0, 0.0, 0.0], [3.0, 10.0, 3.0]), ModelCube::of([24.0, 22.0], [-4.0, 1.5, 0.5], [11.0, 2.0, 2.0]), ModelCube::of([24.0, 22.0], [-4.0, 4.0, 0.5], [11.0, 2.0, 2.0]), ModelCube::of([24.0, 22.0], [-4.0, 6.5, 0.5], [11.0, 2.0, 2.0])], &[]),
        ModelPart::of("right_head", PartPose::offset([-8.0, 4.0, 0.0]), &[ModelCube::of([32.0, 0.0], [-4.0, -4.0, -4.0], [6.0, 6.0, 6.0])], &[]),
        ModelPart::of("shoulders", PartPose::ZERO, &[ModelCube::of([0.0, 16.0], [-10.0, 3.9, -0.5], [20.0, 3.0, 3.0])], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([-2.0, 16.692408, 1.5270092], [0.83252203, 0.0, 0.0]), &[ModelCube::of([12.0, 22.0], [0.0, 0.0, 0.0], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton#boots", 64, 32, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton#chestplate", 64, 32, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton#helmet", 64, 32, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton#leggings", 64, 32, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton#main", 64, 32, ModelPart::of("root", PartPose::new([0.0, -4.803201, 0.0], [0.0, 0.0, 0.0], [1.2, 1.2, 1.2]), &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -1.0], [2.0, 12.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 12.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skeleton_skull#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wither_skull#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 35.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wolf#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 14.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([18.0, 14.0], [-3.0, -2.0, -3.0], [6.0, 9.0, 6.0])], &[]),
        ModelPart::of("head", PartPose::offset([-1.0, 13.5, -7.0]), &[], &[
            ModelPart::of("real_head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-2.0, -3.0, -2.0], [6.0, 6.0, 4.0]), ModelCube::of([16.0, 14.0], [-2.0, -5.0, 0.0], [2.0, 2.0, 1.0]), ModelCube::of([16.0, 14.0], [2.0, -5.0, 0.0], [2.0, 2.0, 1.0]), ModelCube::of([0.0, 10.0], [-0.5, -0.001, -5.0], [3.0, 3.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([0.5, 16.0, -4.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([0.5, 16.0, 7.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 16.0, -4.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).mirror()], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 16.0, 7.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).mirror()], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([-1.0, 12.0, 8.0], [0.62831855, 0.0, 0.0]), &[], &[
            ModelPart::of("real_tail", PartPose::ZERO, &[ModelCube::of([9.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0])], &[]),
        ]),
        ModelPart::of("upper_body", PartPose::offset_and_rotation([-1.0, 14.0, -3.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([21.0, 0.0], [-3.0, -3.0, -3.0], [8.0, 6.0, 7.0])], &[]),
    ])),
    ModelLayer::of("minecraft:wolf_armor#main", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset_and_rotation([0.0, 14.0, 2.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([18.0, 14.0], [-3.0, -2.0, -3.0], [6.0, 9.0, 6.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("head", PartPose::offset([-1.0, 13.5, -7.0]), &[], &[
            ModelPart::of("real_head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-2.0, -3.0, -2.0], [6.0, 6.0, 4.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([16.0, 14.0], [-2.0, -5.0, 0.0], [2.0, 2.0, 1.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([16.0, 14.0], [2.0, -5.0, 0.0], [2.0, 2.0, 1.0]).grow([0.2, 0.2, 0.2]), ModelCube::of([0.0, 10.0], [-0.5, -0.001, -5.0], [3.0, 3.0, 4.0]).grow([0.2, 0.2, 0.2])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([0.5, 16.0, -4.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([0.5, 16.0, 7.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 16.0, -4.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2]).mirror()], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 16.0, 7.0]), &[ModelCube::of([0.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2]).mirror()], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([-1.0, 12.0, 8.0], [0.62831855, 0.0, 0.0]), &[], &[
            ModelPart::of("real_tail", PartPose::ZERO, &[ModelCube::of([9.0, 18.0], [0.0, 0.0, -1.0], [2.0, 8.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
        ]),
        ModelPart::of("upper_body", PartPose::offset_and_rotation([-1.0, 14.0, -3.0], [1.5707964, 0.0, 0.0]), &[ModelCube::of([21.0, 0.0], [-3.0, -3.0, -3.0], [8.0, 6.0, 7.0]).grow([0.2, 0.2, 0.2])], &[]),
    ])),
    ModelLayer::of("minecraft:wolf_baby#main", 32, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-3.0, -2.0, -4.0], [6.0, 4.0, 8.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 18.25, -4.0]), &[ModelCube::of([0.0, 12.0], [-2.99, -3.25, -3.0], [6.0, 5.0, 5.0]).grow([0.025, 0.025, 0.025]), ModelCube::of([17.0, 12.0], [-1.5, -0.24, -5.0], [3.0, 2.0, 2.0])], &[
            ModelPart::of("left_ear", PartPose::offset([2.0, -4.25, -0.5]), &[ModelCube::of([20.0, 5.0], [-1.0, -1.0, -0.5], [2.0, 2.0, 1.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset([-2.0, -4.25, -0.5]), &[ModelCube::of([0.0, 5.0], [-1.0, -1.0, -0.5], [2.0, 2.0, 1.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([1.5, 21.0, -3.0]), &[ModelCube::of([20.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([1.5, 21.0, 3.0]), &[ModelCube::of([8.0, 22.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-1.5, 21.0, -3.0]), &[ModelCube::of([0.0, 0.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-1.5, 21.0, 3.0]), &[ModelCube::of([0.0, 22.0], [-1.0, 0.0, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("tail", PartPose::offset_and_rotation([0.0, 19.0, 3.0], [-0.5236, 0.0, 0.0]), &[], &[
            ModelPart::of("tail_r1", PartPose::offset_and_rotation([0.0, -0.6, 0.2], [-3.1, 0.0, 0.0]), &[ModelCube::of([22.0, 16.0], [-1.0, -5.7, -1.0], [2.0, 6.0, 2.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zoglin#main", 128, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 7.0, 0.0]), &[ModelCube::of([1.0, 1.0], [-8.0, -7.0, -13.0], [16.0, 14.0, 26.0])], &[
            ModelPart::of("mane", PartPose::offset([0.0, -14.0, -7.0]), &[ModelCube::of([90.0, 33.0], [0.0, 0.0, -9.0], [0.0, 10.0, 19.0]).grow([0.001, 0.001, 0.001])], &[]),
        ]),
        ModelPart::of("head", PartPose::offset_and_rotation([0.0, 2.0, -12.0], [0.87266463, 0.0, 0.0]), &[ModelCube::of([61.0, 1.0], [-7.0, -3.0, -19.0], [14.0, 6.0, 19.0])], &[
            ModelPart::of("left_ear", PartPose::offset_and_rotation([6.0, -2.0, -3.0], [0.0, 0.0, 0.6981317]), &[ModelCube::of([1.0, 6.0], [0.0, -1.0, -2.0], [6.0, 1.0, 4.0])], &[]),
            ModelPart::of("left_horn", PartPose::offset([7.0, 2.0, -12.0]), &[ModelCube::of([1.0, 13.0], [-1.0, -11.0, -1.0], [2.0, 11.0, 2.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-6.0, -2.0, -3.0], [0.0, 0.0, -0.6981317]), &[ModelCube::of([1.0, 1.0], [-6.0, -1.0, -2.0], [6.0, 1.0, 4.0])], &[]),
            ModelPart::of("right_horn", PartPose::offset([-7.0, 2.0, -12.0]), &[ModelCube::of([10.0, 13.0], [-1.0, -11.0, -1.0], [2.0, 11.0, 2.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 10.0, -8.5]), &[ModelCube::of([41.0, 42.0], [-3.0, 0.0, -3.0], [6.0, 14.0, 6.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([5.0, 13.0, 10.0]), &[ModelCube::of([0.0, 45.0], [-2.5, 0.0, -2.5], [5.0, 11.0, 5.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 10.0, -8.5]), &[ModelCube::of([66.0, 42.0], [-3.0, 0.0, -3.0], [6.0, 14.0, 6.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-5.0, 13.0, 10.0]), &[ModelCube::of([21.0, 45.0], [-2.5, 0.0, -2.5], [5.0, 11.0, 5.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zoglin_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 24.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-4.0, -14.0, -7.0], [8.0, 8.0, 14.0]).grow([0.02, 0.02, 0.02]), ModelCube::of([24.0, 39.0], [0.0, -18.0, -8.0], [0.0, 6.0, 11.0]).grow([0.02, 0.02, 0.02])], &[]),
        ModelPart::of("head", PartPose::offset_and_rotation([0.0, 13.0, -7.0], [0.8727, 0.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-5.0, -2.2605, -10.547], [10.0, 4.0, 12.0]), ModelCube::of([44.0, 29.0], [-7.0, -4.0981, -8.4879], [2.0, 5.0, 2.0]), ModelCube::of([52.0, 29.0], [5.0, -4.0981, -8.4879], [2.0, 5.0, 2.0])], &[
            ModelPart::of("left_ear", PartPose::offset_and_rotation([5.0, -1.0, -1.5], [0.0, 0.0, 0.8727]), &[ModelCube::of([32.0, 0.0], [-0.9, -0.5, -2.0], [6.0, 1.0, 4.0]).mirror()], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-5.0, -1.0, -1.5], [0.0, 0.0, -0.8727]), &[ModelCube::of([32.0, 5.0], [-5.1, -0.5, -2.0], [6.0, 1.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([2.5, 18.0, -4.5]), &[ModelCube::of([12.0, 38.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.5, 18.0, 4.5]), &[ModelCube::of([12.0, 47.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.5, 18.0, -4.5]), &[ModelCube::of([0.0, 38.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.5, 18.0, 4.5]), &[ModelCube::of([0.0, 47.0], [-1.5, 0.0, -1.5], [3.0, 6.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.9, 0.9, 0.9])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.5, 0.5, 0.5])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([-0.1, 0.5, 0.3])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([-0.2, 0.20000002, 0.20000002])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 17.5, 0.0]), &[ModelCube::of([16.0, 16.0], [-2.0, -2.5, -1.0], [4.0, 5.0, 2.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.25, 0.0]), &[ModelCube::of([3.0, 3.0], [-3.0, -6.25, -3.0], [6.0, 6.0, 6.0]), ModelCube::of([35.0, 3.0], [-3.0, -6.15, -3.0], [6.0, 6.0, 6.0]).grow([0.25, 0.25, 0.25])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([28.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 20.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([36.0, 16.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 20.0, 0.0]), &[ModelCube::of([8.0, 16.0], [-1.0, 0.0, -1.0], [2.0, 4.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_head#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombie_horse#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_horse#saddle", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 11.0, 5.0]), &[ModelCube::of([0.0, 32.0], [-5.0, -8.0, -17.0], [10.0, 10.0, 22.0]).grow([0.05, 0.05, 0.05])], &[
            ModelPart::of("saddle", PartPose::ZERO, &[ModelCube::of([26.0, 0.0], [-5.0, -8.0, -9.0], [10.0, 9.0, 9.0]).grow([0.5, 0.5, 0.5])], &[]),
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -5.0, 2.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([42.0, 36.0], [-1.5, 0.0, 0.0], [3.0, 14.0, 4.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 4.0, -12.0], [0.5235988, 0.0, 0.0]), &[ModelCube::of([0.0, 35.0], [-2.05, -6.0, -2.0], [4.0, 12.0, 7.0])], &[
            ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 13.0], [-3.0, -11.0, -2.0], [6.0, 5.0, 7.0])], &[
                ModelPart::of("left_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [0.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("right_ear", PartPose::ZERO, &[ModelCube::of([19.0, 16.0], [-2.55, -13.0, 4.0], [2.0, 3.0, 1.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("head_saddle", PartPose::ZERO, &[ModelCube::of([1.0, 1.0], [-3.0, -11.0, -1.9], [6.0, 5.0, 6.0]).grow([0.22, 0.22, 0.22])], &[]),
            ModelPart::of("left_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("left_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [2.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("mane", PartPose::ZERO, &[ModelCube::of([56.0, 36.0], [-1.0, -11.0, 5.01], [2.0, 16.0, 2.0])], &[]),
            ModelPart::of("mouth_saddle_wrap", PartPose::ZERO, &[ModelCube::of([19.0, 0.0], [-2.0, -11.0, -4.0], [4.0, 5.0, 2.0]).grow([0.2, 0.2, 0.2])], &[]),
            ModelPart::of("right_saddle_line", PartPose::rotation([-0.5235988, 0.0, 0.0]), &[ModelCube::of([32.0, 2.0], [-3.1, -6.0, -8.0], [0.0, 3.0, 16.0])], &[]),
            ModelPart::of("right_saddle_mouth", PartPose::ZERO, &[ModelCube::of([29.0, 5.0], [-3.0, -9.0, -6.0], [1.0, 2.0, 2.0])], &[]),
            ModelPart::of("upper_mouth", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, -11.0, -7.0], [4.0, 5.0, 5.0])], &[]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.9], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-3.0, -1.01, -1.0], [4.0, 11.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-4.0, 14.0, -10.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.9], [4.0, 11.0, 4.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-4.0, 14.0, 7.0]), &[ModelCube::of([48.0, 21.0], [-1.0, -1.01, -1.0], [4.0, 11.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_horse_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 12.5, 0.0]), &[ModelCube::of([0.0, 13.0], [-4.0, -3.5, -7.0], [8.0, 7.0, 14.0])], &[
            ModelPart::of("tail", PartPose::offset_and_rotation([0.0, -1.0, 7.0], [-0.7418, 0.0, 0.0]), &[ModelCube::of([24.0, 34.0], [-1.5, -1.5, -1.0], [3.0, 3.0, 8.0])], &[]),
        ]),
        ModelPart::of("head_parts", PartPose::offset_and_rotation([0.0, 10.0, -6.0], [0.6109, 0.0, 0.0]), &[ModelCube::of([30.0, 0.0], [-2.0, -6.0, -2.0], [4.0, 8.0, 4.0])], &[
            ModelPart::of("head", PartPose::offset([0.0, -6.0516, -0.2951]), &[ModelCube::of([0.0, 0.0], [-3.0, -3.9484, -6.705], [6.0, 4.0, 9.0])], &[
                ModelPart::of("left_ear", PartPose::offset_and_rotation([2.0, -4.2484, 1.9451], [0.0, 0.0, 0.2618]), &[ModelCube::of([0.0, 4.0], [-1.0, -2.5, -0.8], [2.0, 3.0, 1.0])], &[]),
                ModelPart::of("right_ear", PartPose::offset_and_rotation([-2.0, -4.2484, 1.645], [0.0, 0.0, -0.2618]), &[ModelCube::of([0.0, 0.0], [-1.0, -2.5, -0.5], [2.0, 3.0, 1.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_front_leg", PartPose::offset([2.4, 16.0, -5.4]), &[ModelCube::of([12.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("left_hind_leg", PartPose::offset([2.4, 16.0, 5.4]), &[ModelCube::of([12.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_front_leg", PartPose::offset([-2.4, 16.0, -5.4]), &[ModelCube::of([0.0, 34.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
        ModelPart::of("right_hind_leg", PartPose::offset([-2.4, 16.0, 5.4]), &[ModelCube::of([0.0, 46.0], [-1.5, -1.0, -1.5], [3.0, 9.0, 3.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_nautilus#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 29.0, -6.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -8.5, 12.3]), &[ModelCube::of([0.0, 54.0], [-5.0, -4.51, -3.0], [10.0, 8.0, 14.0]), ModelCube::of([0.0, 76.0], [-5.0, -4.51, 7.0], [10.0, 8.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -0.51, 7.5]), &[ModelCube::of([54.0, 70.0], [-3.0, -2.0, -0.5], [6.0, 4.0, 4.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, 1.49, 7.0]), &[ModelCube::of([54.0, 62.0], [-5.0, -1.98, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.51, 7.0]), &[ModelCube::of([54.0, 54.0], [-5.0, -2.0, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([0.0, -13.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -10.0, -7.0], [14.0, 10.0, 16.0]), ModelCube::of([0.0, 26.0], [-7.0, 0.0, -7.0], [14.0, 8.0, 20.0]), ModelCube::of([48.0, 26.0], [-7.0, 0.0, 6.0], [14.0, 8.0, 0.0])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombie_nautilus_coral#main", 128, 128, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("root", PartPose::offset([0.0, 29.0, -6.0]), &[], &[
            ModelPart::of("body", PartPose::offset([0.0, -8.5, 12.3]), &[ModelCube::of([0.0, 54.0], [-5.0, -4.51, -3.0], [10.0, 8.0, 14.0]), ModelCube::of([0.0, 76.0], [-5.0, -4.51, 7.0], [10.0, 8.0, 0.0])], &[
                ModelPart::of("inner_mouth", PartPose::offset([0.0, -0.51, 7.5]), &[ModelCube::of([54.0, 70.0], [-3.0, -2.0, -0.5], [6.0, 4.0, 4.0])], &[]),
                ModelPart::of("lower_mouth", PartPose::offset([0.0, 1.49, 7.0]), &[ModelCube::of([54.0, 62.0], [-5.0, -1.98, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
                ModelPart::of("upper_mouth", PartPose::offset([0.0, -2.51, 7.0]), &[ModelCube::of([54.0, 54.0], [-5.0, -2.0, 0.0], [10.0, 4.0, 4.0]).grow([-0.001, -0.001, -0.001])], &[]),
            ]),
            ModelPart::of("shell", PartPose::offset([0.0, -13.0, 5.0]), &[ModelCube::of([0.0, 0.0], [-7.0, -10.0, -7.0], [14.0, 10.0, 16.0]), ModelCube::of([0.0, 26.0], [-7.0, 0.0, -7.0], [14.0, 8.0, 20.0]), ModelCube::of([48.0, 26.0], [-7.0, 0.0, 6.0], [14.0, 8.0, 0.0])], &[
                ModelPart::of("corals", PartPose::offset([8.0, 4.5, -8.0]), &[], &[
                    ModelPart::of("blue_coral", PartPose::offset([-14.0, 0.0, 5.5]), &[], &[
                        ModelPart::of("blue_first", PartPose::rotation([0.0, -0.7854, 0.0]), &[ModelCube::of([0.0, 102.0], [-3.5, -5.5, 0.0], [5.0, 10.0, 0.0])], &[]),
                        ModelPart::of("blue_second", PartPose::offset_and_rotation([0.0, 0.0, -2.0], [0.0, 0.7854, 0.0]), &[ModelCube::of([0.0, 102.0], [-3.5, -5.5, 0.0], [5.0, 10.0, 0.0])], &[]),
                    ]),
                    ModelPart::of("pink_coral", PartPose::offset([-12.5, -18.0, 11.0]), &[ModelCube::of([-8.0, 94.0], [-4.5, 4.5, 0.0], [6.0, 0.0, 8.0])], &[
                        ModelPart::of("pink_coral_second", PartPose::offset_and_rotation([-1.5, 4.5, 4.0], [0.0, 0.0, 1.5708]), &[ModelCube::of([-8.0, 94.0], [-3.0, 0.0, -4.0], [6.0, 0.0, 8.0])], &[]),
                    ]),
                    ModelPart::of("red_coral", PartPose::ZERO, &[], &[
                        ModelPart::of("red_coral_first", PartPose::rotation([0.0, 0.7854, 0.0]), &[ModelCube::of([0.0, 112.0], [-4.5, -5.5, 0.0], [6.0, 10.0, 0.0])], &[]),
                        ModelPart::of("red_coral_second", PartPose::offset_and_rotation([-0.5, -1.0, 1.5], [0.0, -0.829, 0.0]), &[ModelCube::of([0.0, 112.0], [-2.5, -5.5, 0.0], [4.0, 10.0, 0.0])], &[]),
                    ]),
                    ModelPart::of("yellow_coral", PartPose::offset([0.0, -11.0, 11.0]), &[], &[
                        ModelPart::of("yellow_coral_first", PartPose::rotation([0.0, 0.7854, 0.0]), &[ModelCube::of([0.0, 85.0], [-4.5, -3.5, 0.0], [6.0, 8.0, 0.0])], &[]),
                        ModelPart::of("yellow_coral_second", PartPose::offset_and_rotation([0.0, 0.0, 2.0], [0.0, -0.7854, 0.0]), &[ModelCube::of([0.0, 85.0], [-4.5, -3.5, 0.0], [6.0, 8.0, 0.0])], &[]),
                    ]),
                ]),
            ]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombie_villager#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[
                ModelPart::of("hat_rim", PartPose::ZERO, &[], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([1.1, 1.1, 1.1]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([1.1, 1.1, 1.1])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.1, 1.1, 1.1])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[
                ModelPart::of("hat_rim", PartPose::ZERO, &[], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.0, 1.0, 1.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 8.0, 8.0]).grow([1.0, 1.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.5, 1.5, 1.5])], &[
                ModelPart::of("hat_rim", PartPose::ZERO, &[], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.6, 0.6, 0.6])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[
                ModelPart::of("hat_rim", PartPose::ZERO, &[], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.6, 0.6, 0.6]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.6, 0.6, 0.6])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.05, 0.05, 0.05])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0]), ModelCube::of([24.0, 0.0], [-1.0, -3.0, -6.0], [2.0, 4.0, 2.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -10.0, -4.0], [8.0, 10.0, 8.0]).grow([0.5, 0.5, 0.5])], &[
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[ModelCube::of([30.0, 47.0], [-8.0, -8.0, -6.0], [16.0, 16.0, 1.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([-0.1, 0.5, 0.3]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([-0.1, 0.5, 0.3])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([-0.1, 0.5, 0.3])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-3.5, 15.5, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([-0.2, 0.20000002, 0.20000002])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([-0.2, 0.20000002, 0.20000002])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.75, 0.0]), &[ModelCube::of([0.0, 15.0], [-2.0, -2.75, -1.5], [4.0, 5.0, 3.0]), ModelCube::of([16.0, 22.0], [-2.0, -2.75, -1.5], [4.0, 6.0, 3.0]).grow([0.1, 0.1, 0.1])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 16.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -3.5], [8.0, 8.0, 7.0])], &[
            ModelPart::of("hat", PartPose::offset([0.0, -4.0, 0.0]), &[ModelCube::of([0.0, 31.0], [-4.0, -4.0, -3.5], [8.0, 8.0, 7.0]).grow([0.3, 0.3, 0.3])], &[]),
            ModelPart::of("hat_rim", PartPose::offset([0.0, -4.5, 0.0]), &[ModelCube::of([0.0, 46.0], [-7.0, -0.5, -6.0], [14.0, 1.0, 12.0])], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -1.0, -4.0]), &[ModelCube::of([23.0, 0.0], [-1.0, -1.0, -0.5], [2.0, 2.0, 1.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([16.0, 15.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 21.5, 0.0]), &[ModelCube::of([0.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([24.0, 15.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 21.5, 0.0]), &[ModelCube::of([8.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_baby_no_hat#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.75, 0.0]), &[ModelCube::of([0.0, 15.0], [-2.0, -2.75, -1.5], [4.0, 5.0, 3.0]), ModelCube::of([16.0, 22.0], [-2.0, -2.75, -1.5], [4.0, 6.0, 3.0]).grow([0.1, 0.1, 0.1])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 16.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::offset([0.0, -4.0, 0.0]), &[], &[]),
            ModelPart::of("hat_rim", PartPose::offset([0.0, -4.5, 0.0]), &[], &[]),
            ModelPart::of("nose", PartPose::offset([0.0, -1.0, -4.0]), &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([3.0, 15.5, 0.0]), &[ModelCube::of([16.0, 15.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.0, 21.5, 0.0]), &[ModelCube::of([0.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-3.0, 15.5, 0.0]), &[ModelCube::of([24.0, 15.0], [-1.0, -0.5, -1.0], [2.0, 5.0, 2.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.0, 21.5, 0.0]), &[ModelCube::of([8.0, 23.0], [-1.0, -0.5, -1.0], [2.0, 3.0, 2.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombie_villager_no_hat#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 20.0], [-4.0, 0.0, -3.0], [8.0, 12.0, 6.0]), ModelCube::of([0.0, 38.0], [-4.0, 0.0, -3.0], [8.0, 20.0, 6.0]).grow([0.05, 0.05, 0.05])], &[]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[
                ModelPart::of("hat_rim", PartPose::rotation([-1.5707964, 0.0, 0.0]), &[], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("left_leg", PartPose::offset([2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).mirror()], &[]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([44.0, 22.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-2.0, 12.0, 0.0]), &[ModelCube::of([0.0, 22.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin#boots", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.91999996, 0.91999996, 0.91999996])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin#chestplate", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02]).mirror()], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin#helmet", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.02, 1.02, 1.02])], &[
            ModelPart::of("hat", PartPose::ZERO, &[ModelCube::of([32.0, 0.0], [-4.0, -8.0, -4.0], [8.0, 8.0, 8.0]).grow([1.52, 1.52, 1.52])], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin#leggings", 64, 32, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.5, 0.5, 0.5])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4]).mirror()], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.4, 0.4, 0.4])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::ZERO, &[ModelCube::of([16.0, 16.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0])], &[
            ModelPart::of("jacket", PartPose::ZERO, &[ModelCube::of([16.0, 32.0], [-4.0, 0.0, -2.0], [8.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("head", PartPose::ZERO, &[ModelCube::of([0.0, 0.0], [-5.0, -8.0, -4.0], [10.0, 8.0, 8.0]), ModelCube::of([31.0, 1.0], [-2.0, -4.0, -5.0], [4.0, 4.0, 1.0]), ModelCube::of([2.0, 4.0], [2.0, -2.0, -5.0], [1.0, 2.0, 1.0]), ModelCube::of([2.0, 0.0], [-3.0, -2.0, -5.0], [1.0, 2.0, 1.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset_and_rotation([4.5, -6.0, 0.0], [0.0, 0.0, -0.5235988]), &[ModelCube::of([51.0, 6.0], [0.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
            ModelPart::of("right_ear", PartPose::offset_and_rotation([-4.5, -6.0, 0.0], [0.0, 0.0, 0.5235988]), &[ModelCube::of([39.0, 6.0], [-1.0, 0.0, -2.0], [1.0, 5.0, 4.0])], &[]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([5.0, 2.0, 0.0]), &[ModelCube::of([32.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_sleeve", PartPose::ZERO, &[ModelCube::of([48.0, 48.0], [-1.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("left_leg", PartPose::offset([1.9, 12.0, 0.0]), &[ModelCube::of([16.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("left_pants", PartPose::ZERO, &[ModelCube::of([0.0, 48.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-5.0, 2.0, 0.0]), &[ModelCube::of([40.0, 16.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_sleeve", PartPose::ZERO, &[ModelCube::of([40.0, 32.0], [-3.0, -2.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
        ModelPart::of("right_leg", PartPose::offset([-1.9, 12.0, 0.0]), &[ModelCube::of([0.0, 16.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0])], &[
            ModelPart::of("right_pants", PartPose::ZERO, &[ModelCube::of([0.0, 32.0], [-2.0, 0.0, -2.0], [4.0, 12.0, 4.0]).grow([0.25, 0.25, 0.25])], &[]),
        ]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin_baby#boots", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[ModelCube::of([0.0, 25.0], [-2.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[ModelCube::of([0.0, 29.0], [-1.0, 2.9, -2.0], [3.0, 1.0, 3.0]).grow([0.7, 0.7, 0.7]).mirror()], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin_baby#chestplate", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[ModelCube::of([0.0, 17.0], [-3.0, -3.0, -1.5], [6.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[ModelCube::of([30.0, 17.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[ModelCube::of([30.0, 25.0], [-1.0, 0.0, -1.53], [2.0, 5.0, 3.0]).grow([0.7, 0.7, 0.7])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin_baby#helmet", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([0.0, 0.0], [-4.5, -7.0, -4.5], [9.0, 8.0, 8.0]).grow([0.7, 0.7, 0.7])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[], &[]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin_baby#leggings", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::ZERO, &[], &[]),
            ModelPart::of("right_ear", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("inner_body", PartPose::offset([0.0, 18.0, 0.0]), &[], &[]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 24.0], [-2.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([0.59999996, 0.59999996, 0.59999996])], &[
            ModelPart::of("right_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.5]), &[ModelCube::of([18.0, 17.0], [-1.0, -0.2, -2.0], [3.0, 4.0, 3.0]).grow([0.59999996, 0.59999996, 0.59999996])], &[
            ModelPart::of("left_foot", PartPose::ZERO, &[], &[]),
        ]),
        ModelPart::of("waist", PartPose::offset([0.0, 19.0, 0.0]), &[ModelCube::of([0.0, 36.0], [-3.0, -1.2, -1.49], [5.9, 2.0, 2.9]).grow([0.59999996, 0.59999996, 0.59999996])], &[]),
    ])),
    ModelLayer::of("minecraft:zombified_piglin_baby#main", 64, 64, ModelPart::of("root", PartPose::ZERO, &[], &[
        ModelPart::of("body", PartPose::offset([0.0, 18.0, -0.5]), &[ModelCube::of([0.0, 13.0], [-3.0, -3.0, -1.0], [6.0, 5.0, 3.0])], &[]),
        ModelPart::of("head", PartPose::offset([0.0, 15.0, 0.0]), &[ModelCube::of([21.0, 30.0], [-1.5, -3.0, -4.5], [3.0, 3.0, 1.0]), ModelCube::of([0.0, 0.0], [-4.5, -6.0, -3.5], [9.0, 6.0, 7.0])], &[
            ModelPart::of("hat", PartPose::ZERO, &[], &[]),
            ModelPart::of("left_ear", PartPose::offset([4.2, -4.0, 0.0]), &[], &[
                ModelPart::of("left_ear_r1", PartPose::offset_and_rotation([1.0, 1.75, 0.0], [0.0, 0.0, -0.6109]), &[ModelCube::of([0.0, 21.0], [-0.5, -3.0, -2.0], [1.0, 6.0, 4.0])], &[]),
            ]),
            ModelPart::of("right_ear", PartPose::offset([-4.2, -4.0, 0.0]), &[], &[
                ModelPart::of("right_ear_r1", PartPose::offset_and_rotation([-1.0, 1.75, 0.0], [0.0, 0.0, 0.6109]), &[ModelCube::of([18.0, 13.0], [-0.5, -3.0, -2.0], [1.0, 6.0, 4.0])], &[]),
            ]),
        ]),
        ModelPart::of("left_arm", PartPose::offset([4.0, 15.0, 0.0]), &[ModelCube::of([28.0, 13.0], [-1.0, 0.0, -1.5], [2.0, 5.0, 3.0])], &[]),
        ModelPart::of("left_leg", PartPose::offset([1.5, 20.0, 0.0]), &[ModelCube::of([10.0, 23.0], [-1.5, 0.0, -1.5], [3.0, 4.0, 3.0])], &[]),
        ModelPart::of("right_arm", PartPose::offset([-4.0, 15.0, 0.0]), &[ModelCube::of([10.0, 30.0], [-1.0, 0.0, -1.5], [2.0, 5.0, 3.0])], &[]),
        ModelPart::of("right_leg", PartPose::offset([-1.5, 20.0, 0.0]), &[ModelCube::of([22.0, 23.0], [-1.5, 0.0, -1.5], [3.0, 4.0, 3.0])], &[]),
    ])),
];
