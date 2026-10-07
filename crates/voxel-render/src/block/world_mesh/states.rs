use super::*;

/// `WeightedList` draws: one variant from each list with the block's random
/// source, a lone variant written outside a list taking no draw.
pub(super) fn pick_variants(
    lists: &[Vec<BakedVariant>],
    random: &mut voxel_core::random::RandomSource,
    mut picked: impl FnMut(Option<u32>),
) {
    for list in lists {
        match list.as_slice() {
            [] => {}
            [only] if !only.listed => picked(only.model),
            _ => {
                let total: i32 = list.iter().map(|variant| variant.weight).sum();
                if let Ok(mut selection) = random.next_int_bound(total)
                    && let Some(variant) = list.iter().find(|variant| {
                        selection -= variant.weight;
                        selection < 0
                    })
                {
                    picked(variant.model);
                }
            }
        }
    }
}

/// Bake every variant the states name, each distinct model and turn once
/// and in parallel, and turn each state's lists into lists of baked models.
pub(super) fn bake_states(
    models: &BTreeMap<String, ResolvedModel>,
    atlas: &BuiltAtlas,
    pending: Vec<PendingState>,
    air: &[bool],
) -> (Vec<BakedModel>, Vec<StateRender>) {
    use rayon::prelude::*;
    let key = |variant: &crate::block::blockstate::Variant| BakeKey {
        model: variant.model.as_string(),
        rotation: (
            variant.x.degrees(),
            variant.y.degrees(),
            variant.z.degrees(),
        ),
        uv_lock: variant.uvlock,
    };
    // Each distinct model and turn, in the order the states first name it.
    let mut keys: Vec<BakeKey> = Vec::new();
    let mut slots: HashMap<BakeKey, usize> = HashMap::new();
    for (lists, _) in pending.iter().flatten() {
        for variant in lists.iter().flatten() {
            slots.entry(key(variant)).or_insert_with_key(|key| {
                keys.push(key.clone());
                keys.len() - 1
            });
        }
    }
    let results: Vec<Option<BakedModel>> = keys
        .par_iter()
        .map(|key| crate::block::bake::bake(models.get(&key.model)?, atlas, key))
        .collect();
    let mut baked = Vec::new();
    let indices: Vec<Option<u32>> = results
        .into_iter()
        .map(|model| {
            let model = model?;
            baked.push(model);
            u32::try_from(baked.len() - 1).ok()
        })
        .collect();
    let states = pending
        .into_iter()
        .zip(air)
        .map(|(drawn, is_air)| {
            if *is_air {
                return StateRender::Air;
            }
            let Some((lists, tints)) = drawn else {
                return StateRender::Missing;
            };
            let lists: Vec<Vec<BakedVariant>> = lists
                .iter()
                .map(|list| {
                    list.iter()
                        .map(|variant| BakedVariant {
                            model: indices[slots[&key(variant)]],
                            weight: variant.weight,
                            listed: variant.listed,
                        })
                        .collect()
                })
                .collect();
            let closes_in = lists.iter().flatten().all(|variant| {
                variant
                    .model
                    .is_none_or(|model| baked[model as usize].closes_in())
            });
            StateRender::Drawn(DrawnState {
                lists,
                closes_in,
                tints,
            })
        })
        .collect();
    (baked, states)
}

