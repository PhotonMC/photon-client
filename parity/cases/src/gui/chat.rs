//! The chat: messages, their tags and fading, the chat options, deletions,
//! hover tooltips and a language change.

use serde_json::{Value, json};

use super::gui;
use crate::ordered::{Ordered, object};

const LONG_TEXT: &str = "this is a fairly long chat message that should wrap onto a second line \
                         when it reaches the edge of the chat box and goes on for a third one too";

/// One chat scenario; `None` and empty fields are left out of the row.
#[derive(Default)]
struct Chat {
    messages: Vec<Value>,
    tick: i64,
    size: Option<(i64, i64)>,
    focused: bool,
    scroll: Option<i64>,
    deletions: Vec<(i64, i64)>,
    options: Option<Value>,
    rescale: Option<Value>,
    lang_after: Option<&'static str>,
    mouse: Option<(i64, i64)>,
    queued: Option<i64>,
    restrictions: Option<Vec<&'static str>>,
    restrictions_after: Option<Vec<&'static str>>,
}

impl Chat {
    fn row(self, label: &str) -> Value {
        let (width, height) = self.size.unwrap_or((427, 240));
        let mut row =
            json!({"label": label, "width": width, "height": height, "messages": self.messages, "tick": self.tick});
        if !self.deletions.is_empty() {
            let deletions: Vec<Value> =
                self.deletions.iter().map(|(signature, at)| json!({"delete": signature, "tick": at})).collect();
            row["deletions"] = json!(deletions);
        }
        let optional = [
            ("focused", self.focused.then_some(json!(true))),
            ("scroll", self.scroll.map(|scroll| json!(scroll))),
            ("options", self.options),
            ("rescale", self.rescale),
            ("lang_after", self.lang_after.map(|language| json!(language))),
            ("mouse", self.mouse.map(|(x, y)| json!([x, y]))),
            ("queued", self.queued.map(|queued| json!(queued))),
            ("restrictions", self.restrictions.map(|names| json!(names))),
            ("restrictions_after", self.restrictions_after.map(|names| json!(names))),
        ];
        for (key, value) in optional {
            if let Some(value) = value {
                row[key] = value;
            }
        }
        row
    }
}

fn texts(messages: &[(&str, i64)]) -> Vec<Value> {
    messages.iter().map(|(text, tick)| json!({"text": text, "tick": tick})).collect()
}

fn numbered(count: i64) -> Vec<Value> {
    (0..count).map(|n| json!({"text": format!("line {n}"), "tick": 0})).collect()
}

fn chat(messages: Vec<Value>, tick: i64) -> Chat {
    Chat { messages, tick, ..Chat::default() }
}

/// Underlined and struck text, which chat draws with a shadow: each effect's shadow copy and the
/// parent's flags reaching a sibling.
fn effect_rows() -> Vec<Value> {
    let one = |component_: Ordered| vec![component(&component_, 0)];
    let under = show("under ").with("underlined", true);
    let both = show("both").with("underlined", true).with("strikethrough", true).with("color", "red");
    vec![
        chat(one(under.clone()), 0).row("effect-underlined"),
        chat(one(show("struck").with("strikethrough", true)), 0).row("effect-struck"),
        chat(one(both.clone()), 0).row("effect-both-coloured"),
        chat(one(under.with("extra", vec![show("child"), both])), 0).row("effect-inherited"),
    ]
}

fn basic_rows() -> Vec<Value> {
    vec![
        chat(texts(&[("hello world", 0)]), 0).row("one-line"),
        chat(texts(&[("one", 0), ("two", 1), ("three", 2)]), 3).row("three-lines"),
        chat(texts(&[(LONG_TEXT, 0)]), 5).row("wrapped"),
        chat(texts(&[("fading out", 0)]), 190).row("fading"),
        chat(texts(&[("gone", 0)]), 200).row("faded"),
        chat(numbered(14), 10).row("many"),
        Chat { size: Some((640, 480)), ..chat(texts(&[("hello", 0)]), 0) }.row("tall-window"),
        chat(texts(&[("first line\nsecond line", 0)]), 0).row("newline"),
        Chat { focused: true, ..chat(numbered(30), 400) }.row("open-unscrolled"),
        Chat { focused: true, scroll: Some(3), ..chat(numbered(30), 400) }.row("open-scrolled"),
        Chat { focused: true, scroll: Some(100), ..chat(numbered(30), 400) }.row("open-scrolled-end"),
        Chat { focused: true, ..chat(texts(&[("only", 0), ("two", 0)]), 400) }.row("open-short"),
    ]
}

