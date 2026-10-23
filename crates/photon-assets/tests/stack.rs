//! The stack's lookups against packs built here, as zips and as folders.

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use photon_assets::{DirectorySource, Pack, PackError, PackFormat, ResourceStack, ZipSource};
use photon_core::identifier::Identifier;

const CURRENT: PackFormat = PackFormat::of(88, 0);
const META: &str = r#"{"pack":{"description":"test","min_format":84,"max_format":88}}"#;

fn scratch(name: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let number = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "photon-assets-{}-{number}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch folder");
    dir
}

fn id(text: &str) -> Identifier {
    Identifier::parse(text).expect("an identifier")
}

/// A zip pack holding `files`, with a valid `pack.mcmeta` unless the list names one.
fn zip_pack(name: &str, files: &[(&str, &str)]) -> Pack {
    let path = scratch(name).join("pack.zip");
    let mut writer = zip::ZipWriter::new(File::create(&path).expect("a zip file"));
    let options = zip::write::SimpleFileOptions::default();
    let mut all = files.to_vec();
    if !all.iter().any(|(file, _)| *file == "pack.mcmeta") {
        all.push(("pack.mcmeta", META));
    }
    for (file, text) in all {
        writer.start_file(file, options).expect("an entry");
        writer.write_all(text.as_bytes()).expect("entry bytes");
    }
    writer.finish().expect("a finished zip");
    let source = ZipSource::open(&path).expect("a readable zip");
    Pack::open(name, Box::new(source), CURRENT).expect("a valid pack")
}

fn text(stack: &ResourceStack, resource: &str) -> Option<String> {
    let bytes = stack.open(&id(resource))?.bytes()?;
    String::from_utf8(bytes).ok()
}

#[test]
fn the_topmost_pack_with_the_file_wins() {
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/minecraft/a.txt", "low"),
            ("assets/minecraft/b.txt", "only low"),
        ],
    );
    let top = zip_pack("top", &[("assets/minecraft/a.txt", "high")]);
    let stack = ResourceStack::new(vec![bottom, top]);
    assert_eq!(text(&stack, "minecraft:a.txt").as_deref(), Some("high"));
    assert_eq!(text(&stack, "minecraft:b.txt").as_deref(), Some("only low"));
    assert_eq!(text(&stack, "minecraft:missing.txt"), None);
    assert_eq!(text(&stack, "other:a.txt"), None);
    assert_eq!(
        stack
            .open(&id("minecraft:a.txt"))
            .map(|r| r.pack_id().to_owned())
            .as_deref(),
        Some("top")
    );
}

#[test]
fn the_stack_of_a_file_runs_from_the_bottom_up() {
    let bottom = zip_pack("one", &[("assets/minecraft/lang/en.json", "1")]);
    let middle = zip_pack("two", &[("assets/minecraft/other", "x")]);
    let top = zip_pack("three", &[("assets/minecraft/lang/en.json", "3")]);
    let stack = ResourceStack::new(vec![bottom, middle, top]);
    let found = stack.stack(&id("minecraft:lang/en.json"));
    let packs: Vec<&str> = found.iter().map(|r| r.pack_id()).collect();
    assert_eq!(packs, ["one", "three"]);
}

#[test]
fn a_filter_hides_the_packs_below_but_not_its_own_files() {
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/minecraft/textures/a.png", "low"),
            ("assets/minecraft/sounds/s.ogg", "low"),
        ],
    );
    let filter = r#"{"pack":{"description":"f"},"filter":{"block":[{"namespace":"minecraft","path":"textures/.*"}]}}"#;
    let top = zip_pack(
        "top",
        &[
            ("pack.mcmeta", filter),
            ("assets/minecraft/textures/b.png", "high"),
        ],
    );
    let stack = ResourceStack::new(vec![bottom, top]);
    assert_eq!(text(&stack, "minecraft:textures/a.png"), None);
    assert_eq!(
        text(&stack, "minecraft:textures/b.png").as_deref(),
        Some("high")
    );
    assert_eq!(
        text(&stack, "minecraft:sounds/s.ogg").as_deref(),
        Some("low")
    );
    let listed = stack.list("textures", |_| true);
    assert_eq!(listed.len(), 1);
}

#[test]
fn a_filter_tests_the_namespace_and_the_path_of_different_patterns() {
    // The reference asks every pattern for the namespace and every pattern for the path, so the
    // path of one pattern hides files of the namespace another pattern named.
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/minecraft/sounds/s.ogg", "low"),
            ("assets/minecraft/b.txt", "low"),
        ],
    );
    let filter = r#"{"pack":{"description":"f"},"filter":{"block":[{"namespace":"minecraft","path":"nothing"},{"namespace":"other","path":"^sounds/"}]}}"#;
    let top = zip_pack(
        "top",
        &[("pack.mcmeta", filter), ("assets/minecraft/z.txt", "high")],
    );
    let stack = ResourceStack::new(vec![bottom, top]);
    assert_eq!(text(&stack, "minecraft:sounds/s.ogg"), None);
    assert_eq!(text(&stack, "minecraft:b.txt").as_deref(), Some("low"));
}

#[test]
fn a_filter_for_a_namespace_the_pack_lacks_still_hides_it() {
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/legacy/x.txt", "low"),
            ("assets/minecraft/y.txt", "low"),
        ],
    );
    let filter = r#"{"pack":{"description":"f"},"filter":{"block":[{"namespace":"legacy"}]}}"#;
    let top = zip_pack(
        "top",
        &[("pack.mcmeta", filter), ("assets/minecraft/z.txt", "high")],
    );
    let stack = ResourceStack::new(vec![bottom, top]);
    assert_eq!(text(&stack, "legacy:x.txt"), None);
    assert_eq!(text(&stack, "minecraft:y.txt").as_deref(), Some("low"));
}

