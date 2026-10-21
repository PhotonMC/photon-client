//! Item models turned into layers once and kept.

use photon_core::ids::BlockStateId;
use photon_data::item::component_types::{DYED_COLOR, LODESTONE_TRACKER};
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, ItemLayer};
use photon_render::item_model::{ItemState, ItemUse};
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
    /// The quick charge enchantment's registry id, for a crossbow's charge time.
    quick_charge: Option<i32>,
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
    tags: Option<std::sync::Arc<photon_world::block::tags::ItemTags>>,
}

/// How the frame's item glints move, where their texture is and which items have the special
/// foil.
#[derive(Clone)]
pub struct GlintFrame {
    pub matrix: [f32; 16],
    pub texture: Option<[f32; 4]>,
    /// The item tags the server sent: `minecraft:compasses` is one of them.
    pub tags: std::sync::Arc<photon_world::block::tags::ItemTags>,
    /// The registry id of the quick charge enchantment, which sets a crossbow's charge time.
    pub quick_charge: Option<i32>,
}

/// `CuboidItemModelWrapper.hasSpecialAnimatedTexture`: the clock and the items of the compasses
/// tag the server sent.
fn has_special_foil(
    item: photon_core::ids::ItemId,
    tags: Option<&photon_world::block::tags::ItemTags>,
) -> bool {
    static CLOCK: std::sync::OnceLock<Option<photon_core::ids::ItemId>> =
        std::sync::OnceLock::new();
    let clock = CLOCK.get_or_init(|| photon_game::item::id_of("minecraft:clock"));
    *clock == Some(item) || tags.is_some_and(|tags| tags.contains(item.0, "minecraft:compasses"))
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
type PatchLayers = ((Vec<u8>, Vec<u8>), Vec<ItemLayer>);

impl ItemLayerCache {
    pub fn new(context: DisplayContext) -> Self {
        Self {
            context,
            layers: std::collections::HashMap::new(),
            blocks: std::collections::HashMap::new(),
            glint: Glint::default(),
            quick_charge: None,
        }
    }

    /// Set how the glints of the frame move (`TextureTransform.GLINT_TEXTURING`) and where their
    /// texture is; without it no glint is drawn.
    pub fn set_glint(&mut self, frame: &GlintFrame) {
        self.glint.moving = frame.texture.map(|texture| (frame.matrix, texture));
        self.glint.tags = Some(frame.tags.clone());
        self.quick_charge = frame.quick_charge;
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
        ticks: Option<i32>,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
        (x, y): (f32, f32),
        out: &mut Vec<photon_render::gui::GuiItemVertex>,
    ) -> usize {
        let first = out.len();
        let layers = self.get_in_use(stack, ticks, (renderer, tints));
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

    /// The GUI's item drawer over this cache (`photon_render::gui::ItemDrawer`): a stack at a
    /// place, with the ticks the player has used it for if it is the one in use.
    pub fn drawer<'a>(
        &'a mut self,
        (renderer, tints): (&'a BlockRenderer, &'a BiomeTints),
    ) -> impl FnMut(
        &ClientItemStackState,
        (f32, f32),
        Option<i32>,
        &mut Vec<photon_render::gui::GuiItemVertex>,
    ) -> usize
    + 'a {
        move |stack, at, ticks, out| self.gui_vertices(stack, ticks, (renderer, tints), at, out)
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

    /// `CrossbowItem.getChargeDuration` of a stack, in ticks.
    pub fn charge_duration(&self, stack: &ClientItemStackState) -> i32 {
        super::crossbow::charge_duration(self.quick_charge, stack)
    }

    /// What the item models read of a stack: the ticks it was used for and what a crossbow holds.
    fn item_state(&self, stack: &ClientItemStackState, ticks: Option<i32>) -> ItemState {
        ItemState {
            item_use: ticks.map(|ticks| ItemUse {
                ticks,
                charge_duration: self.charge_duration(stack),
            }),
            charge: super::crossbow::charge_type(stack),
            broken: stack.breaks_on_next_damage(),
            selected: photon_game::menus::container::bundle_selected_item(stack),
            components: photon_render::item_model::TestedComponents {
                dyed_color: stack.has_added_component(DYED_COLOR),
                lodestone_tracker: stack.has_added_component(LODESTONE_TRACKER),
            },
        }
    }

    /// A stack's layers, its grass tinted by climate.
    pub fn get(
        &mut self,
        stack: &ClientItemStackState,
        renderer: &BlockRenderer,
        tints: &BiomeTints,
    ) -> &[ItemLayer] {
        self.get_in_use(stack, None, (renderer, tints))
    }

    /// A stack's layers as its holder uses it: the bow's pulling models, say.
    pub fn get_in_use(
        &mut self,
        stack: &ClientItemStackState,
        ticks: Option<i32>,
        (renderer, tints): (&BlockRenderer, &BiomeTints),
    ) -> &[ItemLayer] {
        let context = self.context;
        let item = stack.item_registry_id();
        let state = self.item_state(stack, ticks);
        let mut branches = if state == ItemState::default() {
            Vec::new()
        } else {
            renderer.item_branches(item, context, state)
        };
        // The selected item is not in the patch, and decides the nested layers.
        branches.extend(
            state
                .selected
                .into_iter()
                .flat_map(|item| item.0.to_le_bytes()),
        );
        let variants = self.layers.entry(item.0).or_default();
        let index = if let Some(index) = variants
            .iter()
            .position(|((patch, seen), _)| patch == stack.patch_bytes() && *seen == branches)
        {
            index
        } else {
            let layers = renderer.item_layers(
                (item, state),
                context,
                &|temperature, downfall| tints.grass_at_climate(temperature, downfall),
                &StackTints(photon_game::item::values::Values::new(
                    item,
                    stack.patch_bytes(),
                )),
            );
            variants.push(((stack.patch_bytes().to_vec(), branches), layers));
            variants.len() - 1
        };
        let layers = &variants[index].1;
        self.glint.decal = (self.glint.moving.is_some()
            && has_special_foil(stack.item_registry_id(), self.glint.tags.as_deref()))
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

#[cfg(test)]
mod special_foil_tests {
    use super::*;
    use photon_core::identifier::Identifier;
    use photon_core::tags::{RegistryTags, TagEntry};

    fn tags_with_compasses(items: Vec<i32>) -> photon_world::block::tags::ItemTags {
        let id = |text: &str| Identifier::parse(text).expect("an identifier");
        photon_world::block::tags::ItemTags::from_registries(&[RegistryTags::new(
            id("minecraft:item"),
            vec![TagEntry::new(id("minecraft:compasses"), items)],
        )])
        .expect("the item registry")
    }

    #[test]
    fn the_special_foil_follows_the_compasses_tag_the_server_sent_and_the_clock() {
        let compass = photon_game::item::id_of("minecraft:compass").expect("a compass");
        let stick = photon_game::item::id_of("minecraft:stick").expect("a stick");
        let clock = photon_game::item::id_of("minecraft:clock").expect("a clock");
        let tags = tags_with_compasses(vec![compass.0, stick.0]);
        assert!(has_special_foil(compass, Some(&tags)));
        assert!(has_special_foil(stick, Some(&tags)));
        // A tag without the compass leaves it with the standard foil.
        let without = tags_with_compasses(vec![stick.0]);
        assert!(!has_special_foil(compass, Some(&without)));
        // The clock is the one item named in the code; nothing is special before any tags came.
        assert!(has_special_foil(clock, Some(&without)));
        assert!(has_special_foil(clock, None));
        assert!(!has_special_foil(compass, None));
    }
}
