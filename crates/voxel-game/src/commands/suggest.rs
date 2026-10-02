//! What the command tree suggests for text at a cursor
//! (`CommandDispatcher.getCompletionSuggestions` on the client's tree): the
//! literals that continue it and the values each argument type lists on its
//! own. Only what needs the server is left to it: arguments with a custom
//! provider (`ask_server`), and the types whose values come from the game's
//! registries (items, blocks, particles, sounds, dimensions, slots,
//! criteria, resources), which the client reads from registries this port
//! does not hold; the server answers those with the same values.

use super::{ArgumentKind, CommandTree, NodeKind, ParseResults, StringReader};

/// What the client knows that suggestions use.
pub struct SuggestContext<'a> {
    /// `getOnlinePlayerNames`.
    pub players: &'a [String],
    /// `getAllTeams`.
    pub teams: &'a [String],
    /// Whether the source may use entity selectors
    /// (`Permissions.COMMANDS_ENTITY_SELECTORS`).
    pub selectors: bool,
}

/// What the tree itself suggests for text up to a cursor, and whether the
/// server has more to add.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalSuggestions {
    /// Where the suggested text starts in the input.
    pub start: usize,
    /// Sorted without regard to case, each once.
    pub matches: Vec<String>,
    /// An argument at the cursor has suggestions only the server knows.
    pub ask_server: bool,
}

/// `SharedSuggestionProvider.matchesSubStr`: the pattern starts the input or
/// follows one of `._/` in it.
fn matches_sub_str(pattern: &str, input: &str) -> bool {
    let input: Vec<char> = input.chars().collect();
    let pattern: Vec<char> = pattern.chars().collect();
    let mut index = 0;
    while !input[index.min(input.len())..].starts_with(&pattern) {
        match input[index.min(input.len())..]
            .iter()
            .position(|c| matches!(c, '.' | '_' | '/'))
        {
            Some(found) => index += found + 1,
            None => return false,
        }
        if index > input.len() {
            return false;
        }
    }
    true
}

/// `Suggestions.create`'s order: case ignored, then as written.
pub fn sort_suggestions(matches: &mut Vec<String>) {
    matches.sort_by(|a, b| {
        let fold = |text: &str| -> Vec<char> {
            text.chars()
                .map(|c| c.to_uppercase().next().unwrap_or(c))
                .map(|c| c.to_lowercase().next().unwrap_or(c))
                .collect()
        };
        fold(a).cmp(&fold(b))
    });
    matches.dedup();
}

/// `SharedSuggestionProvider.suggest(values, builder)`: the values the
/// remaining text may start (or follow a separator in), but not the text
/// itself.
fn suggest_values<S: AsRef<str>>(values: &[S], remaining: &str) -> Vec<String> {
    let lower = remaining.to_lowercase();
    values
        .iter()
        .map(AsRef::as_ref)
        .filter(|value| matches_sub_str(&lower, &value.to_lowercase()) && *value != remaining)
        .map(str::to_owned)
        .collect()
}

/// `String.split(" ")`: trailing empty fields are dropped.
fn split_fields(text: &str) -> Vec<&str> {
    let mut fields: Vec<&str> = text.split(' ').collect();
    while fields.len() > 1 && fields.last() == Some(&"") {
        fields.pop();
    }
    fields
}

/// `suggestCoordinates` and `suggest2DCoordinates`, with the default
/// coordinates (no block looked at): a `~` or a `^` for each part.
fn coordinates(remaining: &str, parts: usize, kind: &ArgumentKind) -> Vec<String> {
    let mark = if remaining.starts_with('^') { "^" } else { "~" };
    let valid = |text: &str| {
        let mut reader = StringReader::new(text);
        kind.read(&mut reader).is_ok()
    };
    let rest = |count: usize| vec![mark; count].join(" ");
    let mut result = Vec::new();
    if remaining.is_empty() {
        let full = rest(parts);
        if valid(&full) {
            result.push(mark.to_owned());
            if parts == 3 {
                result.push(rest(2));
            }
            result.push(full);
        }
    } else {
        let fields = split_fields(remaining);
        if fields.len() == 1 {
            let full = format!("{} {}", fields[0], rest(parts - 1));
            if valid(&full) {
                if parts == 3 {
                    result.push(format!("{} {}", fields[0], rest(1)));
                }
                result.push(full);
            }
        } else if fields.len() == 2 && parts == 3 {
            let full = format!("{} {} {mark}", fields[0], fields[1]);
            if valid(&full) {
                result.push(full);
            }
        }
    }
    suggest_values(&result, remaining)
}

/// What one argument suggests: the values, where they start, and whether
/// the server is to be asked.
struct Argument {
    matches: Vec<String>,
    start: usize,
    ask_server: bool,
}

impl Argument {
    fn here(matches: Vec<String>, start: usize) -> Self {
        Self {
            matches,
            start,
            ask_server: false,
        }
    }

    fn asks() -> Self {
        Self {
            matches: Vec::new(),
            start: 0,
            ask_server: true,
        }
    }
}

const SELECTORS: [&str; 6] = ["@p", "@a", "@r", "@s", "@e", "@n"];

