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

/// The box's player as it stands now.
fn render_state(player: &photon_world::local_player::LocalPlayer) -> PlayerRenderState {
    let pose = player.model_pose(1.0);
    PlayerRenderState {
        walk_position: pose.walk_position,
        walk_speed: pose.walk_speed,
        age_in_ticks: player.tick_count() as f32 + 1.0,
        swim_amount: pose.swim_amount,
        crouching: pose.crouching,
        spectator: player.spectating(),
        attack_time: player.attack_anim(1.0),
        ..Default::default()
    }
}

/// The local player in the box of its screen (`InventoryScreen.extractEntityInInventoryFollowsMouse`):
/// its model as it stands now, turned to look at the mouse.
fn player_preview(
    player: Option<&photon_world::local_player::LocalPlayer>,
    (player_model, armor, textures): (
        &mut PlayerModel,
        &mut crate::equipment::armor::Armor,
        &photon_render::entity::textures::EntityTextures,
    ),
    (skin, worn, stacks, invisible): (
        photon_render::entity::textures::TextureRect,
        crate::equipment::armor::Worn<'_>,
        Stacks<'_>,
        bool,
    ),
    (area, size, offset_y, mouse): PreviewBox,
    held: HeldTools<'_>,
) -> GuiFigure {
    let Some(player) = player else {
        return GuiFigure::default();
    };
    let state = render_state(player);
    let mut vertices = body_vertices(
        player_model,
        &state,
        (area, size, offset_y, mouse),
        (skin, invisible),
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
    let (in_hands, skull) = held_vertices(
        (player_model, &scene.placement),
        (stacks, head),
        (armor, textures, state.walk_position),
        held,
    );
    let mut dressed = dressed;
    dressed.extend(skull);
    vertices.extend(photon_render::entity::player_renderer::gui_layer_vertices(
        &dressed,
        &scene.view,
    ));
    GuiFigure {
        body: vertices,
        items: photon_render::entity::gui_view::gui_held_vertices(&in_hands, &scene.view),
    }
}

/// The box's player body; an invisible player keeps what it wears and holds, not its body
/// (`LivingEntityRenderer.isBodyVisible`).
fn body_vertices(
    model: &mut PlayerModel,
    state: &PlayerRenderState,
    place: (Area, f32, f32, Mouse),
    (skin, invisible): (photon_render::entity::textures::TextureRect, bool),
) -> Vec<photon_render::gui::GuiItemVertex> {
    if invisible {
        return Vec::new();
    }
    photon_render::entity::player_renderer::gui_player_vertices(model, state, place, skin)
}

/// What the box's player holds in its hands and wears on its head, in the box's pose and without
/// the glint (the world's glint pass would draw it at the origin).
fn held_vertices(
    (model, placement): (
        &PlayerModel,
        &photon_render::entity::player_renderer::Placement,
    ),
    (stacks, head): (Stacks<'_>, Option<&ClientItemStackState>),
    (armor, textures, walk_position): (
        &mut crate::equipment::armor::Armor,
        &photon_render::entity::textures::EntityTextures,
        f32,
    ),
    (layers, tints): HeldTools<'_>,
) -> (
    Vec<photon_render::scene::Vertex>,
    Vec<photon_render::scene::Vertex>,
) {
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
    // A skull on the head is a model of its own, posed by the walk, not an item mesh.
    let skull = armor.skull_vertices(
        hands.skull(),
        walk_position,
        textures,
        (placement.light, false),
    );
    let vertices = crate::equipment::held_items::held_item_vertices(&held, layers, tints);
    for cache in layers.iter_mut() {
        cache.take_glint();
    }
    (vertices, skull)
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
    let invisible = session
        .world()
        .local_player()
        .is_some_and(|state| state.is_invisible());
    move |area, size, offset_y, mouse| {
        player_preview(
            player,
            (&mut *player_model, &mut **armor.borrow_mut(), textures),
            (skin, worn, stacks, invisible),
            (area, size, offset_y, mouse),
            (&mut *layers, tints),
        )
    }
}
