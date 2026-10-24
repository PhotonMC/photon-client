//! The local player in the box of a screen: its model as it stands now, with what it wears.

use super::*;
use crate::equipment::armor::{Armor, Worn};
use photon_render::entity::textures::{EntityTextures, TextureRect};

type Area = (i32, i32, i32, i32);
type Mouse = (f32, f32);

/// Where the model of a screen goes: its box, its size and offset, and the mouse it looks at.
/// What the box's player holds: the layer caches of the hands and the renderers they draw with.
type HeldTools<'a> = (
    &'a mut [ItemLayerCache; 4],
    (&'a BlockRenderer, &'a BiomeTints),
);

/// The two hands' stacks, the right first.
type Stacks<'a> = (
    Option<&'a ClientItemStackState>,
    Option<&'a ClientItemStackState>,
);

type PreviewBox = ((i32, i32, i32, i32), f32, f32, (f32, f32));

/// The local player in the box of its screen (`InventoryScreen.extractEntityInInventoryFollowsMouse`):
/// its model as it stands now, turned to look at the mouse.
fn player_preview(
    player: Option<&photon_world::local_player::LocalPlayer>,
    (player_model, armor, textures): (
        &mut PlayerModel,
        &mut crate::equipment::armor::Armor,
        &photon_render::entity::textures::EntityTextures,
    ),
    (skin, worn, stacks): (
        photon_render::entity::textures::TextureRect,
        crate::equipment::armor::Worn<'_>,
        Stacks<'_>,
    ),
    (area, size, offset_y, mouse): PreviewBox,
    held: HeldTools<'_>,
) -> GuiFigure {
    let Some(player) = player else {
        return GuiFigure::default();
    };
    let pose = player.model_pose(1.0);
    let state = PlayerRenderState {
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + 1.0,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time: player.attack_anim(1.0),
        ..Default::default()
    };
    let mut vertices = photon_render::entity::player_renderer::gui_player_vertices(
        player_model,
        &state,
        (area, size, offset_y, mouse),
        skin,
    );
    // What it wears, as the world draws the player's own model: the pose the box gave the model.
    let scene = photon_render::entity::player_renderer::gui_player(
        player_model,
        &state,
        (area, size, offset_y, mouse),
        skin,
    );
    let head = worn.head_item(armor);
    let dressed = crate::overlays::gui_dressing(
        armor,
        (worn, player_model.slim),
        textures,
        (&scene.state, &scene.placement),
        player.elytra_rotations(1.0),
    );
    let in_hands = held_vertices((player_model, &scene.placement), (stacks, head), held);
    vertices.extend(photon_render::entity::player_renderer::gui_layer_vertices(
        &dressed,
        &scene.view,
    ));
    GuiFigure {
        body: vertices,
        items: photon_render::entity::gui_view::gui_held_vertices(&in_hands, &scene.view),
    }
}

/// What the box's player holds in its hands and wears on its head, in the box's pose and without
/// the glint (the world's glint pass would draw it at the origin).
fn held_vertices(
    (model, placement): (
        &PlayerModel,
        &photon_render::entity::player_renderer::Placement,
    ),
    (stacks, head): (Stacks<'_>, Option<&ClientItemStackState>),
    (layers, tints): HeldTools<'_>,
) -> Vec<photon_render::scene::Vertex> {
    let hands = crate::equipment::held_items::Hands {
        base: &photon_render::entity::player_renderer::placement_pose(placement),
        root: &model.root,
        hand_shift: if model.slim { 0.5 } else { 0.0 },
        baby: false,
        light: placement.light,
        right: stacks.0,
        left: stacks.1,
        head,
        head_transforms: crate::equipment::held_items::HEAD_TRANSFORMS,
        eye: None,
        motions: crate::equipment::held_items::HandActions::default(),
    };
    let mut held = Vec::new();
    hands.collect(&mut held);
    let vertices = crate::equipment::held_items::held_item_vertices(&held, layers, tints);
    for cache in layers.iter_mut() {
        cache.take_glint();
    }
    vertices
}

/// What the local player has on each armor slot.
fn worn(session: &PlaySession) -> Worn<'_> {
    let stack = |slot: i16| {
        session
            .world()
            .local_player()
            .and_then(|state| state.inventory_slot(slot))
            .filter(|stack| stack.item_count() > 0)
    };
    Worn {
        head: stack(5),
        chest: stack(6),
        legs: stack(7),
        feet: stack(8),
    }
}

/// The stacks in the local player's hands: the selected hotbar slot and the off hand.
fn hand_stacks(session: &PlaySession) -> Stacks<'_> {
    let stack = |slot: i16| {
        session
            .world()
            .local_player()
            .and_then(|state| state.inventory_slot(slot))
            .filter(|stack| stack.item_count() > 0)
    };
    (stack(36 + session.status().selected_slot as i16), stack(45))
}

/// The function a screen draws the local player's box with.
pub(super) fn drawer<'a>(
    session: &'a PlaySession,
    (player, player_model): (
        Option<&'a photon_world::local_player::LocalPlayer>,
        &'a mut PlayerModel,
    ),
    (armor, textures): (&'a std::cell::RefCell<&mut Armor>, &'a EntityTextures),
    (skin, layers, tints): (
        TextureRect,
        &'a mut [ItemLayerCache; 4],
        (&'a BlockRenderer, &'a BiomeTints),
    ),
) -> impl FnMut(Area, f32, f32, Mouse) -> GuiFigure + 'a {
    let (worn, stacks) = (worn(session), hand_stacks(session));
    move |area, size, offset_y, mouse| {
        player_preview(
            player,
            (&mut *player_model, &mut **armor.borrow_mut(), textures),
            (skin, worn, stacks),
            (area, size, offset_y, mouse),
            (&mut *layers, tints),
        )
    }
}
