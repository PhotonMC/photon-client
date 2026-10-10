//! Line-level facts about a Rust source file, without parsing it.

/// For every line (index 0 = line 1), whether it holds anything besides
/// whitespace and comments. Lines inside a multi-line string count as code.
pub fn code_lines(text: &str) -> Vec<bool> {
    let mut scanner = Scanner {
        chars: text.chars().collect(),
        at: 0,
        lines: vec![false],
    };
    scanner.scan();
    scanner.lines
}

/// The 1-based numbers of lines that contain a non-ASCII character.
pub fn non_ascii_lines(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.is_ascii())
        .map(|(index, _)| index + 1)
        .collect()
}

struct Scanner {
    chars: Vec<char>,
    at: usize,
    lines: Vec<bool>,
}

impl Scanner {
    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.at + offset).copied()
    }

    /// Consumes one character, starting a new line entry after a newline.
    fn bump(&mut self, code: bool) {
        let Some(current) = self.peek(0) else { return };
        self.at += 1;
        if current == '\n' {
            self.lines.push(false);
        } else if code && !current.is_whitespace() {
            *self.lines.last_mut().expect("never empty") = true;
        }
    }

    fn scan(&mut self) {
        while let Some(current) = self.peek(0) {
            match (current, self.peek(1)) {
                ('/', Some('/')) => self.line_comment(),
                ('/', Some('*')) => self.block_comment(),
                ('"', _) => self.string(),
                ('r', Some('"' | '#')) if self.raw_string_ahead() => self.raw_string(),
                ('\'', _) => self.quote(),
                _ => self.bump(true),
            }
        }
    }

    fn line_comment(&mut self) {
        while self.peek(0).is_some_and(|current| current != '\n') {
            self.bump(false);
        }
    }

    fn block_comment(&mut self) {
        let mut depth = 0usize;
        while let Some(current) = self.peek(0) {
            match (current, self.peek(1)) {
                ('/', Some('*')) => {
                    depth += 1;
                    self.bump(false);
                }
                ('*', Some('/')) => {
                    depth -= 1;
                    self.bump(false);
                    if depth == 0 {
                        self.bump(false);
                        return;
                    }
                }
                _ => {}
            }
            self.bump(false);
        }
    }

    fn string(&mut self) {
        self.bump(true);
        while let Some(current) = self.peek(0) {
            self.bump(true);
            match current {
                '\\' => self.bump(true),
                '"' => return,
                _ => {}
            }
        }
    }

    fn raw_string_ahead(&self) -> bool {
        let mut offset = 1;
        while self.peek(offset) == Some('#') {
            offset += 1;
        }
        // `r#ident` is a raw identifier, not a string.
        self.peek(offset) == Some('"')
    }

    fn raw_string(&mut self) {
        self.bump(true);
        let mut hashes = 0;
        while self.peek(0) == Some('#') {
            hashes += 1;
            self.bump(true);
        }
        self.bump(true);
        while let Some(current) = self.peek(0) {
            self.bump(true);
            if current == '"' && (0..hashes).all(|offset| self.peek(offset) == Some('#')) {
                (0..hashes).for_each(|_| self.bump(true));
                return;
            }
        }
    }

    /// A character literal (`'a'`, `'\n'`) or a lifetime (`'a`).
    fn quote(&mut self) {
        self.bump(true);
        let escaped = self.peek(0) == Some('\\');
        if !escaped && self.peek(1) != Some('\'') {
            return;
        }
        while let Some(current) = self.peek(0) {
            self.bump(true);
            match current {
                '\\' => self.bump(true),
                '\'' => return,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(text: &str) -> usize {
        code_lines(text).into_iter().filter(|code| *code).count()
    }

    #[test]
    fn comments_and_blank_lines_are_not_code() {
        let text = "// a\n\n/* b\n c */\nfn f() {} // d\n/// doc\n";
        assert_eq!(count(text), 1);
    }

    #[test]
    fn comment_markers_inside_strings_and_chars_are_code() {
        let text =
            "let a = \"// not\n/* a comment\";\nlet b = '\"';\nlet c: &'a str = r#\"/*\"#;\n";
        assert_eq!(count(text), 4);
    }

    #[test]
    fn nested_block_comments_end_at_the_outer_close() {
        assert_eq!(count("/* a /* b */ c */ x\n"), 1);
        assert_eq!(count("/* a /* b */ c */\n"), 0);
    }
}
