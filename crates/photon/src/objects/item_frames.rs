//! The item frames (`ItemFrameRenderer`): the frame's model on the wall (none for an invisible
//! one) and the item in it, turned by eighths, or the map in it once the map's data has come
//! (the map's colours in a texture of its own, its decorations over them).

use photon_core::direction::Direction;
use photon_core::math::{Vec3, floor_f64};
use photon_game::item::values::Values;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::item::{entity_block_vertices, item_layer_vertices};
use photon_render::objects::item_frame::{item_frame_poses, item_frame_position, render_offset};
use photon_render::objects::map::{DecorationDraw, Sheet, map_quads};
use photon_render::scene::Vertex;
use photon_world::map::{MAP_SIDE, MapData};

use crate::equipment::item_layers::ItemLayerCache;
use crate::overlays::{FrameView, packed_light};

/// `ItemFrameRenderer.getLightCoords`' light for the item of a glow frame
/// (`LightCoordsUtil.FULL_BRIGHT`): full block and sky light.
const GLOW_ITEM_LIGHT: i32 = 15_728_880;

/// The light of a map in a glow frame (`15728850`).
const GLOW_MAP_LIGHT: i32 = 15_728_850;

/// The block light a glow frame is at least lit by (`ItemFrameRenderer.GLOW_FRAME_BRIGHTNESS`).
const GLOW_FRAME_BRIGHTNESS: i32 = 5;

/// How many maps the maps' image holds at once.
const MAP_SLOTS: usize = 16;

/// The maps' colours as one image, a map to a slot of 128 rows, kept until a map changes.
pub struct MapTextures {
    slots: Vec<Option<(i32, u32)>>,
    pixels: Vec<u8>,
    next_victim: usize,
    dirty: bool,
}

impl MapTextures {
    fn new() -> Self {
        Self {
            slots: vec![None; MAP_SLOTS],
            pixels: vec![0; MAP_SIDE * MAP_SIDE * MAP_SLOTS * 4],
            next_victim: 0,
            dirty: false,
        }
    }

    /// The slot a map is in, its colours written first if it has none or has changed
    /// (`MapTextureManager.prepareMapTexture`).
    fn slot_of(&mut self, id: i32, data: &MapData) -> usize {
        let found = self
            .slots
            .iter()
            .position(|slot| slot.is_some_and(|(held, _)| held == id));
        let slot = found.unwrap_or_else(|| {
            let free = self.slots.iter().position(Option::is_none);
            free.unwrap_or_else(|| {
                let victim = self.next_victim;
                self.next_victim = (victim + 1) % MAP_SLOTS;
                victim
            })
        });
        if self.slots[slot] != Some((id, data.version())) {
            let start = slot * MAP_SIDE * MAP_SIDE * 4;
            for (index, color) in data.colors().iter().enumerate() {
                let at = start + index * 4;
                self.pixels[at..at + 4].copy_from_slice(&photon_data::map::packed_rgba(*color));
            }
            self.slots[slot] = Some((id, data.version()));
            self.dirty = true;
        }
        slot
    }

    /// The image to upload, if a map came or changed since it last was.
    fn take_image(&mut self) -> Option<photon_render::DecodedImage> {
        std::mem::take(&mut self.dirty).then(|| photon_render::DecodedImage {
            width: MAP_SIDE as u32,
            height: (MAP_SIDE * MAP_SLOTS) as u32,
            rgba: self.pixels.clone(),
        })
    }
}

/// What the item frames keep between frames: the layers of their items, the maps' image, and the
/// quads of the maps and their decorations of this frame.
pub struct FrameState {
    pub(crate) layers: ItemLayerCache,
    maps: MapTextures,
    map_vertices: Vec<Vertex>,
    decoration_vertices: Vec<Vertex>,
}

impl FrameState {
    pub fn new() -> Self {
        Self {
            layers: ItemLayerCache::new(photon_render::model::DisplayContext::Fixed),
            maps: MapTextures::new(),
            map_vertices: Vec::new(),
            decoration_vertices: Vec::new(),
        }
    }

    /// A stack in the `FIXED` context at a pose, fully lit (the totem's animation).
    pub fn fixed_item(
        &mut self,
        stack: &photon_world::ClientItemStackState,
        pose: Pose,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
    ) -> Vec<Vertex> {
        item_layer_vertices(
            self.layers.get(stack, renderer, tints),
            &[pose],
            FULL_LIGHT,
            false,
        )
    }

    /// Hand the maps' image (if it changed) and this frame's quads to the renderer.
    pub fn hand_over(
        &mut self,
        backend: &mut photon_render::VulkanBackend,
    ) -> Result<(), photon_render::VulkanError> {
        if let Some(image) = self.maps.take_image() {
            backend.set_map_texture(&image)?;
        }
        backend.set_map_quads((
            &std::mem::take(&mut self.map_vertices),
            &std::mem::take(&mut self.decoration_vertices),
        ))
    }
}

/// The paths (under `textures/`) of the decoration sprites.
/// `LightCoordsUtil.FULL_BRIGHT`.
const FULL_LIGHT: i32 = 15_728_880;

