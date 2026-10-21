//! The book and quill's editing and signing screens, and a written book's
//! view.

use serde_json::{Value, json};

use super::gui;

fn type_text(text: &str) -> Value {
    json!({"do": "type", "text": text})
}

fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

fn keys(name: &str, count: usize) -> Vec<Value> {
    vec![key(name); count]
}

fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y})
}

fn shift_click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "shift": true})
}

/// A book scenario: `extra` are the fields only some screens have.
#[derive(Clone)]
struct Book {
    label: &'static str,
    pages: Vec<Value>,
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: Option<i64>,
}

fn row(label: &'static str, pages: &[&str]) -> Book {
    Book {
        label,
        pages: pages.iter().map(|page| json!(page)).collect(),
        size: (427, 240),
        mouse: (0, 0),
        steps: vec![],
        ms: Some(0),
    }
}

impl Book {
    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "pages": self.pages, "width": self.size.0, "height": self.size.1,
            "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps,
        });
        if let Some(ms) = self.ms {
            row["ms"] = json!(ms);
        }
        row
    }
}

const TWO: &[&str] = &["Hello world", "Second page"];

fn with(label: &'static str, pages: &[&str], steps: Vec<Value>) -> Book {
    Book { steps, ..row(label, pages) }
}

fn on_two(label: &'static str, steps: Vec<Value>) -> Book {
    with(label, TWO, steps)
}

fn on_empty(label: &'static str, steps: Vec<Value>) -> Book {
    with(label, &[], steps)
}

fn long() -> Value {
    type_text("The quick brown fox jumps over the lazy dog and keeps on running far away")
}

fn hover(label: &'static str, x: i64, y: i64, steps: Vec<Value>) -> Book {
    Book { mouse: (x, y), ..on_two(label, steps) }
}

fn typing_rows() -> Vec<Book> {
    vec![
        row("empty", &[]),
        Book { size: (320, 240), ..row("empty small", &[]) },
        Book { size: (640, 360), ..row("empty wide", &[]) },
        row("written", TWO),
        Book { ms: Some(300), ..row("written blink", TWO) },
        Book { ms: Some(650), ..row("written blink back", TWO) },
        on_empty("typed", vec![type_text("Hi there")]),
        on_empty("wrapped", vec![long()]),
        on_empty("one long word", vec![type_text(&"W".repeat(40))]),
        on_empty("new lines", vec![type_text("one"), key("enter"), key("enter"), type_text("three")]),
        on_empty("line limit", [keys("enter", 16), vec![type_text("x")]].concat()),
    ]
}

fn cursor_rows() -> Vec<Book> {
    vec![
        on_two("select all", vec![key("ctrl+a")]),
        on_two("select left", keys("shift+left", 3)),
        on_empty("select lines", vec![long(), key("shift+up"), key("shift+up")]),
        on_two("select typed over", vec![key("ctrl+a"), type_text("new")]),
        on_two("cursor inside", keys("left", 2)),
        on_empty("cursor up", vec![long(), key("up")]),
        on_empty("cursor up down", vec![long(), key("up"), key("up"), key("down")]),
        on_empty("home", vec![long(), key("home")]),
        on_empty("ctrl home", vec![long(), key("ctrl+home")]),
        on_empty("end", vec![long(), key("ctrl+home"), key("end")]),
        on_two("word left", vec![key("ctrl+left"), type_text("X")]),
        on_two("word right", vec![key("ctrl+home"), key("ctrl+right"), type_text("X")]),
        on_two("word erased", vec![key("ctrl+backspace")]),
        on_two("backspace", keys("backspace", 2)),
        on_two("delete", vec![key("ctrl+home"), key("delete")]),
        on_empty("section sign refused", vec![type_text("a\u{a7}b")]),
    ]
}

fn page_rows() -> Vec<Book> {
    let overflow = format!("{}end", "line\n".repeat(20));
    vec![
        on_two("page down new", keys("pagedown", 2)),
        on_two("page down up", vec![key("pagedown"), key("pageup")]),
        on_two("page up first", vec![key("pageup")]),
        on_two("forward clicked", vec![click(240, 164)]),
        on_two("forward clicked twice", vec![click(240, 164); 3]),
        on_two("back clicked", vec![click(240, 164), click(167, 164)]),
        hover("hover forward", 240, 164, vec![]),
        hover("hover back", 167, 164, vec![key("pagedown")]),
        hover("hover sign", 150, 205, vec![]),
        hover("hover done", 260, 205, vec![]),
        on_two("tab", keys("tab", 1)),
        on_two("tab twice", keys("tab", 2)),
        on_two("tab thrice", keys("tab", 3)),
        on_two("tab round", keys("tab", 4)),
        on_two("shift tab", vec![key("shift+tab")]),
        on_two("tab back", vec![key("pagedown"), key("tab")]),
        on_two("tab enter", vec![key("tab"), key("enter")]),
        on_two("tab space", vec![key("tab"), key("space")]),
        on_two("tab typed ignored", vec![key("tab"), type_text("zz")]),
        on_two("click text", vec![click(165, 34)]),
        on_empty("click second line", vec![long(), click(170, 43)]),
        on_two("click below", vec![click(170, 120)]),
        on_two("shift click", vec![key("ctrl+home"), shift_click(175, 34)]),
        Book { ms: Some(100), ..on_two("click back focus", vec![key("tab"), key("tab"), click(165, 34)]) },
        with("overflowing page", &[&overflow], vec![]),
        with("overflowing page up", &[&overflow], vec![key("ctrl+home")]),
    ]
}

