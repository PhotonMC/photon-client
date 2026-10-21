//! The layers an item model draws for a stack and its holder (`ItemModel.update`): conditions,
//! ranges and selects are decided from the properties of the stack in use.

use super::*;

/// The use of the stack by its holder: how many ticks it has been used, and how many a crossbow
/// takes to charge (`CrossbowItem.getChargeDuration`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUse {
    pub ticks: i32,
    pub charge_duration: i32,
    /// `ItemStack.getUseDuration`, which the ticks left of a use count down from.
    pub duration: i32,
}

/// What a `use_cycle` range's property is named in front of: its `period`.
pub const USE_CYCLE: &str = "minecraft:use_cycle/";

/// `CrossbowItem.ChargeType`: what a crossbow holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Charge {
    #[default]
    None,
    Arrow,
    Rocket,
}

impl Charge {
    fn name(self) -> &'static str {
        match self {
            Charge::None => "none",
            Charge::Arrow => "arrow",
            Charge::Rocket => "rocket",
        }
    }
}

/// What the properties read of a stack and its holder.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ItemState {
    pub item_use: Option<ItemUse>,
    pub charge: Charge,
    /// `ItemStack.nextDamageWillBreak` (`broken`).
    pub broken: bool,
    /// The item a bundle has selected (`BundleItem.getSelectedItem`).
    pub selected: Option<photon_core::ids::ItemId>,
    /// The components `has_component` asks about in the item definitions.
    pub components: TestedComponents,
    /// The key of the stack's trim material, if it names one (`trim_material`).
    pub trim_material: Option<&'static str>,
    /// The properties of the stack's `block_state` component, as `name=value` lines
    /// (`BlockState` select).
    pub block_state: Option<&'static str>,
    /// The compass needles the holder's draw works out (`compass` range dispatches).
    pub needles: Needles,
    /// The key of the level's dimension (`context_dimension`).
    pub dimension: Option<&'static str>,
}

/// What a `block_state` select's property is named in front of: the state property it reads.
pub const BLOCK_STATE_SELECT: &str = "minecraft:block_state/";

/// A key held for good, so a state can stay `Copy`: the materials of a registry are few.
pub fn intern_key(key: &str) -> &'static str {
    static KEYS: std::sync::Mutex<std::collections::BTreeSet<&'static str>> =
        std::sync::Mutex::new(std::collections::BTreeSet::new());
    let mut keys = KEYS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(known) = keys.get(key) {
        return known;
    }
    let leaked: &'static str = Box::leak(key.to_owned().into_boxed_str());
    keys.insert(leaked);
    leaked
}

impl ItemState {
    /// A condition property of the item model, decided from this state: `IsUsingItem`, `Broken`
    /// and `HasComponent`. One this client does not read is false.
    pub fn condition(&self, property: &str, component: Option<&(String, bool)>) -> bool {
        match property {
            "minecraft:using_item" => self.item_use.is_some(),
            "minecraft:broken" => self.broken,
            "minecraft:bundle/has_selected_item" => self.selected.is_some(),
            "minecraft:has_component" => match component.map(|(name, _)| name.as_str()) {
                Some("minecraft:dyed_color") => self.components.dyed_color,
                Some("minecraft:lodestone_tracker") => self.components.lodestone_tracker,
                _ => false,
            },
            _ => false,
        }
    }
}

/// Whether a stack has the components the vanilla item definitions test with `has_component`
/// (`ItemStack.has`; `ignore_default` is for a component the item does not declare, which
/// these two never are).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TestedComponents {
    pub dyed_color: bool,
    pub lodestone_tracker: bool,
}

/// What a walk of the model tree reads, and which branches it took (the cache key of the layers).
struct Walk {
    state: ItemState,
    trace: Vec<u8>,
    /// The compass needles the state did not hold when a range asked for them.
    wanted: Vec<(NeedleSource, bool)>,
}

