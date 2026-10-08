//! `BundleMouseActions` for a container screen: the wheel over a bundle in a
//! slot selects another of its items, and leaving the slot selects none. Each
//! choice becomes a [`ScreenAction::SelectBundleItem`], which the session
//! applies to the stack and tells the server of.

use super::ScreenAction;
use photon_game::menus::container::{ContainerMenu, bundle_items_to_show};

/// `ScrollWheelHandler`: the wheel's movement, kept until it makes a whole
/// step.
#[derive(Debug, Clone, Default)]
struct ScrollWheel {
    x: f64,
    y: f64,
}

impl ScrollWheel {
    /// `onMouseScroll`: the whole steps of the movement so far; a change of
    /// direction drops what was kept.
    fn scrolled(&mut self, x: f64, y: f64) -> (i32, i32) {
        if self.x != 0.0 && java_signum(x) != java_signum(self.x) {
            self.x = 0.0;
        }
        if self.y != 0.0 && java_signum(y) != java_signum(self.y) {
            self.y = 0.0;
        }
        self.x += x;
        self.y += y;
        let (whole_x, whole_y) = (self.x as i32, self.y as i32);
        self.x -= f64::from(whole_x);
        self.y -= f64::from(whole_y);
        (whole_x, whole_y)
    }
}

/// `Math.signum`: zero for zero (Rust's gives one).
fn java_signum(value: f64) -> f64 {
    if value == 0.0 { value } else { value.signum() }
}

/// `ScrollWheelHandler.getNextScrollWheelSelection`: one step against the
/// wheel, wrapping within `limit`.
fn next_selection(wheel: i32, current: i32, limit: i32) -> i32 {
    let mut selected = (current - wheel.signum()).max(-1);
    while selected < 0 {
        selected += limit;
    }
    while selected >= limit {
        selected -= limit;
    }
    selected
}

/// The bundle selections a container screen asks for.
#[derive(Debug, Clone, Default)]
pub struct BundleMouse {
    wheel: ScrollWheel,
    queued: Vec<ScreenAction>,
}

impl BundleMouse {
    /// `onMouseScrolled` over the hovered slot: whether the wheel was the
    /// bundle's. A bundle showing nothing leaves the wheel to the screen.
    pub fn scrolled(&mut self, y: f64, hovered: Option<usize>, menu: &ContainerMenu) -> bool {
        let Some(slot) = hovered.filter(|&slot| menu.is_bundle_slot(slot)) else {
            return false;
        };
        let stack = menu.slots[slot].as_ref().expect("a bundle is there");
        let shown = bundle_items_to_show(stack);
        if shown == 0 {
            return false;
        }
        let (wheel_x, wheel_y) = self.wheel.scrolled(0.0, y);
        let wheel = if wheel_y == 0 { -wheel_x } else { wheel_y };
        if wheel != 0 {
            let selected = stack.bundle_selected();
            let next = next_selection(wheel, selected, shown);
            if next != selected {
                self.select(slot, next);
            }
        }
        true
    }

    /// `onStopHovering`: the slot the mouse left selects none of its bundle's
    /// items.
    pub fn hover_changed(
        &mut self,
        previous: Option<usize>,
        now: Option<usize>,
        menu: &ContainerMenu,
    ) {
        if let Some(slot) = previous
            && previous != now
            && menu.is_bundle_slot(slot)
        {
            self.select(slot, -1);
        }
    }

    fn select(&mut self, slot: usize, selected: i32) {
        self.queued.push(ScreenAction::SelectBundleItem {
            slot: slot as i32,
            selected,
        });
    }

    /// The selections asked for since the last call.
    pub fn drain(&mut self) -> Vec<ScreenAction> {
        std::mem::take(&mut self.queued)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_steps_against_its_direction_and_wraps() {
        assert_eq!(next_selection(1, -1, 4), 3);
        assert_eq!(next_selection(1, 0, 4), 3);
        assert_eq!(next_selection(-1, 3, 4), 0);
        assert_eq!(next_selection(-1, -1, 4), 0);
    }

    #[test]
    fn part_steps_are_kept_until_whole() {
        let mut wheel = ScrollWheel::default();
        assert_eq!(wheel.scrolled(0.0, 0.5), (0, 0));
        assert_eq!(wheel.scrolled(0.0, 0.5), (0, 1));
        assert_eq!(wheel.scrolled(0.0, -0.5), (0, 0));
    }
}
