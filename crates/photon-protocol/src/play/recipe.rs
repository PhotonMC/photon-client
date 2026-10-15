use super::*;

/// `SlotDisplay.STREAM_CODEC`.
pub(super) fn read_slot_display(
    input: &mut &[u8],
    depth: usize,
) -> Result<photon_world::SlotDisplay, ConnectionError> {
    use photon_world::SlotDisplay as D;
    if depth > 16 {
        return Err(ConnectionError::Codec("slot display too deep"));
    }
    let next = |input: &mut &[u8]| read_slot_display(input, depth + 1);
    Ok(match read_varint(input, "slot display type")? {
        0 => D::Empty,
        1 => D::AnyFuel,
        2 => D::WithAnyPotion(Box::new(next(input)?)),
        3 => {
            let source = next(input)?;
            D::OnlyWithComponent(Box::new(source), read_varint(input, "display component")?)
        }
        4 => D::Item(read_varint(input, "display item")?),
        5 => {
            let item = read_varint(input, "display stack item")?;
            let count = read_varint(input, "display stack count")?;
            photon_game::item_components::read_patch(input)
                .map_err(|_| ConnectionError::Codec("display stack components"))?;
            D::ItemStack(item, count)
        }
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

/// `Ingredient.CONTENTS_STREAM_CODEC`: a tag, or the items it lists.
pub(super) fn read_ingredient(
    input: &mut &[u8],
) -> Result<photon_world::Ingredient, ConnectionError> {
    let marker = read_varint(input, "ingredient marker")?;
    if marker == 0 {
        return Ok(photon_world::Ingredient::Tag(read_identifier_string(
            input,
            "ingredient tag",
        )?));
    }
    let mut items = Vec::new();
    for _ in 0..marker - 1 {
        items.push(read_varint(input, "ingredient item")?);
    }
    Ok(photon_world::Ingredient::Items(items))
}

pub fn parse_update_recipes_packet(input: &[u8]) -> Result<RecipeUpdate, ConnectionError> {
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
        let ingredient = read_ingredient(&mut input)?;
        let display = read_slot_display(&mut input, 0)?;
        stonecutter.push(photon_world::StonecutterEntry {
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
) -> Result<Vec<photon_world::SlotDisplay>, ConnectionError> {
    let count = read_varint(input, "recipe display count")?;
    let mut all = Vec::new();
    for _ in 0..count {
        all.push(read_slot_display(input, 0)?);
    }
    Ok(all)
}

/// `RecipeDisplay.STREAM_CODEC`: the crafting station each one ends with is
/// read and dropped.
pub fn read_recipe_display(
    input: &mut &[u8],
) -> Result<photon_world::RecipeDisplay, ConnectionError> {
    use photon_world::RecipeDisplay as R;
    let slot = |input: &mut &[u8]| read_slot_display(input, 0);
    Ok(match read_varint(input, "recipe display type")? {
        0 => {
            let ingredients = read_slot_displays(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Shapeless {
                ingredients,
                result,
            }
        }
        1 => {
            let width = read_varint(input, "shaped width")?;
            let height = read_varint(input, "shaped height")?;
            let ingredients = read_slot_displays(input)?;
            if i64::from(width) * i64::from(height) != ingredients.len() as i64 {
                return Err(ConnectionError::Codec("shaped recipe size"));
            }
            let result = slot(input)?;
            slot(input)?;
            R::Shaped {
                width,
                height,
                ingredients,
                result,
            }
        }
        2 => {
            let ingredient = slot(input)?;
            let fuel = slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            read_varint(input, "furnace duration")?;
            read_f32(input, "furnace experience")?;
            R::Furnace {
                ingredient,
                fuel,
                result,
            }
        }
        3 => {
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Stonecutter { result }
        }
        4 => {
            slot(input)?;
            slot(input)?;
            slot(input)?;
            let result = slot(input)?;
            slot(input)?;
            R::Smithing { result }
        }
        _ => return Err(ConnectionError::Codec("unknown recipe display")),
    })
}

/// `ClientboundRecipeBookAddPacket`: the entries with their flag bytes, and
/// whether they replace the book.
pub fn parse_recipe_book_add_packet(
    input: &[u8],
) -> Result<(Vec<(photon_world::RecipeDisplayEntry, u8)>, bool), ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "recipe book add count")?;
    let mut entries = Vec::new();
    for _ in 0..count {
        let id = read_varint(&mut input, "recipe id")?;
        let display = read_recipe_display(&mut input)?;
        let group = match read_varint(&mut input, "recipe group")? {
            0 => None,
            value => Some(value - 1),
        };
        let category = read_varint(&mut input, "recipe category")?;
        let requirements = if read_byte(&mut input, "recipe requirements flag")? != 0 {
            let count = read_varint(&mut input, "recipe requirement count")?;
            let mut all = Vec::new();
            for _ in 0..count {
                all.push(read_ingredient(&mut input)?);
            }
            Some(all)
        } else {
            None
        };
        let flags = read_byte(&mut input, "recipe flags")?;
        entries.push((
            photon_world::RecipeDisplayEntry {
                id,
                display,
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
) -> Result<[photon_world::BookTypeSettings; 4], ConnectionError> {
    let mut input = input;
    let mut settings = [photon_world::BookTypeSettings::default(); 4];
    for entry in &mut settings {
        entry.open = read_byte(&mut input, "recipe book open")? != 0;
        entry.filtering = read_byte(&mut input, "recipe book filtering")? != 0;
    }
    Ok(settings)
}

/// `ClientboundPlaceGhostRecipePacket`.
pub fn parse_place_ghost_recipe_packet(
    input: &[u8],
) -> Result<(i32, photon_world::RecipeDisplay), ConnectionError> {
    let mut input = input;
    let container = read_varint(&mut input, "ghost container")?;
    Ok((container, read_recipe_display(&mut input)?))
}

/// What a clientbound update-recipes packet carries: the item sets by name
/// (the items a furnace, a blast furnace or a smoker accepts, and so on, as
/// registry ids) and the stonecutter's entries.
pub struct RecipeUpdate {
    pub sets: Vec<(String, Vec<i32>)>,
    pub stonecutter: Vec<photon_world::StonecutterEntry>,
}

wrapped_packet!(
    /// `ClientboundRecipeBookAddPacket`: the entries, and whether they replace the book's.
    RecipeBookAdd((Vec<(photon_world::RecipeDisplayEntry, u8)>, bool)) = parse_recipe_book_add_packet,
    OnError::Warn("recipe book add")
);
wrapped_packet!(
    /// `ClientboundRecipeBookRemovePacket`.
    RecipeBookRemove(Vec<i32>) = parse_recipe_book_remove_packet,
    OnError::Warn("recipe book remove")
);
wrapped_packet!(
    /// `ClientboundRecipeBookSettingsPacket`.
    RecipeBookSettings([photon_world::BookTypeSettings; 4]) = parse_recipe_book_settings_packet,
    OnError::Warn("recipe book settings")
);
wrapped_packet!(
    /// `ClientboundPlaceGhostRecipePacket`.
    PlaceGhostRecipe((i32, photon_world::RecipeDisplay)) = parse_place_ghost_recipe_packet,
    OnError::Warn("place ghost recipe")
);
wrapped_packet!(
    /// `ClientboundUpdateRecipesPacket`.
    UpdateRecipes(RecipeUpdate) = parse_update_recipes_packet,
    OnError::Warn("recipes")
);