impl Walk {
    /// `IsUsingItem`: the holder uses this very stack.
    fn condition(&self, property: &str, component: Option<&(String, bool)>) -> bool {
        self.state.condition(property, component)
    }

    /// The numeric property of a range dispatch, before its scale. Properties this client does
    /// not read give zero.
    fn number(&mut self, property: &str, remaining: bool) -> f32 {
        if let Some((target, wobble)) = NeedleSource::from_property(property) {
            if let Some(bits) = self.state.needles.0[target.index()] {
                return f32::from_bits(bits);
            }
            if !self.wanted.contains(&(target, wobble)) {
                self.wanted.push((target, wobble));
            }
            return 0.0;
        }
        let (Some(item_use), false) = (self.state.item_use, remaining) else {
            return 0.0;
        };
        match property {
            "minecraft:use_duration" => item_use.ticks as f32,
            // `UseCycle`: the ticks left of the use, within the period.
            _ if property.starts_with(USE_CYCLE) => {
                let period: f32 = property[USE_CYCLE.len()..].parse().unwrap_or(1.0);
                (item_use.duration - item_use.ticks) as f32 % period
            }
            // `CrossbowPull`: nothing while it holds a projectile; the share of the charge.
            "minecraft:crossbow/pull" if self.state.charge == Charge::None => {
                item_use.ticks as f32 / item_use.charge_duration as f32
            }
            _ => 0.0,
        }
    }

    /// The entry of a range dispatch the value picks, noted in the trace (zero for none).
    fn range_model<'a>(
        &mut self,
        params: (&str, f32, bool),
        entries: &'a [(f32, ItemModelDef)],
    ) -> Option<&'a ItemModelDef> {
        let index = self.range_entry(params, entries);
        self.trace.push(index.map_or(0, |index| index as u8 + 1));
        index.map(|index| &entries[index].1)
    }

    /// `RangeSelectItemModel.update`: the entry with the greatest threshold the value reaches.
    fn range_entry(
        &mut self,
        (property, scale, remaining): (&str, f32, bool),
        entries: &[(f32, ItemModelDef)],
    ) -> Option<usize> {
        let value = self.number(property, remaining) * scale;
        if value.is_nan() {
            return None;
        }
        let mut best: Option<usize> = None;
        for (index, (threshold, _)) in entries.iter().enumerate() {
            let better = best.is_none_or(|best| *threshold >= entries[best].0);
            if *threshold <= value && better {
                best = Some(index);
            }
        }
        best
    }
}

/// The layers a default stack (no components, not in use) draws in a
/// display context. Properties read the default stack: conditions are
/// false, selections other than the display context and numeric
/// properties take their fallbacks.
pub fn layers_for(def: &ItemModelDef, context: DisplayContext, out: &mut Vec<ItemLayerRef>) {
    layers_in_use(def, context, ItemState::default(), out);
}

/// Every layer an item can draw for any state of the properties this client reads: both sides
/// of a `using_item` condition, every entry of a `use_duration` or `crossbow/pull` range and
/// every case of a `charge_type` select, so their models are loaded.
pub fn layers_any_use(
    def: &ItemModelDef,
    context: DisplayContext,
    (local, out): (&Matrix4, &mut Vec<ItemLayerRef>),
) {
    let children: Vec<&ItemModelDef> = match def {
        ItemModelDef::Transformed { matrix, model } => {
            return layers_any_use(model, context, (&multiply(local, matrix), out));
        }
        ItemModelDef::Composite(models) => models.iter().collect(),
        _ if read_children(def).is_some() => read_children(def).unwrap_or_default(),
        ItemModelDef::Model { .. } | ItemModelDef::Special { .. } => {
            out.extend(leaf(def, local));
            return;
        }
        ItemModelDef::Empty | ItemModelDef::Unknown(_) => return,
        // A property this client does not read: the default stack's way.
        _ => {
            let mut walk = Walk {
                state: ItemState::default(),
                trace: Vec::new(),
                wanted: Vec::new(),
            };
            chosen_child(def, context, &mut walk).into_iter().collect()
        }
    };
    for child in children {
        layers_any_use(child, context, (local, out));
    }
}

