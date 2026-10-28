use super::*;

/// `SlotDisplay.STREAM_CODEC`.
pub(super) fn read_slot_display(
    input: &mut &[u8],
    depth: usize,
    registries: &VersionRegistries,
) -> Result<photon_core::recipes::SlotDisplay, ConnectionError> {
    use photon_core::recipes::SlotDisplay as D;
    if depth > 16 {
        return Err(ConnectionError::Codec("slot display too deep"));
    }
    let next = |input: &mut &[u8]| read_slot_display(input, depth + 1, registries);
    Ok(match read_varint(input, "slot display type")? {
        0 => D::Empty,
        1 => D::AnyFuel,
        2 => D::WithAnyPotion(Box::new(next(input)?)),
        3 => {
            let source = next(input)?;
            D::OnlyWithComponent(Box::new(source), read_varint(input, "display component")?)
        }
        4 => D::Item(WireItemId::read(input, "display item")?.canonical(registries)),
        5 => read_display_stack(input, registries)?,
        6 => D::Tag(read_identifier_string(input, "display tag")?),
        7 => {
            let dye = next(input)?;
            D::Dyed(Box::new(dye), Box::new(next(input)?))
        }
        8 => {
            let base = next(input)?;
            let material = next(input)?;
            // `Holder<TrimPattern>`: a registry id plus one, or zero and the
            // pattern written out (its asset, its name and whether it is a decal).
            let pattern = match read_varint(input, "display trim pattern")? {
                0 => {
                    read_identifier_string(input, "trim pattern asset")?;
                    photon_core::nbt::read_any_nbt(input)
                        .map_err(|_| ConnectionError::Codec("trim pattern name"))?;
                    read_bool(input, "trim pattern decal")?;
                    -1
                }
                id => id - 1,
            };
            D::SmithingTrim(Box::new(base), Box::new(material), pattern)
        }
        9 => {
            let source = next(input)?;
            D::WithRemainder(Box::new(source), Box::new(next(input)?))
        }
        10 => {
            let count = read_varint(input, "display count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(next(input)?);
            }
            D::Composite(all)
        }
        _ => return Err(ConnectionError::Codec("unknown slot display")),
    })
}

