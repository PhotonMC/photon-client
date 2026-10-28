//! The trading screen (`MerchantScreen`): the list of offers with its scroller, what each offer
//! asks and gives, the merchant's experience bar, and the buttons that choose a trade.

use crate::ItemSlots;
use photon_game::menus::container::ContainerMenu;
use photon_game::menus::merchant::{MerchantOffer, can_level_up, level_xp_range};
use photon_world::ClientItemStackState as Stack;

use super::{ScreenAction, ScreenDraw};

/// How many offers the list shows at once.
const SHOWN: usize = 7;
/// The trade buttons are 88 by 20.
const BUTTON: (i32, i32) = (88, 20);

#[derive(Debug, Clone)]
pub(super) struct MerchantPanel {
    /// `shopItem`: the offer chosen.
    shop_item: usize,
    /// `scrollOff`: the first offer shown.
    scroll_off: usize,
    dragging: bool,
    /// Which buttons showed after the last frame (a button shows from the frame after its offer
    /// comes).
    visible: [bool; SHOWN],
    /// How many offers there were at the last frame, for the wheel.
    count: usize,
}

impl MerchantPanel {
    pub(super) fn new() -> Self {
        Self {
            shop_item: 0,
            scroll_off: 0,
            dragging: false,
            visible: [false; SHOWN],
            count: 0,
        }
    }

    fn can_scroll(offers: usize) -> bool {
        offers > SHOWN
    }

    /// `extractBackground`: the selected offer's out-of-stock mark.
    pub(super) fn background(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        if let Some(offer) = menu.offers.get(self.shop_item)
            && offer.is_out_of_stock()
        {
            draw.frame.sprite(
                draw.atlas,
                "container/villager/out_of_stock",
                left + 83 + 99,
                top + 35,
                28,
                21,
            );
        }
    }

    fn button_origin(index: usize, (left, top): (i32, i32)) -> (i32, i32) {
        (left + 5, top + 16 + 2 + index as i32 * 20)
    }

    fn over(point: (f64, f64), (x, y): (i32, i32), (w, h): (i32, i32)) -> bool {
        point.0 >= f64::from(x)
            && point.1 >= f64::from(y)
            && point.0 < f64::from(x + w)
            && point.1 < f64::from(y + h)
    }

    /// The trade buttons, as they were left after the last frame.
    pub(super) fn buttons(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        origin: (i32, i32),
        mouse: (f64, f64),
    ) {
        for index in 0..SHOWN {
            if !self.visible[index] {
                continue;
            }
            let (x, y) = Self::button_origin(index, origin);
            let sprite = if Self::over(mouse, (x, y), BUTTON) {
                "widget/button_highlighted"
            } else {
                "widget/button"
            };
            draw.frame.nine_slice(
                draw.atlas,
                sprite,
                (200, 20),
                3,
                false,
                (x, y, BUTTON.0, BUTTON.1),
            );
        }
    }

    /// A payment or result, with its count the way `itemDecorations` shows it.
    fn decorated(draw: &mut ScreenDraw<'_, '_>, stack: &Stack, at: (i32, i32), text: Option<&str>) {
        draw.frame.item_with_count(
            draw.atlas,
            draw.font,
            draw.item,
            stack,
            (stack.item_count(), text.map(|text| (text, 0xFFFF_FFFF))),
            at,
        );
    }

