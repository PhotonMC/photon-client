//! Metrics read from the reference's decompiled source. They print counts only; which packet,
//! screen or type is missing stays in `cargo xtask entities` and the issues.
//!
//! The client side is name-based: a thing is "named" when the client sources spell its identifier.
//! That is a lower bound for what exists, not proof that it behaves right (parity proves that).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::local::group;
use crate::entities::{self, CLIENT, has_word, mentions, registered};
use crate::process;

const SOURCE: &str = ".data/minecraft/source/net/minecraft";
/// Crates that read and write the play packets.
const WIRE: &[&str] = &["crates/photon-net/src", "crates/photon-protocol/src"];
/// Crates that draw screens, besides `CLIENT`.
const SCREENS: &[&str] = &["crates/photon-gui/src", "crates/photon/src"];

fn source(relative: &str) -> Result<String, String> {
    let path = process::root().join(SOURCE).join(relative);
    fs::read_to_string(&path).map_err(|error| {
        format!(
            "{}: {error} (run `cargo xtask setup --named --decompile`)",
            path.display()
        )
    })
}

fn row(label: &str, done: usize, total: usize) -> (String, String) {
    (
        label.to_owned(),
        format!("{} / {}", group(done), group(total)),
    )
}

/// The text between `NAME = ` and the closing `);` of the next template.
fn template<'a>(text: &'a str, name: &str) -> Result<&'a str, String> {
    let from = text
        .find(&format!("{name} = "))
        .ok_or(format!("GameProtocols has no {name}"))?;
    let rest = &text[from..];
    Ok(&rest[..rest.find("\n    );").unwrap_or(rest.len())])
}

/// The packet type names of one direction (`CLIENTBOUND_ADD_ENTITY`), the bundle packet once.
fn packet_types(block: &str, direction: &str) -> BTreeSet<String> {
    block
        .match_indices("PacketTypes.")
        .map(|(at, found)| {
            block[at + found.len()..]
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect::<String>()
        })
        .filter(|name| name.starts_with(direction))
        .collect()
}

/// The text of the `.rs` files below `dirs`, without test directories.
fn wire_text(dirs: &[&str]) -> String {
    let mut files = Vec::new();
    for dir in dirs {
        entities::rust_files(&process::root().join(dir), &mut files);
    }
    files.join("\n")
}

/// Packets of one direction that the connection code names, and the direction's total.
fn packets(wire: &str, direction: &str, template: &str) -> Result<(usize, usize), String> {
    let types = packet_types(template, direction);
    let constants: BTreeSet<String> = types
        .iter()
        .map(|name| format!("PLAY_{}_PACKET_ID", constant_stem(name)))
        .collect();
    let prefix = format!("PLAY_{direction}_");
    let unknown = defined(wire, &prefix)
        .into_iter()
        .filter(|name| !constants.contains(name))
        .count();
    if unknown > 0 {
        return Err(format!(
            "{unknown} {direction} id constants match no packet of the reference"
        ));
    }
    let used = constants.iter().filter(|name| is_used(wire, name)).count();
    Ok((used, types.len()))
}

/// The part of our id constant's name that differs from the reference's packet type name: the
/// bundle packet is the delimiter in our code, and three names differ.
fn constant_stem(packet_type: &str) -> &str {
    match packet_type {
        "CLIENTBOUND_BUNDLE" => "CLIENTBOUND_BUNDLE_DELIMITER",
        "CLIENTBOUND_LEVEL_CHUNK_WITH_LIGHT" => "CLIENTBOUND_CHUNK",
        "CLIENTBOUND_MOVE_MINECART_ALONG_TRACK" => "CLIENTBOUND_MOVE_MINECART",
        "SERVERBOUND_BUNDLE_ITEM_SELECTED" => "SERVERBOUND_SELECT_BUNDLE_ITEM",
        other => other,
    }
}

/// The `const PLAY_<direction>_*_PACKET_ID` names defined in `text`.
fn defined(text: &str, prefix: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.split("const ").nth(1))
        .map(|rest| rest.split(':').next().unwrap_or_default().trim().to_owned())
        .filter(|name| name.starts_with(prefix) && name.ends_with("_PACKET_ID"))
        .collect()
}

/// Whether `name` appears in `text` on a line that does not define it.
fn is_used(text: &str, name: &str) -> bool {
    text.lines()
        .filter(|line| !line.contains(&format!("const {name}")))
        .any(|line| has_word(line, name))
}

fn packet_rows() -> Result<Vec<(String, String)>, String> {
    let text = source("network/protocol/game/GameProtocols.java")?;
    let wire = wire_text(WIRE);
    let (clientbound, serverbound) = (
        template(&text, "CLIENTBOUND_TEMPLATE")?,
        template(&text, "SERVERBOUND_TEMPLATE")?,
    );
    let (done, total) = packets(&wire, "CLIENTBOUND", clientbound)?;
    let mut rows = vec![row("Clientbound play packets handled", done, total)];
    let (done, total) = packets(&wire, "SERVERBOUND", serverbound)?;
    rows.push(row("Serverbound play packets sent", done, total));
    Ok(rows)
}

/// The constants a `public static final` line declares, for `type_marker` declarations.
fn declared(text: &str, type_marker: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.contains("public static final") && line.contains(type_marker))
        .filter_map(|line| line.split(" = ").next()?.split_whitespace().last())
        .map(str::to_lowercase)
        .collect()
}

