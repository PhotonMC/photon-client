//! The item frames (`ItemFrameRenderer`): the frame's model on the wall (none for an invisible
//! one) and the item in it, turned by eighths, or the map in it once the map's data has come
//! (the map's colours in a texture of its own, its decorations over them).

use crate::equipment::held_items::HeldItem;
use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_core::math::Vec3;
use photon_game::item::values::Values;
use photon_net::connection::PlaySession;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::entity::layering::Layering;
use photon_render::entity::textures::EntityTextures;
use photon_render::joml::Pose;
use photon_render::objects::item::{entity_block_vertices, item_layer_vertices};
use photon_render::objects::item_frame::{item_frame_poses, item_frame_position, render_offset};
use photon_render::objects::map::{DecorationDraw, Sheet, map_quads};
use photon_render::scene::Vertex;
use photon_world::EntityView;
use photon_world::map::{MAP_SIDE, MapData};

use crate::equipment::item_layers::ItemLayerCache;
use crate::equipment::needle::Owner;
use crate::overlays::{FrameView, packed_light};
use photon_render::item_model::Holder;

/// `ItemFrameRenderer.getLightCoords`' light for the item of a glow frame
/// (`LightCoordsUtil.FULL_BRIGHT`): full block and sky light.
const GLOW_ITEM_LIGHT: i32 = 15_728_880;

/// The light of a map in a glow frame (`15728850`).
const GLOW_MAP_LIGHT: i32 = 15_728_850;

/// The block light a glow frame is at least lit by (`ItemFrameRenderer.GLOW_FRAME_BRIGHTNESS`).
const GLOW_FRAME_BRIGHTNESS: i32 = 5;

/// The maps' image is a grid of slots, a map to a slot: 16 across, 8 down. A wall of picture
/// maps shows more than a few at once, and a map whose slot is taken in the same frame would
/// show another map's colours.
const MAP_COLUMNS: usize = 16;
const MAP_ROWS: usize = 8;
const MAP_SLOTS: usize = MAP_COLUMNS * MAP_ROWS;
const IMAGE_WIDTH: usize = MAP_SIDE * MAP_COLUMNS;

/// The maps' colours as one image, a map to a slot of 128 by 128, kept until a map changes.
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
            pixels: vec![0; IMAGE_WIDTH * MAP_SIDE * MAP_ROWS * 4],
            next_victim: 0,
            dirty: false,
        }
    }

    /// The slot a map is in, its colours written first if it has none or has changed
    /// (`MapTextureManager.prepareMapTexture`).
    pub(super) fn slot_of(&mut self, id: i32, data: &MapData) -> usize {
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
            let (left, top) = (
                (slot % MAP_COLUMNS) * MAP_SIDE,
                (slot / MAP_COLUMNS) * MAP_SIDE,
            );
            for (index, color) in data.colors().iter().enumerate() {
                let (x, y) = (index % MAP_SIDE, index / MAP_SIDE);
                let at = ((top + y) * IMAGE_WIDTH + left + x) * 4;
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
            width: IMAGE_WIDTH as u32,
            height: (MAP_SIDE * MAP_ROWS) as u32,
            rgba: self.pixels.clone(),
        })
    }
}

/// What the item frames keep between frames: the layers of their items, the maps' image, and the
/// quads of the maps and their decorations of this frame.
pub struct FrameState {
    pub(crate) layers: ItemLayerCache,
    pub(super) maps: MapTextures,
    map_vertices: Vec<Vertex>,
    decoration_vertices: Vec<Vertex>,
    /// The map in the hand: the picture behind it, its colours, its decorations.
    pub(super) hand_map: [Vec<Vertex>; 3],
}

impl FrameState {
    pub fn new() -> Self {
        Self {
            layers: ItemLayerCache::new(photon_render::model::DisplayContext::Fixed),
            maps: MapTextures::new(),
            map_vertices: Vec::new(),
            decoration_vertices: Vec::new(),
            hand_map: Default::default(),
        }
    }

