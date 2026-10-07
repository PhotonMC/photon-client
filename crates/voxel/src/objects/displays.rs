//! The block and item displays (`DisplayRenderer`): the block or item, oriented as its billboard
//! constraint says and moved by its transformation (eased while it changes), lit by the world
//! unless the server overrode the light. Text displays, shadows and the glow outline are not here.

use voxel_core::math::Vec3;
use voxel_net::connection::PlaySession;
use voxel_render::block::biome_tint::BiomeTints;
use voxel_render::block::world_mesh::BlockRenderer;
use voxel_render::joml::Pose;
use voxel_render::model::DisplayContext;
use voxel_render::objects::display::{
    DisplayTurns, display_pose, displayed_light, item_display_pose,
};
use voxel_render::objects::item::{entity_block_vertices, item_layer_vertices};
use voxel_render::scene::Vertex;

use crate::equipment::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// The layers of the items the displays show, by the `ItemDisplayContext` each names (by its id).
pub struct DisplayLayers {
    contexts: Vec<ItemLayerCache>,
}

impl DisplayLayers {
    pub fn new() -> Self {
        let contexts = [
            DisplayContext::None,
            DisplayContext::ThirdPersonLeftHand,
            DisplayContext::ThirdPersonRightHand,
            DisplayContext::FirstPersonLeftHand,
            DisplayContext::FirstPersonRightHand,
            DisplayContext::Head,
            DisplayContext::Gui,
            DisplayContext::Ground,
            DisplayContext::Fixed,
            DisplayContext::OnShelf,
        ];
        Self {
            contexts: contexts.map(ItemLayerCache::new).into(),
        }
    }
}

/// `ItemDisplayContext.BY_ID`'s order, with an id out of range the first.
fn context_index(id: u8) -> usize {
    usize::from(id).min(9) * usize::from(id <= 9)
}

/// `Entity.getViewScale`: the entity distance scaling option, which is one.
pub const VIEW_SCALE: f64 = 1.0;

/// The triangles of every block and item display the server added.
pub fn displays(
    session: &PlaySession,
    layers: &mut DisplayLayers,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    let partial = view.partial as f32;
    for entity in world.entities().all() {
        let Some(display) = entity.display() else {
            continue;
        };
        let (Some(state), Some(snapshot)) = (
            display.render_state(),
            world.entity_snapshot(entity.id(), partial),
        ) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        // `Display.shouldRenderAtSqrDistance`.
        let camera = (
            f64::from(view.origin[0]) + f64::from(view.camera[0]),
            f64::from(view.origin[1]) + f64::from(view.camera[1]),
            f64::from(view.origin[2]) + f64::from(view.camera[2]),
        );
        let distance_sqr = (x - camera.0).powi(2) + (y - camera.1).powi(2) + (z - camera.2).powi(2);
        if !display.should_render_at_sqr_distance(distance_sqr, VIEW_SCALE) {
            continue;
        }
        let progress = display.interpolation_progress(entity.tick_count(), partial);
        let transformation = state.transformation.get(progress);
        let turns = DisplayTurns {
            entity: (entity.y_rot_at(partial), entity.x_rot_at(partial)),
            camera: (view.rotation.0, view.rotation.1),
        };
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let pose = display_pose(&base, (state.billboard, &transformation), &turns);
        // `Entity.getLightProbePosition`: the eyes, which a display has at its feet.
        let light = displayed_light(
            packed_light(world, Vec3::new(x, y, z)),
            state.brightness_override,
        );
        if let Some(block) = display.block_state() {
            let cache = &mut layers.contexts[0];
            if let Some((layer, block_light)) = cache.block(block, renderer) {
                vertices.extend(entity_block_vertices(
                    layer,
                    &pose,
                    (
                        voxel_render::entity::block::submit_light(light, *block_light),
                        0.0,
                    ),
                ));
            }
        } else if let Some(context) = display.item_context()
            && let Some(stack) = entity.item_stack().filter(|stack| stack.item_count() > 0)
        {
            let cache = &mut layers.contexts[context_index(context)];
            vertices.extend(item_layer_vertices(
                cache.get(stack, renderer, tints),
                &[item_display_pose(&pose)],
                light,
                false,
            ));
        }
    }
}

