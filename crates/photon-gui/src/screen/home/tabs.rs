//! Which tabs of the top bar fit, and how tightly.

use super::HomeAction;

/// The space between two tabs.
pub(super) const GAP: i32 = 2;

/// Space on each tab beside its label, from the roomiest to the tightest.
const PADDINGS: [i32; 4] = [14, 8, 4, 2];

/// A destination: its translation key, its label and what it does (nothing: the open page, or
/// not available yet).
#[derive(Debug, Clone)]
pub(super) struct Tab {
    pub key: &'static str,
    pub label: String,
    pub action: Option<HomeAction>,
}

/// The tabs that fit and the padding they are drawn with.
pub(super) struct Fit {
    pub tabs: Vec<Tab>,
    pub padding: i32,
}

fn used(tabs: &[Tab], width: &dyn Fn(&str) -> i32, padding: i32) -> i32 {
    tabs.iter()
        .map(|tab| width(&tab.label) + padding + GAP)
        .sum()
}

fn tightest(tabs: &[Tab], width: &dyn Fn(&str) -> i32, room: i32) -> Option<i32> {
    PADDINGS
        .into_iter()
        .find(|&padding| used(tabs, width, padding) <= room)
}

/// Fits `tabs` into `room`. Pages that are not available yet drop out first, from the end; then
/// the padding shrinks; then the tabs that are left drop out from the end, keeping the first and
/// the settings. Nothing is wider than `room` unless only those two are left.
pub(super) fn fit(mut tabs: Vec<Tab>, width: &dyn Fn(&str) -> i32, room: i32) -> Fit {
    while used(&tabs, width, PADDINGS[0]) > room {
        match tabs.iter().rposition(|tab| tab.action.is_none()) {
            Some(index) if index > 0 => tabs.remove(index),
            _ => break,
        };
    }
    loop {
        if let Some(padding) = tightest(&tabs, width, room) {
            return Fit { tabs, padding };
        }
        let droppable = tabs
            .iter()
            .rposition(|tab| tab.action != Some(HomeAction::Options));
        match droppable {
            Some(index) if index > 0 => tabs.remove(index),
            _ => {
                let padding = PADDINGS[PADDINGS.len() - 1];
                return Fit { tabs, padding };
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tab(key: &'static str, label: &str, action: Option<HomeAction>) -> Tab {
        Tab {
            key,
            label: label.to_owned(),
            action,
        }
    }

    /// Six units a letter, as a font of a fixed width.
    fn six(text: &str) -> i32 {
        text.chars().count() as i32 * 6
    }

    fn turkish() -> Vec<Tab> {
        vec![
            tab("home", "Ana Sayfa", None),
            tab("servers", "Sunucular", Some(HomeAction::Servers)),
            tab("friends", "Arkadaslar", Some(HomeAction::Friends)),
            tab("avatar", "Karakter", None),
            tab("marketplace", "Pazar", None),
            tab("settings", "Ayarlar", Some(HomeAction::Options)),
        ]
    }

    #[test]
    fn a_wide_window_keeps_every_tab_roomy() {
        let fit = fit(turkish(), &six, 1000);
        assert_eq!((fit.tabs.len(), fit.padding), (6, 14));
    }

    #[test]
    fn unavailable_pages_drop_before_the_padding_shrinks() {
        let room = used(&turkish(), &six, 14) - 1;
        let fit = fit(turkish(), &six, room);
        assert!(fit.tabs.iter().all(|tab| tab.key != "marketplace"));
        assert!(used(&fit.tabs, &six, fit.padding) <= room);
    }

    #[test]
    fn a_narrow_window_never_overlaps_and_keeps_settings() {
        for room in [120, 150, 180, 206, 240] {
            let fit = fit(turkish(), &six, room);
            assert!(used(&fit.tabs, &six, fit.padding) <= room, "room {room}");
            assert!(fit.tabs.iter().any(|tab| tab.key == "settings"));
        }
    }
}
