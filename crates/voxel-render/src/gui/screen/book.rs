//! The book screen (`BookViewScreen`, and the lectern's `LecternScreen` over it): the open page,
//! its number, the page buttons and the buttons under the book. The pages are those of the book
//! in the menu's one slot, the page the menu's data says.

use std::sync::Arc;

use voxel_game::container_menu::{ClickType, ContainerMenu};
use voxel_game::text::{ClickEvent, Component, Lang, Run, Style};

use super::container::ContainerLayout;
use super::{KeyMods, KeyResult, Screen, ScreenAction, ScreenDraw};
use crate::gui::chat::split;
use crate::gui::edit_box::Clipboard;
use crate::gui::font::Font;
use crate::gui::layout::Arena;
use crate::gui::text_hit::style_under;
use crate::gui::widgets::{Kind, Widget};

/// `BookViewScreen.IMAGE_WIDTH`, and the page's text box.
const IMAGE: i32 = 192;
const TEXT_WIDTH: i32 = 114;
const TEXT_LINES: usize = 128 / 9;
/// A page button is 23 by 13.
const PAGE_BUTTON: (i32, i32) = (23, 13);

/// `PAGE_TEXT_STYLE`: black, with no shadow.
fn page_style() -> Style {
    Style {
        colour: Some(0),
        shadow: Some(0),
        ..Style::default()
    }
}

/// What a button of the screen does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Press {
    /// `Done`: the container is closed.
    Done,
    /// `Take Book`: the lectern's button 3.
    Take,
}

pub struct BookScreen {
    size: (i32, i32),
    lang: Arc<Lang>,
    font: Arc<Font>,
    /// `Player.mayBuild`: whether the player may also take the book.
    may_build: bool,
    arena: Arena,
    buttons: Vec<(Widget, Press)>,
}

impl BookScreen {
    pub fn new(width: i32, height: i32, lang: Arc<Lang>, font: Arc<Font>, may_build: bool) -> Self {
        let mut screen = Self {
            size: (width, height),
            lang,
            font,
            may_build,
            arena: Arena::default(),
            buttons: Vec::new(),
        };
        screen.init();
        screen
    }

    fn left(&self) -> i32 {
        (self.size.0 - IMAGE) / 2
    }

    /// `BookViewScreen.backgroundTop`.
    fn top() -> i32 {
        2
    }

    /// `createMenuControls`: Done and Take Book beside each other, or Done alone.
    fn init(&mut self) {
        self.arena = Arena::default();
        self.buttons.clear();
        let y = Self::top() + IMAGE + 2;
        let middle = self.size.0 / 2;
        let add = |screen: &mut Self, x: i32, width: i32, key: &str, press: Press| {
            let rect = screen.arena.add(width, 20);
            screen.arena.0[rect].x = x;
            screen.arena.0[rect].y = y;
            let label = vec![Run {
                text: screen.lang.text(key),
                style: Style::default(),
            }];
            screen.buttons.push((
                Widget {
                    rect,
                    active: true,
                    kind: Kind::Button { label },
                },
                press,
            ));
        };
        if self.may_build {
            add(self, middle - 98 - 2, 98, "gui.done", Press::Done);
            add(self, middle + 2, 98, "lectern.take_book", Press::Take);
        } else {
            add(self, (self.size.0 - 200) / 2, 200, "gui.done", Press::Done);
        }
    }

    /// The pages of the book in the menu (`BookAccess.fromItem`).
    fn pages(menu: &ContainerMenu) -> Vec<Component> {
        menu.slots
            .first()
            .and_then(Option::as_ref)
            .and_then(|book| {
                voxel_game::item_values::Values::new(book.item_registry_id(), book.patch_bytes())
                    .book_pages()
            })
            .unwrap_or_default()
    }

    /// The page the menu's data says, kept in the book (`setPage`).
    fn page(menu: &ContainerMenu, count: usize) -> usize {
        let wanted = i32::from(menu.data.first().copied().unwrap_or(0));
        wanted.clamp(0, (count as i32 - 1).max(0)) as usize
    }

