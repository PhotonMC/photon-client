//! Item models turned into layers once and kept.

use photon_core::ids::BlockStateId;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, ItemLayer};
use photon_render::model::DisplayContext;
use photon_render::objects::item::item_glint_vertices;
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

/// Each stack's layers in one display context, built the first time a slot
/// shows it: by item, then by component patch.
pub struct ItemLayerCache {
    context: DisplayContext,
    layers: std::collections::HashMap<i32, Vec<PatchLayers>>,
    /// Block states drawn on entities, by state id, with the light each gives off.
    blocks: std::collections::HashMap<BlockStateId, Option<(ItemLayer, i32)>>,
    glint: Glint,
}

/// The glints of the enchanted stacks drawn since they were last taken.
#[derive(Default)]
struct Glint {
    /// The glint's texture matrix now and the texture's place in the packed image.
    moving: Option<([f32; 16], [f32; 4])>,
    vertices: Vec<Vertex>,
    /// The special foil's coordinates of the stack [`ItemLayerCache::get`] gave last, if it has
    /// that foil: a compass or a clock.
    decal: Option<Vec<[f32; 2]>>,
}

/// `CuboidItemModelWrapper.hasSpecialAnimatedTexture`: the items whose foil is the decal one. The
/// compasses tag is the vanilla one here; the server's version of it is not read.
fn has_special_foil(item: photon_core::ids::ItemId) -> bool {
    static SPECIAL: std::sync::OnceLock<Vec<photon_core::ids::ItemId>> = std::sync::OnceLock::new();
    SPECIAL
        .get_or_init(|| {
            [
                "minecraft:compass",
                "minecraft:recovery_compass",
                "minecraft:clock",
            ]
            .into_iter()
            .filter_map(photon_game::item::id_of)
            .collect()
        })
        .contains(&item)
}

/// `ItemFeatureRenderer.computeFoilDecalPose`: how much the display context scales the pose the
/// decal is made from.
fn foil_pose_scale(context: DisplayContext) -> f32 {
    match context {
        DisplayContext::Gui => 0.5,
        DisplayContext::FirstPersonLeftHand | DisplayContext::FirstPersonRightHand => 0.75,
        _ => 1.0,
    }
}

/// The layers of an item with one component patch.
type PatchLayers = (Vec<u8>, Vec<ItemLayer>);

impl ItemLayerCache {
    pub fn new(context: DisplayContext) -> Self {
        Self {
            context,
            layers: std::collections::HashMap::new(),
            blocks: std::collections::HashMap::new(),
            glint: Glint::default(),
        }
    }

    /// Set how the glints of the frame move (`TextureTransform.GLINT_TEXTURING`) and where their
    /// texture is; without it no glint is drawn.
    pub fn set_glint(&mut self, matrix: [f32; 16], texture: Option<[f32; 4]>) {
        self.glint.moving = texture.map(|texture| (matrix, texture));
    }

    /// `ItemFeatureRenderer.prepareFoilSubmit`: the glint over the triangles a stack drew, if it
    /// has one (`ItemStack.hasFoil`).
    pub fn add_glint(&mut self, stack: &ClientItemStackState, drawn: &[Vertex]) {
        let Some((matrix, texture)) = self.glint.moving else {
            return;
        };
        let foil =
            photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has_foil();
        if foil {
            self.glint.vertices.extend(item_glint_vertices(
                drawn,
                &matrix,
                texture,
                self.glint.decal.as_deref(),
            ));
        }
    }

    /// A stack drawn in the GUI slot at `(x, y)`: its triangles, then its glint's if it has one
    /// (`ItemStack.hasFoil`). Gives back how many of the vertices added are the stack's own.
    pub fn gui_vertices(
        &mut self,
        stack: &ClientItemStackState,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
        (x, y): (f32, f32),
        out: &mut Vec<photon_render::gui::GuiItemVertex>,
    ) -> usize {
        let first = out.len();
        let layers = self.get(stack, renderer, tints);
        photon_render::gui::items::item_vertices(layers, x, y, out);
        let drawn = out.len() - first;
        let foil =
            photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .has_foil();
        if let (true, Some((matrix, texture))) = (foil, self.glint.moving) {
            let glint = photon_render::gui::items::glint_vertices(
                &out[first..],
                &matrix,
                texture,
                self.glint.decal.as_deref(),
            );
            out.extend(glint);
        }
        drawn
    }

    /// Add what a stack drew to `out`, with its glint kept for the glint pass.
    pub fn extend_with_glint(
        &mut self,
        stack: &ClientItemStackState,
        drawn: Vec<Vertex>,
        out: &mut Vec<Vertex>,
    ) {
        self.add_glint(stack, &drawn);
        out.extend(drawn);
    }

    /// The glints added since the last call.
    pub fn take_glint(&mut self) -> Vec<Vertex> {
        std::mem::take(&mut self.glint.vertices)
    }

    /// The model of a block state drawn on an entity (`BlockModelResolver`),
    /// built the first time it is drawn.
    pub fn block(
        &mut self,
        state: BlockStateId,
        renderer: &BlockRenderer,
    ) -> Option<&(ItemLayer, i32)> {
        self.blocks
            .entry(state)
            .or_insert_with(|| renderer.entity_block(state))
            .as_ref()
    }

    /// A stack's layers, its grass tinted by climate.
    pub fn get(
        &mut self,
        stack: &ClientItemStackState,
        renderer: &BlockRenderer,
        tints: &BiomeTints,
    ) -> &[ItemLayer] {
        let context = self.context;
        let variants = self.layers.entry(stack.item_registry_id().0).or_default();
        let index = if let Some(index) = variants
            .iter()
            .position(|(patch, _)| patch == stack.patch_bytes())
        {
            index
        } else {
            let layers = renderer.item_layers(
                stack.item_registry_id(),
                context,
                &|temperature, downfall| tints.grass_at_climate(temperature, downfall),
                &StackTints(photon_game::item::values::Values::new(
                    stack.item_registry_id(),
                    stack.patch_bytes(),
                )),
            );
            variants.push((stack.patch_bytes().to_vec(), layers));
            variants.len() - 1
        };
        let layers = &variants[index].1;
        self.glint.decal = (self.glint.moving.is_some()
            && has_special_foil(stack.item_registry_id()))
        .then(|| photon_render::objects::item::foil_decal_corners(layers, foil_pose_scale(context)))
        .filter(|corners| !corners.is_empty());
        layers
    }
}

/// The tints' view of a stack's components.
struct StackTints(photon_game::item::values::Values);

impl photon_render::item_model::ItemTints for StackTints {
    fn dyed_color(&self) -> Option<i32> {
        self.0.dyed_color()
    }

    fn potion_color_or(&self, default: i32) -> i32 {
        self.0.potion_color_or(default)
    }

    fn map_color(&self) -> Option<i32> {
        self.0.map_color()
    }

    fn firework_colors(&self) -> Vec<i32> {
        self.0.firework_colors()
    }
}
