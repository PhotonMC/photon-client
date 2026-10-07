//! The command tree the server sends (`ClientboundCommandsPacket`) and
//! Brigadier's reading of a command against it: which node took which part
//! of the text, what could not be read and why, and the usage a node
//! offers.
//!
//! Argument types are read exactly where their grammar is small (the
//! Brigadier ones, coordinates, closed word sets, identifiers) and by shape
//! where it is large (selectors, block states, items, NBT, components): a
//! balanced stretch up to the next space, which agrees with the reference
//! for well-formed text and is silent about malformed text.

#![expect(
    clippy::result_large_err,
    reason = "a syntax error carries its message and the input it was read from, as in the reference"
)]

pub mod reader;

use std::collections::HashMap;

use crate::text::Component;
pub use reader::{CommandSyntaxException, StringReader, java_double, java_float};

/// `StringArgumentType.StringType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringType {
    Word,
    Phrase,
    Greedy,
}

/// How an argument is read.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentKind {
    Bool,
    Float {
        min: f32,
        max: f32,
    },
    Double {
        min: f64,
        max: f64,
    },
    Integer {
        min: i32,
        max: i32,
    },
    Long {
        min: i64,
        max: i64,
    },
    String(StringType),
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    Rotation,
    Angle,
    Identifier,
    GameMode,
    EntityAnchor,
    Operation,
    ScoreboardSlot,
    TeamColor,
    HexColor,
    Swizzle,
    Uuid,
    Time {
        min: i32,
    },
    /// A selector or a name; `single` takes one result, `players_only` no
    /// other entities.
    Entity {
        single: bool,
        players_only: bool,
    },
    /// `readUnquotedString`.
    Unquoted,
    /// Up to the next space.
    Word,
    /// Up to the next space outside quotes and brackets.
    Balanced,
    /// A score holder: a selector (when the source may use them), a name or
    /// `*`; read by shape as `Balanced` is.
    ScoreHolder,
    /// The rest of the text.
    Greedy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Root,
    Literal(String),
    Argument { name: String, kind: ArgumentKind },
}

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    /// Literals by name, then arguments by name (`CommandNode.children`).
    pub children: Vec<usize>,
    /// Arguments in the order they were added (`CommandNode.arguments`).
    arguments: Vec<usize>,
    literals: HashMap<String, usize>,
    pub redirect: Option<usize>,
    pub executable: bool,
    /// The server answers this argument's suggestions (`ask_server` and the
    /// other custom providers).
    pub custom_suggestions: bool,
    /// The argument type's id on the wire (`COMMAND_ARGUMENT_TYPE`), 0 for
    /// the rest.
    pub type_id: u8,
}

impl Node {
    /// `getUsageText`.
    pub fn usage_text(&self) -> String {
        match &self.kind {
            NodeKind::Root => String::new(),
            NodeKind::Literal(name) => name.clone(),
            NodeKind::Argument { name, .. } => format!("<{name}>"),
        }
    }

    fn sort_key(&self) -> &str {
        match &self.kind {
            NodeKind::Literal(name) | NodeKind::Argument { name, .. } => name,
            NodeKind::Root => "",
        }
    }

    fn is_literal(&self) -> bool {
        matches!(self.kind, NodeKind::Literal(_))
    }
}

#[derive(Debug, Clone)]
pub struct CommandTree {
    nodes: Vec<Node>,
    root: usize,
}

/// A node as the packet carries it, before it is linked.
pub struct NodeStub {
    pub kind: NodeKind,
    pub children: Vec<usize>,
    pub redirect: Option<usize>,
    pub executable: bool,
    pub custom_suggestions: bool,
    pub type_id: u8,
}

impl CommandTree {
    /// Link the packet's entries (`NodeResolver.resolve`); `None` when an
    /// index is out of range.
    pub fn build(stubs: Vec<NodeStub>, root: usize) -> Option<Self> {
        let mut nodes: Vec<Node> = stubs
            .iter()
            .map(|stub| Node {
                kind: stub.kind.clone(),
                children: Vec::new(),
                arguments: Vec::new(),
                literals: HashMap::new(),
                redirect: stub.redirect,
                executable: stub.executable,
                custom_suggestions: stub.custom_suggestions,
                type_id: stub.type_id,
            })
            .collect();
        if root >= nodes.len()
            || stubs
                .iter()
                .any(|stub| stub.redirect.is_some_and(|r| r >= nodes.len()))
        {
            return None;
        }
        for (index, stub) in stubs.iter().enumerate() {
            for &child in &stub.children {
                if child >= nodes.len() {
                    return None;
                }
                if nodes[child].kind == NodeKind::Root {
                    continue;
                }
                let node = &mut nodes[index];
                match &stubs[child].kind {
                    NodeKind::Literal(name) => {
                        node.literals.insert(name.clone(), child);
                    }
                    _ => node.arguments.push(child),
                }
                node.children.push(child);
            }
            let mut children = std::mem::take(&mut nodes[index].children);
            children.sort_by(|a, b| {
                let (a, b) = (&nodes[*a], &nodes[*b]);
                match (a.is_literal(), b.is_literal()) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.sort_key().cmp(b.sort_key()),
                }
            });
            nodes[index].children = children;
        }
        Some(Self { nodes, root })
    }

    pub fn root(&self) -> usize {
        self.root
    }

    pub fn node(&self, index: usize) -> &Node {
        &self.nodes[index]
    }

    /// `getRelevantNodes`: the literal the next word names, or the arguments.
    fn relevant(&self, node: usize, reader: &StringReader) -> Vec<usize> {
        let node = &self.nodes[node];
        if !node.literals.is_empty() {
            let mut end = reader.cursor();
            while end < reader.total_length() && reader.slice(end, end + 1) != " " {
                end += 1;
            }
            let word = reader.slice(reader.cursor(), end);
            if let Some(&literal) = node.literals.get(&word) {
                return vec![literal];
            }
        }
        node.arguments.clone()
    }
}