/// Whether a range dispatch's property is one this client reads.
fn is_read_range(property: &str) -> bool {
    READ_RANGES.contains(&property)
        || [USE_CYCLE, COMPASS, TIME]
            .iter()
            .any(|prefix| property.starts_with(prefix))
}

/// The children of a node whose property this client reads, all of them: both sides of a
/// `using_item` condition, every entry and the fallback of a `use_duration` or `crossbow/pull`
/// range, every case and the fallback of a `charge_type` select.
fn read_children(def: &ItemModelDef) -> Option<Vec<&ItemModelDef>> {
    match def {
        ItemModelDef::Condition {
            property,
            on_true,
            on_false,
            ..
        } if READ_CONDITIONS.contains(&property.as_str()) => Some(vec![on_true, on_false]),
        ItemModelDef::RangeDispatch {
            property,
            entries,
            fallback,
            ..
        } if is_read_range(property) => Some(
            entries
                .iter()
                .map(|(_, model)| model)
                .chain(fallback.as_deref())
                .collect(),
        ),
        ItemModelDef::Select {
            property,
            cases,
            fallback,
        } if READ_SELECTS.contains(&property.as_str())
            || property.starts_with(BLOCK_STATE_SELECT) =>
        {
            Some(
                cases
                    .iter()
                    .map(|(_, model)| model)
                    .chain(fallback.as_deref())
                    .collect(),
            )
        }
        _ => None,
    }
}

/// The conditions decided here.
const READ_CONDITIONS: [&str; 4] = [
    "minecraft:using_item",
    "minecraft:broken",
    "minecraft:bundle/has_selected_item",
    "minecraft:has_component",
];

/// The selects decided here besides the display context.
const READ_SELECTS: [&str; 3] = [
    "minecraft:charge_type",
    "minecraft:trim_material",
    "minecraft:context_dimension",
];

/// The numeric properties a range dispatch is decided from here.
const READ_RANGES: [&str; 2] = ["minecraft:use_duration", "minecraft:crossbow/pull"];

/// The layers a stack draws for its holder, and the branches the tree took: two stacks of one
/// item with the same trace and components draw the same.
pub fn layers_in_use(
    def: &ItemModelDef,
    context: DisplayContext,
    state: ItemState,
    out: &mut Vec<ItemLayerRef>,
) -> Vec<u8> {
    let mut walk = Walk {
        state,
        trace: Vec::new(),
        wanted: Vec::new(),
    };
    layers_under(def, context, (&IDENTITY, &mut walk), out);
    walk.trace
}

/// The compass needles a model reads for a state that holds none of them yet: the targets of
/// the `compass` ranges the walk reaches, which is what the reference works out and no more.
pub fn needles_wanted(
    def: &ItemModelDef,
    context: DisplayContext,
    state: ItemState,
) -> Vec<(NeedleSource, bool)> {
    let mut walk = Walk {
        state,
        trace: Vec::new(),
        wanted: Vec::new(),
    };
    layers_under(def, context, (&IDENTITY, &mut walk), &mut Vec::new());
    walk.wanted
}

/// The layer a model or a special renderer draws.
fn leaf(def: &ItemModelDef, local: &Matrix4) -> Option<ItemLayerRef> {
    match def {
        ItemModelDef::Model { model, tints } => Some(ItemLayerRef {
            model: model.clone(),
            tints: tints.clone(),
            special: None,
            local: *local,
            nested: None,
        }),
        ItemModelDef::Special { base, model } => Some(ItemLayerRef {
            model: base.clone(),
            tints: Vec::new(),
            special: Some(model.clone()),
            local: *local,
            nested: None,
        }),
        _ => None,
    }
}

