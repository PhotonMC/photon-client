//! The local player in the box of a screen: its model as it stands now, with what it wears.

use super::*;
use crate::equipment::armor::{Armor, Worn};
use photon_render::entity::textures::{EntityTextures, TextureRect};

type Area = (i32, i32, i32, i32);
type Mouse = (f32, f32);

/// Where the model of a screen goes: its box, its size and offset, and the mouse it looks at.
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
    (skin, worn): (
        photon_render::entity::textures::TextureRect,
        crate::equipment::armor::Worn<'_>,
    ),
    (area, size, offset_y, mouse): PreviewBox,
) -> Vec<GuiItemVertex> {
    let Some(player) = player else {
        return Vec::new();
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
    let dressed = crate::overlays::gui_dressing(
        armor,
        (worn, player_model.slim),
        textures,
        (&scene.state, &scene.placement),
        player.elytra_rotations(1.0),
    );
    vertices.extend(photon_render::entity::player_renderer::gui_layer_vertices(
        &dressed,
        &scene.view,
    ));
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

/// The function a screen draws the local player's box with.
pub(super) fn drawer<'a>(
    session: &'a PlaySession,
    (player, player_model): (
        Option<&'a photon_world::local_player::LocalPlayer>,
        &'a mut PlayerModel,
    ),
    (armor, textures): (&'a std::cell::RefCell<&mut Armor>, &'a EntityTextures),
    skin: TextureRect,
) -> impl FnMut(Area, f32, f32, Mouse) -> Vec<GuiItemVertex> + 'a {
    let worn = worn(session);
    move |area, size, offset_y, mouse| {
        player_preview(
            player,
            (&mut *player_model, &mut **armor.borrow_mut(), textures),
            (skin, worn),
            (area, size, offset_y, mouse),
        )
    }
}