/// What a display casts as a shadow (`DisplayRenderer.getShadowRadius` and `getShadowStrength`
/// at its easing's progress): the radius and the strength, if it is drawn and has any.
pub fn display_shadow(
    entity: &voxel_world::ClientEntity,
    drawn: (f64, f64, f64),
    camera: (f64, f64, f64),
    partial: f32,
) -> Option<(f32, f32)> {
    let display = entity.display()?;
    let state = display.render_state()?;
    let distance_sqr =
        (drawn.0 - camera.0).powi(2) + (drawn.1 - camera.1).powi(2) + (drawn.2 - camera.2).powi(2);
    if entity.is_invisible() || !display.should_render_at_sqr_distance(distance_sqr, VIEW_SCALE) {
        return None;
    }
    let progress = display.interpolation_progress(entity.tick_count(), partial);
    let radius = state.shadow_radius.get(progress);
    (radius > 0.0).then(|| (radius, state.shadow_strength.get(progress)))
}

/// The triangles of every text display the server added: the text seen through blocks and the text
/// tested against the depth, each as triangles from the mesh origin.
pub fn display_texts(
    session: &PlaySession,
    (lang, font, atlas): (
        &voxel_game::text::Lang,
        &voxel_render::gui::font::Font,
        &voxel_render::gui::GuiAtlas,
    ),
    view: FrameView,
) -> (Vec<Vertex>, Vec<Vertex>) {
    use voxel_render::objects::display::text_submit;
    let world = session.world();
    let partial = view.partial as f32;
    let (mut see_through, mut normal) = (Vec::new(), Vec::new());
    for entity in world.entities().all() {
        let Some(display) = entity.display() else {
            continue;
        };
        let (Some(state), Some((text, _)), Some(snapshot)) = (
            display.render_state(),
            display.text_render_state(),
            world.entity_snapshot(entity.id(), partial),
        ) else {
            continue;
        };
        let (x, y, z) = snapshot.position;
        let camera = (
            f64::from(view.origin[0]) + f64::from(view.camera[0]),
            f64::from(view.origin[1]) + f64::from(view.camera[1]),
            f64::from(view.origin[2]) + f64::from(view.camera[2]),
        );
        let distance_sqr = (x - camera.0).powi(2) + (y - camera.1).powi(2) + (z - camera.2).powi(2);
        if !display.should_render_at_sqr_distance(distance_sqr, VIEW_SCALE) {
            continue;
        }
        let progress = display.interpolation_progress(entity.tick_count(), partial);
        let turns = DisplayTurns {
            entity: (entity.y_rot_at(partial), entity.x_rot_at(partial)),
            camera: (view.rotation.0, view.rotation.1),
        };
        let mut base = Pose::default();
        base.translate(
            (x - f64::from(view.origin[0])) as f32,
            (y - f64::from(view.origin[1])) as f32,
            (z - f64::from(view.origin[2])) as f32,
        );
        let pose = display_pose(
            &base,
            (state.billboard, &state.transformation.get(progress)),
            &turns,
        );
        // The text a component carries, split into lines `line_width` wide.
        let mut data = text.text.as_slice();
        let Some(component) = voxel_core::nbt::read_any_nbt(&mut data)
            .ok()
            .flatten()
            .and_then(|tag| voxel_game::text::Component::from_nbt(&tag))
        else {
            continue;
        };
        let runs = lang.runs(&component, &voxel_game::text::Style::default());
        let lines = voxel_render::gui::chat::split(&runs, text.line_width, font);
        let widths: Vec<i32> = lines.iter().map(|line| font.width_of_runs(line)).collect();
        let submit = text_submit(
            &pose,
            text.flags,
            (
                text.text_opacity.get(progress),
                text.background.get(progress),
            ),
            (&widths, 0.25),
        );
        let light = displayed_light(
            packed_light(world, Vec3::new(x, y, z)),
            state.brightness_override,
        );
        let vertices =
            voxel_render::objects::display::text_vertices(&submit, &lines, (font, atlas), light);
        if submit.see_through {
            see_through.extend(vertices);
        } else {
            normal.extend(vertices);
        }
    }
    (see_through, normal)
}
