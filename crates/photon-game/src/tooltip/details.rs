//! Tooltip lines of the items and components that read more than their own
//! value: smithing templates, maps (with the client's map data), container
//! contents, written books, and the advanced (`F3+H`) lines.

use super::*;
use crate::item_components::{Cursor, varint_value};

/// What the client holds of a map, as a tooltip reads it (`MapItemSavedData`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapInfo {
    pub scale: i8,
    pub locked: bool,
}

/// `Context.mapData`: the map data the client holds for a map id.
pub type Maps<'a> = &'a dyn Fn(i32) -> Option<MapInfo>;

/// `Item.Context`: the map data the client holds and the tick rate effect times are read at.
#[derive(Clone, Copy)]
pub struct Context<'a> {
    pub maps: Maps<'a>,
    pub tick_rate: f32,
}

impl Context<'static> {
    /// `Context.EMPTY`, and `of(registries)` as the creative search makes it: no map data,
    /// and twenty ticks a second.
    pub const EMPTY: Self = Self {
        maps: &no_maps,
        tick_rate: 20.0,
    };
}

fn no_maps(_: i32) -> Option<MapInfo> {
    None
}

/// `SmithingTemplateItem.appendHoverText`: the netherite upgrade and every
/// armour trim template (`createNetheriteUpgradeTemplate`,
/// `createArmorTrimTemplate`).
pub(super) fn smithing_template_lines(item: &str, out: &mut Vec<Component>) {
    let kind = if item == "minecraft:netherite_upgrade_smithing_template" {
        "netherite_upgrade"
    } else if item.ends_with("_armor_trim_smithing_template") {
        "armor_trim"
    } else {
        return;
    };
    let gray = |key: &str| Component::translatable(key).with_colour("gray");
    let described = |part: &str| {
        Component::literal(" ").append(
            Component::translatable(format!("item.minecraft.smithing_template.{kind}.{part}"))
                .with_colour("blue"),
        )
    };
    out.push(gray("item.minecraft.smithing_template"));
    out.push(Component::empty());
    out.push(gray("item.minecraft.smithing_template.applies_to"));
    out.push(described("applies_to"));
    out.push(gray("item.minecraft.smithing_template.ingredients"));
    out.push(described("ingredients"));
}

/// `MapId.addToTooltip`: an unknown map, else its id unless named or being
/// post-processed, whether it is locked, and (advanced) its scale.
pub(super) fn map_lines(values: &Values, flags: Flags, maps: Maps, out: &mut Vec<Component>) {
    let Some(id) = values.raw(ids::MAP_ID).and_then(varint_value) else {
        return;
    };
    let Some(map) = maps(id) else {
        out.push(Component::translatable("filled_map.unknown").with_colour("gray"));
        return;
    };
    // `MapPostProcessing`: 0 locks, 1 scales.
    let post = values.raw(ids::MAP_POST_PROCESSING).and_then(varint_value);
    if !values.has(ids::CUSTOM_NAME) && post.is_none() {
        out.push(
            Component::translatable_with("filled_map.id", vec![Component::literal(id.to_string())])
                .with_colour("gray"),
        );
    }
    if map.locked || post == Some(0) {
        out.push(Component::translatable("filled_map.locked").with_colour("gray"));
    }
    if flags.advanced {
        let scale = (i32::from(map.scale) + i32::from(post == Some(1))).min(4);
        out.push(
            Component::translatable_with(
                "filled_map.scale",
                vec![Component::literal((1 << scale).to_string())],
            )
            .with_colour("gray"),
        );
        out.push(
            Component::translatable_with(
                "filled_map.level",
                vec![
                    Component::literal(scale.to_string()),
                    Component::literal("4"),
                ],
            )
            .with_colour("gray"),
        );
    }
}

