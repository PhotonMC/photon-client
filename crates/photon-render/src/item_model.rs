//! Item models: the flat item extruded into a slab of pixels, as the
//! reference's item model generator bakes `builtin/generated`.

use std::collections::BTreeSet;

/// Which way a side face of an extruded item faces in the texture: up and
/// down along the rows, left and right along the columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SideDirection {
    Up,
    Down,
    Left,
    Right,
}

impl SideDirection {
    /// The reference's name, for reports.
    pub fn name(self) -> &'static str {
        match self {
            SideDirection::Up => "UP",
            SideDirection::Down => "DOWN",
            SideDirection::Left => "LEFT",
            SideDirection::Right => "RIGHT",
        }
    }

    /// The step, in texture pixels, the side looks towards (`Direction`'s
    /// step: up is +y, down is -y, left is east +x, right is west -x).
    fn step(self) -> (i32, i32) {
        match self {
            SideDirection::Up => (0, 1),
            SideDirection::Down => (0, -1),
            SideDirection::Left => (1, 0),
            SideDirection::Right => (-1, 0),
        }
    }

    /// The model direction the face points.
    pub fn direction(self) -> photon_core::direction::Direction {
        use photon_core::direction::Direction;
        match self {
            SideDirection::Up => Direction::Up,
            SideDirection::Down => Direction::Down,
            SideDirection::Left => Direction::East,
            SideDirection::Right => Direction::West,
        }
    }
}

use crate::model::DisplayContext;
use photon_core::identifier::Identifier;
use photon_core::json::{JsonObject, JsonValue};

/// A tint source a model layer names (`ItemTintSources`), for the
/// default stack.
#[derive(Debug, Clone, PartialEq)]
pub enum TintSource {
    /// A colour of its own (RGB; made opaque).
    Constant(u32),
    /// The grass colour map at a climate.
    Grass { temperature: f32, downfall: f32 },
    /// A source that reads a component the stack may carry: the colour the
    /// component gives, or the source's default when the stack has none
    /// (RGB; made opaque).
    Component { source: ComponentTint, default: u32 },
    /// A source this client does not read; the stack takes its default
    /// colour (RGB; made opaque).
    Default(u32),
}

/// The tint sources that read a component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentTint {
    /// `dye`: `dyed_color`.
    Dye,
    /// `potion`: the colour of `potion_contents`.
    Potion,
    /// `firework`: the average of `firework_explosion`'s colours.
    Firework,
    /// `map_color`.
    MapColor,
}

/// A client item's model tree (`ItemModel.Unbaked`), as `items/*.json`
/// writes it.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemModelDef {
    Model {
        model: Identifier,
        tints: Vec<TintSource>,
    },
    Composite(Vec<ItemModelDef>),
    Condition {
        property: String,
        /// `has_component`'s component and its `ignore_default`.
        component: Option<(String, bool)>,
        on_true: Box<ItemModelDef>,
        on_false: Box<ItemModelDef>,
    },
    Select {
        property: String,
        cases: Vec<(Vec<String>, ItemModelDef)>,
        fallback: Option<Box<ItemModelDef>>,
    },
    RangeDispatch {
        property: String,
        /// The property's value is multiplied by this (`scale`), and `remaining` counts a use
        /// down instead of up (`use_duration`).
        scale: f32,
        remaining: bool,
        entries: Vec<(f32, ItemModelDef)>,
        fallback: Option<Box<ItemModelDef>>,
    },
    /// A model tree under its own `transformation`, which every layer below
    /// it draws through.
    Transformed {
        matrix: Matrix4,
        model: Box<ItemModelDef>,
    },
    /// A special renderer over a base model (chests, banners, heads...): the
    /// base model gives the display transforms and the light. Only chests
    /// are drawn yet.
    Special {
        base: Identifier,
        model: SpecialModel,
    },
    Empty,
    /// A kind this client does not know.
    /// `bundle/selected_item`: the layers of the item the bundle has selected.
    BundleSelectedItem,
    Unknown(String),
}

/// A 4x4 matrix, row by row, applied to column vectors.
pub type Matrix4 = [[f32; 4]; 4];

pub const IDENTITY: Matrix4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// The special renderers this client draws.
pub use crate::entity::special_models::SpecialKind as SpecialModel;