/// The value of a property in `name=value` lines.
fn state_value<'a>(lines: &'a str, name: &str) -> Option<&'a str> {
    lines
        .lines()
        .find_map(|line| line.split_once('=').filter(|(key, _)| *key == name))
        .map(|(_, value)| value)
}

/// The `name=value` lines of a stack's `block_state` properties, as an [`ItemState`] holds them.
pub fn block_state_key(properties: &[(String, String)]) -> Option<&'static str> {
    (!properties.is_empty()).then(|| {
        let lines: Vec<String> = properties
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        intern_key(&lines.join("\n"))
    })
}

/// The value a select reads from the holder: the properties this client reads, the display
/// context and what a crossbow holds. `None` takes the fallback.
pub fn select_value<'a>(
    property: &str,
    (context, state): (DisplayContext, &ItemState),
) -> Option<&'a str> {
    Some(match property {
        "minecraft:display_context" => context.name(),
        "minecraft:charge_type" => state.charge.name(),
        "minecraft:trim_material" => state.trim_material?,
        "minecraft:context_dimension" => state.dimension?,
        other => state_value(state.block_state?, other.strip_prefix(BLOCK_STATE_SELECT)?)?,
    })
}

/// `SelectItemModel`: the case of the value the property reads.
fn select_case<'a>(
    property: &str,
    cases: &'a [(Vec<String>, ItemModelDef)],
    holder: (DisplayContext, &ItemState),
) -> Option<&'a ItemModelDef> {
    let value = select_value(property, holder)?;
    cases
        .iter()
        .find(|(when, _)| when.iter().any(|name| name == value))
        .map(|(_, model)| model)
}

/// The one child a node leads to for this holder, if it is not a leaf, a composite or a
/// transformation: the branch of a condition, the case of a select, the entry of a range.
fn chosen_child<'a>(
    def: &'a ItemModelDef,
    context: DisplayContext,
    walk: &mut Walk,
) -> Option<&'a ItemModelDef> {
    match def {
        ItemModelDef::Condition {
            property,
            component,
            on_true,
            on_false,
        } => {
            let chosen = walk.condition(property, component.as_ref());
            walk.trace.push(u8::from(chosen));
            Some(if chosen { on_true } else { on_false })
        }
        ItemModelDef::Select {
            property,
            cases,
            fallback,
        } => select_case(property, cases, (context, &walk.state)).or(fallback.as_deref()),
        ItemModelDef::RangeDispatch {
            property,
            scale,
            remaining,
            entries,
            fallback,
        } => walk
            .range_model((property, *scale, *remaining), entries)
            .or(fallback.as_deref()),
        _ => None,
    }
}