    /// `extractContents` after the slots: the scroller, each offer shown, the experience bar and
    /// the tooltips; the lines of the tooltip to draw, if any.
    pub(super) fn contents(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        origin: (i32, i32),
        mouse: (f64, f64),
        deprecated: &str,
    ) -> Option<Vec<Vec<photon_game::text::Run>>> {
        let offers = &menu.offers;
        self.count = offers.len();
        if offers.is_empty() {
            return None;
        }
        let (xo, yo) = origin;
        self.scroller(draw, origin, offers.len());
        let mut offer_y = yo + 16 + 1;
        let first_item_x = xo + 5 + 5;
        for (index, offer) in offers.iter().enumerate() {
            if Self::can_scroll(offers.len())
                && !(index >= self.scroll_off && index < SHOWN + self.scroll_off)
            {
                continue;
            }
            let decor = offer_y + 2;
            let (cost_a, base_a) = (offer.cost_a_now(), offer.base_cost_a());
            if base_a.item_count() == cost_a.item_count() {
                Self::decorated(draw, &cost_a, (first_item_x, decor), None);
            } else {
                // The price the demand raised, and the one it was, struck through.
                let one = |stack: &Stack| (stack.item_count() == 1).then_some("1");
                Self::decorated(draw, &base_a, (first_item_x, decor), one(&base_a));
                Self::decorated(draw, &cost_a, (first_item_x + 14, decor), one(&cost_a));
                draw.frame.sprite(
                    draw.atlas,
                    "container/villager/discount_strikethrough",
                    first_item_x + 7,
                    decor + 12,
                    9,
                    2,
                );
            }
            if let Some(cost_b) = offer.cost_b_now() {
                Self::decorated(draw, &cost_b, (xo + 5 + 35, decor), None);
            }
            let arrow = if offer.is_out_of_stock() {
                "container/villager/trade_arrow_out_of_stock"
            } else {
                "container/villager/trade_arrow"
            };
            draw.frame
                .sprite(draw.atlas, arrow, xo + 5 + 35 + 20, decor + 3, 10, 9);
            Self::decorated(draw, &offer.result, (xo + 5 + 68, decor), None);
            offer_y += 20;
        }
        let selected = offers.get(self.shop_item)?;
        if menu.trader.show_progress {
            self.progress_bar(draw, menu, origin);
        }
        let mut tooltip = None;
        if selected.is_out_of_stock()
            && Self::over(mouse, (xo + 186, yo + 35), (22, 21))
            && menu.trader.can_restock
        {
            tooltip = Some(vec![crate::chat::plain_runs(deprecated)]);
        }
        for index in 0..SHOWN {
            let (x, y) = Self::button_origin(index, origin);
            if Self::over(mouse, (x, y), BUTTON) && self.visible[index] {
                tooltip = self
                    .button_tooltip(draw, offers, index, x, mouse.0)
                    .or(tooltip);
            }
            self.visible[index] = index < offers.len();
        }
        tooltip
    }

    /// `TradeOfferButton.extractToolTip`: the item of the part of the button the mouse is on.
    fn button_tooltip(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        offers: &[MerchantOffer],
        index: usize,
        button_x: i32,
        mouse_x: f64,
    ) -> Option<Vec<Vec<photon_game::text::Run>>> {
        let offer = offers.get(index + self.scroll_off)?;
        let x = mouse_x as i32;
        let stack = if x < button_x + 20 {
            Some(offer.cost_a_now())
        } else if x < button_x + 50 && x > button_x + 30 {
            offer.cost_b_now()
        } else if x > button_x + 65 {
            Some(offer.result.clone())
        } else {
            None
        }?;
        Some((draw.tooltip)(&stack, false))
    }

    /// `extractScroller`.
    fn scroller(&self, draw: &mut ScreenDraw<'_, '_>, (xo, yo): (i32, i32), offers: usize) {
        let steps = offers as i32 + 1 - SHOWN as i32;
        if steps > 1 {
            let left_over = 139 - (27 + (steps - 1) * 139 / steps);
            let step_height = 1 + left_over / steps + 139 / steps;
            let mut offset = (self.scroll_off as i32 * step_height).min(113);
            if self.scroll_off as i32 == steps - 1 {
                offset = 113;
            }
            draw.frame.sprite(
                draw.atlas,
                "container/villager/scroller",
                xo + 94,
                yo + 18 + offset,
                6,
                27,
            );
        } else {
            draw.frame.sprite(
                draw.atlas,
                "container/villager/scroller_disabled",
                xo + 94,
                yo + 18,
                6,
                27,
            );
        }
    }

    /// `extractProgressBar`.
    fn progress_bar(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (xo, yo): (i32, i32),
    ) {
        let (level, xp) = (menu.trader.level, menu.trader.xp);
        if level >= 5 {
            return;
        }
        draw.frame.sprite(
            draw.atlas,
            "container/villager/experience_bar_background",
            xo + 136,
            yo + 16,
            102,
            5,
        );
        let (min, max) = level_xp_range(level);
        if xp >= min && can_level_up(level) {
            let multiplier = 102.0_f32 / (max - min) as f32;
            let width = ((multiplier * (xp - min) as f32).floor() as i32).min(102);
            draw.frame.sprite_cut(
                draw.atlas,
                "container/villager/experience_bar_current",
                (102, 5),
                (0, 0),
                (xo + 136, yo + 16, width, 5),
            );
            if menu.future_xp > 0 {
                let future = ((menu.future_xp as f32 * multiplier).floor() as i32).min(102 - width);
                draw.frame.sprite_cut(
                    draw.atlas,
                    "container/villager/experience_bar_result",
                    (102, 5),
                    (width, 0),
                    (xo + 136 + width, yo + 16, future, 5),
                );
            }
        }
    }

