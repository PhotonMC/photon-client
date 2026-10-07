#[test]
fn recipe_book_add_reads_a_shaped_entry_and_its_flags() {
    // One entry: id 7, a 2x1 shaped display (an item, an empty slot), the
    // result item 20, an empty station, no group, category 3, one
    // requirement of item 10, the highlight flag, replacing the book.
    let payload = [
        1, 7, 1, 2, 1, 2, 4, 10, 0, 4, 20, 0, 0, 3, 1, 1, 2, 10, 2, 1,
    ];
    let (entries, replace) = super::super::parse_recipe_book_add_packet(&payload).unwrap();
    assert!(replace);
    let (entry, flags) = &entries[0];
    assert_eq!(
        (entry.id, entry.group, entry.category, *flags),
        (7, None, 3, 2)
    );
    assert_eq!(
        entry.display,
        voxel_world::RecipeDisplay::Shaped {
            width: 2,
            height: 1,
            ingredients: vec![
                voxel_world::SlotDisplay::Item(10),
                voxel_world::SlotDisplay::Empty
            ],
            result: voxel_world::SlotDisplay::Item(20),
        }
    );
    assert_eq!(
        entry.requirements,
        Some(vec![voxel_world::Ingredient::Items(vec![10])])
    );
    let mut book = voxel_world::RecipeBook::default();
    book.add(entries, replace);
    assert!(book.highlight.contains(&7));
}
