//! `BundleItem`'s click overrides and `BundleContents.Mutable`: a bundle
//! carried onto a slot takes its stack in or puts one out, and a stack
//! carried onto a bundle goes in (or one comes out onto the cursor).
//!
//! The selected item (`SelectBundleItem`, from scrolling over the bundle's
//! tooltip) is not ported: every override resets it, so the first stack is
//! the one taken out.

use super::*;
use photon_world::item_components::ids;

/// The sounds a bundle click plays (`BundleItem.playInsertSound` and the like).
pub const BUNDLE_INSERT: &str = "minecraft:item.bundle.insert";
pub const BUNDLE_INSERT_FAIL: &str = "minecraft:item.bundle.insert_fail";
pub const BUNDLE_REMOVE_ONE: &str = "minecraft:item.bundle.remove_one";

/// Apache Commons `Fraction`, reduced; exact for a bundle's weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fraction {
    numerator: i64,
    denominator: i64,
}

impl Fraction {
    const ZERO: Self = Self::new(0, 1);
    const ONE: Self = Self::new(1, 1);

    const fn new(numerator: i64, denominator: i64) -> Self {
        let (mut a, mut b) = (numerator.abs(), denominator.abs());
        while b != 0 {
            (a, b) = (b, a % b);
        }
        let gcd = if a == 0 { 1 } else { a };
        let sign = if denominator < 0 { -1 } else { 1 };
        Self {
            numerator: sign * numerator / gcd,
            denominator: sign * denominator / gcd,
        }
    }

    const fn add(self, other: Self) -> Self {
        Self::new(
            self.numerator * other.denominator + other.numerator * self.denominator,
            self.denominator * other.denominator,
        )
    }

    const fn subtract(self, other: Self) -> Self {
        self.add(Self::new(-other.numerator, other.denominator))
    }

    const fn times(self, count: i64) -> Self {
        Self::new(self.numerator * count, self.denominator)
    }

    /// `divideBy(...).intValue()`: truncated toward zero.
    const fn whole_times_in(self, divisor: Self) -> i64 {
        (self.numerator * divisor.denominator) / (self.denominator * divisor.numerator)
    }
}

/// A bundle's contents, or `None` for a stack with no `bundle_contents`
/// (the bundles' own is empty).
fn contents(stack: &Stack) -> Option<Vec<Stack>> {
    let value = match stack.patched(ids::BUNDLE_CONTENTS) {
        Some(Some(value)) => value,
        Some(None) => return None,
        None => {
            let bundle = stacking(stack).is_some_and(|item| {
                item.class.owner_of(ItemMethod::OverrideStackedOnOther) == ItemClass::BundleItem
            });
            return bundle.then(Vec::new);
        }
    };
    let mut input = value.as_slice();
    let count = photon_core::wire::read_varint(&mut input).ok()?;
    let mut items = Vec::new();
    for _ in 0..count {
        let item = photon_core::wire::read_varint(&mut input).ok()?;
        let count = photon_core::wire::read_varint(&mut input).ok()?;
        let start = input;
        let patch = photon_world::item_components::read_patch(&mut input).ok()?;
        let bytes = start[..start.len() - input.len()].to_vec();
        items.push(
            Stack::from_wire(
                count,
                item,
                patch.added.iter().map(|(id, _)| *id).collect(),
                patch.removed,
            )
            .with_patch_bytes(bytes),
        );
    }
    Some(items)
}

/// `stack.set(BUNDLE_CONTENTS, ...)`: an empty bundle is the item's own, so
/// the patch drops the entry; a changed one keeps its place in the patch.
fn set_contents(stack: &mut Stack, items: &[Stack]) {
    let mut patch = stack.patch();
    let place = patch
        .added
        .iter()
        .position(|(id, _)| *id == ids::BUNDLE_CONTENTS);
    if items.is_empty() {
        if let Some(place) = place {
            patch.added.remove(place);
        }
    } else {
        let mut value = Vec::new();
        photon_core::wire::write_varint(items.len() as i32, &mut value);
        for item in items {
            photon_core::wire::write_varint(item.item_registry_id(), &mut value);
            photon_core::wire::write_varint(item.item_count(), &mut value);
            value.extend(item.patch().trusted());
        }
        match place {
            Some(place) => patch.added[place].1 = value,
            None => patch.added.push((ids::BUNDLE_CONTENTS, value)),
        }
    }
    stack.set_patch(&patch);
}

/// `BundleContents.getWeight`: a bundle weighs its contents and a
/// sixteenth, a hive with bees a whole bundle, anything else one over its
/// stack size. `None` where the reference's result is an error.
fn weight(stack: &Stack) -> Option<Fraction> {
    if let Some(items) = contents(stack) {
        return Some(content_weight(&items)?.add(Fraction::new(1, 16)));
    }
    let bees = match stack.patched(ids::BEES) {
        Some(Some(value)) => photon_core::wire::read_varint(&mut value.as_slice()).ok()? > 0,
        _ => false,
    };
    Some(if bees {
        Fraction::ONE
    } else {
        Fraction::new(1, i64::from(max_stack_size(stack)))
    })
}

/// `BundleContents.computeContentWeight`.
fn content_weight(items: &[Stack]) -> Option<Fraction> {
    items.iter().try_fold(Fraction::ZERO, |sum, item| {
        Some(sum.add(weight(item)?.times(i64::from(item.item_count()))))
    })
}