pub fn textures() -> impl Iterator<Item = &'static str> {
    photon_data::map::decoration_types()
        .iter()
        .map(|kind| kind.texture)
}

/// The id of the map a stack holds (`DataComponents.MAP_ID`), if it holds one.
fn framed_map_id(stack: &photon_world::ClientItemStackState) -> Option<i32> {
    let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
    let mut raw = values.raw(photon_data::item::component_types::MAP_ID)?;
    photon_core::wire::read_varint(&mut raw).ok()
}

/// The triangles of every item frame the server added, with the frame's model and the item in it;
/// the quads of the maps in them are kept in `state` for [`FrameState::hand_over`].
pub fn item_frames(
    session: &PlaySession,
    state: &mut FrameState,
    (renderer, tints, textures): (&BlockRenderer, &BiomeTints, &EntityTextures),
    view: FrameView,
    vertices: &mut Vec<Vertex>,
) {
    let world = session.world();
    for entity in world
        .entities()
        .drawn()
        .filter(|entity| entity.is_item_frame())
    {
        let glowing =
            photon_data::entity_type::name(entity.type_id) == Some("minecraft:glow_item_frame");
        // `ClientboundAddEntityPacket.getData`: the way it faces; the block is its position.
        let direction = Direction::from_3d_data_value(entity.data);
        let (x, y, z) = entity.position();
        let (x, y, z) = item_frame_position((floor_f64(x), floor_f64(y), floor_f64(z)), direction);
        // `Entity.getLightProbePosition`: the eyes, which an item frame has at its feet; a glow
        // frame has at least a little block light (`getBlockLightLevel`).
        let mut light = packed_light(world, Vec3::new(x, y, z));
        if glowing {
            let block = ((light >> 4) & 15).max(GLOW_FRAME_BRIGHTNESS);
            light = (light & !(15 << 4)) | (block << 4);
        }
        let offset = render_offset(direction);
        let mut base = Pose::default();
        base.translate(
            ((x - f64::from(view.origin[0])) + offset.0) as f32,
            ((y - f64::from(view.origin[1])) + offset.1) as f32,
            ((z - f64::from(view.origin[2])) + offset.2) as f32,
        );
        let poses = item_frame_poses(
            &base,
            direction,
            entity.is_invisible(),
            entity.item_frame_rotation(),
        );
        let stack = entity.item_stack().filter(|stack| stack.item_count() > 0);
        // `getFramedMapId` and `getMapData`: a map whose data has come.
        let map = stack
            .and_then(framed_map_id)
            .and_then(|id| world.map_data(id).map(|data| (id, data)));
        if let Some(pose) = poses.frame {
            // `updateForItemFrame`: the map's own model while a map is shown.
            let model = renderer.item_frame_state(glowing, map.is_some());
            if let Some((layer, block_light)) = state.layers.block(model, renderer) {
                vertices.extend(entity_block_vertices(
                    layer,
                    &pose,
                    (
                        photon_render::entity::block::submit_light(light, *block_light),
                        0.0,
                    ),
                ));
            }
        }
        if let Some((id, data)) = map {
            let map_light = if glowing { GLOW_MAP_LIGHT } else { light };
            let slot = state.maps.slot_of(id, data);
            frame_map(state, (&poses.map, data), (slot, map_light), textures);
        } else if let Some(stack) = stack {
            let item_light = if glowing { GLOW_ITEM_LIGHT } else { light };
            vertices.extend(item_layer_vertices(
                state.layers.get(stack, renderer, tints),
                &[poses.item],
                item_light,
                false,
            ));
        }
    }
}

/// The quads of a map in a frame (`MapRenderer.render` with only the decorations a frame shows):
/// the map's rows of the maps' image, and each decoration's sprite among the entity textures.
fn frame_map(
    state: &mut FrameState,
    (pose, data): (&Pose, &MapData),
    (slot, light): (usize, i32),
    textures: &EntityTextures,
) {
    let decorations: Vec<DecorationDraw> = data
        .decorations()
        .iter()
        .filter_map(|decoration| {
            let kind = photon_data::map::decoration_type(decoration.type_id)?;
            let rect = textures.rect(kind.texture)?;
            Some(DecorationDraw {
                x: decoration.x,
                y: decoration.y,
                rot: decoration.rot,
                render_on_frame: kind.show_on_item_frame,
                sprite: rect,
            })
        })
        .collect();
    let light = photon_render::block::light::light_uv(light);
    let rows = MAP_SLOTS as f32;
    for quad in map_quads(pose, &decorations, true) {
        for index in [0, 1, 2, 0, 2, 3] {
            let uv = quad.uvs[index];
            let (uv, target) = match quad.sheet {
                // The map's own rows of the image.
                Sheet::Map => (
                    [uv[0], (slot as f32 + uv[1]) / rows],
                    &mut state.map_vertices,
                ),
                Sheet::Decorations => (uv, &mut state.decoration_vertices),
            };
            target.push(Vertex {
                position: quad.corners[index],
                uv,
                shade: 1.0,
                tint: [1.0; 4],
                light,
            });
        }
    }
}