/// A slot display of an item stack: the item, the count and the components (which are skipped).
fn read_display_stack(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_core::recipes::SlotDisplay, ConnectionError> {
    let item = WireItemId::read(input, "display stack item")?.canonical(registries);
    let count = read_varint(input, "display stack count")?;
    photon_data::item_components::read_patch(input)
        .map_err(|_| ConnectionError::Codec("display stack components"))?;
    Ok(photon_core::recipes::SlotDisplay::ItemStack(item, count))
}

/// `Ingredient.CONTENTS_STREAM_CODEC`: a tag, or the items it lists.
pub(super) fn read_ingredient(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_core::recipes::Ingredient, ConnectionError> {
    let marker = read_varint(input, "ingredient marker")?;
    if marker == 0 {
        return Ok(photon_core::recipes::Ingredient::Tag(
            read_identifier_string(input, "ingredient tag")?,
        ));
    }
    let mut items = Vec::new();
    for _ in 0..marker - 1 {
        items.push(WireItemId::read(input, "ingredient item")?.canonical(registries));
    }
    Ok(photon_core::recipes::Ingredient::Items(items))
}

pub fn parse_update_recipes_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<RecipeUpdate, ConnectionError> {
    let mut input = input;
    let sets = read_varint(&mut input, "recipe property set count")?;
    let mut out = Vec::new();
    for _ in 0..sets {
        let name = read_identifier_string(&mut input, "recipe property set name")?;
        let count = read_varint(&mut input, "recipe property set size")?;
        let mut items = Vec::new();
        for _ in 0..count {
            items.push(read_varint(&mut input, "recipe property set item")?);
        }
        out.push((name, items));
    }
    let entries = read_varint(&mut input, "stonecutter recipe count")?;
    let mut stonecutter = Vec::new();
    for _ in 0..entries {
        let ingredient = read_ingredient(&mut input, registries)?;
        let display = read_slot_display(&mut input, 0, registries)?;
        stonecutter.push(photon_core::recipes::StonecutterEntry {
            input: ingredient,
            display,
        });
    }
    Ok(RecipeUpdate {
        sets: out,
        stonecutter,
    })
}

pub(super) fn read_slot_displays(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<Vec<photon_core::recipes::SlotDisplay>, ConnectionError> {
    let count = read_varint(input, "recipe display count")?;
    let mut all = Vec::new();
    for _ in 0..count {
        all.push(read_slot_display(input, 0, registries)?);
    }
    Ok(all)
}

type DisplayWithStation = (
    photon_core::recipes::RecipeDisplay,
    photon_core::recipes::SlotDisplay,
);

fn read_shaped_display(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<DisplayWithStation, ConnectionError> {
    let width = read_varint(input, "shaped width")?;
    let height = read_varint(input, "shaped height")?;
    let ingredients = read_slot_displays(input, registries)?;
    if i64::from(width) * i64::from(height) != ingredients.len() as i64 {
        return Err(ConnectionError::Codec("shaped recipe size"));
    }
    let result = read_slot_display(input, 0, registries)?;
    let station = read_slot_display(input, 0, registries)?;
    let shaped = photon_core::recipes::RecipeDisplay::Shaped {
        width,
        height,
        ingredients,
        result,
    };
    Ok((shaped, station))
}

fn read_furnace_display(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<DisplayWithStation, ConnectionError> {
    let ingredient = read_slot_display(input, 0, registries)?;
    let fuel = read_slot_display(input, 0, registries)?;
    let result = read_slot_display(input, 0, registries)?;
    let station = read_slot_display(input, 0, registries)?;
    read_varint(input, "furnace duration")?;
    read_f32(input, "furnace experience")?;
    let furnace = photon_core::recipes::RecipeDisplay::Furnace {
        ingredient,
        fuel,
        result,
    };
    Ok((furnace, station))
}

/// `RecipeDisplay.STREAM_CODEC`: the display and the crafting station each one ends with.
pub fn read_recipe_display(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<DisplayWithStation, ConnectionError> {
    use photon_core::recipes::RecipeDisplay as R;
    let slot = |input: &mut &[u8]| read_slot_display(input, 0, registries);
    Ok(match read_varint(input, "recipe display type")? {
        0 => {
            let ingredients = read_slot_displays(input, registries)?;
            let result = slot(input)?;
            let station = slot(input)?;
            (
                R::Shapeless {
                    ingredients,
                    result,
                },
                station,
            )
        }
        1 => read_shaped_display(input, registries)?,
        2 => read_furnace_display(input, registries)?,
        3 => {
            slot(input)?;
            let result = slot(input)?;
            let station = slot(input)?;
            (R::Stonecutter { result }, station)
        }
        4 => {
            slot(input)?;
            slot(input)?;
            slot(input)?;
            let result = slot(input)?;
            let station = slot(input)?;
            (R::Smithing { result }, station)
        }
        _ => return Err(ConnectionError::Codec("unknown recipe display")),
    })
}

/// `ClientboundRecipeBookAddPacket`: the entries with their flag bytes, and
/// whether they replace the book.
pub fn parse_recipe_book_add_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<(Vec<(photon_core::recipes::RecipeDisplayEntry, u8)>, bool), ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book add count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let id = read_varint(&mut input, "recipe id")?;
        let (display, station) = read_recipe_display(&mut input, registries)?;
        let group = match read_varint(&mut input, "recipe group")? {
            0 => None,
            value => Some(value - 1),
        };
        let category = read_varint(&mut input, "recipe category")?;
        let requirements = if read_byte(&mut input, "recipe requirements flag")? != 0 {
            let count = read_varint(&mut input, "recipe requirement count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(read_ingredient(&mut input, registries)?);
            }
            Some(all)
        } else {
            None
        };
        let flags = read_byte(&mut input, "recipe flags")?;
        entries.push((
            photon_core::recipes::RecipeDisplayEntry {
                id,
                display,
                station,
                group,
                category,
                requirements,
            },
            flags,
        ));
    }
    let replace = read_byte(&mut input, "recipe book replace")? != 0;
    Ok((entries, replace))
}

pub fn parse_recipe_book_remove_packet(input: &[u8]) -> Result<Vec<i32>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book remove count")?;
    let mut ids = Vec::new();
    for _ in 0..count {
        ids.push(read_varint(&mut input, "recipe book remove id")?);
    }
    Ok(ids)
}

/// `RecipeBookSettings.read`: open and filtering flags for crafting,
/// furnace, blast furnace and smoker, in that order.
pub fn parse_recipe_book_settings_packet(
    input: &[u8],
) -> Result<[photon_core::recipes::BookTypeSettings; 4], ConnectionError> {
    let mut input = input;
    let mut settings = [photon_core::recipes::BookTypeSettings::default(); 4];
    for entry in &mut settings {
        entry.open = read_byte(&mut input, "recipe book open")? != 0;
        entry.filtering = read_byte(&mut input, "recipe book filtering")? != 0;
    }
    Ok(settings)
}

/// `ClientboundPlaceGhostRecipePacket`.
pub fn parse_place_ghost_recipe_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<(i32, photon_core::recipes::RecipeDisplay), ConnectionError> {
    let mut input = input;
    let container = read_varint(&mut input, "ghost container")?;
    Ok((container, read_recipe_display(&mut input, registries)?.0))
}

/// What a clientbound update-recipes packet carries: the item sets by name
/// (the items a furnace, a blast furnace or a smoker accepts, and so on, as
/// registry ids) and the stonecutter's entries.
pub struct RecipeUpdate {
    pub sets: Vec<(String, Vec<i32>)>,
    pub stonecutter: Vec<photon_core::recipes::StonecutterEntry>,
}

wrapped_wire_packet!(
    /// `ClientboundRecipeBookAddPacket`: the entries, and whether they replace the book's.
    RecipeBookAdd((Vec<(photon_core::recipes::RecipeDisplayEntry, u8)>, bool)) = parse_recipe_book_add_packet,
    OnError::Warn("recipe book add")
);
wrapped_packet!(
    /// `ClientboundRecipeBookRemovePacket`.
    RecipeBookRemove(Vec<i32>) = parse_recipe_book_remove_packet,
    OnError::Warn("recipe book remove")
);
wrapped_packet!(
    /// `ClientboundRecipeBookSettingsPacket`.
    RecipeBookSettings([photon_core::recipes::BookTypeSettings; 4]) = parse_recipe_book_settings_packet,
    OnError::Warn("recipe book settings")
);
wrapped_wire_packet!(
    /// `ClientboundPlaceGhostRecipePacket`.
    PlaceGhostRecipe((i32, photon_core::recipes::RecipeDisplay)) = parse_place_ghost_recipe_packet,
    OnError::Warn("place ghost recipe")
);
wrapped_wire_packet!(
    /// `ClientboundUpdateRecipesPacket`.
    UpdateRecipes(RecipeUpdate) = parse_update_recipes_packet,
    OnError::Warn("recipes")
);
