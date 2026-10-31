//! The friends overlay over the lists the service answered: its states, entries, the pending
//! tab and the tooltips of its buttons.

use serde_json::{Value, json};

use super::gui;
use super::setting::{at, click};

const ALEX: &str = "00000000-0000-0000-0000-000000000a01";
const BOB: &str = "00000000-0000-0000-0000-000000000b02";
const CARA: &str = "00000000-0000-0000-0000-000000000c03";
const DAN: &str = "00000000-0000-0000-0000-000000000d04";
const EVE: &str = "00000000-0000-0000-0000-000000000e05";

fn person(id: &str, name: &str) -> Value {
    json!({"id": id, "name": name})
}

fn friend(id: &str, name: &str, presence: Option<&str>) -> Value {
    let mut friend = person(id, name);
    if let Some(presence) = presence {
        friend["presence"] = json!(presence);
    }
    friend
}

fn state(state: &str) -> Value {
    json!({"state": state, "name": "Steve"})
}

fn lists() -> Value {
    json!({
        "state": "success", "name": "Steve",
        "friends": [
            friend(ALEX, "Alex", Some("online")),
            friend(BOB, "Bob", None),
            friend(CARA, "Cara", Some("playing_server")),
        ],
        "incoming": [person(DAN, "Dan")],
        "outgoing": [person(EVE, "Eve")],
    })
}

fn only_requests() -> Value {
    json!({"state": "success", "name": "Steve", "incoming": [person(DAN, "Dan")], "outgoing": [person(EVE, "Eve")]})
}

/// The tab of the pending requests, in a 427 by 240 window: 213 across, 13 down.
fn pending_tab() -> Value {
    click(260, 23)
}

fn hover(label: &str, facts: Value, x: i64, y: i64, steps: Vec<Value>) -> Value {
    let mut all = steps;
    all.push(json!({"do": "frame", "ms": 0}));
    at(label).friends(facts).mouse(x, y).steps(all).ms(1000).row()
}

pub fn friends_overlay() -> Value {
    let rows = vec![
        at("empty").friends(state("success")).row(),
        at("loading").friends(state("loading")).ms(450).row(),
        at("error-connection").friends(state("connection_issue")).row(),
        at("error-generic").friends(state("generic_error")).row(),
        at("friends").friends(lists()).row(),
        at("friends-small").window(640, 360).friends(lists()).row(),
        at("friends-scale-three").scale(3).friends(lists()).row(),
        at("pending").friends(lists()).steps(vec![pending_tab()]).row(),
        at("pending-empty").friends(state("success")).steps(vec![pending_tab()]).row(),
        at("pending-only-requests").friends(only_requests()).steps(vec![pending_tab()]).row(),
        at("pending-loading").friends(state("loading")).steps(vec![pending_tab()]).row(),
        hover("hover-tab", lists(), 260, 23, vec![]),
        hover("hover-unfriend", lists(), 305, 98, vec![]),
        hover("hover-send", lists(), 290, 62, vec![]),
        hover("hover-name", lists(), 150, 82, vec![]),
        hover("hover-decline", lists(), 285, 62, vec![pending_tab()]),
    ];
    gui("friends-overlay", "friends_overlay", rows)
}
