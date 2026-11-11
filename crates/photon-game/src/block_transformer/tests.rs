use super::*;
use photon_core::json::parse;
use photon_core::nbt::{Field, FieldValue};

const TILL: &str = r#"[{"block_state_provider":{"type":"minecraft:rule_based","rules":[
    {"if_true":{"type":"minecraft:all_of","predicates":[
        {"type":"minecraft:matching_blocks","blocks":"minecraft:air"},
        {"type":"minecraft:matching_blocks","offset":[0,1,0],"blocks":"minecraft:air"}]},
     "then":{"id":"minecraft:farmland","properties":{"moisture":"0"}}}]},
  "disallowed_faces":["down"],"sound":"minecraft:item.hoe.till","particle":"scrape",
  "update_from_neighbors":false,"item_damage_per_use":1}]"#;

fn transforms(text: &str) -> Vec<Transform> {
    from_json(&parse(text).expect("json")).expect("a transformer")
}

fn field(key: &str, value: FieldValue) -> Field {
    Field {
        key: key.into(),
        value,
    }
}

fn text(value: &str) -> FieldValue {
    FieldValue::Str(value.into())
}

#[test]
fn an_entry_reads_with_its_defaults() {
    let [transform] = transforms(TILL).try_into().expect("one transform");
    assert_eq!(transform.disallowed_faces, vec![0]);
    assert_eq!(transform.sound, Some("minecraft:item.hoe.till"));
    assert_eq!(transform.particle.level_event(), Some(3005));
    assert!(!transform.update_from_neighbors);
    assert!(transform.consume_on_use);
    assert!(!transform.copper_chest);
    let plain = transforms(r#"[{"block_state_provider":{"id":"minecraft:dirt"}}]"#);
    assert_eq!(plain[0].sound, None);
    assert_eq!(plain[0].particle, Particle::None);
    assert!(plain[0].update_from_neighbors);
}

#[test]
fn the_nbt_a_server_sends_reads_as_the_json_does() {
    let provider = FieldValue::Compound(vec![field("id", text("minecraft:dirt"))]);
    let entry = FieldValue::Compound(vec![
        field("block_state_provider", provider),
        field("sound", text("minecraft:item.hoe.till")),
        field("update_from_neighbors", FieldValue::Byte(0)),
        field("transform_type", text("copper_chest")),
        field("disallowed_faces", FieldValue::List(vec![text("up")])),
    ]);
    let read = from_nbt(&FieldValue::List(vec![entry])).expect("an entry");
    let expected = transforms(
        r#"[{"block_state_provider":{"id":"minecraft:dirt"},"sound":"minecraft:item.hoe.till",
            "update_from_neighbors":false,"transform_type":"copper_chest",
            "disallowed_faces":["up"]}]"#,
    );
    assert_eq!(read, expected);
    assert!(read[0].copper_chest && !read[0].update_from_neighbors);
}

#[test]
fn an_entry_that_does_not_read_is_none() {
    for bad in [
        "[]",
        r#"[{"sound":"minecraft:item.hoe.till"}]"#,
        r#"[{"block_state_provider":{"id":"minecraft:nothing"}}]"#,
        r#"[{"block_state_provider":{"id":"minecraft:dirt"},"particle":"smoke"}]"#,
        r#"[{"block_state_provider":{"id":"minecraft:dirt"},"disallowed_faces":["sideways"]}]"#,
        r#"{"block_state_provider":{"id":"minecraft:dirt"}}"#,
    ] {
        assert!(from_json(&parse(bad).expect("json")).is_none(), "{bad}");
    }
}

#[test]
fn a_provider_offers_a_state_where_its_rule_holds() {
    let world = ClientWorld::default();
    let [transform] = transforms(TILL).try_into().expect("one transform");
    // An unloaded level reads as air, and so does the block above it.
    let offered = transform.provider.optional_state(&world, (0, 64, 0));
    let state = offered.expect("known").expect("a state");
    assert_eq!(block_name(state), Some("minecraft:farmland"));
    let none = transforms(
        r#"[{"block_state_provider":{"type":"minecraft:rule_based","rules":[
            {"if_true":{"type":"minecraft:matching_blocks","blocks":["minecraft:dirt","minecraft:stone"]},
             "then":{"id":"minecraft:dirt"}}]}}]"#,
    );
    assert_eq!(
        none[0].provider.optional_state(&world, (0, 64, 0)),
        Ok(None)
    );
}