/// A client item (`ClientItem`): its model tree and the properties the
/// hand reads when the held stack changes.
#[derive(Debug, Clone, PartialEq)]
pub struct ClientItem {
    pub model: ItemModelDef,
    /// Whether a new stack lowers the hand and raises it again.
    pub hand_animation_on_swap: bool,
    /// How far the hand drops while it swaps.
    pub swap_animation_scale: f32,
}

/// One model layer the item draws, with its tints.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemLayerRef {
    pub model: Identifier,
    pub tints: Vec<TintSource>,
    /// The special renderer drawn over `model`'s transforms, if any.
    pub special: Option<SpecialModel>,
    /// The transformations of the nodes above the layer, composed: applied
    /// to the layer's geometry before its display transform.
    pub local: Matrix4,
    /// For `bundle/selected_item`: the item whose own layers are drawn here instead of `model`.
    pub nested: Option<photon_core::ids::ItemId>,
}

/// A texture's alpha, row by row, with its width and height.
pub type LayerAlpha = (Vec<u8>, i32, i32);

mod needle;
pub use needle::*;
mod parse;
pub use parse::*;
mod resolve;
pub use resolve::*;
mod generated;
pub use generated::*;

#[cfg(test)]
mod special_tests {
    use super::*;

    #[test]
    fn a_chest_draws_its_model_over_the_base_models_transforms() {
        let text = r#"{"model":{"type":"minecraft:select","property":"minecraft:local_time",
            "pattern":"MM-dd","cases":[],"fallback":{"type":"minecraft:special",
            "base":"minecraft:item/chest","model":{"type":"minecraft:chest",
            "texture":"minecraft:normal"}}}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert_eq!(layers.len(), 1);
        assert_eq!(layers[0].model.as_string(), "minecraft:item/chest");
        assert_eq!(
            layers[0].special,
            Some(SpecialModel::Chest {
                texture: "entity/chest/normal".to_owned()
            })
        );
    }

    #[test]
    fn a_local_time_select_reads_today_as_month_and_day() {
        let (month, day) = photon_core::calendar::today();
        let text = format!(
            r#"{{"model":{{"type":"minecraft:select","property":"minecraft:local_time",
            "pattern":"MM-dd","cases":[{{"when":"{month:02}-{day:02}","model":
            {{"type":"minecraft:model","model":"minecraft:item/today"}}}}],
            "fallback":{{"type":"minecraft:model","model":"minecraft:item/other"}}}}}}"#
        );
        let item = parse_client_item(&text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert_eq!(layers[0].model.as_string(), "minecraft:item/today");
    }

    #[test]
    fn other_special_renderers_draw_nothing() {
        let text = r#"{"model":{"type":"minecraft:special","base":"minecraft:item/trident",
            "model":{"type":"minecraft:not_a_renderer"}}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert!(layers.is_empty());
    }
}

#[cfg(test)]
mod transformation_tests {
    use super::*;

    #[test]
    fn a_child_transformation_moves_only_that_child() {
        // A bed: two models, the second a block further along z.
        let text = r#"{"model":{"type":"minecraft:composite","models":[
            {"type":"minecraft:model","model":"minecraft:block/white_bed_head"},
            {"type":"minecraft:model","model":"minecraft:block/white_bed_foot","transformation":
             {"left_rotation":[0.0,0.0,0.0,1.0],"right_rotation":[0.0,0.0,0.0,1.0],
              "scale":[1.0,1.0,1.0],"translation":[0.0,0.0,1.0]}}]}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        assert_eq!(layers.len(), 2);
        assert_eq!(layers[0].local, IDENTITY);
        assert_eq!(layers[1].local[2][3], 1.0);
        assert_eq!(layers[1].local[0][0], 1.0);
    }

    #[test]
    fn a_transformation_composes_translation_rotation_and_scale() {
        // A half turn about x, scaled by 2, moved by (1, 2, 3): a point at
        // (0, 1, 0) goes to (1, 2 - 2, 3) = (1, 0, 3).
        let text = r#"{"model":{"type":"minecraft:model","model":"minecraft:item/x",
            "transformation":{"translation":[1.0,2.0,3.0],"left_rotation":[1.0,0.0,0.0,0.0],
            "scale":[2.0,2.0,2.0],"right_rotation":[0.0,0.0,0.0,1.0]}}}"#;
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        layers_for(&item.model, DisplayContext::ALL[0], &mut layers);
        let m = layers[0].local;
        let moved: Vec<f32> = (0..3).map(|row| m[row][1] + m[row][3]).collect();
        assert_eq!(moved, vec![1.0, 0.0, 3.0]);
    }
}
