use super::*;

fn entry(id: &str, hash: &str) -> ModEntry {
    ModEntry {
        id: id.to_owned(),
        hash: hash.to_owned(),
        size: 10,
        url: "https://h/x.wasm".to_owned(),
        caps: Vec::new(),
    }
}

fn manifest(entries: &[(&str, &str)]) -> Manifest {
    Manifest {
        mods: entries.iter().map(|(id, hash)| entry(id, hash)).collect(),
    }
}

fn all(_: &ModEntry) -> Admission {
    Admission::Load
}

fn none(_: &str, _: &str) -> bool {
    false
}

fn ids(plan: &Plan) -> Vec<&str> {
    plan.fetch.iter().map(|e| e.id.as_str()).collect()
}

#[test]
fn the_join_hello_is_due_once_with_nothing_loaded() {
    let mut sync = ModSync::new();
    assert!(sync.take_hello_due());
    assert!(!sync.take_hello_due());
    assert!(sync.refused().is_empty());
}

#[test]
fn a_manifest_is_fetched_and_the_hello_follows_the_last_answer() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let plan = sync.manifest(&manifest(&[("a", "h1"), ("b", "h2")]), all, none);
    assert_eq!(ids(&plan), ["a", "b"]);
    assert_eq!(sync.waiting(), 2);
    assert!(!sync.take_hello_due(), "fetches are still out");
    assert_eq!(sync.fetched("h1", Ok(())).len(), 1);
    assert!(!sync.take_hello_due(), "one is still out");
    assert_eq!(sync.fetched("h2", Ok(())).len(), 1);
    assert!(sync.take_hello_due());
}

#[test]
fn a_manifest_with_nothing_new_answers_at_once() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let loaded = |id: &str, hash: &str| (id, hash) == ("a", "h1");
    let plan = sync.manifest(&manifest(&[("a", "h1")]), all, loaded);
    assert!(plan.fetch.is_empty());
    assert_eq!(plan.keep, [("a".to_owned(), "h1".to_owned())]);
    assert!(sync.take_hello_due());
}

#[test]
fn a_new_manifest_does_not_load_what_it_dropped_or_changed() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    sync.manifest(&manifest(&[("a", "h1"), ("b", "h2")]), all, none);
    // The server now drops b and re-hashes a, before either fetch has answered.
    let plan = sync.manifest(&manifest(&[("a", "h3")]), all, none);
    assert_eq!(ids(&plan), ["a"]);
    assert!(
        sync.fetched("h1", Ok(())).is_empty(),
        "the old a is not loaded"
    );
    assert!(
        sync.fetched("h2", Ok(())).is_empty(),
        "the dropped b is not loaded"
    );
    assert!(!sync.take_hello_due(), "the new a is still out");
    assert_eq!(sync.fetched("h3", Ok(()))[0].hash, "h3");
    assert!(sync.take_hello_due());
}

#[test]
fn a_hash_that_is_already_being_fetched_is_not_fetched_twice() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let first = sync.manifest(&manifest(&[("a", "h1")]), all, none);
    assert_eq!(first.fetch.len(), 1);
    // The same file again, also under a second id: nothing new to fetch.
    let second = sync.manifest(&manifest(&[("a", "h1"), ("c", "h1")]), all, none);
    assert!(second.fetch.is_empty());
    let ready = sync.fetched("h1", Ok(()));
    assert_eq!(
        ready.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        ["a", "c"]
    );
    // Once answered, a later manifest may ask for it again.
    assert_eq!(
        sync.manifest(&manifest(&[("a", "h1")]), all, none)
            .fetch
            .len(),
        1
    );
}

#[test]
fn one_file_under_two_ids_is_fetched_once_and_loaded_twice() {
    let mut sync = ModSync::new();
    let plan = sync.manifest(&manifest(&[("a", "h1"), ("b", "h1")]), all, none);
    assert_eq!(plan.fetch.len(), 1);
    assert_eq!(sync.fetched("h1", Ok(())).len(), 2);
}

