use super::*;

impl GenericScreen {
    /// The anvil's name box, its error mark and what the repair costs.
    pub(super) fn anvil_extras(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        let has = |slot: usize| menu.slots.get(slot).is_some_and(Option::is_some);
        let editable = has(0);
        // The box is filled with the item's name when the item changes.
        let key = key_of(menu, 0);
        if self.extras.filled_for != Some(key) {
            self.extras.filled_for = Some(key);
            let item = menu.slots.first().and_then(Option::as_ref);
            let name = item
                .map(|stack| {
                    (draw.tooltip)(stack, false)
                        .first()
                        .map(|line| line.iter().map(|run| run.text.as_str()).collect::<String>())
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            self.extras.named_item = item.map(|stack| {
                let custom = voxel_game::item::values::Values::new(
                    stack.item_registry_id(),
                    stack.patch_bytes(),
                )
                .has(voxel_data::item::component_types::CUSTOM_NAME);
                (name.clone(), custom)
            });
            let font = self.titles.font.clone();
            self.extras.name.set_value(&name, &font);
            self.extras.focused_at = Instant::now();
            // `setValue` tells the responder, which sends the name.
            self.name_changed();
        }
        let field = if editable {
            "container/anvil/text_field"
        } else {
            "container/anvil/text_field_disabled"
        };
        draw.frame
            .sprite(draw.atlas, field, left + 59, top + 20, 110, 16);
        if (has(0) || has(1)) && !has(2) {
            draw.frame.sprite(
                draw.atlas,
                "container/anvil/error",
                left + 99,
                top + 45,
                28,
                21,
            );
        }
        if editable {
            self.extras.name.draw(
                draw.frame,
                draw.atlas,
                draw.font,
                (left + 62, top + 24),
                self.extras.focused_at,
            );
        }
        // The cost of taking the result.
        let cost = i32::from(menu.data.first().copied().unwrap_or(0));
        if cost > 0 {
            let (text, colour) = if cost >= 40 && !menu.creative() {
                (self.titles.repair_expensive.clone(), 0xFFFF_6060)
            } else if has(2) {
                let colour = if menu.can_pick_up_result() {
                    0xFF80_FF20
                } else {
                    0xFFFF_6060
                };
                (
                    self.titles
                        .repair_cost
                        .replace("%1$s", &cost.to_string())
                        .replace("%s", &cost.to_string()),
                    colour,
                )
            } else {
                return;
            };
            let x = 176 - 8 - draw.font.width(&text) - 2;
            draw.frame.fill(
                draw.atlas,
                (left + x - 2, top + 67, left + 176 - 8, top + 79),
                0x4F00_0000,
            );
            draw.frame
                .text_with_shadow(draw.atlas, draw.font, &text, left + x, top + 69, colour);
        }
    }
}
