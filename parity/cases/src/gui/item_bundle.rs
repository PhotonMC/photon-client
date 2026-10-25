//! The `bundle/fullness` range property: a bundle's weight as a float, errors included.

use serde_json::{Value, json};

use super::gui;
use crate::components::{BUNDLE_CONTENTS, MAX_STACK_SIZE, Stack, bundle, patch, patch_of, stack, vi};

/// A stack of `item` with its maximum size patched.
pub fn sized(item: &'static str, count: i64, max: i64) -> Stack {
    (item, count, patch(&[(MAX_STACK_SIZE, vi(max))]))
}

/// One stack of the primes as maximum sizes: weights whose denominators multiply up.
pub fn primes(count: usize) -> Vec<Stack> {
    [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31].iter().take(count).map(|max| sized("stone", 1, *max)).collect()
}

/// A bundle holding `items`.
fn holding(label: &str, item: &str, items: &[Stack]) -> Value {
    json!({"label": label, "item": item, "components": [], "patch": bundle(items)})
}

fn rows() -> Vec<Value> {
    let inner_bundle = ("bundle", 1, bundle(&[stack("stone", 32)]));
    let mut rows = vec![
        json!({"label": "bundle-new", "item": "minecraft:bundle", "components": []}),
        json!({"label": "stone-new", "item": "minecraft:stone", "components": []}),
        json!({"label": "bundle-removed", "item": "minecraft:bundle", "components": [],
            "patch": patch_of(&[], &[BUNDLE_CONTENTS])}),
        holding("bundle-empty", "minecraft:bundle", &[]),
        holding("bundle-one-stone", "minecraft:bundle", &[stack("stone", 1)]),
        holding("bundle-32-stone", "minecraft:bundle", &[stack("stone", 32)]),
        holding("bundle-64-stone", "minecraft:bundle", &[stack("stone", 64)]),
        holding("bundle-63-stone", "minecraft:bundle", &[stack("stone", 63)]),
        holding("bundle-3-pearls", "minecraft:bundle", &[stack("ender_pearl", 3)]),
        holding("bundle-pearls-and-stone", "minecraft:bundle", &[stack("ender_pearl", 5), stack("stone", 7)]),
        holding("bundle-tools", "minecraft:bundle", &[stack("iron_sword", 1), stack("iron_sword", 1)]),
        holding("bundle-nested-empty", "minecraft:bundle", &[("bundle", 1, patch(&[]))]),
        holding("bundle-nested-full", "minecraft:bundle", std::slice::from_ref(&inner_bundle)),
        holding("bundle-nested-twice", "minecraft:bundle", &[inner_bundle.clone(), inner_bundle]),
        holding("bundle-over-full", "minecraft:bundle", &[stack("stone", 64), stack("dirt", 64)]),
        holding("stone-with-contents", "minecraft:stone", &[stack("stone", 16)]),
        holding("bundle-sized-3", "minecraft:bundle", &[sized("stone", 7, 3)]),
        holding("bundle-sized-99", "minecraft:bundle", &[sized("stone", 5, 99)]),
        holding("bundle-count-int-max", "minecraft:bundle", &[stack("stone", 2_147_483_647)]),
        holding(
            "bundle-count-twice-int-max",
            "minecraft:bundle",
            &[stack("stone", 2_147_483_647), stack("stone", 2_147_483_647)],
        ),
        holding("bundle-count-2-30", "minecraft:bundle", &[stack("stone", 1 << 30)]),
    ];
    // Denominators that share factors take the other branch of the addition.
    let powers = |sizes: &[i64]| {
        let mut items = primes(9);
        items.extend(sizes.iter().map(|max| sized("stone", 1, *max)));
        items
    };
    rows.push(holding("bundle-powers-ok", "minecraft:bundle", &powers(&[4, 8, 16])));
    rows.push(holding("bundle-powers-overflow", "minecraft:bundle", &powers(&[4, 8, 16, 32])));
    rows.push(holding(
        "bundle-numerator-overflow",
        "minecraft:bundle",
        &[sized("stone", 2_147_483_647, 2), sized("stone", 2_147_483_647, 4)],
    ));
    // Each way the checked addition fails (the denominator or the numerator of either kind of addition) as
    // the last of the sizes, and the sizes before it, which still add up.
    let failing: [(&str, &[i64]); 4] = [
        ("coprime-denominator", &[65, 86, 69, 77, 89]),
        ("shared-denominator", &[94, 77, 95, 67, 96]),
        ("shared-numerator", &[87, 89, 2, 85, 91, 2, 4, 32]),
        ("coprime-sum", &[67, 74, 81, 65, 2, 13, 79, 2]),
    ];
    for (name, sizes) in failing {
        let items = |sizes: &[i64]| sizes.iter().map(|max| sized("stone", 1, *max)).collect::<Vec<_>>();
        let (_, before) = sizes.split_last().expect("sizes");
        rows.push(holding(&format!("bundle-before-{name}"), "minecraft:bundle", &items(before)));
        rows.push(holding(&format!("bundle-fails-{name}"), "minecraft:bundle", &items(sizes)));
    }
    for count in [1, 5, 9, 10, 11] {
        rows.push(holding(&format!("bundle-primes-{count}"), "minecraft:bundle", &primes(count)));
    }
    rows
}

/// The `bundle/fullness` range over the rows.
pub fn item_fullness() -> Value {
    let rows = rows().into_iter().map(|mut row| {
        row["property"] = json!({"property": "minecraft:bundle/fullness"});
        row
    });
    gui("item-bundle-fullness", "item_number", rows.collect())
}
