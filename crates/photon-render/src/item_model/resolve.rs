//! The layers an item model draws for a stack and its holder (`ItemModel.update`): conditions,
//! ranges and selects are decided from the properties of the stack in use.

use super::*;

/// The use of the stack by its holder: how many ticks it has been used, and how many a crossbow
/// takes to charge (`CrossbowItem.getChargeDuration`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUse {
    pub ticks: i32,
    pub charge_duration: i32,
}

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
}

/// What a walk of the model tree reads, and which branches it took (the cache key of the layers).
struct Walk {
    state: ItemState,
    trace: Vec<u8>,
}

impl Walk {
    /// `IsUsingItem`: the holder uses this very stack.
    fn condition(&self, property: &str) -> bool {
        property == "minecraft:using_item" && self.state.item_use.is_some()
    }

    /// The numeric property of a range dispatch, before its scale. Properties this client does
    /// not read give zero.
    fn number(&self, property: &str, remaining: bool) -> f32 {
        let (Some(item_use), false) = (self.state.item_use, remaining) else {
            return 0.0;
        };
        match property {
            "minecraft:use_duration" => item_use.ticks as f32,
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
        &self,
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
            };
            chosen_child(def, context, &mut walk).into_iter().collect()
        }
    };
    for child in children {
        layers_any_use(child, context, (local, out));
    }
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
        } if property == "minecraft:using_item" => Some(vec![on_true, on_false]),
        ItemModelDef::RangeDispatch {
            property,
            entries,
            fallback,
            ..
        } if READ_RANGES.contains(&property.as_str()) => Some(
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
        } if property == "minecraft:charge_type" => Some(
            cases
                .iter()
                .map(|(_, model)| model)
                .chain(fallback.as_deref())
                .collect(),
        ),
        _ => None,
    }
}

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
    };
    layers_under(def, context, (&IDENTITY, &mut walk), out);
    walk.trace
}

/// The layer a model or a special renderer draws.
fn leaf(def: &ItemModelDef, local: &Matrix4) -> Option<ItemLayerRef> {
    match def {
        ItemModelDef::Model { model, tints } => Some(ItemLayerRef {
            model: model.clone(),
            tints: tints.clone(),
            special: None,
            local: *local,
        }),
        ItemModelDef::Special { base, model } => Some(ItemLayerRef {
            model: base.clone(),
            tints: Vec::new(),
            special: Some(model.clone()),
            local: *local,
        }),
        _ => None,
    }
}

/// `SelectItemModel`: the case of the value of the properties this client reads, the display
/// context and what a crossbow holds.
fn select_case<'a>(
    property: &str,
    cases: &'a [(Vec<String>, ItemModelDef)],
    (context, charge): (DisplayContext, Charge),
) -> Option<&'a ItemModelDef> {
    let value = match property {
        "minecraft:display_context" => context.name(),
        "minecraft:charge_type" => charge.name(),
        _ => return None,
    };
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
            on_true,
            on_false,
        } => {
            let chosen = walk.condition(property);
            walk.trace.push(u8::from(chosen));
            Some(if chosen { on_true } else { on_false })
        }
        ItemModelDef::Select {
            property,
            cases,
            fallback,
        } => select_case(property, cases, (context, walk.state.charge)).or(fallback.as_deref()),
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
            }),
            charge,
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
}
