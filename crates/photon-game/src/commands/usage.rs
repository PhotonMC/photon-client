//! What the chat box shows of a command being typed
//! (`CommandSuggestions.formatText` and `updateUsageInfo`): the text in
//! colours by what read it, and the usage or error lines under it.

use super::{CommandTree, NodeKind, ParseResults};
use crate::text::Component;

/// `ARGUMENT_STYLES`: aqua, yellow, green, light purple, gold.
const ARGUMENT_COLOURS: [u32; 5] = [0x55_FFFF, 0xFF_FF55, 0x55_FF55, 0xFF_55FF, 0xFF_AA00];
const LITERAL_COLOUR: u32 = 0xAA_AAAA;
const UNPARSED_COLOUR: u32 = 0xFF_5555;

impl CommandTree {
    /// `getSmartUsage`: a line for each child that is not a literal's own
    /// (the caller drops literals).
    pub fn smart_usage(&self, node: usize) -> Vec<(usize, String)> {
        let optional = self.nodes[node].executable;
        self.nodes[node]
            .children
            .iter()
            .filter_map(|&child| {
                self.smart_usage_of(child, optional, false)
                    .map(|usage| (child, usage))
            })
            .collect()
    }

    fn smart_usage_of(&self, node: usize, optional: bool, deep: bool) -> Option<String> {
        let data = &self.nodes[node];
        let own = if optional {
            format!("[{}]", data.usage_text())
        } else {
            data.usage_text()
        };
        if deep {
            return Some(own);
        }
        if let Some(redirect) = data.redirect {
            let target = if redirect == self.root {
                "...".to_owned()
            } else {
                format!("-> {}", self.nodes[redirect].usage_text())
            };
            return Some(format!("{own} {target}"));
        }
        let continued = match data.children.as_slice() {
            [] => None,
            [only] => self
                .smart_usage_of(*only, data.executable, data.executable)
                .map(|usage| format!("{own} {usage}")),
            children => self.branch_usage(children, data.executable, &own),
        };
        Some(continued.unwrap_or(own))
    }

    /// The usage of a node with several children: the one line they share, or their names.
    fn branch_usage(&self, children: &[usize], optional: bool, own: &str) -> Option<String> {
        let mut usages: Vec<String> = Vec::new();
        for &child in children {
            if let Some(usage) = self.smart_usage_of(child, optional, true)
                && !usages.contains(&usage)
            {
                usages.push(usage);
            }
        }
        match usages.as_slice() {
            [] => None,
            [usage] if optional => Some(format!("{own} [{usage}]")),
            [usage] => Some(format!("{own} {usage}")),
            _ => {
                let (open, close) = if optional { ('[', ']') } else { ('(', ')') };
                let names: Vec<String> = children
                    .iter()
                    .map(|&child| self.nodes[child].usage_text())
                    .collect();
                Some(format!("{own} {open}{}{close}", names.join("|")))
            }
        }
    }
}

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

/// `CommandSuggestions.hasMessageArguments`: whether the command read has a message in it
/// (`minecraft:message`, which the tree keeps as the rest of the text).
pub fn has_message_arguments(tree: &CommandTree, parse: &ParseResults) -> bool {
    message_in_contexts(tree, parse, false)
}

/// `SignableCommand.hasSignableArguments`: the same, but what comes after a redirect to the root
/// (`execute run ...`) is left out, the server being unable to take a signed message there.
pub fn has_signable_arguments(tree: &CommandTree, parse: &ParseResults) -> bool {
    message_in_contexts(tree, parse, true)
}

