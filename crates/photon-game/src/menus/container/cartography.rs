//! What the cartography table's screen draws of its menu (`CartographyTableScreen`): the picture
//! for the item in the second slot, the error mark and the map shown, decided from the stacks
//! and what the client knows of the map.

use photon_core::wire::read_varint;
use photon_data::item::component_types as ids;

use super::{ContainerMenu, Stack, stacking};

/// What the client knows of a map that the table reads (`MapItemSavedData.scale`, `locked`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapFacts {
    pub scale: i8,
    pub locked: bool,
}

/// Which picture the table draws behind the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartographyLayout {
    /// The paper's: a map that will grow.
    Scaled,
    /// A map in the second slot: two maps.
    Duplicated,
    /// Glass: a map to lock, with the lock.
    Locking,
    /// Anything else.
    Plain,
}

/// The table's drawing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CartographyView {
    pub layout: CartographyLayout,
    /// How many times the error mark is drawn (a locked map with paper at too great a scale
    /// draws it twice).
    pub error_marks: u8,
    /// The map to draw in the picture: its id, once the client has its data.
    pub map: Option<i32>,
}

/// What the second slot holds that the table does something with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Extra {
    Map,
    Paper,
    GlassPane,
}

/// The table's use of the item in its second slot, if it has one.
pub(super) fn extra_of(stack: &Stack) -> Option<Extra> {
    match stacking(stack)?.name {
        "minecraft:map" => Some(Extra::Map),
        "minecraft:paper" => Some(Extra::Paper),
        "minecraft:glass_pane" => Some(Extra::GlassPane),
        _ => None,
    }
}

/// `DataComponents.MAP_ID` of a stack.
pub fn map_id_of(stack: &Stack) -> Option<i32> {
    let values = crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    let mut raw = values.raw(ids::MAP_ID)?;
    read_varint(&mut raw).ok()
}

impl ContainerMenu {
    /// Whether paper cannot make the map larger: it is at its largest scale or, from 26.3 on,
    /// not an item that holds a map that can be extended.
    fn cannot_grow(&self, map: Option<&Stack>, facts: MapFacts) -> bool {
        facts.scale >= 4
            || (photon_core::versions::rules()
                .hud
                .cartography_scaling_needs_extendable_map
                && !map.is_some_and(|stack| self.in_tag(stack, "minecraft:extendable_maps")))
    }

    /// `CartographyTableScreen.extractBackground`'s error marks, and whether the map is locked
    /// against the picture for paper.
    fn error_marks(
        &self,
        extra: Option<Extra>,
        map: Option<&Stack>,
        facts: MapFacts,
    ) -> (u8, bool) {
        let (mut marks, mut locked) = (0, false);
        if facts.locked {
            locked = true;
            marks += u8::from(matches!(extra, Some(Extra::Paper | Extra::GlassPane)));
        }
        if extra == Some(Extra::Paper) && self.cannot_grow(map, facts) {
            locked = true;
            marks += 1;
        }
        (marks, locked)
    }

    /// `CartographyTableScreen.extractBackground` and `extractResultingMap`'s choices.
    pub fn cartography_view(&self, maps: &dyn Fn(i32) -> Option<MapFacts>) -> CartographyView {
        let extra = self
            .slots
            .get(1)
            .and_then(Option::as_ref)
            .and_then(extra_of);
        let map = self.slots.first().and_then(Option::as_ref);
        let id = map.and_then(map_id_of);
        let facts = id.and_then(maps);
        let (error_marks, locked) =
            facts.map_or((0, false), |facts| self.error_marks(extra, map, facts));
        let layout = match extra {
            Some(Extra::Paper) if !locked => CartographyLayout::Scaled,
            Some(Extra::Map) => CartographyLayout::Duplicated,
            Some(Extra::GlassPane) => CartographyLayout::Locking,
            _ => CartographyLayout::Plain,
        };
        CartographyView {
            layout,
            error_marks,
            map: id.filter(|_| facts.is_some()),
        }
    }
}
