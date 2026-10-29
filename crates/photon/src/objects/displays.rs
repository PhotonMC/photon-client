//! The block and item displays (`DisplayRenderer`): the block or item, oriented as its billboard
//! constraint says and moved by its transformation (eased while it changes), lit by the world
//! unless the server overrode the light. Text displays, shadows and the glow outline are not here.

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, ConduitMeshes};
use photon_render::block_entity::conduit::Piece;
use photon_render::joml::Pose;
use photon_render::model::DisplayContext;
use photon_render::objects::display::{
    DisplayTurns, display_pose, displayed_light, item_display_pose,
};
use photon_render::objects::item::{entity_block_vertices, item_layer_vertices};
use photon_render::scene::Vertex;
use photon_world::EntityView;

use crate::equipment::item_layers::{GlintFrame, ItemLayerCache};
use crate::mobs::MobModels;
use crate::overlays::{FrameView, packed_light};
use crate::world::special_models::special_block_vertices;
use photon_render::block_entity::special_model::SpecialModel;
use photon_render::entity::textures::EntityTextures;

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

impl DisplayLayers {
    /// Set how the frame's glints move and where their texture is, in every context.
    pub fn set_glint(&mut self, frame: &GlintFrame) {
        for cache in &mut self.contexts {
            cache.set_glint(frame);
        }
    }

    /// The caches of every context.
    pub fn caches(&mut self) -> impl Iterator<Item = &mut ItemLayerCache> {
        self.contexts.iter_mut()
    }

    /// The glints of the displays' items drawn this frame.
    pub fn take_glint(&mut self) -> Vec<Vertex> {
        self.contexts
            .iter_mut()
            .flat_map(ItemLayerCache::take_glint)
            .collect()
    }
}

/// `ItemDisplayContext.BY_ID`'s order, with an id out of range the first.
fn context_index(id: u8) -> usize {
    usize::from(id).min(9) * usize::from(id <= 9)
}

/// `Entity.getViewScale`: the entity distance scaling option, which is one.
pub const VIEW_SCALE: f64 = 1.0;

/// Where a display is drawn: its entity, the pose its transformation gives it, and its light.
struct Shown<'a> {
    entity: photon_world::EntityView<'a>,
    display: &'a photon_world::objects::display::DisplayState,
    pose: Pose,
    light: i32,
}

/// Every display the server added, in the pose it is drawn in this frame.
fn shown(session: &PlaySession, view: FrameView) -> Vec<Shown<'_>> {
    let world = session.world();
    let partial = view.partial as f32;
    let mut out = Vec::new();
    for entity in world.entities().drawn() {
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
        out.push(Shown {
            entity,
            display,
            pose,
            light,
        });
    }
    out
}

/// `Display.getGlowColorOverride`: -1 for none.
fn glow_colour(display: &photon_world::objects::display::DisplayState) -> i32 {
    display
        .render_state()
        .map_or(-1, |state| state.glow_color_override)
}

/// A display's special-model triangles (over the entity textures) join its outline when it glows.
fn outlined(shown: &Shown<'_>, (atlas, drawn): (bool, &[Vertex])) {
    if !shown.entity.is_glowing() {
        return;
    }
    let colour = glow_colour(shown.display);
    crate::mobs::record_display_outline(shown.entity, colour, (atlas, drawn.to_vec()));
}

/// The block displays whose state has a special model (`BuiltInBlockModels`), which are drawn
/// in the entity pass with the entity textures.
pub fn special_displays(
    session: &PlaySession,
    (models, textures): (&mut MobModels, &EntityTextures),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    for shown in shown(session, view) {
        let Some(block) = shown.display.block_state() else {
            continue;
        };
        let at = (block, &shown.pose, shown.light);
        let drawn = special_block_vertices(models, textures, at).unwrap_or_default();
        outlined(&shown, (false, &drawn));
        vertices.extend(drawn);
        end_cube(models, block, &shown.pose);
    }
}

