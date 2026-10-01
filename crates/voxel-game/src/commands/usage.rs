//! What the chat box shows of a command being typed
//! (`CommandSuggestions.formatText` and `updateUsageInfo`): the text in
//! colours by what read it, and the usage or error lines under it.

use super::{CommandTree, NodeKind, ParseResults};
use crate::text::Component;

/// `ARGUMENT_STYLES`: aqua, yellow, green, light purple, gold.
const ARGUMENT_COLOURS: [u32; 5] = [0x55_FFFF, 0xFF_FF55, 0x55_FF55, 0xFF_55FF, 0xFF_AA00];
const LITERAL_COLOUR: u32 = 0xAA_AAAA;
const UNPARSED_COLOUR: u32 = 0xFF_5555;

/// `formatText`: the visible `text`, which starts `offset` characters into
/// the box, cut into parts of one colour each.
pub fn format_text(parse: &ParseResults, text: &str, offset: usize) -> Vec<(String, u32)> {
    let chars: Vec<char> = text.chars().collect();
    let slice = |from: usize, to: usize| -> String { chars[from..to].iter().collect() };
    let mut parts: Vec<(String, u32)> = Vec::new();
    let mut unformatted = 0;
    let mut next_colour = ARGUMENT_COLOURS.len() - 1;
    for (_, range) in &parse.context.last_child().arguments {
        next_colour = (next_colour + 1) % ARGUMENT_COLOURS.len();
        let start = (range.start - offset as i32).max(0) as usize;
        if start >= chars.len() {
            break;
        }
        let end = (range.end - offset as i32).min(chars.len() as i32);
        if end > 0 {
            let end = end as usize;
            parts.push((slice(unformatted, start), LITERAL_COLOUR));
            parts.push((slice(start, end), ARGUMENT_COLOURS[next_colour]));
            unformatted = end;
        }
    }
    if parse.can_read() {
        let start = (parse.cursor() as i32 - offset as i32).max(0) as usize;
        if start < chars.len() {
            let end = (start + parse.remaining_length()).min(chars.len());
            parts.push((slice(unformatted, start), LITERAL_COLOUR));
            parts.push((slice(start, end), UNPARSED_COLOUR));
            unformatted = end;
        }
    }
    parts.push((slice(unformatted, chars.len()), LITERAL_COLOUR));
    parts.retain(|(text, _)| !text.is_empty());
    parts
}

/// `getExceptionMessage`.
fn exception_line(error: &super::CommandSyntaxException) -> Component {
    match error.context_text() {
        None => error.message.clone(),
        Some(context) => Component::translatable_with(
            "command.context.parse_error",
            vec![
                error.message.clone(),
                Component::literal(error.cursor().to_string()),
                Component::literal(context),
            ],
        ),
    }
}

/// The lines under the box, and the text position they stand under.
pub struct CommandSuggestionsView {
    pub lines: Vec<Component>,
    pub start: i32,
}

/// `updateUsageInfo`: error lines when nothing is suggested at the end of
/// text that does not read, otherwise the usage of what could come next.
pub fn usage_lines(
    tree: &CommandTree,
    parse: &ParseResults,
    cursor: usize,
    suggestions_empty: bool,
) -> CommandSuggestionsView {
    let mut lines: Vec<Component> = Vec::new();
    let mut trailing = false;
    if cursor == parse.text().chars().count() {
        if suggestions_empty && !parse.exceptions.is_empty() {
            let mut literals = 0;
            for (_, error) in &parse.exceptions {
                if error.literal_incorrect {
                    literals += 1;
                } else {
                    lines.push(exception_line(error));
                }
            }
            if literals > 0 {
                let mut reader = super::StringReader::new(&parse.text());
                reader.set_cursor(parse.cursor());
                lines.push(exception_line(&super::CommandSyntaxException::at(
                    &reader,
                    "command.unknown.argument",
                    &[],
                )));
            }
        } else if parse.can_read() {
            trailing = true;
        }
    }
    let (parent, start) = parse.context.suggestion_context(cursor as i32);
    if lines.is_empty() {
        let entries: Vec<Component> = tree
            .smart_usage(parent)
            .into_iter()
            .filter(|(node, _)| !matches!(tree.node(*node).kind, NodeKind::Literal(_)))
            .map(|(_, usage)| Component::literal(usage).with_colour("gray"))
            .collect();
        if entries.is_empty()
            && trailing
            && let Some(error) = parse.parse_exception()
        {
            lines.push(exception_line(&error));
        }
        lines.extend(entries);
    }
    CommandSuggestionsView { lines, start }
}
