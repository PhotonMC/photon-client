use super::*;

fn stack(name: &str, count: i32) -> Stack {
    let index = photon_data::item::registry::index(name).expect("an item");
    Stack::new(count, ItemId(i32::try_from(index).unwrap()))
}

/// A directory of the temp folder that goes again when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("photon-hotbar-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn a_new_manager_has_nine_empty_hotbars() {
    let dir = Scratch::new("new");
    let mut manager = HotbarManager::new(&dir.0);
    for id in 0..HOTBARS {
        assert!(manager.get(id).is_empty());
        assert_eq!(manager.get(id).load(), vec![None; SLOTS]);
    }
}

#[test]
fn a_saved_hotbar_is_read_back_by_a_new_manager() {
    let dir = Scratch::new("round");
    let mut inventory = vec![None; 36];
    inventory[0] = Some(stack("minecraft:stone", 64));
    inventory[3] = Some(stack("minecraft:diamond_sword", 1));
    HotbarManager::new(&dir.0).store(4, &inventory);

    let mut again = HotbarManager::new(&dir.0);
    assert!(!again.get(4).is_empty());
    assert!(again.get(0).is_empty());
    let loaded = again.get(4).load();
    assert_eq!(loaded[0], Some(stack("minecraft:stone", 64)));
    assert_eq!(loaded[3], Some(stack("minecraft:diamond_sword", 1)));
    assert_eq!(loaded[1], None);
}

#[test]
fn the_file_is_uncompressed_with_an_empty_root_name() {
    let dir = Scratch::new("layout");
    HotbarManager::new(&dir.0).store(0, &[]);
    let bytes = std::fs::read(dir.0.join("hotbar.nbt")).unwrap();
    // A compound tag, a name of length zero, then DataVersion first.
    assert_eq!(&bytes[..3], &[10, 0, 0]);
    let fields = parse_file(&bytes).unwrap();
    assert_eq!(
        field(&fields, "DataVersion"),
        Some(&FieldValue::Int(data_version()))
    );
    let mut keys: Vec<_> = fields.iter().map(|field| field.key.as_str()).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        ["0", "1", "2", "3", "4", "5", "6", "7", "8", "DataVersion"]
    );
}

#[test]
fn an_entry_is_the_item_and_its_count() {
    let entry = encode_entry(Some(&stack("minecraft:stone", 12)));
    assert_eq!(
        entry,
        FieldValue::Compound(vec![
            Field {
                key: "id".into(),
                value: FieldValue::Str("minecraft:stone".into())
            },
            Field {
                key: "count".into(),
                value: FieldValue::Int(12)
            },
        ])
    );
    assert_eq!(encode_entry(None), FieldValue::Compound(Vec::new()));
}

#[test]
fn a_missing_count_is_one_and_a_bad_entry_is_empty() {
    let named = |count: Option<FieldValue>| {
        let mut fields = vec![Field {
            key: "id".into(),
            value: FieldValue::Str("minecraft:stone".into()),
        }];
        fields.extend(count.map(|value| Field {
            key: "count".into(),
            value,
        }));
        FieldValue::Compound(fields)
    };
    assert_eq!(
        decode_entry(&named(None)),
        Some(stack("minecraft:stone", 1))
    );
    assert_eq!(decode_entry(&named(Some(FieldValue::Int(0)))), None);
    assert_eq!(decode_entry(&named(Some(FieldValue::Int(100)))), None);
    assert_eq!(decode_entry(&FieldValue::Compound(Vec::new())), None);
    let unknown = FieldValue::Compound(vec![Field {
        key: "id".into(),
        value: FieldValue::Str("minecraft:no_such_item".into()),
    }]);
    assert_eq!(decode_entry(&unknown), None);
}

#[test]
fn a_hotbar_of_the_wrong_size_is_empty() {
    let dir = Scratch::new("size");
    let short = FieldValue::List(vec![FieldValue::Compound(Vec::new()); 3]);
    let full = FieldValue::List(vec![
        encode_entry(Some(&stack("minecraft:stone", 2)));
        SLOTS
    ]);
    let fields = vec![
        Field {
            key: "0".into(),
            value: short,
        },
        Field {
            key: "1".into(),
            value: full,
        },
    ];
    let mut network = Vec::new();
    nbt::write_nbt(Some(&fields), &mut network);
    let mut file = vec![network[0], 0, 0];
    file.extend_from_slice(&network[1..]);
    std::fs::write(dir.0.join("hotbar.nbt"), file).unwrap();

    let mut manager = HotbarManager::new(&dir.0);
    assert!(manager.get(0).is_empty());
    assert!(!manager.get(1).is_empty());
    assert!(manager.get(2).is_empty());
}

#[test]
fn a_stack_with_the_wire_form_of_no_components_is_stored_whole() {
    // A stack read off the wire or taken from a creative tab carries the patch "0 added, 0 removed".
    let plain = stack("minecraft:stone", 3).with_patch_bytes(vec![0, 0]);
    assert_eq!(
        decode_entry(&encode_entry(Some(&plain))).map(|stack| stack.item_count()),
        Some(3)
    );
}

#[test]
fn the_placeholder_is_named_after_the_two_keys() {
    use crate::text::Component;
    use photon_data::item::component_types::ITEM_NAME;
    let stack = placeholder(Component::literal("C"), Component::literal("1"));
    let values = crate::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    let name = values.raw(ITEM_NAME).expect("the placeholder has a name");
    let tag = nbt::read_any_nbt(&mut &*name)
        .ok()
        .flatten()
        .expect("a tag");
    let read = Component::from_nbt(&tag).expect("a component");
    assert_eq!(
        read,
        Component::translatable_with(
            "inventory.hotbarInfo",
            vec![Component::literal("C"), Component::literal("1")]
        )
    );
}