/// The triangles of every block and item display the server added.
pub fn displays(
    session: &PlaySession,
    layers: &mut DisplayLayers,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
    conduit: Option<&ConduitMeshes>,
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    for Shown {
        entity,
        display,
        pose,
        light,
    } in shown(session, view)
    {
        if let Some(block) = display.block_state() {
            if let Some(shell) = conduit.filter(|_| is_conduit(block)) {
                // `BuiltInBlockModels` with `ConduitRenderer.DEFAULT_TRANSFORMATION`.
                let at = SpecialModel::Conduit.item_pose(&pose).unwrap_or(pose);
                if let Some(layer) = shell.get(Piece::Shell) {
                    vertices.extend(entity_block_vertices(layer, &at, (light, 0.0)));
                }
            }
            let cache = &mut layers.contexts[0];
            if let Some((layer, block_light)) = cache.block(block, renderer) {
                let light = photon_render::entity::block::submit_light(light, *block_light);
                let drawn = entity_block_vertices(layer, &pose, (light, 0.0));
                if entity.is_glowing() {
                    let colour = glow_colour(display);
                    crate::mobs::record_display_outline(entity, colour, (true, drawn.clone()));
                }
                vertices.extend(drawn);
            }
        } else if let Some(context) = display.item_context()
            && let Some(stack) = entity.item_stack().filter(|stack| stack.item_count() > 0)
        {
            let cache = &mut layers.contexts[context_index(context)];
            let layers = cache.get(stack, renderer, tints);
            let drawn = item_layer_vertices(layers, &[item_display_pose(&pose)], light, false);
            if entity.is_glowing() {
                let colour = glow_colour(display);
                crate::mobs::record_display_outline(entity, colour, (true, drawn.clone()));
            }
            cache.extend_with_glint(stack, drawn, vertices);
        }
    }
}

/// What a display casts as a shadow (`DisplayRenderer.getShadowRadius` and `getShadowStrength`
/// at its easing's progress): the radius and the strength, if it is drawn and has any.
pub fn display_shadow(
    entity: EntityView<'_>,
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

/// The lines a display's runs split into at its width, as the draw layer takes them.
fn display_lines(
    runs: &[photon_game::text::Run],
    width: i32,
    font: &photon_render::gui::font::Font,
) -> Vec<Vec<photon_render::gui::TextRun<'static>>> {
    let lines = photon_gui::chat::split(runs, width, font);
    photon_gui::draw_lines(&lines)
        .into_iter()
        .map(|line| {
            line.into_iter()
                .map(photon_render::gui::TextRun::into_owned)
                .collect()
        })
        .collect()
}

/// The triangles of every text display the server added: the text seen through blocks and the text
/// tested against the depth, each as triangles from the mesh origin.
pub fn display_texts(
    session: &PlaySession,
    (lang, font, atlas): (
        &photon_game::text::Lang,
        &photon_render::gui::font::Font,
        &photon_render::gui::GuiAtlas,
    ),
    (view, default_background): (FrameView, f32),
) -> (Vec<Vertex>, Vec<Vertex>) {
    use photon_render::objects::display::text_submit;
    let world = session.world();
    let partial = view.partial as f32;
    let (mut see_through, mut normal) = (Vec::new(), Vec::new());
    for entity in world.entities().drawn() {
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
        let Some(component) = photon_core::nbt::read_any_nbt(&mut data)
            .ok()
            .flatten()
            .and_then(|tag| photon_game::text::Component::from_nbt(&tag))
        else {
            continue;
        };
        let runs = lang.runs(&component, &photon_game::text::Style::default());
        let lines = display_lines(&runs, text.line_width, font);
        let widths: Vec<i32> = lines.iter().map(|l| font.width_of_text_runs(l)).collect();
        let submit = text_submit(
            &pose,
            text.flags,
            (
                text.text_opacity.get(progress),
                text.background.get(progress),
            ),
            (&widths, default_background),
        );
        let light = displayed_light(
            packed_light(world, Vec3::new(x, y, z)),
            state.brightness_override,
        );
        let vertices =
            photon_render::objects::display::text_vertices(&submit, &lines, (font, atlas), light);
        if submit.see_through {
            see_through.extend(vertices);
        } else {
            normal.extend(vertices);
        }
    }
    (see_through, normal)
}

/// Whether a block state is the conduit.
fn is_conduit(state: photon_core::ids::BlockStateId) -> bool {
    SpecialModel::of(state) == Some(SpecialModel::Conduit)
}

/// `EndCubeSpecialRenderer`: the whole cube of an end gateway, or the slab of an end portal, for
/// the portal pipeline.
fn end_cube(models: &mut MobModels, state: photon_core::ids::BlockStateId, pose: &Pose) {
    use crate::world::end_portals::{GATEWAY_LAYERS, PORTAL_LAYERS, push_cube};
    let faces: Vec<_> = photon_core::direction::Direction::stream().collect();
    match SpecialModel::of(state) {
        Some(SpecialModel::EndCube { portal: false }) => {
            push_cube(models, (pose, GATEWAY_LAYERS), &faces);
        }
        Some(SpecialModel::EndCube { portal: true }) => {
            let slab = SpecialModel::EndCube { portal: true }
                .item_pose(pose)
                .unwrap_or(*pose);
            push_cube(models, (&slab, PORTAL_LAYERS), &faces);
        }
        _ => {}
    }
}
