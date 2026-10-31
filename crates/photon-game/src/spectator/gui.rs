use super::{Action, FADE_OUT_DELAY, FADE_OUT_TIME, Item, Sources, SpectatorMenu};

/// `SpectatorGui`: the menu while it is open, and when it was last used, which fades it out.
#[derive(Debug, Default)]
pub struct SpectatorGui {
    menu: Option<SpectatorMenu>,
    last_selection: i64,
}

impl SpectatorGui {
    /// `isMenuActive`.
    pub fn is_menu_active(&self) -> bool {
        self.menu.is_some()
    }

    pub fn menu(&self) -> Option<&SpectatorMenu> {
        self.menu.as_ref()
    }

    /// `onHotbarSelected`: a number key. The first press opens the menu, later ones select.
    pub fn on_hotbar_selected(&mut self, slot: i32, now: i64, sources: &Sources) -> Option<Action> {
        self.last_selection = now;
        if let Some(menu) = self.menu.as_mut() {
            let action = menu.select_slot(slot);
            self.close_if_asked();
            return action;
        }
        self.menu = Some(SpectatorMenu::new(sources));
        None
    }

    /// `onHotbarActionKeyPressed`: the key opens the menu, or runs the selected slot.
    pub fn on_action_key(&mut self, now: i64, sources: &Sources) -> Option<Action> {
        self.last_selection = now;
        let Some(menu) = self.menu.as_mut() else {
            self.menu = Some(SpectatorMenu::new(sources));
            return None;
        };
        let selected = menu.selected_slot();
        let action = (selected != -1)
            .then(|| menu.select_slot(selected))
            .flatten();
        self.close_if_asked();
        action
    }

    /// `onMouseScrolled`: the next enabled slot in the wheel's direction.
    pub fn on_mouse_scrolled(&mut self, wheel: i32, now: i64) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let usable = |menu: &SpectatorMenu, slot: i32| {
            let item = menu.item(slot);
            item != Item::Empty && item.is_enabled()
        };
        let mut slot = menu.selected_slot() + wheel;
        while (0..=8).contains(&slot) && !usable(menu, slot) {
            slot += wheel;
        }
        if (0..=8).contains(&slot) {
            menu.select_slot(slot);
            self.last_selection = now;
        }
    }

    /// `getHotbarAlpha`: opaque for five seconds, then two seconds of fade.
    pub fn alpha(&self, now: i64) -> f32 {
        let delta = self.last_selection - now + FADE_OUT_DELAY;
        (delta as f32 / FADE_OUT_TIME).clamp(0.0, 1.0)
    }

    /// `extractHotbar`'s exit: a menu that has faded out closes.
    pub fn close_faded(&mut self, now: i64) {
        if self.menu.is_some() && self.alpha(now) <= 0.0 {
            self.close();
        }
    }

    fn close_if_asked(&mut self) {
        if self.menu.as_ref().is_some_and(SpectatorMenu::is_closed) {
            self.close();
        }
    }

    /// `onSpectatorMenuClosed`.
    fn close(&mut self) {
        self.menu = None;
        self.last_selection = 0;
    }
}