/// `ArgumentVisitor.visitArguments` looking for a message argument.
fn message_in_contexts(
    tree: &CommandTree,
    parse: &ParseResults,
    reject_root_redirects: bool,
) -> bool {
    let root = parse.context.root_node();
    let mut context = Some(&parse.context);
    while let Some(each) = context {
        if reject_root_redirects && !std::ptr::eq(each, &parse.context) && each.root_node() == root
        {
            break;
        }
        let found = each.nodes.iter().any(|(node, _)| {
            matches!(
                &tree.node(*node).kind,
                NodeKind::Argument { name, kind: super::ArgumentKind::Greedy }
                    if each.arguments.iter().any(|(read, _)| read == name)
            )
        });
        if found {
            return true;
        }
        context = each.child.as_deref();
    }
    false
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

#[cfg(test)]
mod tests {
    use super::super::SuggestContext;
    use super::super::{ArgumentKind, CommandTree, NodeKind, NodeStub};
    use super::*;

    fn tree() -> CommandTree {
        let stubs = vec![
            NodeStub {
                kind: NodeKind::Root,
                children: vec![1],
                redirect: None,
                executable: false,
                custom_suggestions: false,
                restricted: false,
                type_id: 0,
            },
            NodeStub {
                kind: NodeKind::Literal("give".into()),
                children: vec![2],
                redirect: None,
                executable: false,
                custom_suggestions: false,
                restricted: false,
                type_id: 0,
            },
            NodeStub {
                kind: NodeKind::Argument {
                    name: "who".into(),
                    kind: ArgumentKind::Word,
                },
                children: vec![3],
                redirect: None,
                executable: false,
                custom_suggestions: false,
                restricted: false,
                type_id: 24,
            },
            NodeStub {
                kind: NodeKind::Argument {
                    name: "n".into(),
                    kind: ArgumentKind::Integer { min: 1, max: 9 },
                },
                children: vec![],
                redirect: None,
                executable: true,
                custom_suggestions: false,
                restricted: false,
                type_id: 0,
            },
        ];
        CommandTree::build(stubs, 0).unwrap()
    }

    /// Whatever the box shows, the parts put back together are the text.
    #[test]
    fn formatting_any_stretch_keeps_the_text() {
        let tree = tree();
        for text in [
            "/",
            "/g",
            "/give",
            "/give ",
            "/give a",
            "/give a ",
            "/give a 5",
            "/give a x",
            "/give a 5 z",
            "/zzz",
            "/give  a",
        ] {
            let parse = tree.parse(text, 1, true);
            let chars: Vec<char> = text.chars().collect();
            for cursor in 0..=chars.len() {
                for display in 0..=cursor {
                    let half: String = chars[display..cursor].iter().collect();
                    let rest: String = chars[cursor..].iter().collect();
                    let first: String = format_text(&parse, &half, display)
                        .into_iter()
                        .map(|p| p.0)
                        .collect();
                    assert_eq!(first, half, "{text:?} first {display}..{cursor}");
                    let second: String = format_text(&parse, &rest, cursor)
                        .into_iter()
                        .map(|p| p.0)
                        .collect();
                    assert_eq!(second, rest, "{text:?} rest from {cursor}");
                }
            }
        }
    }

    fn context() -> SuggestContext<'static> {
        SuggestContext {
            selectors: true,
            players: &[],
            teams: &[],
            selected_entities: &[],
        }
    }

    /// The tree suggests its own literals, case ignored, and leaves out what
    /// it does not hold (a command the player may not use).
    #[test]
    fn the_tree_suggests_only_what_it_holds() {
        let tree = tree();
        let parse = tree.parse("/G", 1, true);
        let suggested = tree.suggestions(&parse, 2, &context());
        assert_eq!(suggested.matches, ["give"]);
        assert!(!suggested.ask_server);
        // After "give " the argument is an objective, which the server holds.
        let parse = tree.parse("/give ", 1, true);
        let suggested = tree.suggestions(&parse, 6, &context());
        assert!(suggested.ask_server);
        assert!(suggested.matches.is_empty());
        assert_eq!(
            tree.suggestions(&tree.parse("/zz", 1, true), 3, &context())
                .matches,
            Vec::<String>::new()
        );
    }
}
