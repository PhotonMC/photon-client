use super::*;

/// `block_declarations`: the declarations as the oracle describes them.
pub(super) fn block_declarations(label: &str) -> Value {
    use photon_data::block::classes::BlockClass;
    use photon_data::block::definition::{Emission, Fluid, Rule};
    use photon_data::block::properties::FIELDS;
    use photon_data::block::registry::{self, PropertyKind};
    let classes: Vec<Value> = BlockClass::ALL
        .iter()
        .map(|class| {
            let parent = class
                .parent()
                .map_or(String::new(), |parent| format!("{parent:?}"));
            let interfaces: Vec<String> = class
                .interfaces()
                .iter()
                .map(|interface| format!("{interface:?}"))
                .collect();
            let declares: Vec<String> = class
                .declares()
                .iter()
                .map(|method| {
                    let name = format!("{method:?}");
                    name[..1].to_lowercase() + &name[1..]
                })
                .collect();
            json!({
                "name": format!("{class:?}"),
                "parent": parent,
                "interfaces": interfaces,
                "declares": declares,
            })
        })
        .collect();
    let interfaces: Vec<Value> = photon_data::block::classes::BlockInterface::ALL
        .iter()
        .map(|interface| {
            let parents: Vec<String> = interface
                .parents()
                .iter()
                .map(|parent| format!("{parent:?}"))
                .collect();
            json!({"name": format!("{interface:?}"), "parents": parents})
        })
        .collect();
    let properties: Vec<Value> = FIELDS
        .iter()
        .map(|(field, property)| {
            json!({
                "field": field,
                "name": property.name,
                "kind": match property.kind {
                    PropertyKind::Boolean => "boolean",
                    PropertyKind::Integer => "integer",
                    PropertyKind::Enum => "enum",
                },
                "values": property.values,
            })
        })
        .collect();
    let field = |property: &registry::Property| {
        FIELDS
            .iter()
            .find(|(_, listed)| std::ptr::eq(*listed, property))
            .map(|(field, _)| *field)
            .expect("every property is listed")
    };
    let blocks: Vec<Value> = crate::case_ids::blocks()
        .map(|block| {
            let count = block.state_count();
            let mut per_state = serde_json::Map::new();
            if block.light_emission != Emission::Constant(0) {
                let levels: Vec<u8> = (0..count).map(|offset| block.light_emission.value(offset)).collect();
                per_state.insert("lightEmission".into(), json!(levels));
            }
            for (key, rule) in [
                ("emissiveRendering", block.emissive_rendering),
                ("isSuffocating", block.suffocating),
                ("isViewBlocking", block.view_blocking),
                ("isRedstoneConductor", block.redstone_conductor),
            ] {
                if rule != Rule::Default {
                    let values: Vec<bool> = (0..count)
                        .map(|offset| rule.value(offset).expect("not the default"))
                        .collect();
                    per_state.insert(key.into(), json!(values));
                }
            }
            let mut described = json!({
                "name": block.name,
                "class": format!("{:?}", block.class),
                "properties": block.properties.iter().map(|property| field(property)).collect::<Vec<_>>(),
                "default": block.default,
                "declared": declared(block),
                "per_state": per_state,
                "item": block.item_name(),
            });
            if block.class.is(BlockClass::LiquidBlock) {
                described["fluid"] = json!(match block.fluid {
                    Fluid::Water => "minecraft:water",
                    Fluid::FlowingWater => "minecraft:flowing_water",
                    Fluid::Lava => "minecraft:lava",
                    Fluid::FlowingLava => "minecraft:flowing_lava",
                    Fluid::Empty => "minecraft:empty",
                });
            }
            described
        })
        .collect();
    json!({
        "label": label,
        "outcome": "listed",
        "classes": classes,
        "interfaces": interfaces,
        "properties": properties,
        "blocks": blocks,
    })
}

