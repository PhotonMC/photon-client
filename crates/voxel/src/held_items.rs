//! What humanoids hold in their hands (`ItemInHandLayer`): each stack where
//! its arm's hand is, drawn like the items lying in the world.

use voxel_render::biome_tint::BiomeTints;
use voxel_render::held_item::{hand_pose, head_item_pose};
use voxel_render::item_entity::item_layer_vertices;
use voxel_render::joml::Pose;
use voxel_render::player_model::{Arm, Part};
use voxel_render::scene::Vertex;
use voxel_render::world_mesh::BlockRenderer;
use voxel_world::ClientItemStackState;

use crate::item_layers::ItemLayerCache;

/// Where a stack is carried: in a hand, or on the head.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Right,
    Left,
    Head,
}

impl Place {
    /// The index of the layer cache of the place's display context.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// A carried stack with the pose of where it is carried.
pub struct HeldItem {
    pub stack: ClientItemStackState,
    pub pose: Pose,
    pub place: Place,
    pub light: i32,
}

/// The posed model a humanoid's hands are found on.
pub struct Hands<'a> {
    /// The pose the model is drawn from.
    pub base: &'a Pose,
    pub root: &'a Part,
    /// How far out the hand is of its arm's own place (`translateToHand`).
    pub hand_shift: f32,
    pub baby: bool,
    pub light: i32,
    /// What the right and the left arm hold.
    pub right: Option<&'a ClientItemStackState>,
    pub left: Option<&'a ClientItemStackState>,
    /// What sits on the head (armor aside), with the renderer's
    /// `CustomHeadLayer.Transforms`: its offset and its horizontal scale.
    pub head: Option<&'a ClientItemStackState>,
    pub head_transforms: (f32, f32),
}

/// `CustomHeadLayer.Transforms.DEFAULT` less its skull offset.
pub const HEAD_TRANSFORMS: (f32, f32) = (0.0, 1.0);

impl Hands<'_> {
    /// Adds each held stack, the right arm's first.
    pub fn collect(&self, out: &mut Vec<HeldItem>) {
        for (side, stack) in [(Arm::Right, self.right), (Arm::Left, self.left)] {
            let Some(stack) = stack else {
                continue;
            };
            let name = if side == Arm::Left {
                "left_arm"
            } else {
                "right_arm"
            };
            let Some(arm) = self.root.children().iter().find(|part| part.name() == name) else {
                continue;
            };
            out.push(HeldItem {
                stack: stack.clone(),
                pose: hand_pose(self.base, self.root, arm, side, self.hand_shift, self.baby),
                place: if side == Arm::Left {
                    Place::Left
                } else {
                    Place::Right
                },
                light: self.light,
            });
        }
        if let (Some(stack), Some(head)) = (
            self.head,
            self.root
                .children()
                .iter()
                .find(|part| part.name() == "head"),
        ) {
            out.push(HeldItem {
                stack: stack.clone(),
                pose: head_item_pose(self.base, self.root, head, self.head_transforms),
                place: Place::Head,
                light: self.light,
            });
        }
    }
}

/// The layers of each held stack, in the display context of its hand.
pub fn held_item_vertices(
    held: &[HeldItem],
    layers: &mut [ItemLayerCache; 3],
    (renderer, tints): (&BlockRenderer, &BiomeTints),
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for item in held {
        let cache = &mut layers[item.place.index()];
        let item_layers = cache.get(&item.stack, renderer, tints);
        vertices.extend(item_layer_vertices(
            item_layers,
            &[item.pose],
            item.light,
            item.place == Place::Left,
        ));
    }
    vertices
}
