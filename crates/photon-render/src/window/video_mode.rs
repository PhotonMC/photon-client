//! A monitor's video mode, as the fullscreen resolution option stores and shows it.

use std::fmt;

/// One video mode of a monitor: its size, colour depth and refresh rate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoMode {
    pub width: i32,
    pub height: i32,
    pub red_bits: i32,
    pub green_bits: i32,
    pub blue_bits: i32,
    pub refresh_rate: i32,
}

impl VideoMode {
    /// The bits of all three colour channels together.
    pub fn bits(&self) -> i32 {
        self.red_bits + self.green_bits + self.blue_bits
    }

    /// `VideoMode.read`: `WIDTHxHEIGHT`, optionally `@RATE` and then `:BITS`; a missing rate is 60
    /// and missing bits are 24, split evenly over the channels.
    pub fn read(text: &str) -> Option<Self> {
        let (size, rest) = match text.split_once('@') {
            Some((size, rest)) => (size, Some(rest)),
            None => (text, None),
        };
        let (width, height) = size.split_once('x')?;
        let (rate, bits) = match rest {
            None => (None, None),
            Some(rest) => match rest.split_once(':') {
                Some((rate, bits)) => (Some(rate), Some(bits)),
                None => (Some(rest), None),
            },
        };
        let number = |digits: &str| -> Option<i32> {
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            digits.parse().ok()
        };
        let component = number(bits.unwrap_or("24"))? / 3;
        Some(Self {
            width: number(width)?,
            height: number(height)?,
            red_bits: component,
            green_bits: component,
            blue_bits: component,
            refresh_rate: rate.map_or(Some(60), number)?,
        })
    }

    /// `VideoMode.write`: the text `read` takes back.
    pub fn write(&self) -> String {
        format!(
            "{}x{}@{}:{}",
            self.width,
            self.height,
            self.refresh_rate,
            self.bits()
        )
    }
}

impl From<glfw::VidMode> for VideoMode {
    fn from(mode: glfw::VidMode) -> Self {
        let signed = |value: u32| i32::try_from(value).unwrap_or(i32::MAX);
        Self {
            width: signed(mode.width),
            height: signed(mode.height),
            red_bits: signed(mode.red_bits),
            green_bits: signed(mode.green_bits),
            blue_bits: signed(mode.blue_bits),
            refresh_rate: signed(mode.refresh_rate),
        }
    }
}

impl fmt::Display for VideoMode {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "{}x{}@{} ({}bit)",
            self.width,
            self.height,
            self.refresh_rate,
            self.bits()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mode(width: i32, height: i32, rate: i32, bits: i32) -> VideoMode {
        VideoMode {
            width,
            height,
            red_bits: bits / 3,
            green_bits: bits / 3,
            blue_bits: bits / 3,
            refresh_rate: rate,
        }
    }

    #[test]
    fn a_mode_reads_what_it_writes() {
        let wide = mode(2560, 1440, 144, 24);
        assert_eq!(wide.write(), "2560x1440@144:24");
        assert_eq!(VideoMode::read(&wide.write()), Some(wide));
    }

    #[test]
    fn a_short_text_gets_sixty_hertz_and_24_bits() {
        assert_eq!(VideoMode::read("1920x1080"), Some(mode(1920, 1080, 60, 24)));
        assert_eq!(
            VideoMode::read("1920x1080@75"),
            Some(mode(1920, 1080, 75, 24))
        );
    }

    #[test]
    fn a_text_that_is_no_mode_reads_as_none() {
        for text in [
            "",
            "1920",
            "1920x",
            "ax1080",
            "1920x1080@",
            "1920x1080@60:",
            "1920x1080@-1",
        ] {
            assert_eq!(VideoMode::read(text), None, "{text}");
        }
    }

    #[test]
    fn the_name_is_size_rate_and_bits() {
        assert_eq!(mode(1280, 720, 60, 24).to_string(), "1280x720@60 (24bit)");
    }
}