    /// A stack in the `FIXED` context at a pose, fully lit (the totem's animation): its
    /// triangles, and its glint's if it has one.
    pub fn fixed_item(
        &mut self,
        stack: &photon_world::ClientItemStackState,
        pose: Pose,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
    ) -> (Vec<Vertex>, Vec<Vertex>) {
        self.layers.set_owner(None);
        let drawn = item_layer_vertices(
            self.layers.get(stack, renderer, tints),
            &[pose],
            FULL_LIGHT,
            false,
        );
        self.layers.add_glint(stack, &drawn);
        (drawn, self.layers.take_glint())
    }

    /// A map in the hand (`ItemInHandRenderer.renderMap`): the picture behind it, the checkered
    /// one once its data has come, then its colours and all of its decorations.
    pub(super) fn set_hand_map(
        &mut self,
        map: Option<(i32, &MapData)>,
        (pose, light): (&Pose, i32),
        textures: &EntityTextures,
    ) {
        let name = if map.is_some() {
            HAND_MAP_BACKGROUND_CHECKERBOARD
        } else {
            HAND_MAP_BACKGROUND
        };
        if let Some(rect) = textures.rect(name) {
            let uv = |[u, v]: [f32; 2]| {
                [
                    rect[0] + (rect[2] - rect[0]) * u,
                    rect[1] + (rect[3] - rect[1]) * v,
                ]
            };
            let corners = [[-7.0, 135.0], [135.0, 135.0], [135.0, -7.0], [-7.0, -7.0]];
            let uvs = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
            let light = photon_render::block::light::light_uv(light);
            for index in [0, 1, 2, 0, 2, 3] {
                let [x, y] = corners[index];
                self.hand_map[0].push(Vertex {
                    position: pose.pose.transform_position(x, y, 0.0),
                    uv: uv(uvs[index]),
                    shade: 1.0,
                    tint: [1.0; 4],
                    light,
                });
            }
        }
        if let Some((id, data)) = map {
            let slot = self.maps.slot_of(id, data);
            let (colours, decorations) = map_vertices((pose, data), (slot, light), textures, false);
            self.hand_map[1].extend(colours);
            self.hand_map[2].extend(decorations);
        }
    }

    /// Hand the maps' image (if it changed) and this frame's quads to the renderer.
    pub fn hand_over(
        &mut self,
        backend: &mut photon_render::VulkanBackend,
    ) -> Result<(), photon_render::VulkanError> {
        if let Some(image) = self.maps.take_image() {
            backend.set_map_texture(&image)?;
        }
        let [back, map, decorations] = std::mem::take(&mut self.hand_map);
        backend.set_hand_map_quads((&back, &map, &decorations))?;
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
        .chain([HAND_MAP_BACKGROUND, HAND_MAP_BACKGROUND_CHECKERBOARD])
}

/// The picture behind a map in the hand whose data has not come, and behind one whose has.
pub(super) const HAND_MAP_BACKGROUND: &str = "map/map_background";
pub(super) const HAND_MAP_BACKGROUND_CHECKERBOARD: &str = "map/map_background_checkerboard";

/// The id of the map a stack holds (`DataComponents.MAP_ID`), if it holds one.
pub(crate) fn framed_map_id(stack: &photon_world::ClientItemStackState) -> Option<i32> {
    let values = Values::new(stack.item_registry_id(), stack.patch_bytes());
    let mut raw = values.raw(photon_data::item::component_types::MAP_ID)?;
    photon_core::wire::read_varint(&mut raw).ok()
}

/// The frame as the holder of its item (`ItemFrame.getVisualRotationYInDegrees`): where its
/// entity is, and the heading its facing and the item's turn make.
fn frame_owner(entity: EntityView<'_>, direction: Direction) -> Owner {
    let vertical = direction.axis().is_vertical();
    let correction = if vertical {
        90 * direction.get_step_y()
    } else {
        0
    };
    let yaw = photon_core::math::wrap_degrees_i32(
        180 + direction.data2d() * 90 + entity.item_frame_rotation() * 45 + correction,
    );
    // `HangingEntity.recalculateBoundingBox`: the holder is where the frame's box is centred.
    let (x, y, z) = entity.position();
    Owner {
        holder: Holder {
            position: [x, y, z],
            yaw: yaw as f32,
        },
        seed: entity.id().0,
    }
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
        .drawn_of_kind(|entity| entity.is_item_frame())
    {
        let glowing =
            photon_data::entity_type::name(entity.type_id) == Some("minecraft:glow_item_frame");
        // `ClientboundAddEntityPacket.getData`: the way it faces; the block is its position.
        let direction = Direction::from_3d_data_value(entity.data);
        let (x, y, z) = item_frame_position(entity.start_block(), direction);
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
            let model = renderer.item_frame_state(glowing, map.is_some());
            let drawn = vertices.len();
            frame_model(state, (model, &pose), (renderer, light), vertices);
            // `submitWithZOffset`: `entity_solid_z_offset_forward`, `VIEW_OFFSET_Z_LAYERING_FORWARD`.
            Layering::ViewOffsetForward.apply(&mut vertices[drawn..], view.camera);
            outlined(entity, &vertices[drawn..]);
        }
        if let Some((id, data)) = map {
            let map_light = if glowing { GLOW_MAP_LIGHT } else { light };
            let slot = state.maps.slot_of(id, data);
            frame_map(state, (&poses.map, data), (slot, map_light), textures);
        } else if let Some(stack) = stack {
            let item_light = if glowing { GLOW_ITEM_LIGHT } else { light };
            let layers = framed_layers(state, (entity, direction), stack, (renderer, tints));
            let drawn = item_layer_vertices(layers, &[poses.item], item_light, false);
            outlined(entity, &drawn);
            state.layers.extend_with_glint(stack, drawn, vertices);
        }
    }
}