/// The vanilla data pack's block tags, which is what the oracle's registry
/// holds.
pub(super) fn vanilla_tags() -> Result<photon_world::block::tags::BlockTags, String> {
    photon_world::block::tags::BlockTags::from_registries(
        &vanilla_block_tags()?.registries,
        &crate::case_ids::registries().blocks,
    )
    .ok_or_else(|| "the vanilla tags have no block registry".to_owned())
}

/// `face_occlusion`: the state cache's faces and skip rules, numbered and
/// grouped the way the oracle groups them.
pub(super) fn face_occlusion(label: &str) -> Result<Value, String> {
    use photon_data::block::registry;
    use photon_world::block::culling::skip_rendering;
    use photon_world::block::state::{DIRECTIONS, states};
    let tags = vanilla_tags()?;
    let states = states();
    let info = |block: &registry::Block, offset: u32| {
        states
            .get(block.state_at(offset))
            .expect("a registry state")
    };
    let mut tuples: Vec<[u16; 6]> = Vec::new();
    let mut blocks = Vec::new();
    for block in crate::case_ids::blocks() {
        let per_state: Vec<usize> = (0..block.state_count())
            .map(|offset| {
                let faces = info(block, offset).faces;
                tuples
                    .iter()
                    .position(|tuple| *tuple == faces)
                    .unwrap_or_else(|| {
                        tuples.push(faces);
                        tuples.len() - 1
                    })
            })
            .collect();
        let listed = if per_state.iter().all(|tuple| *tuple == per_state[0]) {
            vec![per_state[0]]
        } else {
            per_state
        };
        blocks.push(json!({"name": block.name, "tuples": listed}));
    }
    let count = states.faces().len();
    let matrix = |test: &dyn Fn(u16, u16) -> bool| -> Vec<String> {
        (0..count as u16)
            .map(|shape| {
                (0..count as u16)
                    .map(|occluder| if test(shape, occluder) { '1' } else { '0' })
                    .collect()
            })
            .collect()
    };
    let joins = matrix(&|shape, occluder| states.joins(shape, occluder));
    let covers = matrix(&|shape, occluder| states.covers(shape, occluder));
    let mut skips = Vec::new();
    for block in crate::case_ids::blocks() {
        let own = info(block, block.default_offset());
        for other in crate::case_ids::blocks() {
            let neighbour = info(other, other.default_offset());
            if !DIRECTIONS
                .iter()
                .any(|direction| skip_rendering(own, neighbour, *direction, &tags))
            {
                continue;
            }
            let mut all = true;
            let mut directions = std::collections::BTreeSet::new();
            for offset in 0..block.state_count() {
                for other_offset in 0..other.state_count() {
                    for direction in DIRECTIONS {
                        if skip_rendering(
                            info(block, offset),
                            info(other, other_offset),
                            direction,
                            &tags,
                        ) {
                            directions.insert(direction.name());
                        } else {
                            all = false;
                        }
                    }
                }
            }
            skips.push(json!({
                "block": block.name,
                "neighbour": other.name,
                "always": all,
                "directions": directions,
            }));
        }
    }
    let state_dependent = skips.iter().any(|pair| pair["always"] == json!(false));
    Ok(json!({
        "label": label,
        "outcome": "listed",
        "tuples": tuples,
        "blocks": blocks,
        "shape_count": count,
        "joins": joins,
        "covers": covers,
        "skip_pairs": skips,
        "skip_is_state_dependent": state_dependent,
    }))
}

