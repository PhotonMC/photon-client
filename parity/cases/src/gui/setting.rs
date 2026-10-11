//! A settings screen scenario, built by chaining the few fields a row
//! changes, and the input steps those screens take.

use serde_json::{Value, json};

/// One settings screen scenario.
#[derive(Clone)]
pub struct Setting {
    label: String,
    framebuffer: (i64, i64),
    gui_scale: i64,
    in_world: bool,
    mouse: (i64, i64),
    steps: Vec<Value>,
    ms: i64,
    options: Option<Value>,
}

/// A scenario in the default window, in a world.
pub fn at(label: &str) -> Setting {
    Setting {
        label: label.to_owned(),
        framebuffer: (854, 480),
        gui_scale: 2,
        in_world: true,
        mouse: (0, 0),
        steps: vec![],
        ms: 0,
        options: None,
    }
}

impl Setting {
    pub fn window(self, width: i64, height: i64) -> Self {
        Self { framebuffer: (width, height), ..self }
    }

    pub fn scale(self, gui_scale: i64) -> Self {
        Self { gui_scale, ..self }
    }

    pub fn outside(self) -> Self {
        Self { in_world: false, ..self }
    }

    pub fn mouse(self, x: i64, y: i64) -> Self {
        Self { mouse: (x, y), ..self }
    }

    pub fn steps(self, steps: Vec<Value>) -> Self {
        Self { steps, ..self }
    }

    pub fn ms(self, ms: i64) -> Self {
        Self { ms, ..self }
    }

    pub fn options(self, options: Value) -> Self {
        Self { options: Some(options), ..self }
    }

    /// The row; `extra` fields come after the common ones.
    pub fn row(self) -> Value {
        let mut row = json!({
            "label": self.label, "framebuffer": [self.framebuffer.0, self.framebuffer.1], "gui_scale": self.gui_scale,
            "in_world": self.in_world, "mouse": [self.mouse.0, self.mouse.1], "steps": self.steps, "ms": self.ms,
        });
        if let Some(options) = self.options {
            row["options"] = options;
        }
        row
    }
}

pub fn click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "shift": false})
}

pub fn shift_click(x: i64, y: i64) -> Value {
    json!({"do": "click", "x": x, "y": y, "shift": true})
}

pub fn drag(x: i64, y: i64) -> Value {
    json!({"do": "drag", "x": x, "y": y})
}

pub fn release(x: i64, y: i64) -> Value {
    json!({"do": "release", "x": x, "y": y})
}

/// The wheel turned by `dy` (an integer or a fraction, as written) at a
/// point.
pub fn wheel_at(dy: impl Into<Value>, x: i64, y: i64, ctrl: bool) -> Value {
    json!({"do": "scroll", "x": x, "y": y, "dy": dy.into(), "ctrl": ctrl})
}

pub fn wheel(dy: impl Into<Value>) -> Value {
    wheel_at(dy, 200, 100, false)
}

pub fn ctrl_wheel(dy: impl Into<Value>) -> Value {
    wheel_at(dy, 200, 100, true)
}

pub fn frame(ms: i64) -> Value {
    json!({"do": "frame", "ms": ms})
}

pub fn key(name: &str) -> Value {
    json!({"do": "key", "key": name})
}

/// `step` `count` times.
pub fn times(step: Value, count: usize) -> Vec<Value> {
    vec![step; count]
}

/// A click at `from`, a drag to `to` and a release there.
pub fn slide(from: (i64, i64), to: (i64, i64)) -> Vec<Value> {
    vec![click(from.0, from.1), drag(to.0, to.1), release(to.0, to.1)]
}
