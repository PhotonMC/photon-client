use super::*;

impl GenericScreen {
    /// `EnchantmentScreen.extractBackground`'s three rows, and the tooltip of
    /// the hovered one (`extractRenderState`).
    pub(super) fn enchant_draw(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        const WORDS: [&str; 62] = [
            "the",
            "elder",
            "scrolls",
            "klaatu",
            "berata",
            "niktu",
            "xyzzy",
            "bless",
            "curse",
            "light",
            "darkness",
            "fire",
            "air",
            "earth",
            "water",
            "hot",
            "dry",
            "cold",
            "wet",
            "ignite",
            "snuff",
            "embiggen",
            "twist",
            "shorten",
            "stretch",
            "fiddle",
            "destroy",
            "imbue",
            "galvanize",
            "enchant",
            "free",
            "limited",
            "range",
            "of",
            "towards",
            "inside",
            "sphere",
            "cube",
            "self",
            "other",
            "ball",
            "mental",
            "physical",
            "grow",
            "shrink",
            "demon",
            "elemental",
            "spirit",
            "animal",
            "creature",
            "beast",
            "humanoid",
            "undead",
            "fresh",
            "stale",
            "phnglui",
            "mglwnafh",
            "cthulhu",
            "rlyeh",
            "wgahnagl",
            "fhtagn",
            "baguette",
        ];
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        // `EnchantmentNames.initSeed`: the same names each frame.
        let mut random = photon_core::random::RandomSource::create(i64::from(data(3)));
        let mut tip = None;
        for row in 0..3_usize {
            let x = left + 60;
            let y = top + 14 + 19 * row as i32;
            let cost = data(row);
            if cost == 0 {
                draw.frame.sprite(
                    draw.atlas,
                    "container/enchanting_table/enchantment_slot_disabled",
                    x,
                    y,
                    108,
                    19,
                );
                continue;
            }
            let cost_text = cost.to_string();
            let text_width = 86 - draw.font.width(&cost_text);
            // `EnchantmentNames.getRandomName`, cut to the width.
            let count = random.next_int_bound(2).unwrap_or(0) + 3;
            let name = (0..count)
                .map(|_| WORDS[random.next_int_bound(WORDS.len() as i32).unwrap_or(0) as usize])
                .collect::<Vec<_>>()
                .join(" ");
            let alt = self.titles.alt_font.clone();
            let mut shown = String::new();
            for character in name.chars() {
                shown.push(character);
                if alt.width(&shown) > text_width {
                    shown.pop();
                    break;
                }
            }
            let level = format!("container/enchanting_table/level_{}", row + 1);
            let affordable = Self::enchant_can_afford(menu, row);
            let (name_colour, cost_colour) = if affordable {
                let (xx, yy) = (mouse.0 - f64::from(x), mouse.1 - f64::from(y));
                let over = (0.0..108.0).contains(&xx) && (0.0..19.0).contains(&yy);
                let (sprite, colour) = if over {
                    ("enchantment_slot_highlighted", 0xFFFF_FF80)
                } else {
                    ("enchantment_slot", 0xFF68_5E4A)
                };
                draw.frame.sprite(
                    draw.atlas,
                    &format!("container/enchanting_table/{sprite}"),
                    x,
                    y,
                    108,
                    19,
                );
                draw.frame.sprite(draw.atlas, &level, x + 1, y + 1, 16, 16);
                (colour, 0xFF80_FF20)
            } else {
                draw.frame.sprite(
                    draw.atlas,
                    "container/enchanting_table/enchantment_slot_disabled",
                    x,
                    y,
                    108,
                    19,
                );
                draw.frame.sprite(
                    draw.atlas,
                    &format!("{level}_disabled"),
                    x + 1,
                    y + 1,
                    16,
                    16,
                );
                (0xFF34_2F25, 0xFF40_7F10)
            };
            draw.frame
                .text(draw.atlas, &alt, &shown, x + 20, y + 2, name_colour);
            draw.frame.text_with_shadow(
                draw.atlas,
                draw.font,
                &cost_text,
                x + 20 + 86 - draw.font.width(&cost_text),
                y + 9,
                cost_colour,
            );
            // The tooltip of the row under the mouse.
            let (xx, yy) = (mouse.0 - f64::from(x), mouse.1 - f64::from(y));
            if tip.is_none() && (0.0..108.0).contains(&xx) && (0.0..17.0).contains(&yy) {
                tip = Some(row);
            }
        }
        if let Some(row) = tip {
            self.enchant_tooltip(menu, row);
        }
    }

