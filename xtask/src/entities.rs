//! `cargo xtask entities`: every entity type the reference registers a renderer for, with what the
//! sources say of it, written to `.data/reports/entities.md`.
//!
//! "Named" counts the files of the client crates that spell the type's id or bare name as a
//! string literal; a type nobody names cannot be drawn on purpose. It is a lower bound for the
//! gaps, not proof that a type is drawn right: a shared renderer names its types in a table.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::process;

/// The reference's renderer registry, below `.data/minecraft/source`.
pub(crate) const REGISTRY: &str = "net/minecraft/client/renderer/entity/EntityRenderers.java";
/// Where the client's behaviour lives; the generated tables list every type and prove nothing.
pub(crate) const CLIENT: &[&str] = &[
    "crates/photon/src",
    "crates/photon-render/src",
    "crates/photon-world/src",
];
const CASES: &str = "parity/cases/src";

/// The `register(EntityTypes.X, Renderer...)` pairs, in the registry's order.
pub(crate) fn registered(source: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = source;
    while let Some(at) = rest.find("EntityTypes.") {
        rest = &rest[at + "EntityTypes.".len()..];
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
            .collect();
        let after = &rest[name.len()..];
        let class: String = after
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .find(|word| word.ends_with("Renderer") || word.ends_with("Renderer::new"))
            .unwrap_or("?")
            .to_owned();
        if !name.is_empty() && source_before_register(source, &name) {
            out.push((name.to_lowercase(), class));
        }
    }
    out.dedup_by(|a, b| a.0 == b.0);
    out
}

/// Whether the first use of the constant is a `register` call (not a layer or a comment).
fn source_before_register(source: &str, name: &str) -> bool {
    source.contains(&format!("EntityTypes.{name},"))
        || source.contains(&format!("EntityTypes.{name})"))
}

pub(crate) fn rust_files(dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.extend(fs::read_to_string(&path));
        }
    }
}

pub(crate) fn corpus(dirs: &[&str]) -> Vec<String> {
    let root = process::root();
    let mut out = Vec::new();
    for dir in dirs {
        rust_files(&root.join(dir), &mut out);
    }
    out
}

/// Whether `text` holds `needle` as a whole word: not part of a longer name.
pub(crate) fn has_word(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + needle.len()..].chars().next();
        let word = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
        !word(before) && !word(after)
    })
}

/// How many files spell the type as `minecraft:name`, as a quoted bare `name` (alone or before a
/// model layer's `#`), or as the hyphenated case name `snow-golem`.
pub(crate) fn mentions(files: &[String], name: &str) -> usize {
    let spellings = [
        format!("minecraft:{name}"),
        format!("\"{name}"),
        name.replace('_', "-"),
    ];
    files
        .iter()
        .filter(|file| spellings.iter().any(|spelling| has_word(file, spelling)))
        .count()
}

pub fn run(_args: &[String]) -> Result<(), String> {
    let root = process::root();
    let path = root.join(".data/minecraft/source").join(REGISTRY);
    let source = fs::read_to_string(&path).map_err(|error| {
        format!(
            "{}: {error} (run `cargo xtask setup --named --decompile`)",
            path.display()
        )
    })?;
    let (client, cases) = (corpus(CLIENT), corpus(&[CASES]));
    let mut table = BTreeMap::new();
    for (name, class) in registered(&source) {
        table.insert(
            name.clone(),
            (class, mentions(&client, &name), mentions(&cases, &name)),
        );
    }
    let mut report =
        String::from("| type | renderer | named in client | named in cases |\n|---|---|---|---|\n");
    let mut unnamed = Vec::new();
    let uncased = table
        .values()
        .filter(|(_, _, in_cases)| *in_cases == 0)
        .count();
    for (name, (class, in_client, in_cases)) in &table {
        report.push_str(&format!(
            "| {name} | {class} | {in_client} | {in_cases} |\n"
        ));
        if *in_client == 0 && class != "NoopRenderer" {
            unnamed.push(name.as_str());
        }
    }
    let dir = root.join(".data/reports");
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    fs::write(dir.join("entities.md"), &report).map_err(|error| error.to_string())?;
    println!(
        "{} types with a renderer in the reference; report in .data/reports/entities.md",
        table.len()
    );
    println!("named by no parity case: {uncased}");
    println!(
        "not named by the client ({}): {}",
        unnamed.len(),
        unnamed.join(", ")
    );
    Ok(())
}
