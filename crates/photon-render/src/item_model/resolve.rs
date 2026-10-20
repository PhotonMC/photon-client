//! The layers an item model draws for a stack and its holder (`ItemModel.update`): conditions,
//! ranges and selects are decided from the properties of the stack in use.

use super::*;

/// The use of the stack by its holder: how many ticks it has been used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemUse {
    pub ticks: i32,
}

/// What a walk of the model tree reads, and which branches it took (the cache key of the layers).
struct Walk {
    item_use: Option<ItemUse>,
    trace: Vec<u8>,
}

impl Walk {
    /// `IsUsingItem`: the holder uses this very stack.
    fn condition(&self, property: &str) -> bool {
        property == "minecraft:using_item" && self.item_use.is_some()
    }

    /// The numeric property of a range dispatch, before its scale. Properties this client does
    /// not read give zero.
    fn number(&self, property: &str, remaining: bool) -> f32 {
        match (property, self.item_use) {
            ("minecraft:use_duration", Some(item_use)) if !remaining => item_use.ticks as f32,
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
    layers_in_use(def, context, None, out);
}

/// Every layer an item can draw in any use of it: the default ones, and the models of the
/// `using_item` conditions and `use_duration` ranges, so their models are loaded.
pub fn layers_any_use(def: &ItemModelDef, context: DisplayContext, out: &mut Vec<ItemLayerRef>) {
    for ticks in [None, Some(0), Some(i32::MAX)] {
        layers_in_use(def, context, ticks.map(|ticks| ItemUse { ticks }), out);
    }
    // Between the thresholds of a draw: every entry's own threshold, as a tick count.
    let mut thresholds = Vec::new();
    use_thresholds(def, &mut thresholds);
    for ticks in thresholds {
        layers_in_use(def, context, Some(ItemUse { ticks }), out);
    }
}

/// The tick counts at which the `use_duration` ranges below `def` change their entry.
fn use_thresholds(def: &ItemModelDef, out: &mut Vec<i32>) {
    match def {
        ItemModelDef::Transformed { model, .. } => use_thresholds(model, out),
        ItemModelDef::Composite(models) => models.iter().for_each(|m| use_thresholds(m, out)),
        ItemModelDef::Condition {
            on_true, on_false, ..
        } => {
            use_thresholds(on_true, out);
            use_thresholds(on_false, out);
        }
        ItemModelDef::RangeDispatch {
            property,
            scale,
            entries,
            fallback,
            ..
        } => {
            if property == "minecraft:use_duration" && *scale > 0.0 {
                out.extend(
                    entries
                        .iter()
                        .map(|(threshold, _)| (threshold / scale).ceil() as i32),
                );
            }
            for model in entries
                .iter()
                .map(|(_, model)| model)
                .chain(fallback.as_deref())
            {
                use_thresholds(model, out);
            }
        }
        _ => {}
    }
}

/// The layers a stack draws for its holder, and the branches the tree took: two stacks of one
/// item with the same trace and components draw the same.
pub fn layers_in_use(
    def: &ItemModelDef,
    context: DisplayContext,
    item_use: Option<ItemUse>,
    out: &mut Vec<ItemLayerRef>,
) -> Vec<u8> {
    let mut walk = Walk {
        item_use,
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

/// `SelectItemModel`: the case of the display context, the one property this client reads.
fn select_case<'a>(
    property: &str,
    cases: &'a [(Vec<String>, ItemModelDef)],
    context: DisplayContext,
) -> Option<&'a ItemModelDef> {
    (property == "minecraft:display_context")
        .then(|| {
            cases
                .iter()
                .find(|(when, _)| when.iter().any(|name| name == context.name()))
                .map(|(_, model)| model)
        })
        .flatten()
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
        } => select_case(property, cases, context).or(fallback.as_deref()),
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

    fn drawn(item_use: Option<ItemUse>) -> (String, Vec<u8>) {
        let item = parse_client_item(BOW).expect("a client item");
        let mut layers = Vec::new();
        let trace = layers_in_use(&item.model, DisplayContext::ALL[0], item_use, &mut layers);
        (layers[0].model.as_string(), trace)
    }

    /// The bow draws its pulling models as the draw's ticks (times the scale, in `f32`) reach
    /// the thresholds, and only while this stack is the one in use.
    #[test]
    fn a_drawn_bow_steps_through_its_pulling_models() {
        assert_eq!(drawn(None).0, "minecraft:item/bow");
        let at = |ticks| drawn(Some(ItemUse { ticks })).0;
        assert_eq!(at(0), "minecraft:item/bow_pulling_0");
        assert_eq!(at(12), "minecraft:item/bow_pulling_0");
        assert_eq!(at(13), "minecraft:item/bow_pulling_1");
        assert_eq!(at(17), "minecraft:item/bow_pulling_1");
        assert_eq!(at(18), "minecraft:item/bow_pulling_2");
        assert_eq!(at(400), "minecraft:item/bow_pulling_2");
    }

    #[test]
    fn the_trace_tells_the_branches_apart_and_not_the_ticks() {
        let trace = |ticks| drawn(Some(ItemUse { ticks })).1;
        assert_eq!(trace(20), trace(40));
        assert_ne!(trace(0), trace(13));
        assert_ne!(drawn(None).1, trace(0));
    }
}