    /// `EnchantmentScreen.extractRenderState`'s tooltip over a row: the clue,
    /// and what it costs.
    pub(super) fn enchant_tooltip(&mut self, menu: &ContainerMenu, row: usize) {
        use photon_game::text::{Component, Run, Style};
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        let cost = data(row);
        let clue = usize::try_from(data(4 + row)).ok();
        let level = data(7 + row);
        let Some(enchantment) = clue.and_then(|id| self.titles.enchantments.get(id)) else {
            return;
        };
        if cost <= 0 || level < 0 {
            return;
        }
        let lang = &self.titles.lang;
        // `Enchantment.getFullname`: the name in grey, red for a curse, and
        // the level unless the enchantment has one level only.
        let mut full = Component::literal(enchantment.name.clone())
            .with_colour(if enchantment.curse { "red" } else { "gray" });
        if level != 1 || enchantment.max_level != 1 {
            full = full
                .append(Component::literal(" "))
                .append(Component::translatable(format!(
                    "enchantment.level.{level}"
                )));
        }
        let clue_line =
            Component::translatable_with("container.enchant.clue", vec![full]).with_colour("white");
        let line = |text: String, colour: &str| {
            vec![Run {
                text,
                style: Style::coloured(colour),
            }]
        };
        let mut lines = vec![lang.runs(&clue_line, &Style::default())];
        if !menu.creative() {
            lines.push(Vec::new());
            let gold = menu
                .slots
                .get(1)
                .and_then(Option::as_ref)
                .map_or(0, photon_world::ClientItemStackState::item_count);
            let row_cost = row as i32 + 1;
            if menu.xp_level < cost {
                lines.push(line(
                    lang.text("container.enchant.level.requirement")
                        .replace("%s", &cost.to_string()),
                    "red",
                ));
            } else {
                let lapis = if row_cost == 1 {
                    lang.text("container.enchant.lapis.one")
                } else {
                    lang.text("container.enchant.lapis.many")
                        .replace("%s", &row_cost.to_string())
                };
                lines.push(line(lapis, if gold >= row_cost { "gray" } else { "red" }));
                let levels = if row_cost == 1 {
                    lang.text("container.enchant.level.one")
                } else {
                    lang.text("container.enchant.level.many")
                        .replace("%s", &row_cost.to_string())
                };
                lines.push(line(levels, "gray"));
            }
        }
        self.extras.pending_tooltip = Some(lines);
    }

    /// `AnvilScreen.onNameChanged`: with an item in the slot, the box's text
    /// is the name to send, or none at all when it is what the item is
    /// called already; `AnvilMenu.setItemName` tells whether that is news.
    pub(super) fn name_changed(&mut self) {
        let Some((hover, custom)) = &self.extras.named_item else {
            return;
        };
        let value = self.extras.name.value();
        let name = if !custom && value == *hover {
            String::new()
        } else {
            value
        };
        if name.chars().count() <= 50 && self.extras.item_name.as_ref() != Some(&name) {
            self.extras.item_name = Some(name.clone());
            self.extras.queued.push(ScreenAction::Rename(name));
        }
    }

    /// A key for the anvil's name box, when it can take input.
    pub(in super::super) fn name_key(
        &mut self,
        key: glfw::Key,
        control: bool,
        shift: bool,
        menu: &ContainerMenu,
        clipboard: &mut dyn Clipboard,
    ) -> Option<KeyResult> {
        if self.kind != MenuKind::Anvil || menu.slots.first().is_none_or(Option::is_none) {
            return None;
        }
        if key == glfw::Key::Escape {
            return Some(KeyResult::Close);
        }
        if let Some(edit) = edit_key(key) {
            let font = self.titles.font.clone();
            self.extras
                .name
                .key_pressed(edit, control, shift, &font, clipboard);
            self.name_changed();
        }
        // The box takes every key while it can, so the inventory key is typed.
        Some(KeyResult::Consumed(Vec::new()))
    }

    /// A character typed into the anvil's name box.
    pub(in super::super) fn name_char(&mut self, character: char) {
        if self.kind == MenuKind::Anvil && self.extras.named_item.is_some() {
            let font = self.titles.font.clone();
            if self.extras.name.char_typed(character, &font) {
                self.name_changed();
            }
        }
    }

    /// The marks, previews and boxes of the menu over its image.
    pub(in super::super) fn draw_extras(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
    ) {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        match self.kind {
            MenuKind::Grindstone => {
                if (has(0) || has(1)) && !has(2) {
                    draw.frame.sprite(
                        draw.atlas,
                        "container/grindstone/error",
                        left + 92,
                        top + 31,
                        28,
                        21,
                    );
                }
            }
            MenuKind::Smithing => {
                if data(0) > 0 {
                    draw.frame.sprite(
                        draw.atlas,
                        "container/smithing/error",
                        left + 65,
                        top + 46,
                        28,
                        21,
                    );
                }
            }
            MenuKind::Brewing => brewing_progress(draw, (data(0), data(1)), (left, top)),
            MenuKind::Cartography => self.cartography_preview(draw, menu, (left, top)),
            MenuKind::Anvil => self.anvil_extras(draw, menu, (left, top)),
            MenuKind::Stonecutter => self.stone_draw(draw, menu, mouse, (left, top)),
            MenuKind::Loom => self.loom_draw(draw, menu, mouse, (left, top)),
            MenuKind::Enchantment => self.enchant_draw(draw, menu, mouse, (left, top)),
            _ => {}
        }
    }

    /// `CartographyTableScreen.extractBackground` without the map drawn.
    pub(super) fn cartography_preview(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        let item = |slot: usize| {
            menu.slots
                .get(slot)
                .and_then(Option::as_ref)
                .and_then(|stack| {
                    photon_data::item::registry::ITEMS
                        .get(usize::try_from(stack.item_registry_id().0).ok()?)
                })
                .map_or("", |item| item.name)
        };
        let extra = item(1);
        let sprite = |draw: &mut ScreenDraw<'_, '_>, name: &str, x: i32, y: i32, w: i32, h: i32| {
            draw.frame.sprite(
                draw.atlas,
                &format!("container/cartography_table/{name}"),
                left + x,
                top + y,
                w,
                h,
            );
        };
        if extra == "minecraft:paper" {
            sprite(draw, "scaled_map", 67, 13, 66, 66);
        } else if extra == "minecraft:map" {
            sprite(draw, "duplicated_map", 83, 13, 50, 66);
            sprite(draw, "duplicated_map", 67, 29, 50, 66);
        } else if extra == "minecraft:glass_pane" {
            sprite(draw, "map", 67, 13, 66, 66);
            sprite(draw, "locked", 118, 60, 10, 14);
        } else {
            sprite(draw, "map", 67, 13, 66, 66);
        }
    }
}