/// `StringRange`: `end` is past the last character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StringRange {
    pub start: i32,
    pub end: i32,
}

impl StringRange {
    fn encompassing(a: Self, b: Self) -> Self {
        Self {
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }
}

/// `CommandContextBuilder`: what the nodes along one command read.
#[derive(Debug, Clone)]
pub struct Context {
    root: usize,
    pub nodes: Vec<(usize, StringRange)>,
    /// Argument names and ranges, in the order read (a name read again
    /// replaces the earlier one).
    pub arguments: Vec<(String, StringRange)>,
    pub range: StringRange,
    pub child: Option<Box<Context>>,
    /// Whether the last node read ends a command (`getCommand() != null`).
    pub executable: bool,
    /// Whether the source may use entity selectors
    /// (`EntitySelectorParser.allowSelectors`).
    selectors: bool,
}

impl Context {
    fn new(root: usize, start: i32, selectors: bool) -> Self {
        Self {
            selectors,
            root,
            nodes: Vec::new(),
            arguments: Vec::new(),
            range: StringRange { start, end: start },
            child: None,
            executable: false,
        }
    }

    /// `getLastChild`.
    pub fn last_child(&self) -> &Context {
        self.child.as_ref().map_or(self, |child| child.last_child())
    }

    /// `findSuggestionContext`: the node before the cursor, and where the
    /// text it would read starts.
    pub fn suggestion_context(&self, cursor: i32) -> (usize, i32) {
        if self.range.end < cursor {
            if let Some(child) = &self.child {
                return child.suggestion_context(cursor);
            }
            if let Some(&(node, range)) = self.nodes.last() {
                return (node, range.end + 1);
            }
            return (self.root, self.range.start);
        }
        let mut previous = self.root;
        for &(node, range) in &self.nodes {
            if range.start <= cursor && cursor <= range.end {
                return (previous, range.start);
            }
            previous = node;
        }
        (previous, self.range.start)
    }
}

/// `ParseResults`.
#[derive(Debug, Clone)]
pub struct ParseResults {
    pub context: Context,
    reader: StringReader,
    /// Why nodes did not take the text, in the order tried.
    pub exceptions: Vec<(usize, CommandSyntaxException)>,
}

impl ParseResults {
    pub fn can_read(&self) -> bool {
        self.reader.can_read()
    }

    pub fn cursor(&self) -> usize {
        self.reader.cursor()
    }

    pub fn remaining_length(&self) -> usize {
        self.reader.remaining_length()
    }

    pub fn text(&self) -> String {
        self.reader.string()
    }

    /// `Commands.getParseException`.
    pub fn parse_exception(&self) -> Option<CommandSyntaxException> {
        if !self.reader.can_read() {
            None
        } else if self.exceptions.len() == 1 {
            Some(self.exceptions[0].1.clone())
        } else if self.context.range.start == self.context.range.end {
            Some(CommandSyntaxException::at(
                &self.reader,
                "command.unknown.command",
                &[],
            ))
        } else {
            Some(CommandSyntaxException::at(
                &self.reader,
                "command.unknown.argument",
                &[],
            ))
        }
    }
}

impl CommandTree {
    /// `CommandDispatcher.parse` of `text` from `start` (past a slash), for
    /// a source that may use entity selectors or not.
    pub fn parse(&self, text: &str, start: usize, selectors: bool) -> ParseResults {
        let mut reader = StringReader::new(text);
        reader.set_cursor(start);
        let context = Context::new(self.root, start as i32, selectors);
        self.parse_nodes(self.root, &reader, context)
    }