fn layers_under(
    def: &ItemModelDef,
    context: DisplayContext,
    (local, walk): (&Matrix4, &mut Walk),
    out: &mut Vec<ItemLayerRef>,
) {
    match def {
        ItemModelDef::Transformed { matrix, model } => {
            layers_under(model, context, (&multiply(local, matrix), walk), out);
        }
        ItemModelDef::Model { .. } | ItemModelDef::Special { .. } => out.extend(leaf(def, local)),
        ItemModelDef::Composite(models) => {
            for model in models {
                layers_under(model, context, (local, walk), out);
            }
        }
        ItemModelDef::BundleSelectedItem => {
            out.extend(walk.state.selected.map(|item| ItemLayerRef {
                model: Identifier::parse("minecraft:bundle/selected_item").expect("fixed"),
                tints: Vec::new(),
                special: None,
                local: *local,
                nested: Some(item),
            }));
        }
        ItemModelDef::Empty | ItemModelDef::Unknown(_) => {}
        _ => {
            if let Some(child) = chosen_child(def, context, walk) {
                layers_under(child, context, (local, walk), out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOW: &str = r#"{"model":{"type":"minecraft:condition","property":"minecraft:using_item",
        "on_false":{"type":"minecraft:model","model":"minecraft:item/bow"},
        "on_true":{"type":"minecraft:range_dispatch","property":"minecraft:use_duration","scale":0.05,
          "entries":[{"threshold":0.65,"model":{"type":"minecraft:model","model":"minecraft:item/bow_pulling_1"}},
                     {"threshold":0.9,"model":{"type":"minecraft:model","model":"minecraft:item/bow_pulling_2"}}],
          "fallback":{"type":"minecraft:model","model":"minecraft:item/bow_pulling_0"}}}}"#;

    fn drawn_by(text: &str, state: ItemState) -> (String, Vec<u8>) {
        let item = parse_client_item(text).expect("a client item");
        let mut layers = Vec::new();
        let trace = layers_in_use(&item.model, DisplayContext::ALL[0], state, &mut layers);
        (layers[0].model.as_string(), trace)
    }

    fn drawn(item_use: Option<ItemUse>) -> (String, Vec<u8>) {
        drawn_by(
            BOW,
            ItemState {
                item_use,
                ..ItemState::default()
            },
        )
    }

    fn bow_use(ticks: i32) -> Option<ItemUse> {
        Some(ItemUse {
            ticks,
            charge_duration: 25,
            duration: 72000,
        })
    }

    /// The bow draws its pulling models as the draw's ticks (times the scale, in `f32`) reach
    /// the thresholds, and only while this stack is the one in use.
    #[test]
    fn a_drawn_bow_steps_through_its_pulling_models() {
        assert_eq!(drawn(None).0, "minecraft:item/bow");
        let at = |ticks| drawn(bow_use(ticks)).0;
        assert_eq!(at(0), "minecraft:item/bow_pulling_0");
        assert_eq!(at(12), "minecraft:item/bow_pulling_0");
        assert_eq!(at(13), "minecraft:item/bow_pulling_1");
        assert_eq!(at(17), "minecraft:item/bow_pulling_1");
        assert_eq!(at(18), "minecraft:item/bow_pulling_2");
        assert_eq!(at(400), "minecraft:item/bow_pulling_2");
    }

    #[test]
    fn the_trace_tells_the_branches_apart_and_not_the_ticks() {
        let trace = |ticks| drawn(bow_use(ticks)).1;
        assert_eq!(trace(20), trace(40));
        assert_ne!(trace(0), trace(13));
        assert_ne!(drawn(None).1, trace(0));
    }

    const CROSSBOW: &str = r#"{"model":{"type":"minecraft:select","property":"minecraft:charge_type",
        "cases":[{"when":"arrow","model":{"type":"minecraft:model","model":"minecraft:item/crossbow_arrow"}},
                 {"when":"rocket","model":{"type":"minecraft:model","model":"minecraft:item/crossbow_firework"}}],
        "fallback":{"type":"minecraft:condition","property":"minecraft:using_item",
          "on_false":{"type":"minecraft:model","model":"minecraft:item/crossbow"},
          "on_true":{"type":"minecraft:range_dispatch","property":"minecraft:crossbow/pull",
            "entries":[{"threshold":0.58,"model":{"type":"minecraft:model","model":"minecraft:item/crossbow_pulling_1"}},
                       {"threshold":1.0,"model":{"type":"minecraft:model","model":"minecraft:item/crossbow_pulling_2"}}],
            "fallback":{"type":"minecraft:model","model":"minecraft:item/crossbow_pulling_0"}}}}}"#;

    fn crossbow(ticks: i32, charge_duration: i32, charge: Charge) -> String {
        let state = ItemState {
            item_use: Some(ItemUse {
                ticks,
                charge_duration,
                duration: 72000,
            }),
            charge,
            ..ItemState::default()
        };
        drawn_by(CROSSBOW, state).0
    }

    /// A crossbow charges through its pulling models by the share of its charge time, and the
    /// loaded one shows what it holds whether or not it is used.
    #[test]
    fn a_crossbow_charges_and_then_shows_its_projectile() {
        let at = |ticks, duration| crossbow(ticks, duration, Charge::None);
        // The plain charge is 25 ticks, quick charge I 20.
        assert_eq!(at(14, 25), "minecraft:item/crossbow_pulling_0");
        assert_eq!(at(15, 25), "minecraft:item/crossbow_pulling_1");
        assert_eq!(at(24, 25), "minecraft:item/crossbow_pulling_1");
        assert_eq!(at(25, 25), "minecraft:item/crossbow_pulling_2");
        assert_eq!(at(11, 20), "minecraft:item/crossbow_pulling_0");
        assert_eq!(at(12, 20), "minecraft:item/crossbow_pulling_1");
        assert_eq!(at(19, 20), "minecraft:item/crossbow_pulling_1");
        assert_eq!(at(20, 20), "minecraft:item/crossbow_pulling_2");
        assert_eq!(
            crossbow(3, 25, Charge::Arrow),
            "minecraft:item/crossbow_arrow"
        );
        assert_eq!(
            drawn_by(
                CROSSBOW,
                ItemState {
                    charge: Charge::Rocket,
                    ..ItemState::default()
                }
            )
            .0,
            "minecraft:item/crossbow_firework"
        );
        assert_eq!(
            drawn_by(CROSSBOW, ItemState::default()).0,
            "minecraft:item/crossbow"
        );
    }

    /// The models of every branch are loaded, not only the idle ones.
    #[test]
    fn every_branch_of_the_read_properties_is_loaded() {
        let item = parse_client_item(CROSSBOW).expect("a client item");
        let mut layers = Vec::new();
        layers_any_use(
            &item.model,
            DisplayContext::ALL[0],
            (&IDENTITY, &mut layers),
        );
        let names: Vec<String> = layers.iter().map(|l| l.model.as_string()).collect();
        for name in [
            "crossbow",
            "crossbow_arrow",
            "crossbow_firework",
            "crossbow_pulling_0",
            "crossbow_pulling_1",
            "crossbow_pulling_2",
        ] {
            assert!(names.contains(&format!("minecraft:item/{name}")), "{name}");
        }
    }

    const ELYTRA: &str = r#"{"model":{"type":"minecraft:condition","property":"minecraft:broken",
        "on_false":{"type":"minecraft:model","model":"minecraft:item/elytra"},
        "on_true":{"type":"minecraft:model","model":"minecraft:item/elytra_broken"}}}"#;

    const WOLF_ARMOR: &str = r#"{"model":{"type":"minecraft:condition","property":"minecraft:has_component",
        "component":"minecraft:dyed_color",
        "on_false":{"type":"minecraft:model","model":"minecraft:item/wolf_armor"},
        "on_true":{"type":"minecraft:model","model":"minecraft:item/wolf_armor_dyed"}}}"#;

    #[test]
    fn a_broken_elytra_draws_the_cracked_model() {
        let at = |broken| {
            drawn_by(
                ELYTRA,
                ItemState {
                    broken,
                    ..ItemState::default()
                },
            )
            .0
        };
        assert_eq!(at(false), "minecraft:item/elytra");
        assert_eq!(at(true), "minecraft:item/elytra_broken");
    }

    #[test]
    fn dyed_wolf_armor_draws_the_dyed_model() {
        let at = |dyed_color| {
            let components = TestedComponents {
                dyed_color,
                ..TestedComponents::default()
            };
            drawn_by(
                WOLF_ARMOR,
                ItemState {
                    components,
                    ..ItemState::default()
                },
            )
            .0
        };
        assert_eq!(at(false), "minecraft:item/wolf_armor");
        assert_eq!(at(true), "minecraft:item/wolf_armor_dyed");
    }

    const BUNDLE: &str = r#"{"model":{"type":"minecraft:condition","property":"minecraft:bundle/has_selected_item",
        "on_false":{"type":"minecraft:model","model":"minecraft:item/bundle"},
        "on_true":{"type":"minecraft:composite","models":[
            {"type":"minecraft:model","model":"minecraft:item/bundle_open_back"},
            {"type":"minecraft:bundle/selected_item"},
            {"type":"minecraft:model","model":"minecraft:item/bundle_open_front"}]}}}"#;

    #[test]
    fn an_open_bundle_draws_its_selected_item_between_its_halves() {
        let item = parse_client_item(BUNDLE).expect("a client item");
        let layers_of = |selected| {
            let state = ItemState {
                selected,
                ..ItemState::default()
            };
            let mut layers = Vec::new();
            layers_in_use(&item.model, DisplayContext::ALL[0], state, &mut layers);
            layers
        };
        let closed = layers_of(None);
        assert_eq!(closed.len(), 1);
        assert_eq!(closed[0].model.as_string(), "minecraft:item/bundle");
        let open = layers_of(Some(photon_core::ids::ItemId(7)));
        assert_eq!(open.len(), 3);
        assert_eq!(open[0].model.as_string(), "minecraft:item/bundle_open_back");
        assert!(open[1].nested.is_some());
        assert_eq!(
            open[2].model.as_string(),
            "minecraft:item/bundle_open_front"
        );
    }

    const TRIMMED: &str = r#"{"model":{"type":"minecraft:select","property":"minecraft:trim_material",
        "cases":[{"model":{"type":"minecraft:model","model":"minecraft:item/iron_chestplate_gold_trim"},"when":"minecraft:gold"},
                 {"model":{"type":"minecraft:model","model":"minecraft:item/iron_chestplate_iron_trim"},"when":"minecraft:iron"}],
        "fallback":{"type":"minecraft:model","model":"minecraft:item/iron_chestplate"}}}"#;

    #[test]
    fn a_trimmed_armor_draws_the_model_of_its_material() {
        let at = |trim_material| {
            drawn_by(
                TRIMMED,
                ItemState {
                    trim_material,
                    ..ItemState::default()
                },
            )
            .0
        };
        assert_eq!(at(None), "minecraft:item/iron_chestplate");
        assert_eq!(
            at(Some(intern_key("minecraft:gold"))),
            "minecraft:item/iron_chestplate_gold_trim"
        );
        assert_eq!(
            at(Some(intern_key("minecraft:iron"))),
            "minecraft:item/iron_chestplate_iron_trim"
        );
        // A material with no case of its own takes the fallback.
        assert_eq!(
            at(Some(intern_key("minecraft:custom"))),
            "minecraft:item/iron_chestplate"
        );
    }

    #[test]
    fn a_case_without_a_namespace_names_the_vanilla_material() {
        let text = TRIMMED.replace("minecraft:gold\"", "gold\"");
        assert_ne!(text, TRIMMED);
        let (model, _) = drawn_by(
            &text,
            ItemState {
                trim_material: Some(intern_key("minecraft:gold")),
                ..ItemState::default()
            },
        );
        assert_eq!(model, "minecraft:item/iron_chestplate_gold_trim");
    }

    const COMPASS_MODEL: &str = r#"{"model":{"type":"minecraft:condition","property":"minecraft:has_component",
        "component":"minecraft:lodestone_tracker",
        "on_true":{"type":"minecraft:range_dispatch","property":"minecraft:compass","scale":2.0,"target":"lodestone",
            "entries":[{"threshold":0.0,"model":{"type":"minecraft:model","model":"minecraft:item/lode_0"}},
                       {"threshold":1.0,"model":{"type":"minecraft:model","model":"minecraft:item/lode_1"}}]},
        "on_false":{"type":"minecraft:range_dispatch","property":"minecraft:compass","scale":2.0,"target":"spawn","wobble":false,
            "entries":[{"threshold":0.0,"model":{"type":"minecraft:model","model":"minecraft:item/spawn_0"}},
                       {"threshold":1.0,"model":{"type":"minecraft:model","model":"minecraft:item/spawn_1"}}]}}}"#;

    #[test]
    fn a_compass_asks_only_for_the_needle_of_the_target_it_reaches() {
        let item = parse_client_item(COMPASS_MODEL).expect("a client item");
        let wanted = |lodestone_tracker| {
            let state = ItemState {
                components: TestedComponents {
                    lodestone_tracker,
                    ..TestedComponents::default()
                },
                ..ItemState::default()
            };
            needles_wanted(&item.model, DisplayContext::Gui, state)
        };
        assert_eq!(wanted(false), [(NeedleSource::Spawn, false)]);
        assert_eq!(wanted(true), [(NeedleSource::Lodestone, true)]);
        let mut needles = Needles::default();
        needles.0[NeedleSource::Spawn.index()] = Some(0.75_f32.to_bits());
        let state = ItemState {
            needles,
            ..ItemState::default()
        };
        assert!(needles_wanted(&item.model, DisplayContext::Gui, state).is_empty());
        assert_eq!(drawn_by(COMPASS_MODEL, state).0, "minecraft:item/spawn_1");
    }

    const LIGHT: &str = r#"{"model":{"type":"minecraft:select","property":"minecraft:block_state",
        "block_state_property":"level",
        "cases":[{"model":{"type":"minecraft:model","model":"minecraft:item/light_00"},"when":"0"},
                 {"model":{"type":"minecraft:model","model":"minecraft:item/light_07"},"when":"7"}],
        "fallback":{"type":"minecraft:model","model":"minecraft:item/light_15"}}}"#;

    #[test]
    fn a_light_block_draws_the_model_of_its_level() {
        let at = |block_state| {
            drawn_by(
                LIGHT,
                ItemState {
                    block_state,
                    ..ItemState::default()
                },
            )
            .0
        };
        assert_eq!(at(Some(intern_key("level=7"))), "minecraft:item/light_07");
        assert_eq!(
            at(Some(intern_key("other=1\nlevel=0"))),
            "minecraft:item/light_00"
        );
        // Another property, or none at all, takes the fallback.
        assert_eq!(at(Some(intern_key("other=7"))), "minecraft:item/light_15");
        assert_eq!(at(None), "minecraft:item/light_15");
    }

    const BRUSH: &str = r#"{"model":{"type":"minecraft:range_dispatch","property":"minecraft:use_cycle",
        "period":10.0,"scale":0.1,
        "entries":[{"threshold":0.25,"model":{"type":"minecraft:model","model":"minecraft:item/brush_brushing_0"}},
                   {"threshold":0.5,"model":{"type":"minecraft:model","model":"minecraft:item/brush_brushing_1"}},
                   {"threshold":0.75,"model":{"type":"minecraft:model","model":"minecraft:item/brush_brushing_2"}}],
        "fallback":{"type":"minecraft:model","model":"minecraft:item/brush"}}}"#;

    #[test]
    fn a_brush_cycles_its_models_as_the_use_runs_down() {
        let at = |item_use| {
            drawn_by(
                BRUSH,
                ItemState {
                    item_use,
                    ..ItemState::default()
                },
            )
            .0
        };
        let using = |ticks| {
            Some(ItemUse {
                ticks,
                charge_duration: 0,
                duration: 200,
            })
        };
        assert_eq!(at(None), "minecraft:item/brush");
        // 200 left is a whole cycle, so zero; 3 left is 0.3 of it, and so on.
        assert_eq!(at(using(0)), "minecraft:item/brush");
        assert_eq!(at(using(197)), "minecraft:item/brush_brushing_0");
        assert_eq!(at(using(194)), "minecraft:item/brush_brushing_1");
        assert_eq!(at(using(191)), "minecraft:item/brush_brushing_2");
        assert_eq!(at(using(190)), "minecraft:item/brush");
    }
}
