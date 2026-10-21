//! What humanoids hold in their hands (`ItemInHandLayer`): each stack where
//! its arm's hand is, drawn like the items lying in the world.

use photon_core::ids::BlockStateId;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::block_entity::skull::{SkullKind, skull_pose};
use photon_render::entity::held_item::{HandMotion, hand_pose, head_item_pose, held_to_eye_pose};
use photon_render::entity::player_model::{Arm, Part};
use photon_render::joml::Pose;
use photon_render::objects::item::item_layer_vertices;
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

use crate::equipment::item_layers::ItemLayerCache;
use crate::equipment::needle::Owner;
use photon_render::item_model::Holder;

/// Where a stack is carried: in a hand, or on the head.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Place {
    Right,
    Left,
    Head,
    /// Held on folded arms or at a witch's nose: shown as it lies on the
    /// ground (`HoldingEntityRenderState.heldItem`).
    Ground,
}

impl Place {
    /// `ItemDisplayContext.ordinal()` of the place's display context.
    fn context_ordinal(self) -> i32 {
        match self {
            Place::Left => 1,
            Place::Right => 2,
            Place::Head => 5,
            Place::Ground => 7,
        }
    }

    /// The index of the layer cache of the place's display context.
    pub fn index(self) -> usize {
        self as usize
    }
}

/// What is carried: an item stack, or a block state drawn as a block model
/// (the enderman's block, the snow golem's pumpkin, the iron golem's flower).
pub enum Carried {
    Stack(ClientItemStackState),
    /// A block state by its id, with the pose its model is drawn from (the
    /// layer has already moved it to the block's corner).
    Block {
        state: BlockStateId,
        /// The overlay's code (`overlay_code`): zero where the layer submits
        /// `NO_OVERLAY`.
        overlay: f32,
    },
}

/// A carried thing with the pose of where it is carried.
pub struct HeldItem {
    pub stack: Carried,
    pub pose: Pose,
    pub place: Place,
    pub light: i32,
    /// The use of the stack by its holder, which picks the model's branches.
    pub item_use: Option<i32>,
    /// The stack is the rod whose bobber is out.
    pub cast: bool,
    /// The entity that holds the stack when it is not the local player.
    pub owner: Option<Owner>,
    /// The stack is the local player's own, on its body: the seed of its needle's spin.
    pub local: Option<i32>,
}

impl HeldItem {
    /// A stack carried at a place, from the pose its layer leaves.
    pub fn stack(stack: &ClientItemStackState, pose: Pose, place: Place, light: i32) -> Self {
        Self {
            stack: Carried::Stack(stack.clone()),
            pose,
            place,
            light,
            item_use: None,
            cast: false,
            owner: None,
            local: None,
        }
    }

    /// A block state drawn on an entity, from the pose its layer leaves.
    pub fn block(state: BlockStateId, pose: Pose, light: i32, overlay: f32) -> Self {
        Self {
            stack: Carried::Block { state, overlay },
            pose,
            place: Place::Head,
            light,
            item_use: None,
            cast: false,
            owner: None,
            local: None,
        }
    }
}

/// The state of the block a stack shows on an entity
/// (`BlockItem.getBlock().defaultBlockState()` with the stack's
/// `BlockItemStateProperties` applied), or none for an item that is not a
/// block item.
pub fn block_item_state(stack: &ClientItemStackState) -> Option<BlockStateId> {
    let item = usize::try_from(stack.item_registry_id().0)
        .ok()
        .and_then(|index| photon_data::item::registry::ITEMS.get(index))?;
    let block = photon_data::block::registry::block(item.block?)?;
    let values =
        photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    // A value the property cannot hold, or a property the block lacks, leaves
    // the state as it is.
    let mut chosen = std::collections::BTreeMap::new();
    for (name, value) in values.block_state_properties() {
        let mut attempt = chosen.clone();
        attempt.insert(name, value);
        if block.state_id(&attempt).is_some() {
            chosen = attempt;
        }
    }
    block.state_id(&chosen)
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
    /// The arm whose stack is held to the eye instead of in the hand (`renderItemHeldToEye`).
    pub eye: Option<Arm>,
    /// What each hand's item does besides being held.
    pub motions: HandActions,
}

