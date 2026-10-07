//! An item's tooltip: `ItemStack.getTooltipLines`, as the components a stack
//! carries and the item's defaults give it.
//!
//! The name, then what each component says in the reference's order.
//! Ported: the name (custom, item, book title, potion, tipped arrow, shield,
//! player head, compass), painting variants, disc fragments, instruments,
//! fireworks and stars, potion and ominous bottle effects, jukebox songs,
//! armour trims, enchantments, dye, lore, attribute modifiers, unbreakable,
//! banner patterns, suspicious stew and beehive honey. Not ported: smithing
//! templates, map, container, book and bee lines, the adventure predicates
//! and the advanced lines (the `F3+H` toggle is off).

use crate::item_components::ids;
use crate::item_values::{AttributeDisplay, Explosion, PaintingVariant, Values};
use crate::registries::Registries;
use crate::text::{Component, Lang, Style};
use voxel_core::json::JsonValue;
use voxel_data::item::attributes::ATTRIBUTES;
use voxel_data::item::effect::{attribute_id, effect_id};
use voxel_data::item::mob_effects::MOB_EFFECTS;
use voxel_data::item::potions::POTIONS;
use voxel_world::ClientItemStackState as Stack;

/// `TooltipFlag`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    /// `isCreative`: the creative inventory's hovered slot.
    pub creative: bool,
}

/// The two base attributes a player's attribute modifiers are added to.
const PLAYER_ATTACK_DAMAGE: f64 = 1.0;
const PLAYER_ATTACK_SPEED: f64 = 4.0;

/// `DyeColor.fireworkColor` by id.
const FIREWORK_COLOURS: [i32; 16] = [
    15_790_320, 15_435_844, 12_801_229, 6_719_955, 14_602_026, 4_312_372, 14_188_952, 4_408_131,
    11_250_603, 2_651_799, 8_073_150, 2_437_522, 5_320_730, 3_887_386, 11_743_532, 1_973_019,
];

const SHAPES: [&str; 5] = ["small_ball", "large_ball", "star", "creeper", "burst"];

#[cfg(test)]
mod tests {
    use super::*;

    fn registries() -> Registries {
        crate::registries::for_tests()
    }

    fn lang() -> Lang {
        Lang::load(std::path::Path::new(
            &std::env::var("VOXEL_ASSETS").unwrap_or_default(),
        ))
    }

    #[test]
    fn decimals_are_cut_to_two_places() {
        assert_eq!(format_decimal(7.0), "7");
        assert_eq!(format_decimal(1.6), "1.6");
        assert_eq!(format_decimal(-0.4), "-0.4");
        assert_eq!(format_decimal(0.125), "0.12");
        assert_eq!(format_decimal(-0.0), "0");
    }

    #[test]
    fn ticks_read_as_minutes_and_seconds() {
        assert_eq!(format_ticks(3600), "03:00");
        assert_eq!(format_ticks(20), "00:01");
        assert_eq!(format_ticks(20 * 3600), "01:00:00");
    }

    #[test]
    fn a_plain_item_has_only_its_name() {
        let stone = crate::item::id_of("minecraft:stone").expect("an item");
        let lines = lines(
            &Stack::new(1, stone),
            false,
            Flags::default(),
            &registries(),
        );
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn every_creative_stack_has_a_name_and_no_panic() {
        let lang = lang();
        if lang.get("item.minecraft.stone").is_none() {
            return;
        }
        let registries = registries();
        for tab in crate::creative_menu::shown_tabs() {
            for entry in voxel_data::item::creative_tabs::CREATIVE_TABS[tab].items {
                let stack = crate::creative_menu::table_stack(entry, &registries);
                let text = search_lines(&stack, &lang, &registries);
                assert!(!text.is_empty(), "{}", item_name_of(&stack));
            }
        }
    }
}

mod name;
pub use name::*;
mod lines;
pub use lines::*;
