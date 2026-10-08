use super::*;

impl GenericScreen {
    /// `CrafterScreen.slotClicked`: a click on an empty slot of the grid switches it on or off
    /// before the click itself is made.
    pub(super) fn crafter_toggle(
        kind: MenuKind,
        click: &SlotClick,
        menu: &ContainerMenu,
    ) -> Option<ScreenAction> {
        let slot = usize::try_from(click.slot).ok().filter(|slot| *slot < 9)?;
        if kind != MenuKind::Crafter || menu.slots[slot].is_some() {
            return None;
        }
        let disabled = menu.data.get(slot) == Some(&1);
        let enabled = match click.kind {
            ClickType::Pickup if disabled => true,
            ClickType::Pickup if menu.carried.is_none() => false,
            // A number key over a switched-off slot, when the hotbar slot holds something.
            ClickType::Swap if disabled => {
                let hotbar = usize::try_from(click.button).ok().filter(|key| *key < 9)?;
                menu.slots.get(36 + hotbar)?.as_ref()?;
                true
            }
            _ => return None,
        };
        Some(ScreenAction::SlotState {
            slot: slot as i32,
            enabled,
        })
    }

    /// `MerchantScreen.extractLabels`: the title, with the merchant's level when its bar shows, the
    /// inventory's label and the list's.
    pub(super) fn draw_merchant_labels(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        use photon_game::text::{Component, Style};
        let lang = &self.titles.lang;
        let level = menu.trader.level;
        let title = if level > 0 && level <= 5 && menu.trader.show_progress {
            let named = Component::translatable_with(
                "merchant.title",
                vec![
                    Component::literal(self.titles.title.clone()),
                    Component::translatable(format!("merchant.level.{level}")),
                ],
            );
            lang.runs(&named, &Style::default())
        } else {
            lang.runs(
                &Component::literal(self.titles.title.clone()),
                &Style::default(),
            )
        };
        let width = draw.font.width_of_runs(&title);
        let colour = 0xFF40_4040;
        draw.frame.styled_text(
            draw.atlas,
            draw.font,
            &title,
            left + 49 + 276 / 2 - width / 2,
            top + 6,
            colour,
            false,
        );
        draw.frame.text(
            draw.atlas,
            draw.font,
            &self.titles.inventory,
            left + 107,
            top + 166 - 94,
            colour,
        );
        let trades = lang.text("merchant.trades");
        draw.frame.text(
            draw.atlas,
            draw.font,
            &trades,
            left + 5 - draw.font.width(&trades) / 2 + 48,
            top + 6,
            colour,
        );
    }

    /// `CrafterScreen.extractRedstone`, from the middle of the screen; and the hint over a
    /// switched-on empty slot.
    pub(super) fn draw_crafter_redstone(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
    ) {
        if self.kind != MenuKind::Crafter {
            return;
        }
        let crafter_data = |slot: usize| menu.data.get(slot).copied().unwrap_or(0);
        // `CrafterScreen.extractRedstone`, from the middle of the screen; and the hint over
        // a switched-on empty slot.
        let sprite = if crafter_data(9) == 1 {
            "container/crafter/powered_redstone"
        } else {
            "container/crafter/unpowered_redstone"
        };
        draw.frame.sprite(
            draw.atlas,
            sprite,
            self.layout.width / 2 + 9,
            self.layout.height / 2 - 48,
            16,
            16,
        );
        if let Some(slot) = self.hovered.filter(|slot| *slot < 9)
            && crafter_data(slot) != 1
            && menu.carried.is_none()
            && menu.slots[slot].is_none()
        {
            let hint = self.titles.lang.text("gui.togglable_slot");
            self.extras.pending_tooltip = Some(vec![crate::gui::chat::plain_runs(&hint)]);
        }
    }

    /// The labels of the screen (and, for the beacon, its buttons), which the reference draws
    /// with them: the title and the inventory's, or the beacon's two powers.
    pub(super) fn draw_labels(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        mouse: (f64, f64),
        (left, top): (i32, i32),
        (width, height): (i32, i32),
    ) {
        if let Some(panel) = &self.beacon {
            if let Some((key, suffix)) = panel.draw(draw, (left, top), mouse) {
                let name = format!("{}{suffix}", self.titles.lang.text(&key));
                self.extras.pending_tooltip = Some(vec![crate::gui::chat::plain_runs(&name)]);
            }
            // `BeaconScreen.extractLabels`: two centred labels instead of the title.
            for (key, x) in [
                ("block.minecraft.beacon.primary", 62),
                ("block.minecraft.beacon.secondary", 169),
            ] {
                let label = self.titles.lang.text(key);
                draw.frame.text_with_shadow(
                    draw.atlas,
                    draw.font,
                    &label,
                    left + x - draw.font.width(&label) / 2,
                    top + 10,
                    0xFFE0_E0E0,
                );
            }
        } else if self.kind == MenuKind::Merchant {
            self.draw_merchant_labels(draw, menu, (left, top));
        } else {
            // The dispenser centres its title; the others keep it at the corner.
            let (label_x, label_y) = transform::title_label(self.kind);
            let title_x = if matches!(
                self.kind,
                MenuKind::Dispenser | MenuKind::Furnace(_) | MenuKind::Brewing | MenuKind::Crafter
            ) {
                (width - draw.font.width(&self.titles.title)) / 2
            } else {
                label_x
            };
            draw.frame.text(
                draw.atlas,
                draw.font,
                &self.titles.title,
                left + title_x,
                top + label_y,
                0xFF40_4040,
            );
            draw.frame.text(
                draw.atlas,
                draw.font,
                &self.titles.inventory,
                left + 8,
                top + height - 94,
                0xFF40_4040,
            );
        }
    }
}
