use std::time::Duration;

use photon_api::{CAPABILITIES, IMPORTS, Import, Owner};

use super::*;
use crate::host::Host;

const MS: Duration = Duration::from_secs(1);

/// Only the capability `name`.
fn only(name: &str) -> Caps {
    Caps {
        tick: name == photon_api::EVENTS_TICK,
        hud_text: name == HUD_TEXT,
        sound: name == SOUND_PLAY,
        game_time: name == GAME_TIME,
        player: name == PLAYER_STATE,
        mod_data: name == FS_MOD_DATA,
    }
}

fn all_but(name: &str) -> Caps {
    Caps {
        tick: true,
        hud_text: name != HUD_TEXT,
        sound: name != SOUND_PLAY,
        game_time: name != GAME_TIME,
        player: name != PLAYER_STATE,
        mod_data: name != FS_MOD_DATA,
    }
}

/// A component that imports `import` with the signature the table declares, and does nothing.
fn importer(import: &Import) -> String {
    let params: String = import
        .params
        .iter()
        .map(|(name, ty)| format!(" (param \"{name}\" {})", ty.wat()))
        .collect();
    let result = import
        .result
        .map_or(String::new(), |ty| format!(" (result {})", ty.wat()));
    format!(
        r#"(component
  (import "{}" (func{params}{result}))
  (core module $m (func (export "tick")))
  (core instance $i (instantiate $m))
  (func (export "tick") (canon lift (core func $i "tick"))))"#,
        import.name
    )
}

fn load(host: &Host, import: &Import, caps: Caps) -> Result<(), String> {
    host.load_with(
        Owner::new("link-test"),
        importer(import).as_bytes(),
        (&[], caps),
        MS,
    )
    .map(drop)
    .map_err(|error| error.to_string())
}

#[test]
fn every_capability_of_the_table_is_one_the_caps_know() {
    for cap in CAPABILITIES {
        let caps = only(cap.name);
        assert!(caps.grants(cap.name), "{} is not in Caps::grants", cap.name);
        assert!(!Caps::default().grants(cap.name));
    }
}

#[test]
fn every_import_of_the_table_is_linked_with_its_declared_signature() {
    let host = Host::new().unwrap();
    for import in IMPORTS {
        let result = load(&host, import, only(import.capability));
        assert_eq!(result, Ok(()), "{}", import.name);
    }
}

#[test]
fn an_import_is_refused_without_its_capability() {
    let host = Host::new().unwrap();
    for import in IMPORTS {
        let refused = load(&host, import, all_but(import.capability));
        assert!(
            refused.is_err(),
            "{} linked without {}",
            import.name,
            import.capability
        );
    }
}

#[test]
fn a_wrong_signature_is_refused() {
    let host = Host::new().unwrap();
    for import in IMPORTS {
        let wrong = Import {
            result: match import.result {
                Some(_) => None,
                None => Some(photon_api::Ty::U32),
            },
            ..*import
        };
        assert!(
            load(&host, &wrong, only(import.capability)).is_err(),
            "{}",
            import.name
        );
    }
}

#[test]
fn an_import_the_table_does_not_have_is_not_linked() {
    let host = Host::new().unwrap();
    let unknown = Import {
        name: "registry-read",
        ..IMPORTS[0]
    };
    assert!(load(&host, &unknown, all_but("")).is_err());
}