/// `BundleItem.getWeightSafe` of a bundle's contents, as a numerator and a
/// denominator (zero for an error); none for a stack that is no bundle.
pub(super) fn bundle_weight(stack: &Stack) -> Option<(i64, i64)> {
    let bundle = stacking(stack).is_some_and(|item| {
        item.class.owner_of(ItemMethod::OverrideStackedOnOther) == ItemClass::BundleItem
    });
    if !bundle {
        return None;
    }
    let weight = contents(stack)
        .and_then(|items| content_weight(&items))
        .unwrap_or(Fraction::ZERO);
    Some((weight.numerator, weight.denominator))
}

/// `BundleContents.canItemBeInBundle`.
fn can_be_in_bundle(stack: &Stack) -> bool {
    stack.item_count() > 0 && can_fit_inside_container_items(stack)
}

/// `BundleContents.Mutable`.
struct Contents {
    items: Vec<Stack>,
    weight: Fraction,
}

impl Contents {
    /// A bundle whose weight is an error starts empty.
    fn new(items: Vec<Stack>) -> Self {
        match content_weight(&items) {
            Some(weight) => Self { items, weight },
            None => Self {
                items: Vec::new(),
                weight: Fraction::ZERO,
            },
        }
    }

    fn max_amount_to_add(&self, item_weight: Fraction) -> i32 {
        Fraction::ONE
            .subtract(self.weight)
            .whole_times_in(item_weight)
            .max(0) as i32
    }

    /// `tryInsert`: as much of the stack as fits goes in first, merged with
    /// a stack of the same item and components; the stack shrinks by it.
    fn try_insert(&mut self, stack: &mut Stack) -> i32 {
        if !can_be_in_bundle(stack) {
            return 0;
        }
        let Some(item_weight) = weight(stack) else {
            return 0;
        };
        let amount = stack.item_count().min(self.max_amount_to_add(item_weight));
        if amount == 0 {
            return 0;
        }
        self.weight = self.weight.add(item_weight.times(i64::from(amount)));
        let found = is_stackable(stack)
            .then(|| {
                self.items
                    .iter()
                    .position(|item| item.is_same_item_same_components(stack))
            })
            .flatten();
        if let Some(found) = found {
            let removed = self.items.remove(found);
            let merged = removed.copy_with_count(removed.item_count() + amount);
            stack.set_count(stack.item_count() - amount);
            self.items.insert(0, merged);
        } else {
            self.items.insert(0, stack.split(amount));
        }
        amount
    }

    /// `removeOne`: the first stack, with no item selected.
    fn remove_one(&mut self) -> Option<Stack> {
        if self.items.is_empty() {
            return None;
        }
        let removed = self.items.remove(0);
        let stack = removed.copy_with_count(removed.item_count());
        let item_weight = weight(&stack).expect("it went in with a weight");
        self.weight = self
            .weight
            .subtract(item_weight.times(i64::from(stack.item_count())));
        Some(stack)
    }
}

impl ContainerMenu {
    /// `AbstractContainerMenu.tryItemClickBehaviourOverride` for bundles:
    /// whether the click was the bundle's, not the slot's.
    pub(super) fn bundle_click(&mut self, index: usize, primary: bool, player: Player) -> bool {
        if let Some(items) = self.carried.as_ref().and_then(contents) {
            let slot_empty = self.slots[index].is_none();
            if primary && !slot_empty {
                let mut bundle = Contents::new(items);
                let moved = self.try_transfer(&mut bundle, index, player);
                self.sounds.push(if moved > 0 {
                    BUNDLE_INSERT
                } else {
                    BUNDLE_INSERT_FAIL
                });
                set_contents(self.carried.as_mut().expect("checked above"), &bundle.items);
                return true;
            }
            if !primary && slot_empty {
                let mut bundle = Contents::new(items);
                if let Some(stack) = bundle.remove_one() {
                    let count = stack.item_count();
                    match self.safe_insert(index, stack, count) {
                        Some(mut remainder) if remainder.item_count() > 0 => {
                            bundle.try_insert(&mut remainder);
                        }
                        _ => self.sounds.push(BUNDLE_REMOVE_ONE),
                    }
                }
                set_contents(self.carried.as_mut().expect("checked above"), &bundle.items);
                return true;
            }
        }
        let Some(items) = self.slots[index].as_ref().and_then(contents) else {
            return false;
        };
        let mut bundle = Contents::new(items);
        let handled = match (primary, self.carried.is_some()) {
            (true, true) => {
                let allowed = self.allow_modification(index);
                let carried = self.carried.as_mut().expect("checked above");
                let inserted = allowed && bundle.try_insert(carried) > 0;
                settle(&mut self.carried);
                self.sounds.push(if inserted {
                    BUNDLE_INSERT
                } else {
                    BUNDLE_INSERT_FAIL
                });
                true
            }
            (false, false) => {
                if self.allow_modification(index)
                    && let Some(removed) = bundle.remove_one()
                {
                    self.sounds.push(BUNDLE_REMOVE_ONE);
                    self.carried = Some(removed);
                }
                true
            }
            // `toggleSelectedItem(-1)`, which only rewrites the contents.
            _ => false,
        };
        set_contents(
            self.slots[index].as_mut().expect("checked above"),
            &bundle.items,
        );
        handled
    }

    /// `BundleContents.Mutable.tryTransfer`: as much of the slot's stack as
    /// fits, taken the way the player takes it.
    fn try_transfer(&mut self, bundle: &mut Contents, index: usize, player: Player) -> i32 {
        let Some(other) = self.slots[index].clone() else {
            return 0;
        };
        let Some(item_weight) = weight(&other) else {
            return 0;
        };
        let max = bundle.max_amount_to_add(item_weight);
        if !can_be_in_bundle(&other) {
            return 0;
        }
        self.safe_take(index, other.item_count(), max, player)
            .map_or(0, |mut taken| bundle.try_insert(&mut taken))
    }
}