/// The layers of the stack in a frame, drawn for the frame that holds it (and, when it glows, in
/// its outline).
fn framed_layers<'a>(
    state: &'a mut FrameState,
    (entity, direction): (EntityView<'_>, Direction),
    stack: &photon_world::ClientItemStackState,
    (renderer, tints): (&BlockRenderer, &BiomeTints),
) -> &'a [photon_render::block::world_mesh::ItemLayer] {
    let glow = HeldItem::glow_holder(entity);
    state
        .layers
        .set_owner_glowing(frame_owner(entity, direction), glow);
    state.layers.get(stack, renderer, tints)
}

/// A glowing frame's model and item join its outline (a map does not: `MapRenderer.render` takes
/// no outline colour).
fn outlined(entity: EntityView<'_>, drawn: &[Vertex]) {
    if entity.is_glowing() {
        crate::mobs::record_item_outline(entity, drawn.to_vec());
    }
}

/// The frame's own model on the wall (`updateForItemFrame`: the map's model while a map shows).
fn frame_model(
    state: &mut FrameState,
    (model, pose): (BlockStateId, &Pose),
    (renderer, light): (&BlockRenderer, i32),
    vertices: &mut Vec<Vertex>,
) {
    if let Some((layer, block_light)) = state.layers.block(model, renderer) {
        vertices.extend(entity_block_vertices(
            layer,
            pose,
            (
                photon_render::entity::block::submit_light(light, *block_light),
                0.0,
            ),
        ));
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
    let (map, decorations) = map_vertices((pose, data), (slot, light), textures, true);
    state.map_vertices.extend(map);
    state.decoration_vertices.extend(decorations);
}

/// The vertices of a map's colours and of its decorations (the frame's, or all of them).
pub(super) fn map_vertices(
    (pose, data): (&Pose, &MapData),
    (slot, light): (usize, i32),
    textures: &EntityTextures,
    only_frame: bool,
) -> (Vec<Vertex>, Vec<Vertex>) {
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
    let (column, row) = ((slot % MAP_COLUMNS) as f32, (slot / MAP_COLUMNS) as f32);
    let (mut maps, mut sprites) = (Vec::new(), Vec::new());
    for quad in map_quads(pose, &decorations, only_frame) {
        for index in [0, 1, 2, 0, 2, 3] {
            let uv = quad.uvs[index];
            let (uv, target) = match quad.sheet {
                // The map's own cell of the image.
                Sheet::Map => (
                    [
                        (column + uv[0]) / MAP_COLUMNS as f32,
                        (row + uv[1]) / MAP_ROWS as f32,
                    ],
                    &mut maps,
                ),
                Sheet::Decorations => (uv, &mut sprites),
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
    (maps, sprites)
}