#[test]
fn a_kind_the_client_does_not_decide_is_unported_only_when_reached() {
    let world = ClientWorld::default();
    let reached = transforms(
        r#"[{"block_state_provider":{"type":"minecraft:rule_based","rules":[
            {"if_true":{"type":"minecraft:solid"},"then":{"id":"minecraft:dirt"}}]}}]"#,
    );
    assert_eq!(
        reached[0].provider.optional_state(&world, (0, 0, 0)),
        Err(Unported)
    );
    let skipped = transforms(
        r#"[{"block_state_provider":{"type":"minecraft:rule_based","rules":[
            {"if_true":{"type":"minecraft:all_of","predicates":[
                {"type":"minecraft:matching_blocks","blocks":"minecraft:stone"},
                {"type":"minecraft:solid"}]},"then":{"id":"minecraft:dirt"}}]}}]"#,
    );
    assert_eq!(
        skipped[0].provider.optional_state(&world, (0, 0, 0)),
        Ok(None)
    );
}

#[test]
fn copied_properties_come_from_the_block_that_is_replaced() {
    use photon_world::block::named_state::state_named;
    use photon_world::block::rules::property;
    let copy = transforms(
        r#"[{"block_state_provider":{"type":"minecraft:copy_properties",
            "source":{"id":"minecraft:stripped_oak_log"}}}]"#,
    );
    let Provider::CopyProperties(_) = &copy[0].provider else {
        panic!("a copy");
    };
    let source = state_named("minecraft:stripped_oak_log", &[]).expect("a state");
    let log = state_named("minecraft:oak_log", &[("axis".into(), "z".into())]).expect("a state");
    let copied = with_properties_of(source, log);
    assert_eq!(property(copied, "axis"), Some("z"));
}

/// The vanilla entries are numbered alphabetically (`axe`, `hoe`, `shovel`) in the registry the
/// server syncs, and each tool's default patch, as the reference writes it, names its own.
#[test]
fn the_default_patches_name_the_entries_by_their_place() {
    const PATCHES: &str =
        include_str!("../../../../tests/behavior/fixtures/item-default-patches.txt");
    let registries = photon_data::version_registries::VersionRegistries::of(
        photon_core::version::Version::V26_3,
    );
    let entry = |name: &str| Transformer {
        name: format!("minecraft:{name}"),
        transforms: None,
    };
    let loaded = crate::registries::Registries {
        block_transformers: vec![entry("axe"), entry("hoe"), entry("shovel")],
        ..Default::default()
    };
    let mut seen = 0;
    for line in PATCHES.lines() {
        let (item, wire) = line.split_once(' ').expect("a name and its bytes");
        if !["axe", "hoe", "shovel"]
            .into_iter()
            .any(|kind| item.ends_with(&format!("_{kind}")))
        {
            continue;
        }
        let bytes: Vec<u8> = (0..wire.len())
            .step_by(2)
            .map(|at| u8::from_str_radix(&wire[at..at + 2], 16).expect("hex"))
            .collect();
        let patch =
            photon_data::item_components::read_wire_patch(&mut bytes.as_slice(), &registries)
                .expect("a patch");
        let written = patch
            .added
            .iter()
            .find(|(id, _)| *id == photon_data::item::component_types::BLOCK_TRANSFORMER)
            .map(|(_, value)| value.clone());
        let declared = photon_data::item::registry::ITEMS
            .iter()
            .find(|candidate| candidate.name == item)
            .and_then(|candidate| {
                candidate.components.iter().find_map(|component| {
                    crate::item::default_components::encode(component, Some(&loaded))
                        .filter(|(id, _)| {
                            *id == photon_data::item::component_types::BLOCK_TRANSFORMER
                        })
                        .map(|(_, value)| value)
                })
            });
        assert!(written.is_some(), "{item}");
        assert_eq!(declared, written, "{item}");
        seen += 1;
    }
    assert_eq!(seen, 21);
}