#[test]
fn refused_mods_are_named_with_their_reason_and_the_rest_goes_on() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let admit = |e: &ModEntry| {
        if e.id == "odd" {
            Admission::Refuse(Reason::Capability)
        } else {
            Admission::Load
        }
    };
    let plan = sync.manifest(
        &manifest(&[("odd", "h0"), ("a", "h1"), ("b", "h2"), ("c", "h3")]),
        admit,
        none,
    );
    assert_eq!(ids(&plan), ["a", "b", "c"]);
    assert!(sync.fetched("h1", Err(Reason::Hash)).is_empty());
    assert!(sync.fetched("h2", Err(Reason::Fetch)).is_empty());
    let ready = sync.fetched("h3", Ok(()));
    sync.load_failed(&ready[0].id, Reason::Load);
    let named: Vec<(&str, &str)> = sync
        .refused()
        .iter()
        .map(|r| (r.id.as_str(), r.reason.word()))
        .collect();
    assert_eq!(
        named,
        [
            ("odd", "capability"),
            ("a", "hash"),
            ("b", "fetch"),
            ("c", "load")
        ]
    );
    assert!(sync.take_hello_due());
}

#[test]
fn a_manifest_of_only_refused_mods_answers_at_once() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let admit = |_: &ModEntry| Admission::Refuse(Reason::Consent);
    sync.manifest(&manifest(&[("a", "h1")]), admit, none);
    assert!(sync.take_hello_due());
    assert_eq!(sync.refused()[0].reason, Reason::Consent);
}

#[test]
fn a_connection_that_starts_over_owes_a_new_join_hello() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    sync.manifest(&manifest(&[("a", "h1")]), all, none);
    sync.reset();
    assert!(sync.take_hello_due());
    assert_eq!(sync.waiting(), 0);
    // The old fetch answers late and changes nothing.
    assert!(sync.fetched("h1", Ok(())).is_empty());
    assert!(!sync.take_hello_due());
}

#[test]
fn the_reasons_are_the_fixed_words() {
    let words: Vec<&str> = [
        Reason::Capability,
        Reason::Hash,
        Reason::Size,
        Reason::Fetch,
        Reason::Load,
        Reason::Consent,
    ]
    .iter()
    .map(|r| r.word())
    .collect();
    assert_eq!(
        words,
        ["capability", "hash", "size", "fetch", "load", "consent"]
    );
}

fn asks(entry: &ModEntry) -> Admission {
    if entry.id == "ask" {
        Admission::Ask(vec!["fs.mod_data".to_owned()])
    } else {
        Admission::Load
    }
}

#[test]
fn a_mod_that_needs_an_answer_is_held_and_named_as_refused_for_consent() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    let plan = sync.manifest(&manifest(&[("ask", "h1"), ("a", "h2")]), asks, none);
    assert_eq!(ids(&plan), ["a"], "the held mod is not fetched");
    assert_eq!(
        sync.questions(),
        [Question {
            id: "ask".to_owned(),
            hash: "h1".to_owned(),
            caps: vec!["fs.mod_data".to_owned()]
        }]
    );
    assert_eq!(sync.refused()[0].reason, Reason::Consent);
    // A manifest of only held mods answers at once.
    sync.fetched("h2", Ok(()));
    assert!(sync.take_hello_due());
}

#[test]
fn a_grant_resumes_the_held_mod_and_a_refusal_ends_it() {
    let mut sync = ModSync::new();
    sync.take_hello_due();
    sync.manifest(&manifest(&[("ask", "h1")]), asks, none);
    assert!(sync.take_hello_due());
    let fetch = sync.answer("ask", true);
    assert_eq!(fetch.len(), 1);
    assert!(sync.refused().is_empty() && sync.questions().is_empty());
    assert!(!sync.take_hello_due(), "it is being fetched");
    assert_eq!(sync.fetched("h1", Ok(())).len(), 1);
    assert!(sync.take_hello_due());

    sync.manifest(&manifest(&[("ask", "h9")]), asks, none);
    sync.take_hello_due();
    assert!(sync.answer("ask", false).is_empty());
    assert_eq!(sync.refused()[0].reason, Reason::Consent);
    assert!(sync.questions().is_empty());
    assert!(sync.take_hello_due());
    // A second answer to the same question, or one nobody asked, does nothing.
    assert!(sync.answer("ask", true).is_empty());
    assert!(sync.answer("nobody", true).is_empty());
}

#[test]
fn a_new_manifest_drops_the_questions_of_the_old_one() {
    let mut sync = ModSync::new();
    sync.manifest(&manifest(&[("ask", "h1")]), asks, none);
    sync.manifest(&manifest(&[("a", "h2")]), asks, none);
    assert!(sync.questions().is_empty());
    assert!(sync.answer("ask", true).is_empty());
}