    fn page_button_origin(&self, forward: bool) -> (i32, i32) {
        (
            self.left() + if forward { 116 } else { 43 },
            Self::top() + 157,
        )
    }

    fn over(point: (f64, f64), (x, y): (i32, i32), (w, h): (i32, i32)) -> bool {
        point.0 >= f64::from(x)
            && point.1 >= f64::from(y)
            && point.0 < f64::from(x + w)
            && point.1 < f64::from(y + h)
    }

    /// The lines of the open page, in the page's style (`ComponentUtils.mergeStyles` with black
    /// and no shadow) and split to the text box.
    fn lines(&self, pages: &[Component], page: usize) -> Vec<Vec<Run>> {
        let component = pages.get(page).cloned().unwrap_or_else(Component::empty);
        let parent = page_style();
        split(&self.lang.runs(&component, &parent), TEXT_WIDTH, &self.font)
    }

    /// `BookViewScreen.handleClickEvent` for a change of page: the lectern asks for it unless it
    /// is the page open.
    fn clicked_text(&self, mouse: (f64, f64), menu: &ContainerMenu) -> Option<Vec<ScreenAction>> {
        let pages = Self::pages(menu);
        let page = Self::page(menu, pages.len());
        let (left, top) = (self.left(), Self::top());
        for (index, line) in self.lines(&pages, page).iter().take(TEXT_LINES).enumerate() {
            let origin = (left + 36, top + 30 + index as i32 * 9);
            let style = style_under(&self.font, line, origin, (mouse.0 as f32, mouse.1 as f32));
            if let Some(ClickEvent::ChangePage(target)) = style.and_then(|s| s.click.as_deref()) {
                let target = (target - 1).clamp(0, (pages.len() as i32 - 1).max(0)) as usize;
                return Some(if target == page {
                    Vec::new()
                } else {
                    vec![ScreenAction::Button(100 + target as i32)]
                });
            }
        }
        None
    }
}

impl Screen for BookScreen {
    fn resize(&mut self, width: i32, height: i32) {
        self.size = (width, height);
        self.init();
    }

    fn mouse_clicked(
        &mut self,
        mouse: (f64, f64),
        button: i32,
        _shift: bool,
        _now: std::time::Instant,
        menu: &ContainerMenu,
    ) -> Vec<ScreenAction> {
        if button != 0 {
            return Vec::new();
        }
        // The buttons are the screen's widgets, which take the click first.
        for (widget, press) in &self.buttons {
            if widget.contains(&self.arena, (mouse.0 as i32, mouse.1 as i32)) {
                return vec![match press {
                    Press::Done => ScreenAction::Close,
                    Press::Take => ScreenAction::Button(3),
                }];
            }
        }
        let pages = Self::pages(menu);
        let page = Self::page(menu, pages.len());
        for (forward, visible) in [(true, page + 1 < pages.len()), (false, page > 0)] {
            if visible && Self::over(mouse, self.page_button_origin(forward), PAGE_BUTTON) {
                return vec![ScreenAction::Button(if forward { 2 } else { 1 })];
            }
        }
        self.clicked_text(mouse, menu).unwrap_or_default()
    }

    fn mouse_released(
        &mut self,
        _mouse: (f64, f64),
        _button: i32,
        _shift: bool,
        _menu: &ContainerMenu,
    ) -> Vec<ScreenAction> {
        Vec::new()
    }

    fn mouse_dragged(&mut self, _mouse: (f64, f64), _menu: &ContainerMenu) {}