pub(super) fn is_air_block(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

/// Every weighted list a blockstate file holds: its variants, then its
/// multipart pieces.
pub(super) fn variant_lists(
    file: &crate::block::blockstate::BlockStateFile,
) -> impl Iterator<Item = &Vec<crate::block::blockstate::Variant>> {
    file.variants
        .iter()
        .flat_map(|variants| variants.values())
        .chain(
            file.multipart
                .iter()
                .flat_map(|parts| parts.iter().map(|part| &part.apply)),
        )
}

/// Every state's variant lists and colours, block by block in parallel, and
/// which states are air.
pub(super) fn state_lists(
    blockstates: &BTreeMap<String, crate::block::blockstate::BlockStateFile>,
) -> (Vec<PendingState>, Vec<bool>) {
    use rayon::prelude::*;
    let per_block: Vec<Vec<(PendingState, bool)>> = registry::BLOCKS
        .par_iter()
        .map(|block| {
            let is_air = is_air_block(block.name);
            (0..block.state_count())
                .map(|offset| {
                    let drawn = blockstates.get(block.name).filter(|_| !is_air).map(|file| {
                        let properties: BTreeMap<String, String> = block
                            .properties_at(offset)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|(key, value)| (key.to_owned(), value.to_owned()))
                            .collect();
                        (
                            file.matched_lists(&properties),
                            StateTints::of(block.name, &properties),
                        )
                    });
                    (drawn, is_air)
                })
                .collect()
        })
        .collect();
    per_block.into_iter().flatten().unzip()
}

/// What one block state draws, worked out once at load rather than for
/// every block.
pub(super) enum StateRender {
    Air,
    /// Not air, but nothing to draw it with.
    Missing,
    Drawn(DrawnState),
}

pub(super) struct DrawnState {
    /// The weighted lists its variants are picked from.
    pub(super) lists: Vec<Vec<BakedVariant>>,
    /// Whether every quad of every model it can draw hides behind a side,
    /// so a block closed in on all six sides draws nothing.
    pub(super) closes_in: bool,
    /// Where each tint slot's colour comes from.
    pub(super) tints: StateTints,
}

/// A variant as the random pick sees it, with its baked model.
pub(super) struct BakedVariant {
    /// Index into [`BlockRenderer::baked`]; `None` when the model could not
    /// be baked, which still takes its draw.
    pub(super) model: Option<u32>,
    pub(super) weight: i32,
    /// Whether it was written inside a list, and so drawn for.
    pub(super) listed: bool,
}

/// `BlockColors` for a state: fixed colours, or colours read from the
/// biome around the block.
pub(super) enum StateTints {
    Fixed(Vec<u32>),
    Biome {
        layers: &'static [Option<crate::block::biome_tint::TintKind>],
        constant: &'static [u32],
        /// A tall plant's upper half takes the colour of the block below.
        below: bool,
    },
}

/// A state's variant lists and colours, before they are baked.
pub(super) type PendingState = Option<(Vec<Vec<crate::block::blockstate::Variant>>, StateTints)>;

impl StateTints {
    /// The reference's colour providers for a block's state
    /// (`BlockColors.createDefault`).
    pub(super) fn of(block: &str, properties: &BTreeMap<String, String>) -> Self {
        // The lily pad has a world colour of its own.
        if block == "minecraft:lily_pad" {
            return Self::Fixed(vec![crate::block::tint::LILY_PAD_IN_WORLD]);
        }
        if let Some(tint) = crate::block::tint::state_tint(block, properties) {
            return Self::Fixed(vec![tint]);
        }
        let constant = registry::tints(block);
        match crate::block::biome_tint::tint_layers(block) {
            None => Self::Fixed(constant.to_vec()),
            Some(layers) => Self::Biome {
                layers,
                constant,
                below: properties.get("half").map(String::as_str) == Some("upper")
                    && matches!(block, "minecraft:tall_grass" | "minecraft:large_fern"),
            },
        }
    }

    /// Each slot's colour for the block at `position`, into `out`.
    pub(super) fn colours(
        &self,
        position: [i32; 3],
        biome_tint: &impl Fn(crate::block::biome_tint::TintKind, [i32; 3]) -> u32,
        out: &mut Vec<u32>,
    ) {
        out.clear();
        match self {
            Self::Fixed(colours) => out.extend_from_slice(colours),
            Self::Biome {
                layers,
                constant,
                below,
            } => {
                let sample = if *below {
                    [position[0], position[1] - 1, position[2]]
                } else {
                    position
                };
                out.extend(layers.iter().enumerate().map(|(slot, layer)| match layer {
                    Some(kind) => biome_tint(*kind, sample),
                    None => constant.get(slot).copied().unwrap_or(0xFFFF_FFFF),
                }));
            }
        }
    }
}