    /// `mouseClicked`: a press on the scroller starts a drag; one on a visible button chooses
    /// its offer.
    pub(super) fn click(
        &mut self,
        mouse: (f64, f64),
        origin: (i32, i32),
        menu: &ContainerMenu,
        left_button: bool,
    ) -> Option<Vec<ScreenAction>> {
        let (xo, yo) = origin;
        if Self::can_scroll(menu.offers.len())
            && mouse.0 > f64::from(xo + 94)
            && mouse.0 < f64::from(xo + 94 + 6)
            && mouse.1 > f64::from(yo + 18)
            && mouse.1 <= f64::from(yo + 18 + 139 + 1)
        {
            self.dragging = true;
        }
        for index in 0..SHOWN {
            let at = Self::button_origin(index, origin);
            if left_button && self.visible[index] && Self::over(mouse, at, BUTTON) {
                crate::ui_sound::play_down_sound();
                self.shop_item = index + self.scroll_off;
                return Some(vec![ScreenAction::SelectTrade(self.shop_item as i32)]);
            }
        }
        None
    }

    pub(super) fn release(&mut self) {
        self.dragging = false;
    }

    /// `mouseDragged` on the scroller.
    pub(super) fn drag(&mut self, mouse: (f64, f64), top: i32, menu: &ContainerMenu) {
        if !self.dragging {
            return;
        }
        let offers = menu.offers.len() as i32;
        let full_top = top + 18;
        let full_bottom = full_top + 139;
        let max = offers - SHOWN as i32;
        let scrolling =
            (mouse.1 as f32 - full_top as f32 - 13.5) / ((full_bottom - full_top) as f32 - 27.0);
        let scrolling = scrolling * max as f32 + 0.5;
        self.scroll_off = (scrolling as i32).clamp(0, max.max(0)) as usize;
    }

    /// `mouseScrolled`.
    pub(super) fn wheel(&mut self, amount: f64) {
        let offers = self.count;
        if Self::can_scroll(offers) {
            let max = (offers - SHOWN) as i32;
            self.scroll_off = ((self.scroll_off as f64 - amount) as i32).clamp(0, max) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_game::menus::container::MenuKind;

    fn menu(offers: usize) -> ContainerMenu {
        let mut menu = ContainerMenu::for_menu(MenuKind::Merchant, 3, Vec::new(), None, 0);
        let item = photon_game::item::id_of("minecraft:emerald").unwrap();
        let offer = photon_game::menus::merchant::MerchantOffer {
            cost_a: photon_game::menus::merchant::ItemCost {
                item,
                count: 1,
                components: Vec::new(),
            },
            cost_b: None,
            result: Stack::new(1, item),
            uses: 0,
            max_uses: 4,
            xp: 1,
            special_price_diff: 0,
            demand: 0,
            price_multiplier: 0.0,
        };
        menu.offers = std::sync::Arc::new(vec![offer; offers]);
        menu
    }

    #[test]
    fn a_visible_button_chooses_its_offer_plus_the_scroll() {
        let mut panel = MerchantPanel::new();
        let menu = menu(10);
        let origin = (0, 0);
        // Nothing shows before the first frame has run.
        assert!(panel.click((10.0, 30.0), origin, &menu, true).is_none());
        panel.visible = [true; SHOWN];
        panel.scroll_off = 2;
        // The second button, at 38 down.
        let actions = panel.click((10.0, 40.0), origin, &menu, true).unwrap();
        assert_eq!(actions, vec![ScreenAction::SelectTrade(3)]);
        // Only the left button presses it.
        assert!(panel.click((10.0, 40.0), origin, &menu, false).is_none());
    }

    #[test]
    fn the_wheel_and_the_bar_scroll_within_the_offers_that_do_not_fit() {
        let mut panel = MerchantPanel::new();
        let menu = menu(10);
        panel.count = 10;
        panel.wheel(-1.0);
        assert_eq!(panel.scroll_off, 1);
        panel.wheel(-5.0);
        assert_eq!(panel.scroll_off, 3);
        panel.wheel(10.0);
        assert_eq!(panel.scroll_off, 0);
        // A press on the bar, then a drag to its bottom.
        assert!(panel.click((96.0, 100.0), (0, 0), &menu, true).is_none());
        panel.drag((96.0, 160.0), 0, &menu);
        assert_eq!(panel.scroll_off, 3);
        panel.release();
        panel.drag((96.0, 18.0), 0, &menu);
        assert_eq!(panel.scroll_off, 3);
    }
}