/// What each hand's item does besides being held, the right hand's first: a spear's thrust or
/// use, and for how many ticks its holder has used the stack in the hand (`getUseItem() == stack`).
#[derive(Default)]
pub struct HandActions {
    motions: [HandMotion; 2],
    uses: [Option<i32>; 2],
    /// Whether the arm holds a rod whose bobber is out.
    cast: [bool; 2],
    /// Who holds the stacks when it is a world entity, with its id as the seed.
    owner: Option<Owner>,
    /// The id of the local player when the hands are its.
    local: Option<i32>,
}

impl HandActions {
    pub fn new(motions: [HandMotion; 2], uses: [Option<i32>; 2]) -> Self {
        Self {
            motions,
            uses,
            cast: [false; 2],
            owner: None,
            local: None,
        }
    }

    /// The actions of a world entity's hands.
    pub fn of_entity(
        motions: [HandMotion; 2],
        session: &photon_net::connection::PlaySession,
        entity: &photon_world::ClientEntity,
    ) -> Self {
        let (x, y, z) = entity.position();
        let mut actions = Self::new(motions, super::arm_poses::entity_uses(entity))
            .with_cast(crate::objects::fishing_hooks::entity_casts(session, entity));
        // `LivingEntity.getVisualRotationYInDegrees` is the body's.
        actions.owner = Some(Owner {
            holder: Holder {
                position: [x, y, z],
                yaw: entity.y_body_rot(),
            },
            seed: entity.id().0,
        });
        actions
    }

    /// The actions of the local player's hands, `main` being the stack in the main hand.
    pub fn of_local(
        motions: [HandMotion; 2],
        (session, player): (
            &photon_net::connection::PlaySession,
            &photon_world::local_player::LocalPlayer,
        ),
        main: Option<&ClientItemStackState>,
    ) -> Self {
        let mut actions = Self::new(motions, player.arm_uses()).with_cast(
            crate::objects::fishing_hooks::local_casts(session, player, main),
        );
        actions.local = Some(player.entity_id().0);
        actions
    }

    /// The owner of a stack drawn at a place: `ItemModelResolver.updateForLiving` seeds a spinning
    /// needle with the holder's id plus the ordinal of the display context.
    fn owner_at(&self, place: Place) -> Option<Owner> {
        self.owner.map(|owner| Owner {
            seed: owner.seed + place.context_ordinal(),
            ..owner
        })
    }

    /// The seed of a local stack drawn at a place: the player's id plus the display context's
    /// ordinal, as for any living holder.
    fn local_at(&self, place: Place) -> Option<i32> {
        self.local.map(|id| id + place.context_ordinal())
    }

    /// The arms that hold a rod with its bobber out (`FishingRodCast`).
    pub fn with_cast(mut self, cast: [bool; 2]) -> Self {
        self.cast = cast;
        self
    }
}

/// `CustomHeadLayer.Transforms.DEFAULT` less its skull offset.
pub const HEAD_TRANSFORMS: (f32, f32) = (0.0, 1.0);

/// The skull a head stack shows, if it is one (`AbstractSkullBlock.getType` of its block).
pub fn skull_of(stack: &ClientItemStackState) -> Option<SkullKind> {
    photon_world::block::rules::item(stack.item_registry_id())
        .and_then(|item| SkullKind::of_item(item.name))
}