#[test]
fn listing_gives_the_topmost_files_sorted_by_path_then_namespace() {
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/minecraft/models/b.json", "1"),
            ("assets/zeta/models/a.json", "1"),
            ("assets/minecraft/models/a.json", "1"),
        ],
    );
    let top = zip_pack(
        "top",
        &[
            ("assets/minecraft/models/b.json", "2"),
            ("assets/minecraft/other/c.json", "2"),
        ],
    );
    let stack = ResourceStack::new(vec![bottom, top]);
    let listed = stack.list("models", |_| true);
    let names: Vec<String> = listed.keys().map(Identifier::as_string).collect();
    assert_eq!(
        names,
        [
            "minecraft:models/a.json",
            "zeta:models/a.json",
            "minecraft:models/b.json"
        ]
    );
    assert_eq!(listed[&id("minecraft:models/b.json")].pack_id(), "top");
    let only_zeta = stack.list("models", |id| id.namespace() == "zeta");
    assert_eq!(only_zeta.len(), 1);
}

#[test]
fn overlays_apply_to_the_formats_they_name_and_the_last_listed_wins() {
    let meta = r#"{"pack":{"description":"o"},"overlays":{"entries":[
        {"directory":"old","formats":[10,20]},
        {"directory":"new","min_format":80,"max_format":90},
        {"directory":"newer","min_format":[88,0],"max_format":[88,5]}]}}"#;
    let pack = zip_pack(
        "overlaid",
        &[
            ("pack.mcmeta", meta),
            ("assets/minecraft/a.txt", "base"),
            ("old/assets/minecraft/a.txt", "old"),
            ("new/assets/minecraft/a.txt", "new"),
            ("newer/assets/minecraft/a.txt", "newer"),
            ("old/assets/minecraft/c.txt", "old only"),
        ],
    );
    let stack = ResourceStack::new(vec![pack]);
    assert_eq!(text(&stack, "minecraft:a.txt").as_deref(), Some("newer"));
    assert_eq!(text(&stack, "minecraft:c.txt"), None);
    assert_eq!(stack.list("", |_| true).len(), 1);
}

#[test]
fn the_metadata_comes_from_the_topmost_pack_that_has_it() {
    let bottom = zip_pack(
        "bottom",
        &[
            ("assets/minecraft/t.png", "p"),
            ("assets/minecraft/t.png.mcmeta", r#"{"a":1}"#),
        ],
    );
    let top = zip_pack("top", &[("assets/minecraft/t.png.mcmeta", r#"{"a":2}"#)]);
    let stack = ResourceStack::new(vec![bottom, top]);
    let resource = stack.open(&id("minecraft:t.png")).expect("the texture");
    assert_eq!(resource.pack_id(), "bottom");
    let meta = resource.metadata().expect("metadata");
    assert_eq!(meta.i32("a"), Some(2));
}

#[test]
fn a_pack_needs_a_pack_section() {
    let dir = scratch("nometa");
    let source = DirectorySource::new(&dir);
    assert_eq!(
        Pack::open("p", Box::new(source), CURRENT).err(),
        Some(PackError::NoMetadata)
    );
    std::fs::write(dir.join("pack.mcmeta"), r#"{"pack":{}}"#).expect("a file");
    let source = DirectorySource::new(&dir);
    assert_eq!(
        Pack::open("p", Box::new(source), CURRENT).err(),
        Some(PackError::NoPackSection)
    );
}

#[test]
fn a_folder_pack_reads_files_and_refuses_to_leave_its_root() {
    let dir = scratch("folder");
    std::fs::create_dir_all(dir.join("assets/minecraft/lang")).expect("folders");
    std::fs::write(dir.join("assets/minecraft/lang/x.json"), "{}").expect("a file");
    std::fs::write(dir.join("pack.mcmeta"), META).expect("a file");
    std::fs::write(dir.join("assets/secret.txt"), "s").expect("a file");
    std::fs::write(dir.parent().expect("a parent").join("outside.txt"), "o").expect("a file");
    let source = DirectorySource::new(&dir);
    let pack = Pack::open("folder", Box::new(source), CURRENT).expect("a pack");
    let stack = ResourceStack::new(vec![pack]);
    assert_eq!(text(&stack, "minecraft:lang/x.json").as_deref(), Some("{}"));
    assert_eq!(
        stack.namespaces().into_iter().collect::<Vec<_>>(),
        ["minecraft"]
    );
    // `minecraft/../secret.txt` is a real file, and so is one more step up; neither is served.
    assert_eq!(text(&stack, "minecraft:../secret.txt"), None);
    assert_eq!(text(&stack, "minecraft:../../outside.txt"), None);
}

#[test]
fn a_pack_is_compatible_when_its_formats_include_the_games() {
    let compatible = |section: &str| {
        let dir = scratch("compat");
        std::fs::write(
            dir.join("pack.mcmeta"),
            format!(r#"{{"pack":{{"description":"t",{section}}}}}"#),
        )
        .expect("a file");
        let source = DirectorySource::new(&dir);
        Pack::open("p", Box::new(source), CURRENT)
            .expect("a pack")
            .is_compatible()
    };
    assert!(compatible(r#""min_format":84,"max_format":88"#));
    assert!(compatible(r#""min_format":[88,0],"max_format":[88,5]"#));
    assert!(!compatible(r#""min_format":84,"max_format":87"#));
    assert!(!compatible(r#""min_format":89,"max_format":90"#));
    assert!(!compatible(r#""pack_format":34"#));
    assert!(compatible(r#""pack_format":88"#));
    assert!(!compatible(
        r#""supported_formats":[15,34],"pack_format":15"#
    ));
    assert!(!compatible(r#""other":1"#));
}