fn tagged_rows() -> Vec<Value> {
    let tags = vec![
        json!({"text": "system", "tick": 0, "tag": "system"}),
        json!({"text": "not secure", "tick": 1, "tag": "not_secure", "signature": 1}),
        json!({"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2}),
        json!({"text": "error", "tick": 3, "tag": "error"}),
        json!({"text": "no tag", "tick": 4, "signature": 3}),
    ];
    let around = vec![
        json!({"text": "before", "tick": 0, "tag": "system"}),
        json!({"text": "will go", "tick": 1, "signature": 7}),
        json!({"text": "after", "tick": 2, "tag": "system"}),
    ];
    vec![
        chat(tags, 5).row("tags"),
        chat(vec![json!({"text": LONG_TEXT, "tick": 0, "tag": "modified:x", "signature": 4})], 5)
            .row("modified-wraps-narrower"),
        Chat { deletions: vec![(7, 10)], ..chat(vec![json!({"text": "will go", "tick": 0, "signature": 7})], 30) }
            .row("delete-waits"),
        Chat { deletions: vec![(7, 5)], ..chat(around, 100) }.row("delete-after-sixty"),
        Chat { deletions: vec![(9, 5)], ..chat(vec![json!({"text": "kept", "tick": 0, "signature": 8})], 10) }
            .row("delete-unknown"),
    ]
}

fn with_options(messages: Vec<Value>, tick: i64, options: Value) -> Chat {
    Chat { options: Some(options), ..chat(messages, tick) }
}

fn open(messages: Vec<Value>, options: Value, scroll: Option<i64>) -> Chat {
    Chat { focused: true, scroll, options: Some(options), ..chat(messages, 400) }
}

/// The chat options: each alone and together, closed and open.
fn scale_and_width_rows() -> Vec<Value> {
    let long = || texts(&[(LONG_TEXT, 0)]);
    let tags = vec![
        json!({"text": "system", "tick": 0, "tag": "system"}),
        json!({"text": "modified", "tick": 2, "tag": "modified:the original", "signature": 2}),
    ];
    vec![
        with_options(long(), 3, json!({"scale": 0.5})).row("scale-half"),
        with_options(long(), 3, json!({"scale": 0.75})).row("scale-three-quarters"),
        with_options(texts(&[("hello", 0), ("two", 1)]), 3, json!({"scale": 0.33})).row("scale-third"),
        open(numbered(30), json!({"scale": 0.6}), None).row("scale-open"),
        open(numbered(30), json!({"scale": 0.9}), Some(5)).row("scale-open-scrolled"),
        with_options(tags, 5, json!({"scale": 0.7})).row("scale-tags"),
        with_options(long(), 3, json!({"width": 0.0})).row("width-narrow"),
        with_options(long(), 3, json!({"width": 0.5})).row("width-half"),
        Chat { focused: true, ..with_options(texts(&[(LONG_TEXT, 0); 4]), 3, json!({"width": 0.25})) }
            .row("width-quarter-open"),
        with_options(long(), 3, json!({"width": 0.4, "scale": 0.8})).row("width-and-scale"),
    ]
}

fn opacity_and_spacing_rows() -> Vec<Value> {
    let open_tags = vec![
        json!({"text": "system", "tick": 0, "tag": "system"}),
        json!({"text": "error", "tick": 3, "tag": "error"}),
    ];
    let spacing_tags = vec![
        json!({"text": "system", "tick": 0, "tag": "system"}),
        json!({"text": "no tag", "tick": 4, "signature": 3}),
    ];
    let hello = || texts(&[("hello", 0)]);
    vec![
        with_options(hello(), 3, json!({"opacity": 0.0})).row("opacity-zero"),
        with_options(texts(&[("hello", 0), ("fading", 1)]), 3, json!({"opacity": 0.3})).row("opacity-low"),
        with_options(texts(&[("fading out", 0)]), 190, json!({"opacity": 0.5})).row("opacity-fading"),
        Chat { focused: true, ..with_options(open_tags, 5, json!({"opacity": 0.4})) }.row("opacity-open-tags"),
        with_options(hello(), 3, json!({"background_opacity": 0.0})).row("background-zero"),
        with_options(texts(&[("hello", 0), ("two", 0)]), 3, json!({"background_opacity": 1.0})).row("background-full"),
        with_options(texts(&[("fading out", 0)]), 190, json!({"background_opacity": 0.8})).row("background-fading"),
        with_options(texts(&[("one", 0), ("two", 1), ("three", 2)]), 3, json!({"line_spacing": 1.0}))
            .row("spacing-full"),
        with_options(numbered(14), 10, json!({"line_spacing": 0.25})).row("spacing-quarter"),
        open(numbered(30), json!({"line_spacing": 0.5}), None).row("spacing-open"),
        open(numbered(30), json!({"line_spacing": 0.3}), Some(4)).row("spacing-open-scrolled"),
        with_options(spacing_tags, 5, json!({"line_spacing": 0.6})).row("spacing-tags"),
    ]
}

fn height_rows() -> Vec<Value> {
    let everything = json!({
        "scale": 0.7, "width": 0.6, "opacity": 0.6, "background_opacity": 0.3, "line_spacing": 0.2,
        "height_focused": 0.7,
    });
    let rescaled = |messages: Vec<Value>, rescale: Value| Chat { rescale: Some(rescale), ..chat(messages, 3) };
    vec![
        with_options(numbered(14), 10, json!({"height_unfocused": 0.0})).row("height-unfocused-low"),
        with_options(numbered(30), 10, json!({"height_unfocused": 1.0})).row("height-unfocused-full"),
        open(numbered(30), json!({"height_focused": 0.2}), None).row("height-focused-low"),
        open(numbered(30), json!({"height_focused": 0.5}), Some(7)).row("height-focused-half-scrolled"),
        open(numbered(30), json!({"height_focused": 0.6, "line_spacing": 0.4}), None).row("height-and-spacing"),
        open(numbered(30), json!({"height_focused": 0.3}), Some(100)).row("scroll-end-low-height"),
        open(numbered(30), everything, Some(3)).row("everything"),
        rescaled(texts(&[(LONG_TEXT, 0), ("short", 1)]), json!({"width": 0.3})).row("rescale-width"),
        rescaled(texts(&[(LONG_TEXT, 0)]), json!({"scale": 0.5})).row("rescale-scale"),
        Chat { focused: true, scroll: Some(6), rescale: Some(json!({"opacity": 0.9})), ..chat(numbered(30), 400) }
            .row("rescale-resets-scroll"),
        Chat { options: Some(json!({"width": 0.8})), ..rescaled(texts(&[(LONG_TEXT, 0)]), json!({"width": 0.2})) }
            .row("rescale-after-options"),
        Chat { size: Some((640, 480)), ..with_options(texts(&[("hello", 0)]), 0, json!({"scale": 0.5})) }
            .row("tall-window-scaled"),
    ]
}

/// `chatColors`: the formatting codes that are literal text, kept or stripped as the lines are made.
fn colour_rows() -> Vec<Value> {
    let coded = [("a \u{a7}cred \u{a7}Lbold\u{a7}r plain", 0), ("kept \u{a7}g \u{a7}x end \u{a7}", 1)];
    let wide = "\u{a7}c0123456789 \u{a7}a0123456789 \u{a7}b0123456789 \u{a7}d0123456789 \u{a7}e0123456789 \u{a7}f0123456789 tail";
    let styled = vec![
        json!({"json": "{\"text\":\"red \u{a7}lbold \u{a7}rback\",\"color\":\"red\",\"italic\":true}", "tick": 0}),
    ];
    let pieces = vec![json!({"json": "{\"text\":\"half \u{a7}\",\"extra\":[{\"text\":\"c split\"}]}", "tick": 0})];
    vec![
        with_options(texts(&coded), 3, json!({"colors": true})).row("colours-on"),
        with_options(texts(&coded), 3, json!({"colors": false})).row("colours-off"),
        with_options(texts(&[(wide, 0)]), 3, json!({"colors": false})).row("colours-off-wraps-shorter"),
        with_options(texts(&[(wide, 0)]), 3, json!({"colors": true})).row("colours-on-wraps-wider"),
        with_options(pieces, 3, json!({"colors": false})).row("colours-off-split-pieces"),
        with_options(texts(&[("x \u{a7}\u{a7}cc y \u{a7}\u{a7}cX z", 0)]), 3, json!({"colors": false}))
            .row("colours-off-doubled-sign"),
        with_options(styled, 3, json!({"colors": true})).row("colours-on-reset-to-run-style"),
        Chat { rescale: Some(json!({"colors": false})), ..chat(texts(&coded), 3) }.row("colours-off-on-rescale"),
        Chat {
            options: Some(json!({"colors": false})),
            rescale: Some(json!({"colors": true})),
            ..chat(texts(&coded), 3)
        }
        .row("colours-on-on-rescale"),
    ]
}

/// A text component with `text` and further fields in this order.
fn show(text: &str) -> Ordered {
    object(vec![("text", text.into())])
}

fn tip(value: Ordered) -> Ordered {
    object(vec![("action", "show_text".into()), ("value", value)])
}

fn hover(text: &str, value: impl Into<Ordered>) -> Ordered {
    show(text).with("hover_event", tip(value.into()))
}

/// A message holding `component` as JSON text, at `tick`.
fn component(component: &Ordered, tick: i64) -> Value {
    json!({"json": component.dumps(), "tick": tick})
}

fn hovered(mouse: (i64, i64), messages: Vec<Value>) -> Chat {
    Chat { focused: true, mouse: Some(mouse), ..chat(messages, 5) }
}

/// The mouse over an open chat: the tooltip of the text under it. The last
/// line is at the bottom, its text from x 4 and 192 down.
fn hover_rows() -> Vec<Value> {
    let me = || vec![component(&hover("hover me", "tooltip text"), 0)];
    vec![
        hovered((10, 196), me()).row("hover-show-text"),
        Chat { focused: false, ..hovered((10, 196), me()) }.row("hover-show-text-unfocused"),
        hovered((300, 196), me()).row("hover-miss-right"),
        hovered((10, 150), me()).row("hover-miss-above"),
        hovered((4, 196), me()).row("hover-left-edge"),
        hovered((3, 196), me()).row("hover-before-left-edge"),
        hovered((4 + 6 * 8 - 2, 196), me()).row("hover-last-glyph"),
        hovered((4 + 8 * 6 + 1, 196), me()).row("hover-past-end"),
        hovered((10, 192), me()).row("hover-glyph-top"),
        hovered((10, 200), me()).row("hover-glyph-bottom"),
        hovered((10, 201), me()).row("hover-below-line"),
    ]
}

fn styled_hover_rows() -> Vec<Value> {
    let one = |component_: Ordered| vec![component(&component_, 0)];
    let bold = hover("bold", "bold tip").with("bold", true);
    vec![
        hovered((40, 196), one(show("a").with("extra", vec![bold]))).row("hover-sibling"),
        hovered((20, 196), one(hover("parent ", "parent tip").with("extra", vec![show("child")])))
            .row("hover-inherited"),
        hovered((10, 196), one(show("plain ").with("extra", vec![hover("child", "child tip")])))
            .row("hover-on-child-only"),
        hovered((12, 196), one(hover("slanted", "tooltip text").with("italic", true))).row("hover-italic"),
        hovered((11, 196), one(hover("bold", "tooltip text").with("bold", true))).row("hover-bold-edge"),
        hovered((10, 196), one(hover("hover me", show("red tip").with("color", "red")))).row("hover-coloured-tip"),
        hovered((10, 196), one(hover("hover me", LONG_TEXT.repeat(2)))).row("hover-long-tip"),
        hovered((10, 196), one(hover("hover me", "first\nsecond\nthird"))).row("hover-newline-tip"),
    ]
}

fn placed_hover_rows() -> Vec<Value> {
    let me = || vec![component(&hover("hover me", "tooltip text"), 0)];
    let two = || vec![component(&hover("upper", "upper tip"), 0), component(&hover("lower", "lower tip"), 1)];
    let wrapped = || vec![component(&hover(LONG_TEXT, "wrapped tip"), 0)];
    let tagged_with = |tag: &str| {
        let mut message = component(&hover("tagged", "tooltip text"), 0);
        message["tag"] = json!(tag);
        vec![message]
    };
    let tagged = || tagged_with("system");
    let scaled = |mouse: (i64, i64)| Chat { options: Some(json!({"scale": 0.5})), ..hovered(mouse, me()) };
    vec![
        hovered((10, 187), two()).row("hover-two-lines-upper"),
        hovered((10, 196), two()).row("hover-two-lines-lower"),
        hovered((10, 196), wrapped()).row("hover-wrapped-second-line"),
        hovered((10, 178), wrapped()).row("hover-wrapped-first-line"),
        hovered((10, 196), tagged()).row("hover-with-tag-text"),
        hovered((0, 196), tagged()).row("hover-with-tag-bar"),
        hovered((0, 196), tagged_with("system_single_player")).row("hover-tag-bar-single-player"),
        hovered((0, 196), tagged_with("not_secure")).row("hover-tag-bar-not-secure"),
        hovered((0, 196), tagged_with("modified:the original")).row("hover-tag-bar-modified"),
        hovered((0, 196), tagged_with("error")).row("hover-tag-bar-error"),
        scaled((5, 198)).row("hover-scaled"),
        scaled((5, 150)).row("hover-scaled-miss"),
        Chat { options: Some(json!({"line_spacing": 0.5})), ..hovered((10, 196), me()) }.row("hover-spaced"),
        Chat { size: Some((640, 480)), ..hovered((10, 436), me()) }.row("hover-tall-window"),
        Chat { size: Some((200, 150)), ..hovered((10, 106), vec![component(&hover("hover me", LONG_TEXT), 0)]) }
            .row("hover-narrow-window"),
    ]
}

fn event_hover_rows() -> Vec<Value> {
    let one = |component_: Ordered| vec![component(&component_, 0)];
    let uuid: Vec<Ordered> = [1, 2, 3, 4].into_iter().map(Ordered::Int).collect();
    let entity =
        object(vec![("action", "show_entity".into()), ("id", "minecraft:pig".into()), ("uuid", Ordered::List(uuid))]);
    let link = object(vec![("action", "open_url".into()), ("url", "https://example.com".into())]);
    let last = hover("ab", "first").with("extra", vec![hover("", "second")]);
    vec![
        hovered((4 + 5 * 6 + 1, 196), one(hover("hello world", "tooltip text"))).row("hover-space-glyph"),
        hovered((10, 196), one(show("entity").with("hover_event", entity))).row("hover-entity-no-tip"),
        hovered((10, 196), one(show("link").with("click_event", link))).row("click-only-no-tip"),
        hovered((10, 196), one(last)).row("hover-last-wins"),
        hovered((10, 196), vec![component(&hover("hover me", "tooltip text"), 0), json!({"text": "x", "tick": 1})])
            .row("hover-no-mouse-rows-draw"),
    ]
}

/// Messages held back by the chat delay: the line under the chat that counts them.
fn queue_rows() -> Vec<Value> {
    let held = |messages: Vec<Value>, queued: i64| Chat { queued: Some(queued), ..chat(messages, 3) };
    let open_held = |mouse: (i64, i64)| Chat { focused: true, mouse: Some(mouse), ..held(texts(&[("hello", 0)]), 3) };
    vec![
        held(texts(&[("hello", 0)]), 3).row("queue-line"),
        held(texts(&[("hello", 0)]), 12).row("queue-line-two-digits"),
        held(Vec::new(), 3).row("queue-without-messages"),
        Chat {
            options: Some(json!({"scale": 0.5, "opacity": 0.5, "background_opacity": 0.8})),
            ..held(texts(&[("hello", 0)]), 3)
        }
        .row("queue-line-scaled"),
        Chat { focused: true, ..held(texts(&[("hello", 0)]), 3) }.row("queue-line-open"),
        open_held((10, 203)).row("queue-line-hover"),
        open_held((10, 150)).row("queue-line-mouse-elsewhere"),
    ]
}

const COMMANDS_ONLY: &str = "CHAT_DISABLED_BY_OPTIONS";
const HIDDEN: &str = "CHAT_AND_COMMANDS_DISABLED_BY_OPTIONS";

/// What a player's restrictions do to the chat: the line an open chat shows, and the messages of
/// the kinds they take away.
fn restricted_rows() -> Vec<Value> {
    let restricted = |messages: Vec<Value>, restrictions: &[&'static str]| Chat {
        restrictions: Some(restrictions.to_vec()),
        ..chat(messages, 3)
    };
    let open = |messages: Vec<Value>, restrictions: &[&'static str]| Chat {
        focused: true,
        ..restricted(messages, restrictions)
    };
    let mixed = || {
        vec![
            json!({"text": "from the server", "tick": 0}),
            json!({"text": "<Steve> hello", "tick": 1, "signature": 1}),
            json!({"text": "<Alex> hi", "tick": 1, "tag": "system"}),
        ]
    };
    let narrow = |mouse: Option<(i64, i64)>| Chat {
        options: Some(json!({"width": 0.0})),
        mouse,
        ..open(texts(&[("hello", 0)]), &[HIDDEN])
    };
    vec![
        open(Vec::new(), &[COMMANDS_ONLY]).row("restricted-open-empty"),
        open(texts(&[("hello", 0), ("world", 1)]), &[HIDDEN]).row("restricted-open"),
        restricted(texts(&[("hello", 0)]), &[HIDDEN]).row("restricted-closed"),
        Chat { mouse: Some((10, 195)), ..open(Vec::new(), &[HIDDEN]) }.row("restricted-hover-prompt"),
        Chat { mouse: Some((10, 150)), ..open(Vec::new(), &[HIDDEN]) }.row("restricted-mouse-elsewhere"),
        narrow(None).row("restricted-narrow-clipped"),
        narrow(Some((10, 195))).row("restricted-narrow-hover"),
        Chat {
            options: Some(json!({"scale": 0.5, "line_spacing": 1.0, "background_opacity": 0.8})),
            ..open(texts(&[("hello", 0)]), &[COMMANDS_ONLY])
        }
        .row("restricted-scaled"),
        Chat { scroll: Some(2), ..open(numbered(30), &[COMMANDS_ONLY]) }.row("restricted-scrolled"),
        open(mixed(), &[COMMANDS_ONLY]).row("filter-commands-only"),
        open(mixed(), &[HIDDEN]).row("filter-hidden"),
        open(mixed(), &["DISABLED_BY_PROFILE"]).row("filter-profile"),
        open(mixed(), &["DISABLED_BY_LAUNCHER", COMMANDS_ONLY]).row("filter-launcher"),
        Chat { restrictions_after: Some(vec![HIDDEN]), ..chat(mixed(), 3) }.row("filter-applied-later"),
        Chat { restrictions: Some(Vec::new()), restrictions_after: Some(vec![COMMANDS_ONLY]), ..chat(mixed(), 3) }
            .row("filter-from-none-to-commands-only"),
        Chat { restrictions_after: Some(Vec::new()), ..restricted(mixed(), &[HIDDEN]) }
            .row("filter-lifted-keeps-the-unseen-out"),
    ]
}

/// A language change words the translatable messages again, and wraps them.
fn language_rows() -> Vec<Value> {
    let warning = |tick: i64| json!({"key": "options.languageAccuracyWarning", "tick": tick});
    let changed =
        |messages: Vec<Value>, language: &'static str| Chat { lang_after: Some(language), ..chat(messages, 3) };
    let mut tagged_warning = warning(0);
    tagged_warning["tag"] = json!("system");
    let spanish =
        Chat { focused: true, scroll: Some(3), lang_after: Some("es_es"), ..chat((0..8).map(warning).collect(), 400) };
    vec![
        changed(vec![warning(0)], "en_us").row("lang-same"),
        changed(vec![warning(0), json!({"key": "gui.done", "tick": 1})], "de_de").row("lang-german"),
        changed(vec![warning(0), json!({"text": "plain", "tick": 1})], "fr_fr").row("lang-french"),
        changed(vec![tagged_warning, json!({"key": "menu.quit", "tick": 1, "tag": "error"})], "pt_br")
            .row("lang-portuguese-tags"),
        spanish.row("lang-spanish-open"),
        changed(vec![warning(0)], "xx_xx").row("lang-unknown"),
    ]
}

/// The screens' drawing as the reference's own GUI code records it: the
/// rectangles and the text, in order, with their poses applied.
pub fn chat_case() -> Value {
    let rows = [
        basic_rows(),
        effect_rows(),
        tagged_rows(),
        scale_and_width_rows(),
        opacity_and_spacing_rows(),
        height_rows(),
        hover_rows(),
        styled_hover_rows(),
        placed_hover_rows(),
        event_hover_rows(),
        queue_rows(),
        restricted_rows(),
        language_rows(),
        colour_rows(),
    ]
    .concat();
    gui("chat", "chat", rows)
}