    /// `keyPressed`: Page Up and Page Down press the page buttons (visible or not); Escape and
    /// the inventory key close the screen.
    fn key_pressed(
        &mut self,
        key: glfw::Key,
        _mods: KeyMods,
        _menu: &ContainerMenu,
        _clipboard: &mut dyn Clipboard,
    ) -> KeyResult {
        match key {
            glfw::Key::Escape => KeyResult::Close,
            glfw::Key::PageUp => KeyResult::Actions(vec![ScreenAction::Button(1)]),
            glfw::Key::PageDown => KeyResult::Actions(vec![ScreenAction::Button(2)]),
            _ => KeyResult::Actions(Vec::new()),
        }
    }

    fn scripted_click(&mut self, slot: i32, button: i32, kind: ClickType) -> ScreenAction {
        ScreenAction::Container(super::SlotClick { slot, button, kind })
    }

    /// `extractRenderState`: the dimmed world and the book, the buttons, then the page's number
    /// and text.
    fn build(&mut self, draw: &mut ScreenDraw<'_, '_>, menu: &ContainerMenu, mouse: (f64, f64)) {
        ContainerLayout::new(self.size.0, self.size.1, (IMAGE, IMAGE)).background(draw);
        let (left, top) = (self.left(), Self::top());
        draw.frame.blit_texture(
            draw.atlas,
            "book",
            (256, 256),
            (0, 0),
            left,
            top,
            IMAGE,
            IMAGE,
        );
        let point = (mouse.0 as i32, mouse.1 as i32);
        for (widget, _) in &self.buttons {
            let highlighted = widget.contains(&self.arena, point);
            widget.draw(draw.frame, draw.atlas, draw.font, &self.arena, highlighted);
        }
        let pages = Self::pages(menu);
        let page = Self::page(menu, pages.len());
        for (forward, visible) in [(true, page + 1 < pages.len()), (false, page > 0)] {
            if !visible {
                continue;
            }
            let (x, y) = self.page_button_origin(forward);
            let hovered = Self::over(mouse, (x, y), PAGE_BUTTON);
            let sprite = match (forward, hovered) {
                (true, true) => "widget/page_forward_highlighted",
                (true, false) => "widget/page_forward",
                (false, true) => "widget/page_backward_highlighted",
                (false, false) => "widget/page_backward",
            };
            draw.frame
                .sprite(draw.atlas, sprite, x, y, PAGE_BUTTON.0, PAGE_BUTTON.1);
        }
        // `Page %s of %s`, right-aligned at the page's number column.
        let indicator = Component::translatable_with(
            "book.pageIndicator",
            vec![
                Component::literal((page + 1).to_string()),
                Component::literal(pages.len().max(1).to_string()),
            ],
        );
        let runs = self.lang.runs(&indicator, &page_style());
        let width = draw.font.width_of_runs(&runs);
        draw.frame.styled_text(
            draw.atlas,
            draw.font,
            &runs,
            left + 148 - width,
            top + 16,
            0xFFFF_FFFF,
            true,
        );
        for (index, line) in self.lines(&pages, page).iter().take(TEXT_LINES).enumerate() {
            draw.frame.styled_text(
                draw.atlas,
                draw.font,
                line,
                left + 36,
                top + 30 + index as i32 * 9,
                0xFFFF_FFFF,
                true,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voxel_game::container_menu::MenuKind;
    use voxel_world::ClientItemStackState as Stack;

    fn book_menu(pages: usize, page: i16) -> ContainerMenu {
        let mut menu = ContainerMenu::for_menu(MenuKind::Lectern, 3, Vec::new(), None, 0);
        let item = voxel_game::item::id_of("minecraft:written_book").unwrap();
        let patch = {
            // One component, `written_book_content`: a title, an author, the pages, resolved.
            let mut out = vec![1, 0];
            voxel_core::wire::write_varint(
                voxel_data::item::component_types::WRITTEN_BOOK_CONTENT,
                &mut out,
            );
            out.extend_from_slice(&[1, b'T', 0, 1, b'A', 0, pages as u8]);
            for _ in 0..pages {
                out.extend_from_slice(&[8, 0, 1, b'x', 0]);
            }
            out.push(1);
            out
        };
        menu.slots[0] = Some(Stack::new(1, item).with_patch_bytes(patch));
        menu.data = vec![page];
        menu
    }

    fn screen(may_build: bool) -> BookScreen {
        BookScreen::new(
            427,
            240,
            Arc::default(),
            Arc::new(Font::default()),
            may_build,
        )
    }

    fn press(screen: &mut BookScreen, menu: &ContainerMenu, at: (f64, f64)) -> Vec<ScreenAction> {
        screen.mouse_clicked(at, 0, false, std::time::Instant::now(), menu)
    }

    #[test]
    fn the_page_buttons_turn_the_page_only_where_there_is_one() {
        let mut screen = screen(true);
        let forward = (f64::from(screen.left() + 116 + 5), 164.0);
        let back = (f64::from(screen.left() + 43 + 5), 164.0);
        let first = book_menu(3, 0);
        assert_eq!(
            press(&mut screen, &first, forward),
            vec![ScreenAction::Button(2)]
        );
        assert!(press(&mut screen, &first, back).is_empty());
        let last = book_menu(3, 2);
        assert!(press(&mut screen, &last, forward).is_empty());
        assert_eq!(
            press(&mut screen, &last, back),
            vec![ScreenAction::Button(1)]
        );
    }

    #[test]
    fn the_buttons_under_the_book_close_it_or_take_the_book() {
        let menu = book_menu(1, 0);
        let mut owner = screen(true);
        let done = (150.0, 205.0);
        let take = (250.0, 205.0);
        assert_eq!(press(&mut owner, &menu, done), vec![ScreenAction::Close]);
        assert_eq!(
            press(&mut owner, &menu, take),
            vec![ScreenAction::Button(3)]
        );
        // Without the right to build there is one wide Done button, and nothing to take.
        let mut visitor = screen(false);
        assert_eq!(press(&mut visitor, &menu, take), vec![ScreenAction::Close]);
    }

    #[test]
    fn the_page_keys_press_the_page_buttons_and_escape_closes() {
        let mut screen = screen(true);
        let menu = book_menu(2, 0);
        struct NoClip;
        impl Clipboard for NoClip {
            fn get(&mut self) -> String {
                String::new()
            }
            fn set(&mut self, _: &str) {}
        }
        let mut clipboard = NoClip;
        let mut key = |key| screen.key_pressed(key, KeyMods::default(), &menu, &mut clipboard);
        assert_eq!(
            key(glfw::Key::PageDown),
            KeyResult::Actions(vec![ScreenAction::Button(2)])
        );
        assert_eq!(
            key(glfw::Key::PageUp),
            KeyResult::Actions(vec![ScreenAction::Button(1)])
        );
        assert_eq!(key(glfw::Key::Escape), KeyResult::Close);
    }

    #[test]
    fn the_page_is_the_menus_kept_inside_the_book() {
        let menu = book_menu(3, 7);
        assert_eq!(BookScreen::page(&menu, BookScreen::pages(&menu).len()), 2);
        assert_eq!(BookScreen::pages(&menu).len(), 3);
        assert_eq!(BookScreen::page(&book_menu(3, -2), 3), 0);
    }

    #[test]
    fn a_book_and_quill_reads_its_pages_as_plain_text() {
        let mut menu = ContainerMenu::for_menu(MenuKind::Lectern, 3, Vec::new(), None, 0);
        let item = voxel_game::item::id_of("minecraft:writable_book").unwrap();
        let mut patch = vec![1, 0];
        voxel_core::wire::write_varint(
            voxel_data::item::component_types::WRITABLE_BOOK_CONTENT,
            &mut patch,
        );
        // Two pages, each a string and no filtered copy.
        patch.extend_from_slice(&[2, 2, b'h', b'i', 0, 1, b'!', 0]);
        menu.slots[0] = Some(Stack::new(1, item).with_patch_bytes(patch));
        let pages = BookScreen::pages(&menu);
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0], Component::literal("hi"));
        assert_eq!(pages[1], Component::literal("!"));
    }
}
