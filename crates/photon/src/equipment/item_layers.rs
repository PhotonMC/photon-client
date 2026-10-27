//! Item models turned into layers once and kept.

use photon_core::ids::BlockStateId;
use photon_data::item::component_types::{DYED_COLOR, LODESTONE_TRACKER};
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::{BlockRenderer, ItemLayer};
use photon_render::item_model::{ItemState, ItemTints, ItemUse};
use photon_render::model::DisplayContext;
use photon_render::objects::item::item_glint_vertices;
use photon_render::scene::Vertex;
use photon_world::ClientItemStackState;

/// Whether the stack's model draws the enchantment glint: `ItemStack.hasFoil`, but the skull
/// renderers (`SkullSpecialRenderer`, `PlayerHeadSpecialRenderer`) take the flag and leave it
/// unused.
fn shows_foil(stack: &ClientItemStackState) -> bool {
    super::held_items::skull_of(stack).is_none()
        && photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
            .has_foil()
}

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
    /// The server's trim materials, in the order a `trim` component numbers them.
    trim_materials: std::sync::Arc<Vec<String>>,
    /// The compass needles, for the contexts the local player holds the stack in.
    needles: Option<super::needle::SharedNeedles>,
    /// Who holds the stacks drawn next, when it is not the local player.
    owner: Option<super::needle::Owner>,
    /// The stacks drawn next are the local player's own, on its body.
    local_body: bool,
    /// The seed of the local player's stacks (`ItemModelResolver`'s `seed`): a spinning needle is
    /// offset by it.
    seed: i32,
    /// Whether the stacks drawn next are a rod with its bobber out (`FishingRodCast`).
    cast: bool,
    /// Whether the stacks drawn next have no owner (`GuiGraphics.fakeItem`).
    fake: bool,
    /// The player heads this cache draws, in the skins of their profiles (the GUI's, or the world's
    /// once skins are exchanged with it).
    pub heads: super::head_items::GuiHeads,
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
    /// The server's trim materials, in the order a `trim` component numbers them.
    pub trim_materials: std::sync::Arc<Vec<String>>,
    /// The compass needles of the frame.
    pub needles: super::needle::SharedNeedles,
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
            trim_materials: std::sync::Arc::default(),
            needles: None,
            owner: None,
            local_body: false,
            seed: 0,
            cast: false,
            fake: false,
            heads: Default::default(),
        }
    }

    /// Set how the glints of the frame move (`TextureTransform.GLINT_TEXTURING`) and where their
    /// texture is; without it no glint is drawn.
    pub fn set_glint(&mut self, frame: &GlintFrame) {
        self.glint.moving = frame.texture.map(|texture| (frame.matrix, texture));
        self.glint.tags = Some(frame.tags.clone());
        self.quick_charge = frame.quick_charge;
        self.trim_materials = frame.trim_materials.clone();
        // The GUI and the first-person hand show the local player's own stacks; a frame or the
        // ground shows another entity's, which the caller names with `set_owner`.
        self.needles = matches!(
            self.context,
            DisplayContext::Gui
                | DisplayContext::FirstPersonLeftHand
                | DisplayContext::FirstPersonRightHand
                | DisplayContext::ThirdPersonLeftHand
                | DisplayContext::ThirdPersonRightHand
                | DisplayContext::Head
                | DisplayContext::Fixed
                | DisplayContext::Ground
        )
        .then(|| frame.needles.clone());
    }

    /// Whether the stacks drawn next are the local player's own on its body (third person).
    pub fn set_seed(&mut self, seed: i32) {
        self.seed = seed;
    }

    pub fn set_local_body(&mut self, local_body: bool) {
        self.local_body = local_body;
    }

    /// Whether the stacks drawn next are held by the arm of a player whose bobber is out.
    pub fn set_cast(&mut self, cast: bool) {
        self.cast = cast;
    }

    /// The entity that holds the stacks drawn next (`ItemOwner`); none for the local player.
    pub fn set_owner(&mut self, owner: Option<super::needle::Owner>) {
        self.owner = owner;
    }

    /// `ItemFeatureRenderer.prepareFoilSubmit`: the glint over the triangles a stack drew, if it
    /// has one (`ItemStack.hasFoil`).
    pub fn add_glint(&mut self, stack: &ClientItemStackState, drawn: &[Vertex]) {
        let Some((matrix, texture)) = self.glint.moving else {
            return;
        };
        if shows_foil(stack) {
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
        let foil = shows_foil(stack);
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
        photon_render::gui::ItemHolding,
        &mut Vec<photon_render::gui::GuiItemVertex>,
    ) -> (usize, bool)
    + 'a {
        move |stack, at, holding, out| {
            self.set_cast(holding.cast);
            self.fake = holding.fake;
            self.set_seed(holding.seed);
            let first = out.len();
            let drawn = self.gui_vertices(stack, holding.ticks, (renderer, tints), at, out);
            self.fake = false;
            let skinned = self
                .heads
                .skin(stack, renderer, &mut out[first..first + drawn]);
            (drawn, skinned)
        }
    }

    /// Add what a stack drew to `out`, with its glint kept for the glint pass.
    pub fn extend_with_glint(
        &mut self,
        stack: &ClientItemStackState,
        drawn: Vec<Vertex>,
        out: &mut Vec<Vertex>,
    ) {
        self.add_glint(stack, &drawn);
        let mut drawn = drawn;
        self.heads.skin_in_world(stack, &mut drawn);
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

    /// The key of the material of a stack's trim (`TrimMaterialProperty`), if it has one.
    fn trim_material(&self, stack: &ClientItemStackState) -> Option<&'static str> {
        let id =
            photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
                .trim_material()?;
        let name = self.trim_materials.get(usize::try_from(id).ok()?)?;
        Some(photon_render::item_model::intern_key(name))
    }

    /// What the item models read of a stack: the ticks it was used for and what a crossbow holds.
    fn item_state(&self, stack: &ClientItemStackState, ticks: Option<i32>) -> ItemState {
        ItemState {
            item_use: ticks.map(|ticks| ItemUse {
                ticks,
                charge_duration: self.charge_duration(stack),
                duration: super::arm_poses::use_duration(stack),
            }),
            charge: super::crossbow::charge_type(stack),
            broken: stack.breaks_on_next_damage(),
            selected: photon_game::menus::container::bundle_selected_item(stack),
            trim_material: self.trim_material(stack),
            block_state: block_state_lines(stack),
            dimension: None,
            cast: self.cast,
            numbers: photon_gui::stack_numbers(stack),
            custom_model_data: photon_gui::stack_custom_model_data(stack),
            needles: photon_render::item_model::Needles::default(),
            components: photon_render::item_model::TestedComponents {
                dyed_color: stack.has_added_component(DYED_COLOR),
                lodestone_tracker: stack.has_added_component(LODESTONE_TRACKER),
            },
        }
    }

    /// The needles the stack's compass model reads, worked out once for the draw.
    fn work_out_needles(
        &self,
        renderer: &BlockRenderer,
        (item, stack): (photon_core::ids::ItemId, &ClientItemStackState),
        state: &mut ItemState,
    ) {
        let Some(needles) = &self.needles else {
            return;
        };
        // Outside the local player's own views a stack needs the entity that holds it.
        let own_view = matches!(
            self.context,
            DisplayContext::Gui
                | DisplayContext::FirstPersonLeftHand
                | DisplayContext::FirstPersonRightHand
        );
        if !own_view && !self.local_body && self.owner.is_none() {
            return;
        }
        // `context_dimension` picks a clock's source, so the walk needs it.
        state.dimension = Some(photon_render::item_model::intern_key(
            store_dimension(needles).as_str(),
        ));
        let wanted = renderer.item_needles_wanted(item, self.context, state);
        if wanted.is_empty() {
            state.dimension = None;
            return;
        }
        if self.fake {
            // `NeedleDirectionHelper.get`: no owner, no angle.
            for (target, _) in wanted {
                state.needles.0[target.index()] = Some(0.0_f32.to_bits());
            }
            return;
        }
        let lodestone =
            photon_game::item::values::Values::new(item, stack.patch_bytes()).lodestone_target();
        let mut store = needles
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (target, wobble) in wanted {
            let angle = store.angle(
                (item, target, wobble),
                lodestone.clone(),
                (self.owner, self.seed),
            );
            state.needles.0[target.index()] = Some(angle.to_bits());
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
        let mut state = self.item_state(stack, ticks);
        self.work_out_needles(renderer, (item, stack), &mut state);
        let mut branches = renderer.item_branches(item, context, &state);
        // The trim's key comes from the registry, which the patch alone does not name.
        branches.extend(registry_keys(&state));
        // The selected contents decide the nested layers, and the selection is not in the patch.
        let selected = photon_game::menus::container::bundle_selected_stack(stack);
        let selected_state = selected.as_ref().map(|inner| self.item_state(inner, None));
        if let (Some(inner), Some(inner_state)) = (&selected, &selected_state) {
            branches.extend(inner.item_registry_id().0.to_le_bytes());
            branches.extend(inner.patch_bytes());
            branches.extend(renderer.item_branches(inner.item_registry_id(), context, inner_state));
        }
        let variants = self.layers.entry(item.0).or_default();
        let index = if let Some(index) = variants
            .iter()
            .position(|((patch, seen), _)| patch == stack.patch_bytes() && *seen == branches)
        {
            index
        } else {
            let own = photon_gui::stack_tints(item, stack);
            let nested = selected
                .as_ref()
                .map(|inner| photon_gui::stack_tints(inner.item_registry_id(), inner));
            let layers = renderer.item_layers(
                (item, &state),
                context,
                &|temperature, downfall| tints.grass_at_climate(temperature, downfall),
                (
                    &own,
                    nested
                        .as_ref()
                        .zip(selected_state.as_ref())
                        .map(|(tints, state)| (state, tints as &dyn ItemTints)),
                ),
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

/// The keys the draw read from the registries and the level, which the patch alone does not name.
fn registry_keys(state: &ItemState) -> Vec<u8> {
    [state.trim_material, state.block_state, state.dimension]
        .into_iter()
        .flat_map(|key| key.map(str::as_bytes).unwrap_or_default().iter().copied())
        .collect()
}

/// A stack's `block_state` properties as `name=value` lines, held for good.
fn block_state_lines(stack: &ClientItemStackState) -> Option<&'static str> {
    photon_render::item_model::block_state_key(
        &photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes())
            .block_state(),
    )
}

/// The dimension the needle store's frame holds.
fn store_dimension(needles: &super::needle::SharedNeedles) -> String {
    needles
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .dimension()
        .to_owned()
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