/// `register(Owner.NAME,` names in registration order.
fn registrations(text: &str, owner: &str) -> BTreeSet<String> {
    text.match_indices(&format!("{owner}."))
        .map(|(at, found)| {
            text[at + found.len()..]
                .chars()
                .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|name| !name.is_empty())
        .collect()
}

fn named(client: &[String], names: &[String]) -> usize {
    names
        .iter()
        .filter(|name| mentions(client, name) > 0)
        .count()
}

fn type_rows() -> Result<Vec<(String, String)>, String> {
    let client = entities::corpus(CLIENT);
    let with_renderer: Vec<String> =
        registered(&source("client/renderer/entity/EntityRenderers.java")?)
            .into_iter()
            .map(|(name, _)| name)
            .collect();
    let block_renderers: Vec<String> = registrations(
        &source("client/renderer/blockentity/BlockEntityRenderers.java")?,
        "BlockEntityTypes",
    )
    .into_iter()
    .collect();
    let particles = declared(
        &source("core/particles/ParticleTypes.java")?,
        "ParticleType",
    );
    Ok(vec![
        row(
            "Entity types with a renderer, named by the client",
            named(&client, &with_renderer),
            with_renderer.len(),
        ),
        row(
            "Block entity types with a renderer, named by the client",
            named(&client, &block_renderers),
            block_renderers.len(),
        ),
        row(
            "Particle types named by the client",
            named(&client, &particles),
            particles.len(),
        ),
    ])
}

/// Every non-abstract class below `client/gui/screens` that extends `Screen`, directly or not.
fn screen_classes() -> Result<Vec<String>, String> {
    let mut parents = BTreeMap::new();
    let mut abstracts = BTreeSet::new();
    let mut files = Vec::new();
    collect(
        &process::root().join(SOURCE).join("client/gui/screens"),
        &mut files,
    );
    for file in files {
        let text = fs::read_to_string(&file).map_err(|error| error.to_string())?;
        for line in text.lines() {
            if let Some((class, parent)) = declaration(line) {
                if line.contains("abstract class") {
                    abstracts.insert(class.clone());
                }
                parents.insert(class, parent);
            }
        }
    }
    let extends_screen = |class: &String| {
        let mut at = class.clone();
        for _ in 0..16 {
            match parents.get(&at) {
                Some(parent) if parent == "Screen" => return true,
                Some(parent) => at = parent.clone(),
                None => return false,
            }
        }
        false
    };
    Ok(parents
        .keys()
        .filter(|class| !abstracts.contains(*class) && extends_screen(class))
        .cloned()
        .collect())
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "java") {
            out.push(path);
        }
    }
}

/// `(class, parent)` of a `class X extends Y` declaration line, generics removed.
fn declaration(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    let after = line.split("class ").nth(1)?;
    let (class, rest) = after.split_once(" extends ")?;
    let class = class.split('<').next()?.trim();
    let parent = rest
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()?;
    (!class.is_empty() && !parent.is_empty()).then(|| (class.to_owned(), parent.to_owned()))
}

fn screen_rows() -> Result<Vec<(String, String)>, String> {
    let classes = screen_classes()?;
    let mut dirs: Vec<&str> = CLIENT.to_vec();
    dirs.extend(SCREENS);
    let text = entities::corpus(&dirs);
    let done = classes
        .iter()
        .filter(|class| text.iter().any(|file| has_word(file, class)))
        .count();
    Ok(vec![row(
        "Screens named by the client",
        done,
        classes.len(),
    )])
}

/// Packets, screens, renderers and particles against the reference.
pub fn rows() -> Result<Vec<(String, String)>, String> {
    let mut rows = packet_rows()?;
    rows.extend(screen_rows()?);
    rows.extend(type_rows()?);
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_packet_types_of_one_direction() {
        let block = "withBundlePacket(GamePacketTypes.CLIENTBOUND_BUNDLE, X::new)\n\
            .addPacket(CommonPacketTypes.CLIENTBOUND_KEEP_ALIVE, Y)\n\
            .addPacket(GamePacketTypes.SERVERBOUND_CHAT, Z)";
        let types = packet_types(block, "CLIENTBOUND");
        assert_eq!(
            types.into_iter().collect::<Vec<_>>(),
            ["CLIENTBOUND_BUNDLE", "CLIENTBOUND_KEEP_ALIVE"]
        );
    }

    #[test]
    fn template_ends_at_the_closing_line() {
        let text = "A = x\n  .add(PacketTypes.A_ONE)\n    );\nB = y\n";
        assert_eq!(
            template(text, "A").unwrap(),
            "A = x\n  .add(PacketTypes.A_ONE)"
        );
        assert!(template(text, "C").is_err());
    }

    #[test]
    fn use_means_a_line_that_does_not_define() {
        let defined_only = "pub const PLAY_X_PACKET_ID: i32 = 1;";
        assert!(!is_used(defined_only, "PLAY_X_PACKET_ID"));
        let with_use = format!("{defined_only}\nmatch id {{ PLAY_X_PACKET_ID => {{}} }}");
        assert!(is_used(&with_use, "PLAY_X_PACKET_ID"));
    }

    #[test]
    fn reads_class_declarations() {
        let line = "public class AbcScreen<T> extends Other<T> implements Q {";
        assert_eq!(
            declaration(line),
            Some(("AbcScreen".to_owned(), "Other".to_owned()))
        );
        assert_eq!(declaration("public class Plain {"), None);
    }
}
