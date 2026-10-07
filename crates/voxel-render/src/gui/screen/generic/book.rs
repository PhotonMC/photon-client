use super::*;

impl GenericScreen {
    /// A screen for a menu, opened on a GUI of this size.
    pub fn new(kind: MenuKind, width: i32, height: i32, titles: Titles) -> Self {
        // `CraftingScreen` and `AbstractFurnaceScreen` place the toggle button.
        let button = if kind == MenuKind::Crafting {
            (5, -49)
        } else {
            (20, -49)
        };
        let book = BookKind::of_menu(kind).map(|book| {
            RecipeBookPanel::new(book, 176, button, titles.recipe.clone(), (width, height))
        });
        Self {
            kind,
            layout: ContainerLayout::new(width, height, image(kind)),
            titles,
            hovered: None,
            mouse: ContainerMouse::default(),
            book,
            extras: transform::Extras::new(),
            beacon: (kind == MenuKind::Beacon).then(beacon::BeaconPanel::new),
            merchant: (kind == MenuKind::Merchant).then(merchant::MerchantPanel::new),
        }
    }

    /// `AbstractRecipeBookScreen.widthTooNarrow`: the open book covers the
    /// screen, which takes no click or hover.
    pub(super) fn book_covers(&self) -> bool {
        self.book
            .as_ref()
            .is_some_and(RecipeBookPanel::blocks_screen)
    }

    /// `hasClickedOutside`: off the image, and off the book's panel and tabs.
    pub(super) fn outside(&self, mouse: (f64, f64)) -> bool {
        self.layout.outside(mouse)
            && !self.book.as_ref().is_some_and(|book| {
                book.claims(
                    mouse,
                    (self.layout.left(), self.layout.top()),
                    image(self.kind).1,
                )
            })
    }

    pub(super) fn hovered_slot(&self, mouse: (f64, f64)) -> Option<usize> {
        if self.book_covers() {
            return None;
        }
        let kind = self.kind;
        self.layout
            .hovered(mouse, kind.slot_count(), |index| slot_position(kind, index))
    }

    /// The screen's texture, over the image's rectangle.
    pub(super) fn blit_background(&self, draw: &mut ScreenDraw<'_, '_>) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let (width, height) = image(self.kind);
        match self.kind {
            MenuKind::Chest(rows) => {
                let upper = rows as i32 * 18 + 17;
                let texture = "container/generic_54";
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    (256, 256),
                    (0, 0),
                    left,
                    top,
                    width,
                    upper,
                );
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    (256, 256),
                    (0, 126),
                    left,
                    top + upper,
                    width,
                    96,
                );
            }
            other => {
                let texture = match other {
                    MenuKind::Dispenser => "container/dispenser",
                    MenuKind::Hopper => "container/hopper",
                    MenuKind::Crafting => "container/crafting_table",
                    MenuKind::Furnace(Smelter::Furnace) => "container/furnace",
                    MenuKind::Furnace(Smelter::BlastFurnace) => "container/blast_furnace",
                    MenuKind::Furnace(Smelter::Smoker) => "container/smoker",
                    MenuKind::ShulkerBox => "container/shulker_box",
                    other => transform::texture(other).unwrap_or("container/hopper"),
                };
                // The villager's is twice as wide.
                let size = if other == MenuKind::Merchant {
                    (512, 256)
                } else {
                    (256, 256)
                };
                draw.frame.blit_texture(
                    draw.atlas,
                    texture,
                    size,
                    (0, 0),
                    left,
                    top,
                    width,
                    height,
                );
            }
        }
    }

    /// `AbstractFurnaceScreen.extractBackground`: the flame, as much of it as
    /// burns, and the arrow, as much as has cooked.
    pub(super) fn furnace_progress(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        smelter: Smelter,
    ) {
        let (left, top) = (self.layout.left(), self.layout.top());
        let folder = match smelter {
            Smelter::Furnace => "furnace",
            Smelter::BlastFurnace => "blast_furnace",
            Smelter::Smoker => "smoker",
        };
        let data = |index: usize| i32::from(menu.data.get(index).copied().unwrap_or(0));
        if data(0) > 0 {
            let duration = if data(1) == 0 { 200 } else { data(1) };
            let lit = (data(0) as f32 / duration as f32).clamp(0.0, 1.0);
            let height = (lit * 13.0).ceil() as i32 + 1;
            draw.frame.blit_part(
                draw.atlas,
                GuiBlend::Translucent,
                &format!("container/{folder}/lit_progress"),
                (14, 14),
                (0, 14 - height),
                left + 56,
                top + 36 + 14 - height,
                14,
                height,
            );
        }
        let (current, total) = (data(2), data(3));
        let cooked = if total != 0 && current != 0 {
            (current as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let width = (cooked * 24.0).ceil() as i32;
        draw.frame.blit_part(
            draw.atlas,
            GuiBlend::Translucent,
            &format!("container/{folder}/burn_progress"),
            (24, 16),
            (0, 0),
            left + 79,
            top + 34,
            width,
            16,
        );
    }

    /// The clicks as actions, with what the recipe book answers them with.
    pub(super) fn actions(
        &mut self,
        clicks: Vec<SlotClick>,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction> {
        let mut actions = Vec::new();
        for click in clicks {
            actions.extend(Self::crafter_toggle(self.kind, &click, menu));
            if let Some(book) = &mut self.book {
                book.slot_clicked(click.slot);
                actions.extend(book.drain_actions());
            }
            actions.push(ScreenAction::Container(click));
        }
        actions
    }
}