/// `ItemContainerContents.addToTooltip`: the first five stacks by name and
/// count, then how many more there are.
pub(super) fn container_lines(values: &Values, registries: &Registries, out: &mut Vec<Component>) {
    let Some(mut input) = values.raw(ids::CONTAINER) else {
        return;
    };
    let Ok(slots) = photon_core::wire::read_varint(&mut input) else {
        return;
    };
    let (mut shown, mut items) = (0, 0);
    for _ in 0..slots {
        let Some((&present, rest)) = input.split_first() else {
            break;
        };
        input = rest;
        if present == 0 {
            continue;
        }
        let Some(stack) = crate::item::read_template(&mut input) else {
            break;
        };
        items += 1;
        if shown <= 4 {
            shown += 1;
            let inner =
                Values::with_registries(stack.item_registry_id(), stack.patch_bytes(), registries);
            out.push(Component::translatable_with(
                "item.container.item_count",
                vec![
                    hover_name(&stack, &inner),
                    Component::literal(stack.item_count().to_string()),
                ],
            ));
        }
    }
    if items > shown {
        out.push(
            Component::translatable_with(
                "item.container.more_items",
                vec![Component::literal((items - shown).to_string())],
            )
            .italic(),
        );
    }
}

/// A written book's raw title, author and generation (`WrittenBookContent`).
fn book_header(values: &Values) -> Option<(String, String, i32)> {
    let mut cursor = Cursor::new(values.raw(ids::WRITTEN_BOOK_CONTENT)?);
    let string = |cursor: &mut Cursor| -> Option<String> {
        let length = cursor.count().ok()?;
        let text = String::from_utf8_lossy(cursor.data.get(..length)?).into_owned();
        cursor.skip(length).ok()?;
        Some(text)
    };
    let title = string(&mut cursor)?;
    // The title's filtered form, if it has one.
    cursor.optional(Cursor::string).ok()?;
    let author = string(&mut cursor)?;
    let generation = cursor.varint().ok()?;
    Some((title, author, generation))
}

/// `ItemStack.getCustomName`'s second choice: a written book's title unless
/// it is blank.
pub(super) fn book_title(values: &Values) -> Option<Component> {
    book_header(values)
        .map(|(title, _, _)| title)
        .filter(|title| !is_blank(title))
        .map(Component::literal)
}

/// `WrittenBookContent.addToTooltip`: the author unless blank, then the
/// generation.
pub(super) fn written_book_lines(values: &Values, out: &mut Vec<Component>) {
    let Some((_, author, generation)) = book_header(values) else {
        return;
    };
    if !is_blank(&author) {
        out.push(
            Component::translatable_with("book.byAuthor", vec![Component::literal(author)])
                .with_colour("gray"),
        );
    }
    out.push(Component::translatable(format!("book.generation.{generation}")).with_colour("gray"));
}

/// `StringUtil.isBlank`.
fn is_blank(text: &str) -> bool {
    text.chars().all(is_java_whitespace)
}

/// `StringUtil.isWhitespace`: `Character.isWhitespace` or `isSpaceChar`,
/// which together are Rust's white space less U+0085 and plus the four
/// separator controls.
fn is_java_whitespace(character: char) -> bool {
    (character.is_whitespace() && character != '\u{85}')
        || ('\u{1c}'..='\u{1f}').contains(&character)
}

/// The advanced (`F3+H`) lines: durability when damaged, the item's id, and
/// how many components the stack has.
pub(super) fn advanced_lines(
    stack: &Stack,
    values: &Values,
    shows: impl Fn(i32) -> bool,
    out: &mut Vec<Component>,
) {
    if crate::menus::container::is_damaged(stack) && shows(ids::DAMAGE) {
        let max = crate::menus::container::max_damage(stack);
        out.push(Component::translatable_with(
            "item.durability",
            vec![
                Component::literal((max - crate::menus::container::damage(stack)).to_string()),
                Component::literal(max.to_string()),
            ],
        ));
    }
    out.push(Component::literal(item_name_of(stack)).with_colour("dark_gray"));
    let own = usize::try_from(stack.item_registry_id().0)
        .ok()
        .and_then(|id| photon_data::item::component_sets::DEFAULT_COMPONENT_SETS.get(id))
        .copied()
        .unwrap_or(0);
    let count = values.count(own);
    if count > 0 {
        out.push(
            Component::translatable_with(
                "item.components",
                vec![Component::literal(count.to_string())],
            )
            .with_colour("dark_gray"),
        );
    }
}