/// The parts of `EntitySelectorParser` that decide what is suggested for a
/// selector or a name. What is typed inside `@x[...]` is the options'
/// (their keys and values); the server is asked for it.
fn entities(remaining: &str, start: usize, ctx: &SuggestContext) -> Argument {
    let chars: Vec<char> = remaining.chars().collect();
    let selectors = || {
        SELECTORS
            .iter()
            .map(|s| (*s).to_owned())
            .collect::<Vec<_>>()
    };
    match chars.first() {
        None => {
            let mut matches = suggest_values(ctx.players, remaining);
            if ctx.selectors {
                matches.extend(selectors());
            }
            Argument::here(matches, start)
        }
        // Without the permission the text is read as a name, to the end.
        Some('@') if ctx.selectors => match chars.get(1) {
            None => Argument::here(selectors(), start),
            Some(kind) if !"pearsn".contains(*kind) => Argument::here(selectors(), start),
            Some(_) if chars.len() == 2 => Argument::here(vec!["[".to_owned()], start + 2),
            Some(_) if chars[2] == '[' => Argument::asks(),
            Some(_) => Argument::here(vec!["[".to_owned()], start + 2),
        },
        Some(_) => Argument::here(suggest_values(ctx.players, remaining), start),
    }
}

/// `TimeArgument.listSuggestions`: the units after a number.
fn time(remaining: &str, start: usize) -> Argument {
    let mut reader = StringReader::new(remaining);
    if reader.read_float().is_err() {
        return Argument::here(Vec::new(), start);
    }
    let offset = reader.cursor();
    let tail = reader.remaining();
    Argument::here(suggest_values(&["d", "s", "t", ""], &tail), start + offset)
}

/// One argument type's suggestions for the text typed of it (`start` is
/// where that text starts).
fn argument(
    type_id: u8,
    kind: &ArgumentKind,
    remaining: &str,
    start: usize,
    ctx: &SuggestContext,
) -> Argument {
    let list = |values: &[&str]| Argument::here(suggest_values(values, remaining), start);
    match type_id {
        // `BoolArgumentType`: the words the text is the start of.
        0 => {
            let lower = remaining.to_lowercase();
            let words = ["true", "false"]
                .iter()
                .filter(|word| word.starts_with(&lower) && **word != remaining)
                .map(|word| (*word).to_owned())
                .collect();
            Argument::here(words, start)
        }
        6 | 7 => entities(remaining, start, ctx),
        8 | 10 => Argument::here(coordinates(remaining, 3, kind), start),
        9 | 11 => Argument::here(coordinates(remaining, 2, kind), start),
        16 => list(&crate::scoreboard::TEAM_COLOURS),
        17 => list(&["F00", "FF0000"]),
        26 => list(&["=", "+=", "-=", "*=", "/=", "%=", "<", ">", "><"]),
        30 => list(&crate::scoreboard::DISPLAY_SLOT_NAMES),
        33 => Argument::here(suggest_values(ctx.teams, remaining), start),
        38 => list(&["feet", "eyes"]),
        42 => list(&["survival", "creative", "adventure", "spectator"]),
        43 => time(remaining, start),
        49 => list(&["none", "left_right", "front_back"]),
        50 => list(&["none", "clockwise_90", "180", "counterclockwise_90"]),
        51 => list(&[
            "world_surface",
            "motion_blocking",
            "motion_blocking_no_leaves",
            "ocean_floor",
        ]),
        // The objective is the scoreboard's, which the server holds, and
        // loot tables, predicates and modifiers likewise; the registries'
        // values (blocks, items, particles, sounds, dimensions, slots,
        // criteria, resources, dialogs) are the client's own in the
        // reference, the server's here.
        12..=15 | 24 | 25 | 27 | 34 | 35 | 41 | 44..=48 | 52..=55 => Argument::asks(),
        _ => Argument::here(Vec::new(), start),
    }
}

impl CommandTree {
    /// `CommandDispatcher.getCompletionSuggestions` for what needs no
    /// server: each child of the node before the cursor lists its own
    /// suggestions, and the lists are merged.
    pub fn suggestions(
        &self,
        parse: &ParseResults,
        cursor: usize,
        ctx: &SuggestContext,
    ) -> LocalSuggestions {
        let (parent, start) = parse.context.suggestion_context(cursor as i32);
        let start = (start.max(0) as usize).min(cursor);
        let text = parse.text();
        let remaining: String = text.chars().skip(start).take(cursor - start).collect();
        let lower = remaining.to_lowercase();
        let mut matches = Vec::new();
        let mut merged_start: Option<usize> = None;
        let mut ask_server = false;
        let mut add = |found: Vec<String>, at: usize| {
            if !found.is_empty() {
                merged_start = Some(merged_start.map_or(at, |existing| existing.min(at)));
                matches.extend(found);
            }
        };
        for &child in &self.nodes[parent].children {
            let node = &self.nodes[child];
            match &node.kind {
                NodeKind::Literal(literal) => {
                    if literal.to_lowercase().starts_with(&lower) && *literal != remaining {
                        add(vec![literal.clone()], start);
                    }
                }
                NodeKind::Argument { .. } if node.custom_suggestions => ask_server = true,
                NodeKind::Argument { kind, .. } => {
                    let found = argument(node.type_id, kind, &remaining, start, ctx);
                    ask_server |= found.ask_server;
                    add(found.matches, found.start);
                }
                NodeKind::Root => {}
            }
        }
        sort_suggestions(&mut matches);
        LocalSuggestions {
            start: merged_start.unwrap_or(start),
            matches,
            ask_server,
        }
    }
}