impl Hands<'_> {
    /// The skull on the head, and the pose it is drawn from (`CustomHeadLayer`'s `wornHeadType`
    /// branch; a skull is a model, not an item).
    pub fn skull(&self) -> Option<(SkullKind, Pose)> {
        let kind = skull_of(self.head?)?;
        let head = self
            .root
            .children()
            .iter()
            .find(|part| part.name() == "head")?;
        Some((
            kind,
            skull_pose(self.base, self.root, head, (0.0, self.head_transforms.1)),
        ))
    }

    /// The part of the model named so.
    fn part(&self, name: &str) -> Option<&Part> {
        self.root.children().iter().find(|part| part.name() == name)
    }

    /// A stack in an arm, with the use and the cast of that arm.
    fn held(
        &self,
        index: usize,
        stack: &ClientItemStackState,
        (pose, place): (Pose, Place),
    ) -> HeldItem {
        HeldItem {
            stack: Carried::Stack(stack.clone()),
            pose,
            place,
            light: self.light,
            item_use: self.motions.uses[index],
            cast: self.motions.cast[index],
            owner: self.motions.owner_at(place),
            local: self.motions.local_at(place),
        }
    }

    /// Adds each held stack, the right arm's first.
    pub fn collect(&self, out: &mut Vec<HeldItem>) {
        for (index, (side, stack)) in [(Arm::Right, self.right), (Arm::Left, self.left)]
            .into_iter()
            .enumerate()
        {
            let Some(stack) = stack else {
                continue;
            };
            let name = if side == Arm::Left {
                "left_arm"
            } else {
                "right_arm"
            };
            let Some(arm) = self.part(name) else {
                continue;
            };
            let held = |pose, place| self.held(index, stack, (pose, place));
            if self.eye == Some(side)
                && let Some(head) = self.part("head")
            {
                out.push(held(
                    held_to_eye_pose(self.base, self.root, head, side),
                    Place::Head,
                ));
                continue;
            }
            let pose = hand_pose(
                self.base,
                self.root,
                arm,
                side,
                (self.hand_shift, self.baby),
                &self.motions.motions[index],
            );
            let place = if side == Arm::Left {
                Place::Left
            } else {
                Place::Right
            };
            out.push(held(pose, place));
        }
        if let (Some(stack), Some(head)) = (
            self.head.filter(|stack| skull_of(stack).is_none()),
            self.part("head"),
        ) {
            out.push(HeldItem {
                stack: Carried::Stack(stack.clone()),
                pose: head_item_pose(self.base, self.root, head, self.head_transforms),
                place: Place::Head,
                light: self.light,
                item_use: None,
                cast: false,
                owner: self.motions.owner_at(Place::Head),
                local: self.motions.local_at(Place::Head),
            });
        }
    }
}

/// The layers of each held stack, in the display context of its hand.
pub fn held_item_vertices(
    held: &[HeldItem],
    layers: &mut [ItemLayerCache; 4],
    (renderer, tints): (&BlockRenderer, &BiomeTints),
) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for item in held {
        let cache = &mut layers[item.place.index()];
        let stack = match &item.stack {
            Carried::Stack(stack) => stack,
            Carried::Block { state, overlay } => {
                if let Some((layer, block_light)) = cache.block(*state, renderer) {
                    vertices.extend(photon_render::objects::item::entity_block_vertices(
                        layer,
                        &item.pose,
                        (
                            photon_render::entity::block::submit_light(item.light, *block_light),
                            *overlay,
                        ),
                    ));
                }
                continue;
            }
        };
        cache.set_cast(item.cast);
        cache.set_owner(item.owner);
        cache.set_local_body(item.local.is_some());
        cache.set_seed(item.local.unwrap_or(0));
        let item_layers = cache.get_in_use(stack, item.item_use, (renderer, tints));
        let drawn = item_layer_vertices(
            item_layers,
            &[item.pose],
            item.light,
            item.place == Place::Left,
        );
        cache.extend_with_glint(stack, drawn, &mut vertices);
    }
    vertices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hand_seeds_its_needle_by_the_holders_id_and_the_display_context() {
        let mut actions = HandActions::default();
        assert!(actions.owner_at(Place::Left).is_none());
        actions.owner = Some(Owner {
            holder: Holder {
                position: [0.0; 3],
                yaw: 0.0,
            },
            seed: 40,
        });
        let seed = |place| actions.owner_at(place).map(|owner| owner.seed);
        assert_eq!(seed(Place::Left), Some(41));
        assert_eq!(seed(Place::Right), Some(42));
        assert_eq!(seed(Place::Head), Some(45));
    }
}
