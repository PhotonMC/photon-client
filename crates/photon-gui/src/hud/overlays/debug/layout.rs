//! How `DebugScreenOverlay.extractRenderState` sorts what the entries add into the two columns.

/// What one entry adds through `DebugScreenDisplayer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    /// `addPriorityLine`: to the column that is not longer than the other, left first.
    Priority(String),
    /// `addLine`: in the block that is split in two halves.
    Regular(String),
    /// `addToGroup`: lines kept together under a group's id, in the order the groups first appear.
    Group(&'static str, Vec<String>),
}

/// The left and the right column, with the empty lines that separate the blocks. `footer` is the
/// overlay's own help text, which is only there while the overlay is up.
pub fn columns(lines: Vec<Line>, footer: Option<[String; 3]>) -> (Vec<String>, Vec<String>) {
    let mut left: Vec<String> = Vec::new();
    let mut right: Vec<String> = Vec::new();
    let mut regular: Vec<String> = Vec::new();
    let mut groups: Vec<(&'static str, Vec<String>)> = Vec::new();
    for line in lines {
        match line {
            Line::Priority(text) => priority(&mut left, &mut right, text),
            Line::Regular(text) => regular.push(text),
            Line::Group(id, text) => match groups.iter_mut().find(|(group, _)| *group == id) {
                Some((_, kept)) => kept.extend(text),
                None => groups.push((id, text)),
            },
        }
    }
    close_block(&mut left);
    close_block(&mut right);
    split_regular(regular, &mut left, &mut right);
    split_groups(groups, &mut left, &mut right);
    if let Some(footer) = footer {
        left.push(String::new());
        left.extend(footer);
    }
    (left, right)
}

/// `addPriorityLine`: left unless the left is longer.
fn priority(left: &mut Vec<String>, right: &mut Vec<String>, text: String) {
    if left.len() > right.len() {
        right.push(text);
    } else {
        left.push(text);
    }
}

/// An empty line after a block that has lines.
fn close_block(column: &mut Vec<String>) {
    if !column.is_empty() {
        column.push(String::new());
    }
}

/// The regular lines, the first half on the left and the rest on the right.
fn split_regular(mut regular: Vec<String>, left: &mut Vec<String>, right: &mut Vec<String>) {
    if regular.is_empty() {
        return;
    }
    let second = regular.split_off(regular.len().div_ceil(2));
    left.extend(regular);
    left.push(String::new());
    if !second.is_empty() {
        right.extend(second);
        right.push(String::new());
    }
}

/// The groups, the first half (by count) on the left, each closed by an empty line.
fn split_groups(
    groups: Vec<(&'static str, Vec<String>)>,
    left: &mut Vec<String>,
    right: &mut Vec<String>,
) {
    let middle = groups.len().div_ceil(2);
    for (index, (_, text)) in groups.into_iter().enumerate() {
        if text.is_empty() {
            continue;
        }
        let column = if index < middle {
            &mut *left
        } else {
            &mut *right
        };
        column.extend(text);
        column.push(String::new());
    }
}