fn edit_rows() -> Vec<Book> {
    [typing_rows(), cursor_rows(), page_rows()].concat()
}

/// The book and quill's screen: its pages typed, wrapped and broken by
/// Enter up to the page's fourteen lines, the cursor moved by line, word
/// and click and blinking, selections over one line and several, pages
/// turned and added by the buttons and Page Up/Down, the focus moved by
/// Tab, a page longer than the box scrolled to its cursor.
pub fn book_edit_screen() -> Value {
    gui("book-edit-screen", "book_edit_screen", edit_rows().into_iter().map(Book::row).collect())
}

/// A signing scenario: the title kept, whether the screen is shown again.
struct Title {
    label: &'static str,
    title: &'static str,
    reopened: bool,
    size: (i64, i64),
    mouse: (i64, i64),
    steps: Vec<Value>,
}

fn title(label: &'static str) -> Title {
    Title { label, title: "", reopened: false, size: (427, 240), mouse: (0, 0), steps: vec![] }
}

impl Title {
    fn row(self) -> Value {
        json!({
            "label": self.label, "pages": ["Some text"], "title": self.title, "reopened": self.reopened,
            "width": self.size.0, "height": self.size.1, "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps,
        })
    }
}

/// The signing screen: the title typed, centred and cut to fifteen
/// characters, Sign and Close awake only for a title that is not blank, the
/// cursor and selections in the title, the focus moved by Tab and clicks, a
/// title kept from before on a screen shown again (`rebuildWidgets`).
pub fn book_sign_screen() -> Value {
    let with = |label: &'static str, steps: Vec<Value>| Title { steps, ..title(label) };
    let book = |label: &'static str, steps: Vec<Value>| Title { title: "Book", steps, ..title(label) };
    let my_book = || type_text("My Book");
    let rows = vec![
        title("empty"),
        Title { size: (320, 240), ..title("empty small") },
        Title { size: (640, 360), ..title("empty wide") },
        with("typed", vec![my_book()]),
        with("blank", vec![type_text("   ")]),
        with("too long", vec![type_text("A title that is far too long")]),
        with("section sign refused", vec![type_text("a\u{a7}b")]),
        with("cursor inside", vec![my_book(), key("left"), key("left")]),
        with("select all", vec![my_book(), key("ctrl+a")]),
        with("select left", vec![my_book(), key("shift+left"), key("shift+left")]),
        with("erased", vec![my_book(), key("backspace"), key("ctrl+backspace")]),
        Title { title: "Kept title", ..title("kept") },
        Title { title: "Kept title", reopened: true, ..title("reopened") },
        Title { title: "Kept", reopened: true, steps: vec![type_text("zz")], ..title("reopened typed ignored") },
        Title {
            title: "Kept",
            reopened: true,
            steps: vec![click(213, 52), type_text("!")],
            ..title("reopened clicked")
        },
        Title { title: "Kept title", steps: vec![click(205, 52)], ..title("clicked inside") },
        with("tab inactive", vec![key("tab")]),
        with("tab active", vec![type_text("Book"), key("tab")]),
        with("tab twice", vec![type_text("Book"), key("tab"), key("tab")]),
        with("tab round", vec![type_text("Book"), key("tab"), key("tab"), key("tab")]),
        with("shift tab", vec![type_text("Book"), key("shift+tab")]),
        Title { mouse: (150, 205), ..book("hover finalize", vec![]) },
        Title { mouse: (150, 205), ..title("hover finalize inactive") },
        Title { mouse: (260, 205), ..title("hover cancel") },
    ];
    gui("book-sign-screen", "book_sign_screen", rows.into_iter().map(Title::row).collect())
}

/// A written book the server opens in the hand: its page and number, the
/// page buttons where there is a page to turn to, pages turned by clicks
/// and by Page Up/Down, styled and long pages.
pub fn book_view() -> Value {
    let three = || {
        vec![
            json!("First page"),
            json!({"text": "Second", "color": "red"}),
            json!(["", {"text": "Third ", "bold": true}, "page"]),
        ]
    };
    let long_page = vec![json!("The quick brown fox jumps over the lazy dog. ".repeat(8))];
    let view = |label: &'static str, pages: Vec<Value>| Book { pages, ms: None, ..row(label, &[]) };
    let on_three = |label: &'static str, steps: Vec<Value>| Book { steps, ..view(label, three()) };
    let rows = vec![
        view("one page", vec![json!("Only page")]),
        view("three pages", three()),
        Book { size: (320, 240), ..view("small", three()) },
        Book { size: (640, 360), ..view("wide", three()) },
        on_three("forward clicked", vec![click(240, 164)]),
        on_three("forward twice", vec![click(240, 164), click(240, 164)]),
        on_three("forward past last", vec![click(240, 164); 4]),
        on_three("forward back", vec![click(240, 164), click(240, 164), click(167, 164)]),
        on_three("page down", vec![key("pagedown")]),
        on_three("page down up", vec![key("pagedown"), key("pagedown"), key("pageup")]),
        on_three("page up first", vec![key("pageup")]),
        Book { mouse: (240, 164), ..view("hover forward", three()) },
        Book { mouse: (213, 205), ..view("hover done", three()) },
        view("long page", long_page),
        view("empty book", vec![json!("")]),
        view(
            "effects",
            vec![json!({"text": "under ", "underlined": true, "extra": [
                {"text": "child"}, {"text": " both", "strikethrough": true, "color": "red"}
            ]})],
        ),
    ];
    gui("book-view", "book_view", rows.into_iter().map(Book::row).collect())
}