    fn parse_nodes(&self, node: usize, original: &StringReader, so_far: Context) -> ParseResults {
        let mut errors: Vec<(usize, CommandSyntaxException)> = Vec::new();
        let mut potentials: Vec<ParseResults> = Vec::new();
        let cursor = original.cursor();
        for child in self.relevant(node, original) {
            let mut context = so_far.clone();
            let mut reader = original.clone();
            let read = self.nodes[child]
                .read(&mut reader, &mut context, child)
                .and_then(|()| {
                    if reader.can_read() && reader.peek() != ' ' {
                        Err(CommandSyntaxException::at(
                            &reader,
                            "command.expected.separator",
                            &[],
                        ))
                    } else {
                        Ok(())
                    }
                });
            if let Err(error) = read {
                errors.push((child, error));
                continue;
            }
            context.executable = self.nodes[child].executable;
            let redirect = self.nodes[child].redirect;
            if reader.can_read_n(if redirect.is_none() { 2 } else { 1 }) {
                reader.skip();
                if let Some(target) = redirect {
                    let child_context =
                        Context::new(target, reader.cursor() as i32, context.selectors);
                    let parse = self.parse_nodes(target, &reader, child_context);
                    context.child = Some(Box::new(parse.context));
                    return ParseResults {
                        context,
                        reader: parse.reader,
                        exceptions: parse.exceptions,
                    };
                }
                potentials.push(self.parse_nodes(child, &reader, context));
            } else {
                potentials.push(ParseResults {
                    context,
                    reader,
                    exceptions: Vec::new(),
                });
            }
        }
        if !potentials.is_empty() {
            // A stable sort: the first of equals stays first.
            potentials.sort_by(|a, b| {
                (a.reader.can_read(), !a.exceptions.is_empty())
                    .cmp(&(b.reader.can_read(), !b.exceptions.is_empty()))
            });
            return potentials.swap_remove(0);
        }
        let _ = cursor;
        ParseResults {
            context: so_far,
            reader: original.clone(),
            exceptions: errors,
        }
    }

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
        let child_optional = data.executable;
        let (open, close) = if child_optional {
            ('[', ']')
        } else {
            ('(', ')')
        };
        if !deep {
            if let Some(redirect) = data.redirect {
                let target = if redirect == self.root {
                    "...".to_owned()
                } else {
                    format!("-> {}", self.nodes[redirect].usage_text())
                };
                return Some(format!("{own} {target}"));
            }
            let children = &data.children;
            if children.len() == 1 {
                if let Some(usage) =
                    self.smart_usage_of(children[0], child_optional, child_optional)
                {
                    return Some(format!("{own} {usage}"));
                }
            } else if children.len() > 1 {
                let mut usages: Vec<String> = Vec::new();
                for &child in children {
                    if let Some(usage) = self.smart_usage_of(child, child_optional, true)
                        && !usages.contains(&usage)
                    {
                        usages.push(usage);
                    }
                }
                if usages.len() == 1 {
                    let usage = &usages[0];
                    return Some(if child_optional {
                        format!("{own} [{usage}]")
                    } else {
                        format!("{own} {usage}")
                    });
                } else if usages.len() > 1 {
                    let names: Vec<String> = children
                        .iter()
                        .map(|&child| self.nodes[child].usage_text())
                        .collect();
                    return Some(format!("{own} {open}{}{close}", names.join("|")));
                }
            }
        }
        Some(own)
    }
}

impl Node {
    /// `LiteralCommandNode.parse` and `ArgumentCommandNode.parse`.
    fn read(
        &self,
        reader: &mut StringReader,
        context: &mut Context,
        index: usize,
    ) -> Result<(), CommandSyntaxException> {
        let start = reader.cursor();
        match &self.kind {
            NodeKind::Root => Ok(()),
            NodeKind::Literal(literal) => {
                let length = literal.chars().count();
                if reader.can_read_n(length) && reader.slice(start, start + length) == *literal {
                    reader.set_cursor(start + length);
                    if !reader.can_read() || reader.peek() == ' ' {
                        let range = StringRange {
                            start: start as i32,
                            end: (start + length) as i32,
                        };
                        context.add_node(index, range);
                        return Ok(());
                    }
                    reader.set_cursor(start);
                }
                let mut error = CommandSyntaxException::at(
                    reader,
                    "argument.literal.incorrect",
                    &[Component::literal(literal.clone())],
                );
                error.literal_incorrect = true;
                Err(error)
            }
            NodeKind::Argument { name, kind } => {
                kind.read_as(reader, context.selectors)?;
                let range = StringRange {
                    start: start as i32,
                    end: reader.cursor() as i32,
                };
                context.arguments.retain(|(existing, _)| existing != name);
                context.arguments.push((name.clone(), range));
                context.add_node(index, range);
                Ok(())
            }
        }
    }
}

impl Context {
    fn add_node(&mut self, node: usize, range: StringRange) {
        self.nodes.push((node, range));
        self.range = StringRange::encompassing(self.range, range);
    }
}

mod arguments;
mod suggest;
mod usage;
pub use suggest::{LocalSuggestions, SuggestContext, sort_suggestions};
pub use usage::{CommandSuggestionsView, format_text, usage_lines};
