//! `photon.toml` of the game directory: the settings of Photon itself, which the Java client has no
//! counterpart for and `options.txt` therefore does not hold. There is no settings screen.
//!
//! The file reads the part of TOML it needs: `key = value` lines, `[section]` headers, dotted keys
//! and `#` comments. A missing or unreadable file means the defaults, and a bad line is reported and
//! skipped, so a typo never keeps the client from starting.

use std::path::Path;

/// How a frame is built and drawn (ADR 0003, "Addendum: rendering one frame behind").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RenderMode {
    /// Simulation, meshing, vertex building and recording run one after another on one thread.
    /// The reference for the pipelined path.
    Serial,
    /// Vertex building and recording run on a render thread, one frame behind the simulation.
    Pipelined,
}

/// What `photon.toml` holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Settings {
    /// `render.pipelined`; on unless the file turns it off.
    pub render: RenderMode,
}

impl RenderMode {
    /// The name the bench report gives the mode.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Serial => "serial",
            Self::Pipelined => "pipelined",
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render: RenderMode::Pipelined,
        }
    }
}

impl Settings {
    /// The settings of `photon.toml` in the game directory, read once. Both render modes run the
    /// serial path until the render thread exists (#22).
    pub(crate) fn get() -> Self {
        static SETTINGS: std::sync::OnceLock<Settings> = std::sync::OnceLock::new();
        *SETTINGS.get_or_init(|| Self::read(&crate::controls::game_directory().join("photon.toml")))
    }

    fn read(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(text) => Self::parse(&text, |problem| {
                log::warn!("{}: {problem}", path.display());
            }),
            Err(_) => Self::default(),
        }
    }

    /// The settings of `text`; every line that is not understood goes to `problem`.
    pub(crate) fn parse(text: &str, mut problem: impl FnMut(String)) -> Self {
        let mut settings = Self::default();
        let mut section = String::new();
        for (number, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(name) = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
            {
                section = name.trim().to_owned();
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                problem(format!("line {}: expected key = value", number + 1));
                continue;
            };
            let key = key.trim();
            let key = if section.is_empty() {
                key.to_owned()
            } else {
                format!("{section}.{key}")
            };
            match (key.as_str(), value.trim()) {
                ("render.pipelined", "true") => settings.render = RenderMode::Pipelined,
                ("render.pipelined", "false") => settings.render = RenderMode::Serial,
                ("render.pipelined", other) => {
                    problem(format!(
                        "line {}: render.pipelined is {other}, not true or false",
                        number + 1
                    ));
                }
                (other, _) => problem(format!("line {}: unknown setting {other}", number + 1)),
            }
        }
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> (Settings, Vec<String>) {
        let mut problems = Vec::new();
        let settings = Settings::parse(text, |problem| problems.push(problem));
        (settings, problems)
    }

    #[test]
    fn rendering_is_pipelined_unless_the_file_says_otherwise() {
        assert_eq!(parse("").0.render, RenderMode::Pipelined);
        assert_eq!(
            parse("render.pipelined = true").0.render,
            RenderMode::Pipelined
        );
        assert_eq!(
            parse("render.pipelined = false").0.render,
            RenderMode::Serial
        );
    }

    #[test]
    fn a_section_header_prefixes_the_keys_after_it() {
        let (settings, problems) = parse("# render\n[render]\npipelined = false # reference\n");
        assert_eq!(settings.render, RenderMode::Serial);
        assert!(problems.is_empty());
    }

    #[test]
    fn a_bad_line_is_reported_and_the_rest_still_applies() {
        let (settings, problems) =
            parse("render.pipelined = maybe\nnonsense\nrender.speed = 3\nrender.pipelined = false");
        assert_eq!(settings.render, RenderMode::Serial);
        assert_eq!(problems.len(), 3);
        assert!(problems[0].contains("line 1"));
        assert!(problems[2].contains("unknown setting render.speed"));
    }
}
