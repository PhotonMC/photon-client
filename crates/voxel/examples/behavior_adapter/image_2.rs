use super::*;

pub(super) fn image_texture_metadata_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let parsed =
            voxel_core::json::parse(string(row, "json")?)
                .ok()
                .and_then(|value| match value {
                    voxel_core::json::JsonValue::Object(object) => {
                        voxel_render::TextureMetadata::parse(&object)
                    }
                    _ => None,
                });
        match parsed {
            None => out.push(json!({"label": label, "outcome": "rejected"})),
            Some(metadata) => out.push(json!({
                "label": label,
                "outcome": "parsed",
                "blur": metadata.blur,
                "clamp": metadata.clamp,
                "strategy": metadata.mipmap_strategy.name(),
                "bias": format!("{:08x}", metadata.alpha_cutoff_bias.to_bits()),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_sprite_uv_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let width = i32_field(row, "width")?;
        let height = i32_field(row, "height")?;
        let placement = voxel_render::SpritePlacement {
            name: voxel_core::identifier::Identifier::parse("minecraft:block/parity")
                .map_err(|_| "sprite name")?,
            x: i32_field(row, "x")?,
            y: i32_field(row, "y")?,
            padding: i32_field(row, "padding")?,
        };
        let sprite = voxel_render::AtlasSprite::new(
            &placement,
            width,
            height,
            i32_field(row, "atlas_width")?,
            i32_field(row, "atlas_height")?,
        );
        let mut interpolated = Vec::new();
        for offset in row["offsets"]
            .as_array()
            .ok_or("offsets must be an array")?
        {
            let bits = u32::from_str_radix(offset.as_str().ok_or("offset must be a string")?, 16)
                .map_err(|error| format!("offset: {error}"))?;
            let offset = f32::from_bits(bits);
            interpolated.push(json!({
                "u": format!("{:08x}", sprite.u(offset).to_bits()),
                "v": format!("{:08x}", sprite.v(offset).to_bits()),
            }));
        }
        out.push(json!({
            "label": label,
            "outcome": "placed",
            "x": sprite.x,
            "y": sprite.y,
            "u0": format!("{:08x}", sprite.u0.to_bits()),
            "u1": format!("{:08x}", sprite.u1.to_bits()),
            "v0": format!("{:08x}", sprite.v0.to_bits()),
            "v1": format!("{:08x}", sprite.v1.to_bits()),
            "interpolated": interpolated,
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_colour_map_result(request: &Value) -> Result<Value, String> {
    let bytes = hex_bytes(string(request, "colour_map")?)?;
    let map = voxel_render::decode_rgba(&bytes).map_err(|_| "colour map did not decode")?;
    // The reference reads the map as packed colours, not as bytes.
    let pixels: Vec<u32> = map
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|texel| {
            (u32::from(texel[3]) << 24)
                | (u32::from(texel[0]) << 16)
                | (u32::from(texel[1]) << 8)
                | u32::from(texel[2])
        })
        .collect();
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let label = string(row, "label")?;
        let kind = match string(row, "kind")? {
            "grass" => voxel_render::ColourMap::Grass,
            "foliage" => voxel_render::ColourMap::Foliage,
            "dry_foliage" => voxel_render::ColourMap::DryFoliage,
            other => return Err(format!("unknown colour map kind: {other}")),
        };
        let read = |name: &str| -> Result<f64, String> {
            let bits = u64::from_str_radix(string(row, name)?, 16)
                .map_err(|error| format!("{name}: {error}"))?;
            Ok(f64::from_bits(bits))
        };
        match voxel_render::colour_map_lookup(kind, &pixels, read("temperature")?, read("rain")?) {
            Err(_) => out.push(json!({"label": label, "outcome": "out_of_bounds"})),
            Ok(colour) => out.push(json!({
                "label": label,
                "outcome": "resolved",
                "colour": format!("{colour:08x}"),
            })),
        }
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_argb_multiply_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let read = |name: &str| -> Result<u32, String> {
            u32::from_str_radix(string(row, name)?, 16).map_err(|error| format!("{name}: {error}"))
        };
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "multiplied",
            "colour": format!("{:08x}", voxel_render::multiply(read("lhs")?, read("rhs")?)),
        }));
    }
    Ok(json!({"rows": out}))
}

pub(super) fn image_tint_constants_result(request: &Value) -> Result<Value, String> {
    let rows = request["rows"]
        .as_array()
        .ok_or("image rows must be an array")?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(json!({
            "label": string(row, "label")?,
            "outcome": "listed",
            "values": {
                "foliage_default": format!("{:08x}", voxel_render::ColourMap::Foliage.default_colour()),
                "foliage_evergreen": format!("{:08x}", voxel_render::constant_tints("minecraft:spruce_leaves")[0]),
                "foliage_birch": format!("{:08x}", voxel_render::constant_tints("minecraft:birch_leaves")[0]),
                "foliage_mangrove": format!("{:08x}", voxel_render::FOLIAGE_MANGROVE),
                "dry_foliage_default": format!("{:08x}", voxel_render::ColourMap::DryFoliage.default_colour()),
                "lily_pad_in_hand": format!("{:08x}", voxel_render::constant_tints("minecraft:lily_pad")[0]),
                "lily_pad_in_world": format!("{:08x}", voxel_render::LILY_PAD_IN_WORLD),
            },
        }));
    }
    Ok(json!({"rows": out}))
}

/// `block_declarations`: the declarations as the oracle describes them.
pub(super) fn block_declarations(label: &str) -> Value {
    use voxel_data::block::classes::BlockClass;
    use voxel_data::block::definition::{Emission, Fluid, PushReaction, Rule};
    use voxel_data::block::properties::FIELDS;
    use voxel_data::block::registry::{self, PropertyKind};
    let bits = |value: f32| format!("{:08x}", value.to_bits());
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
    let interfaces: Vec<Value> = voxel_data::block::classes::BlockInterface::ALL
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
    let blocks: Vec<Value> = voxel_data::block::registry::BLOCKS
        .iter()
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
                "declared": {
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
                    voxel_data::block::definition::OffsetType::None => "NONE",
                    voxel_data::block::definition::OffsetType::Xz => "XZ",
                    voxel_data::block::definition::OffsetType::Xyz => "XYZ",
                },
                "pushReaction": match block.push_reaction {
                        PushReaction::Normal => "NORMAL",
                        PushReaction::Destroy => "DESTROY",
                        PushReaction::Block => "BLOCK",
                        PushReaction::Ignore => "IGNORE",
                        PushReaction::PushOnly => "PUSH_ONLY",
                    },
                },
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
pub(super) fn vanilla_tags() -> Result<voxel_world::block::tags::BlockTags, String> {
    voxel_world::block::tags::BlockTags::from_registries(&vanilla_block_tags()?.registries)
        .ok_or_else(|| "the vanilla tags have no block registry".to_owned())
}

/// `face_occlusion`: the state cache's faces and skip rules, numbered and
/// grouped the way the oracle groups them.
pub(super) fn face_occlusion(label: &str) -> Result<Value, String> {
    use voxel_data::block::registry;
    use voxel_world::block::culling::skip_rendering;
    use voxel_world::block::state::{DIRECTIONS, states};
    let tags = vanilla_tags()?;
    let states = states();
    let mut tuples: Vec<[u16; 6]> = Vec::new();
    let mut blocks = Vec::with_capacity(registry::BLOCKS.len());
    for block in &registry::BLOCKS {
        let first = block.first_state_id();
        let per_state: Vec<usize> = (0..block.state_count())
            .map(|offset| {
                let faces = states.get(first + offset).expect("a registry state").faces;
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
    let info = |block: &registry::Block, offset: u32| {
        states
            .get(block.first_state_id() + offset)
            .expect("a registry state")
    };
    let mut skips = Vec::new();
    for block in &registry::BLOCKS {
        let own = info(block, block.default_state_id() - block.first_state_id());
        for other in &registry::BLOCKS {
            let neighbour = info(other, other.default_state_id() - other.first_state_id());
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
