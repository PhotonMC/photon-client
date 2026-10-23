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
    let stack = Arc::new(crate::assets::vanilla_stack(jar_assets));
    if let Ok(mut slot) = IN_USE.write() {
        return Arc::clone(slot.get_or_insert(stack));
    }
    stack
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
