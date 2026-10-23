//! The resource packs in use: the one stack every reader of assets shares, so that a reload
//! changes what all of them see.

use std::path::Path;
use std::sync::{Arc, RwLock};

use photon_assets::ResourceStack;

static IN_USE: RwLock<Option<Arc<ResourceStack>>> = RwLock::new(None);

/// The stack in use, which holds the game's own assets alone until packs are chosen.
pub(crate) fn shared(jar_assets: &Path) -> Arc<ResourceStack> {
    if let Some(stack) = IN_USE.read().ok().and_then(|stack| stack.clone()) {
        return stack;
    }
    let stack = Arc::new(ResourceStack::new(selected_packs(jar_assets)));
    if let Ok(mut slot) = IN_USE.write() {
        return Arc::clone(slot.get_or_insert(stack));
    }
    stack
}

/// The pack format of the running game (`SharedConstants.getCurrentVersion().packVersion`).
const PACK_FORMAT: photon_assets::PackFormat = photon_assets::PackFormat::of(88, 0);

/// The game's own assets, then the packs `options.txt` selects (`resourcePacks`, the lowest
/// first). A `file/` entry names a folder or zip in the `resourcepacks` folder of the game
/// directory; one that is missing or cannot be opened is left out.
fn selected_packs(jar_assets: &Path) -> Vec<photon_assets::Pack> {
    let mut stacked = vec![crate::assets::vanilla_pack(jar_assets)];
    let directory = crate::controls::game_directory().join("resourcepacks");
    for id in crate::controls::load_options().resource_packs() {
        let Some(name) = id.strip_prefix("file/") else {
            continue;
        };
        match open_pack(id.clone(), &directory.join(name)) {
            Ok(pack) => stacked.push(pack),
            Err(error) => log::warn!("resource pack {id}: {error}"),
        }
    }
    stacked
}

fn open_pack(name: String, path: &Path) -> Result<photon_assets::Pack, String> {
    let source: Box<dyn photon_assets::Source> = if path.is_dir() {
        Box::new(photon_assets::DirectorySource::new(path))
    } else {
        let zip = photon_assets::ZipSource::open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        Box::new(zip)
    };
    photon_assets::Pack::open(name, source, PACK_FORMAT)
        .map_err(|error| format!("{}: {error:?}", path.display()))
}

/// The selected packs with the downloaded ones over them. `packs` lists the top of the stack
/// first, as the server pack source gives them; a pack that cannot be opened fails the whole.
pub(crate) fn with_packs(
    jar_assets: &Path,
    packs: &crate::packs::LoadedPacks,
) -> Result<ResourceStack, String> {
    let mut stacked = selected_packs(jar_assets);
    for (id, path) in packs.iter().rev() {
        let name = format!("server/{}", id.to_string_java());
        stacked.push(open_pack(name, path)?);
    }
    Ok(ResourceStack::new(stacked))
}

/// Whether the stack in use already holds exactly `packs` (the top first), so that loading them
/// changes nothing.
pub(crate) fn holds(packs: &crate::packs::LoadedPacks) -> bool {
    let Some(stack) = IN_USE.read().ok().and_then(|stack| stack.clone()) else {
        return packs.is_empty();
    };
    let in_use = stack
        .packs()
        .iter()
        .filter(|pack| pack.id().starts_with("server/"))
        .map(|pack| pack.id().to_owned());
    let wanted = packs
        .iter()
        .rev()
        .map(|(id, _)| format!("server/{}", id.to_string_java()));
    in_use.eq(wanted)
}

/// Put `stack` in use: what is read afterwards sees it.
pub(crate) fn replace(stack: Arc<ResourceStack>) {
    if let Ok(mut slot) = IN_USE.write() {
        *slot = Some(stack);
    }
}

#[cfg(test)]
mod tests {
    use photon_assets::{DirectorySource, Pack, ResourceStack};
    use photon_game::text::Lang;

    fn pack(name: &str, files: &[(&str, &str)]) -> (Pack, std::path::PathBuf) {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("photon-res-{}-{n}", std::process::id()));
        for (path, text) in files {
            let file = dir.join(path);
            std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            std::fs::write(file, text).unwrap();
        }
        (
            Pack::built_in(name, Box::new(DirectorySource::new(&dir))),
            dir,
        )
    }

    #[test]
    fn language_files_of_every_pack_are_read_over_each_other() {
        let (low, a) = pack(
            "low",
            &[
                ("assets/minecraft/lang/en_us.json", r#"{"a":"1","b":"2"}"#),
                ("assets/minecraft/texts/splashes.txt", "low\n"),
            ],
        );
        let (high, b) = pack(
            "high",
            &[
                ("assets/minecraft/lang/en_us.json", r#"{"b":"3"}"#),
                ("assets/extra/lang/en_us.json", r#"{"c":"4"}"#),
                ("assets/minecraft/texts/splashes.txt", "high\n"),
            ],
        );
        let stack = ResourceStack::new(vec![low, high]);
        let none = std::path::Path::new("/nonexistent");
        let lang = Lang::load_from(&stack, none);
        assert_eq!(lang.get("a"), Some("1"));
        assert_eq!(lang.get("b"), Some("3"));
        assert_eq!(lang.get("c"), Some("4"));
        assert_eq!(photon_game::splash::read_splashes_from(&stack), ["high"]);
        std::fs::remove_dir_all(a).unwrap();
        std::fs::remove_dir_all(b).unwrap();
    }

    #[test]
    fn the_providers_of_a_higher_pack_come_first() {
        let font = |advance: &str| {
            format!(r#"{{"providers":[{{"type":"space","advances":{{"x":{advance}}}}}]}}"#)
        };
        let (low, a) = pack("low", &[("assets/minecraft/font/default.json", &font("4"))]);
        let (high, b) = pack(
            "high",
            &[("assets/minecraft/font/default.json", &font("7"))],
        );
        let alone = photon_render::gui::font::Font::load_from(&ResourceStack::new(vec![low]));
        assert_eq!(alone.unwrap().width("x"), 4);
        let (low, c) = pack("low", &[("assets/minecraft/font/default.json", &font("4"))]);
        let both = ResourceStack::new(vec![low, high]);
        assert_eq!(
            photon_render::gui::font::Font::load_from(&both)
                .unwrap()
                .width("x"),
            7
        );
        for dir in [a, b, c] {
            std::fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn the_vanilla_stack_reads_the_languages_the_old_way() {
        let Ok(root) = std::env::var("PHOTON_ASSETS") else {
            return;
        };
        let root = std::path::Path::new(&root);
        let stack = crate::assets::vanilla_stack(root);
        let english = Lang::load_from(&stack, root);
        assert_eq!(english.text("gui.yes"), Lang::load(root).text("gui.yes"));
        let source = photon_game::language::LanguageSource::open(root).unwrap();
        for code in ["de_de", "tr_tr"] {
            let old = source.lang(code);
            let new = source.lang_from(code, &stack);
            assert_eq!(new.text("gui.yes"), old.text("gui.yes"), "{code}");
            assert_eq!(new.text("menu.quit"), old.text("menu.quit"), "{code}");
        }
        assert_ne!(
            source.lang_from("de_de", &stack).text("menu.quit"),
            english.text("menu.quit")
        );
    }
}
