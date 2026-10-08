//! The item component codec lives with the world's data; what reads the
//! creative tabs' stacks whole needs the game's registries, so it is here.

use crate::item_components;

#[test]
fn every_creative_stack_reads_whole() {
    let registries = crate::registries::for_tests();
    for tab in photon_data::item::creative_tabs::CREATIVE_TABS.iter() {
        for entry in tab.items {
            let stack = crate::menus::creative::table_stack(entry, &registries);
            // An empty patch is kept as no bytes.
            let empty = [0, 0];
            let mut input = if stack.patch_bytes().is_empty() {
                &empty[..]
            } else {
                stack.patch_bytes()
            };
            let patch =
                item_components::read_patch(&mut input).expect("a patch this port wrote reads");
            assert!(input.is_empty());
            let ids: Vec<i32> = patch.added.iter().map(|(id, _)| *id).collect();
            assert_eq!(ids, stack.positive_component_type_ids());
            assert!(patch.removed.is_empty());
            if !stack.patch_bytes().is_empty() {
                assert_eq!(patch.trusted(), stack.patch_bytes());
            }
        }
    }
}