pub(super) fn vanilla_tags_of(
    folder: &str,
    registry_name: &str,
    index_of: &dyn Fn(&str) -> Option<i32>,
) -> Result<ClientboundUpdateTagsPacket, String> {
    fn resolve(
        root: &std::path::Path,
        name: &str,
        seen: &mut BTreeMap<String, Vec<i32>>,
        index_of: &dyn Fn(&str) -> Option<i32>,
    ) -> Result<Vec<i32>, String> {
        if let Some(done) = seen.get(name) {
            return Ok(done.clone());
        }
        let path = root.join(format!("{name}.json"));
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let parsed: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for entry in parsed["values"].as_array().ok_or("tag values")? {
            let (id, required) = match entry {
                Value::String(id) => (id.as_str(), true),
                other => (
                    other["id"].as_str().ok_or("tag entry id")?,
                    other["required"].as_bool().unwrap_or(true),
                ),
            };
            if let Some(tag) = id.strip_prefix('#') {
                let path = tag.strip_prefix("minecraft:").ok_or("foreign tag")?;
                out.extend(resolve(root, path, seen, index_of)?);
            } else {
                match index_of(id) {
                    Some(index) => out.push(index),
                    None if !required => {}
                    None => return Err(format!("unknown entry {id} in tag {name}")),
                }
            }
        }
        seen.insert(name.to_owned(), out.clone());
        Ok(out)
    }
    fn walk(
        dir: &std::path::Path,
        root: &std::path::Path,
        names: &mut Vec<String>,
    ) -> Result<(), String> {
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path.is_dir() {
                walk(&path, root, names)?;
            } else if let Some(stem) = path.strip_prefix(root).ok().and_then(|p| p.to_str()) {
                names.push(stem.trim_end_matches(".json").replace('\\', "/"));
            }
        }
        Ok(())
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../.data/minecraft/assets/jar/data/minecraft/tags")
        .join(folder);
    let mut names = Vec::new();
    walk(&root, &root, &mut names)?;
    names.sort();
    let mut seen = BTreeMap::new();
    let mut tags = Vec::new();
    for name in names {
        let mut values = resolve(&root, &name, &mut seen, index_of)?;
        values.sort_unstable();
        values.dedup();
        let id = Identifier::parse(&format!("minecraft:{name}")).map_err(|_| "tag id")?;
        tags.push(TagEntry::new(id, values));
    }
    let registry = Identifier::parse(registry_name).map_err(|_| "registry id")?;
    Ok(ClientboundUpdateTagsPacket::new(vec![RegistryTags::new(
        registry, tags,
    )]))
}

/// The `Properties` a block declares, as the reference's own fields read back.
fn declared(block: &photon_data::block::definition::Block) -> Value {
    use photon_data::block::definition::PushReaction;
    json!({
        "destroyTime": bits(block.destroy_time),
        "explosionResistance": bits(block.explosion_resistance),
        "friction": bits(block.friction),
        "speedFactor": bits(block.speed_factor),
        "jumpFactor": bits(block.jump_factor),
        "bounceRestitution": bits(block.bounce_restitution),
        "hasCollision": block.has_collision,
        "requiresCorrectToolForDrops": block.requires_correct_tool_for_drops,
        "isRandomlyTicking": block.randomly_ticking,
        "canOcclude": block.can_occlude,
        "isAir": block.air,
        "ignitedByLava": block.ignited_by_lava,
        "liquid": block.liquid,
        "forceSolidOff": block.force_solid_off,
        "forceSolidOn": block.force_solid_on,
        "spawnTerrainParticles": block.spawn_terrain_particles,
        "replaceable": block.replaceable,
        "dynamicShape": block.dynamic_shape,
        "offsetType": match block.offset_type {
        photon_data::block::definition::OffsetType::None => "NONE",
        photon_data::block::definition::OffsetType::Xz => "XZ",
        photon_data::block::definition::OffsetType::Xyz => "XYZ",
    },
    "instrument": block.instrument,
    "pushReaction": match block.push_reaction {
            PushReaction::Normal => "NORMAL",
            PushReaction::Destroy => "DESTROY",
            PushReaction::Block => "BLOCK",
            PushReaction::Ignore => "IGNORE",
            PushReaction::PushOnly => "PUSH_ONLY",
        },
    })
}

/// A float as the hex of its bits.
pub(crate) fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}
