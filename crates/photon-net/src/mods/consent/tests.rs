use super::*;

const HASH: &str = "ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12ab12";
const OTHER: &str = "cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34cd34";
const SERVER: &str = "play.example.org:25565";

fn entry(hash: &str, caps: &[&str]) -> ModEntry {
    ModEntry {
        id: "m".to_owned(),
        hash: hash.to_owned(),
        size: 10,
        url: "https://h/m.wasm".to_owned(),
        caps: caps.iter().map(|cap| (*cap).to_owned()).collect(),
    }
}

fn local(cap: &str) -> bool {
    matches!(cap, "events.tick" | "hud.text" | "sound.play" | "game.time")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("photon-consent-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn game_local_capabilities_load_without_asking() {
    let consent = Consent::default();
    let all = entry(HASH, &["events.tick", "hud.text", "game.time"]);
    assert_eq!(decide(&consent, SERVER, &all, local), Admission::Load);
    assert_eq!(
        decide(&consent, SERVER, &entry(HASH, &[]), local),
        Admission::Load
    );
}

#[test]
fn a_capability_that_touches_the_machine_asks_first() {
    let consent = Consent::default();
    let wants = entry(HASH, &["hud.text", "fs.mod_data"]);
    assert_eq!(
        decide(&consent, SERVER, &wants, local),
        Admission::Ask(vec!["fs.mod_data".to_owned()])
    );
}

#[test]
fn an_unknown_capability_refuses_the_mod_even_beside_a_consent_one() {
    let consent = Consent::default();
    let odd = entry(HASH, &["fs.mod_data", "net.connect"]);
    assert_eq!(
        decide(&consent, SERVER, &odd, local),
        Admission::Refuse(Reason::Capability)
    );
}

#[test]
fn an_answer_is_remembered_per_server_and_per_hash() {
    let mut consent = Consent::default();
    let wants = entry(HASH, &["fs.mod_data"]);
    consent.answer(SERVER, HASH, "fs.mod_data", true);
    assert_eq!(decide(&consent, SERVER, &wants, local), Admission::Load);
    // Another server, or an updated mod (a new hash), is asked again.
    assert!(matches!(
        decide(&consent, "other.example:25565", &wants, local),
        Admission::Ask(_)
    ));
    assert!(matches!(
        decide(&consent, SERVER, &entry(OTHER, &["fs.mod_data"]), local),
        Admission::Ask(_)
    ));
    // A refusal refuses, and the last answer wins.
    consent.answer(SERVER, HASH, "fs.mod_data", false);
    assert_eq!(
        decide(&consent, SERVER, &wants, local),
        Admission::Refuse(Reason::Consent)
    );
    consent.answer(SERVER, HASH, "fs.mod_data", true);
    assert_eq!(decide(&consent, SERVER, &wants, local), Admission::Load);
}

#[test]
fn nothing_is_remembered_without_a_server_or_for_a_capability_that_needs_no_answer() {
    let mut consent = Consent::default();
    consent.answer("", HASH, "fs.mod_data", true);
    consent.answer(SERVER, HASH, "hud.text", true);
    assert_eq!(consent, Consent::default());
    consent.answer(SERVER, HASH, "fs.mod_data", true);
    // Without a server name there is no one to have said yes.
    let wants = entry(HASH, &["fs.mod_data"]);
    assert!(matches!(
        decide(&consent, "", &wants, local),
        Admission::Ask(_)
    ));
}

#[test]
fn the_answers_survive_a_save_and_a_load() {
    let dir = scratch("roundtrip");
    let mut consent = Consent::default();
    consent.answer(SERVER, HASH, "fs.mod_data", true);
    consent.answer("a \"quoted\" server", OTHER, "fs.mod_data", false);
    consent.save(&dir).expect("save");
    assert_eq!(Consent::load(&dir), consent);
    // The file was replaced, not left beside a temporary one.
    let names: Vec<_> = std::fs::read_dir(&dir).expect("dir").collect();
    assert_eq!(names.len(), 1);
    consent.answer(SERVER, HASH, "fs.mod_data", false);
    consent.save(&dir).expect("save again");
    assert_eq!(Consent::load(&dir), consent);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_missing_or_damaged_file_grants_nothing() {
    let dir = scratch("damaged");
    assert_eq!(Consent::load(&dir), Consent::default());
    std::fs::create_dir_all(&dir).expect("dir");
    for text in ["", "not json", "{\"v\":2,\"entries\":[]}", "[]"] {
        std::fs::write(dir.join(CONSENT_FILE), text).expect("write");
        assert_eq!(Consent::load(&dir), Consent::default(), "{text}");
    }
    // An entry with a bad hash is skipped, a good one beside it is kept, and a capability the
    // client has no question for is dropped on reading.
    let text = format!(
        r#"{{"v":1,"entries":[{{"server":"s","hash":"short","granted":["fs.mod_data"],"denied":[]}},{{"server":"s","hash":"{HASH}","granted":["fs.mod_data","net.connect"],"denied":[]}}]}}"#
    );
    let consent = Consent::from_json(&text);
    assert_eq!(consent.answers("s", HASH).map(|r| r.granted.len()), Some(1));
    assert!(consent.answers("s", "short").is_none());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn two_writers_do_not_collide() {
    let dir = scratch("race");
    let writers: Vec<_> = (0..6)
        .map(|n| {
            let dir = dir.clone();
            std::thread::spawn(move || {
                let mut consent = Consent::default();
                consent.answer(&format!("server{n}"), HASH, "fs.mod_data", true);
                consent.save(&dir)
            })
        })
        .collect();
    for writer in writers {
        writer.join().expect("thread").expect("save");
    }
    assert_eq!(Consent::load(&dir).records.len(), 1);
    assert_eq!(std::fs::read_dir(&dir).expect("dir").count(), 1);
    let _ = std::fs::remove_dir_all(dir);
}
