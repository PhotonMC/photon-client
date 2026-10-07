use super::*;

impl RecipeBookPanel {
    /// `GhostSlots.extractTooltip`: over a ghost slot, the tooltip of the item
    /// it shows now. Whether it drew one (a tooltip asked for earlier wins).
    pub fn ghost_tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        hovered: Option<usize>,
        mouse: (f64, f64),
    ) -> bool {
        let (Some(slot), Some((container, display))) = (hovered, &draw.book.ghost) else {
            return false;
        };
        if *container != menu.container_id || !self.visible {
            return false;
        }
        let empty = |slot: usize| menu.slots.get(slot).is_none_or(Option::is_none);
        let Some((_, shown, _)) = ghost_slots(self.kind, display, &empty)
            .into_iter()
            .find(|(place, _, _)| *place == slot)
        else {
            return false;
        };
        let stacks = display_stacks(&shown, draw.tags);
        if stacks.is_empty() {
            return false;
        }
        let (item, count) = stacks[self.select_time() % stacks.len()];
        let lines = (draw.tooltip)(&Stack::new(count, item), false);
        self.tooltip(draw, mouse, lines);
        true
    }

    /// `OverlayRecipeComponent.extractRenderState`: the box, and in it each
    /// recipe's button with its ingredients drawn small.
    pub(super) fn draw_overlay(&mut self, draw: &mut ScreenDraw<'_, '_>, mouse: (f64, f64)) {
        if self.overlay_pending {
            self.overlay_pending = false;
            if let Some(overlay) = &mut self.overlay {
                for button in &mut overlay.buttons {
                    if let Some(entry) = draw.book.known.get(&button.recipe) {
                        button.slots = overlay_slots(&entry.display, draw.tags);
                    }
                }
            }
        }
        let Some(overlay) = &self.overlay else {
            return;
        };
        let total = overlay.buttons.len();
        let max_row = if total <= 16 { 4 } else { 5 };
        let width = total.min(max_row) as i32;
        let height = total.div_ceil(max_row) as i32;
        draw.frame.nine_slice(
            draw.atlas,
            "recipe_book/overlay_recipe",
            (32, 32),
            4,
            false,
            overlay.x,
            overlay.y,
            width * 25 + 8,
            height * 25 + 8,
        );
        let folder = if matches!(self.kind, BookKind::Furnace(_)) {
            "furnace"
        } else {
            "crafting"
        };
        for button in &overlay.buttons {
            let over = Self::in_rect(mouse, (button.x, button.y, 24, 24));
            let sprite = format!(
                "recipe_book/{folder}_overlay{}{}",
                if button.craftable { "" } else { "_disabled" },
                if over { "_highlighted" } else { "" }
            );
            draw.frame
                .sprite(draw.atlas, &sprite, button.x, button.y, 24, 24);
            for (sx, sy, slot) in &button.slots {
                let stacks = display_stacks(slot, draw.tags);
                if stacks.is_empty() {
                    continue;
                }
                let (item, count) = stacks[self.select_time() % stacks.len()];
                let first = draw.frame.items.len();
                draw.frame.item(draw.item, &Stack::new(count, item), 0, 0);
                // The pose: to the slot's place, 0.375 of the size, about
                // the item's middle.
                let (tx, ty) = ((button.x + 2 + sx) as f32, (button.y + 2 + sy) as f32);
                draw.frame
                    .scale_items_from(first, 0.375, (tx - 0.375 * 8.0, ty - 0.375 * 8.0));
            }
        }
    }

    /// `GhostSlots`: the ghost recipe over the menu's slots, each slot's
    /// item a translucent copy between a red and a white wash.
    pub fn draw_ghost(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
        position: &dyn Fn(usize) -> (i32, i32),
    ) {
        let Some((container, display)) = &draw.book.ghost else {
            return;
        };
        if *container != menu.container_id || matches!(menu.kind, MenuKind::Chest(_)) {
            return;
        }
        // `isBiggerResultSlot`: the inventory's result slot is not.
        let big_result = menu.kind != MenuKind::Inventory;
        let empty = |slot: usize| menu.slots.get(slot).is_none_or(Option::is_none);
        for (slot, shown, is_result) in ghost_slots(self.kind, display, &empty) {
            let stacks = display_stacks(&shown, draw.tags);
            if stacks.is_empty() {
                continue;
            }
            let (x, y) = position(slot);
            let (x, y) = (left + x, top + y);
            let rect = if is_result && big_result {
                (x - 4, y - 4, x + 20, y + 20)
            } else {
                (x, y, x + 16, y + 16)
            };
            draw.frame.fill(draw.atlas, rect, 0x30FF_0000);
            let (item, count) = stacks[self.select_time() % stacks.len()];
            let stack = Stack::new(count, item);
            draw.frame.item(draw.item, &stack, x, y);
            draw.frame
                .fill(draw.atlas, (x, y, x + 16, y + 16), 0x30FF_FFFF);
            if is_result && count != 1 {
                draw.frame.text_with_shadow(
                    draw.atlas,
                    draw.font,
                    &count.to_string(),
                    x + 19 - 2 - draw.font.width(&count.to_string()),
                    y + 6 + 3,
                    0xFFFF_FFFF,
                );
            }
        }
    }
}
