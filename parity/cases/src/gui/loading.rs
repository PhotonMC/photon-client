//! The loading overlay: the logo and bar over a resource reload, in its fades.

use serde_json::{Value, json};

use super::gui;

/// One overlay scenario: a size, the brand colour, the fade it was made with, the progress it
/// shows and the reload reports, the clock, and where the fades began.
struct Overlay {
    label: String,
    size: (i64, i64),
    dark: bool,
    fade_in: bool,
    progress: f64,
    actual: f64,
    ms: i64,
    fade_in_start: Option<i64>,
    fade_out_start: Option<i64>,
}

fn overlay(label: impl Into<String>) -> Overlay {
    Overlay {
        label: label.into(),
        size: (427, 240),
        dark: false,
        fade_in: false,
        progress: 0.0,
        actual: 0.0,
        ms: 0,
        fade_in_start: None,
        fade_out_start: None,
    }
}

impl Overlay {
    fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "width": self.size.0, "height": self.size.1, "dark": self.dark,
            "fade_in": self.fade_in, "progress": self.progress, "actual": self.actual,
            "ms": self.ms,
        });
        if let Some(start) = self.fade_in_start {
            row["fade_in_start"] = json!(start);
        }
        if let Some(start) = self.fade_out_start {
            row["fade_out_start"] = json!(start);
        }
        row
    }
}

/// The overlay of a pack reload (no fade-in) at several progresses and sizes, in the dark
/// brand colour, and through the fade-in of the first load and the fade-out of both.
pub fn loading_overlay() -> Value {
    let sized = |label: &str, width: i64, height: i64| Overlay { size: (width, height), ..overlay(label) };
    let mut rows = vec![
        overlay("empty"),
        Overlay { progress: 0.5, actual: 0.5, ..overlay("half") },
        Overlay { progress: 0.5, actual: 1.0, ..overlay("smoothing") },
        Overlay { progress: 1.0, actual: 1.0, ..overlay("full") },
        Overlay { dark: true, progress: 0.25, ..overlay("dark") },
        sized("small", 320, 240),
        sized("wide", 854, 480),
        sized("flat", 1000, 300),
        sized("tall", 300, 600),
    ];
    let faded_in = |label: String, ms: i64| Overlay {
        fade_in: true,
        progress: 0.3,
        actual: 0.6,
        ms,
        fade_in_start: Some(0),
        ..overlay(label)
    };
    rows.extend([0, 100, 250, 499, 500, 999, 1000].map(|ms| faded_in(format!("fade-in-{ms}"), ms)));
    rows.push(Overlay { fade_in_start: None, ..faded_in("fade-in-first-frame".into(), 700) });
    let faded_out =
        |label: String, ms: i64| Overlay { progress: 1.0, actual: 1.0, ms, fade_out_start: Some(0), ..overlay(label) };
    rows.extend(
        [0, 250, 500, 999, 1000, 1001, 1250, 1500, 1999, 2000, 2500].map(|ms| faded_out(format!("fade-out-{ms}"), ms)),
    );
    rows.extend([1500, 2000].map(|ms| Overlay {
        fade_in: true,
        dark: true,
        fade_in_start: Some(-1000),
        ..faded_out(format!("fade-in-then-out-{ms}"), ms)
    }));
    gui("loading-overlay", "loading_overlay", rows.into_iter().map(Overlay::row).collect())
}
