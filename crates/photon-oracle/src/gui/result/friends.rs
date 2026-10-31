//! The friends overlay over the account's lists, as a row gives them.

use super::*;

use photon_core::binary::Uuid;
use photon_game::text::{Component, Lang, Style};
use photon_gui::screen::friends::{Entry, EntryKind, FriendsOverlay, FriendsView, Listing};
use photon_render::window::scale::WindowScale;

/// A `UUID.toString` text as an id.
fn uuid(text: &str) -> Result<Uuid, String> {
    let digits: String = text.chars().filter(|c| *c != '-').collect();
    let value = u128::from_str_radix(&digits, 16).map_err(|_| format!("a uuid: {text}"))?;
    Ok(Uuid::new((value >> 64) as u64, value as u64))
}

/// `FriendEntry.presenceStatusComponent`: the status in green, or grey when offline.
fn status_runs(lang: &Lang, presence: Option<&str>) -> Vec<photon_game::text::Run> {
    let name = presence.unwrap_or("offline");
    let colour = if name == "offline" {
        0xA0A0A0
    } else {
        0x00FF00
    };
    let style = Style {
        colour: Some(colour),
        ..Style::default()
    };
    let key = format!("gui.friends.presence.status.{name}");
    lang.runs(
        &Component::translatable(key).with_style(&style),
        &Style::default(),
    )
}

/// `PresenceStatusDto` order of `FriendEntry.presenceStatusSortOrder`: those in a game first.
fn sort_order(presence: Option<&str>) -> u8 {
    match presence.unwrap_or("offline") {
        "playing_hosted_server" => 0,
        "playing_server" => 1,
        "playing_realms" => 2,
        "playing_offline" => 3,
        "online" => 4,
        _ => 5,
    }
}

fn entries(list: &Value, kind: EntryKind, lang: &Lang) -> Result<Vec<Entry>, String> {
    let mut found = Vec::new();
    for item in list.as_array().map(Vec::as_slice).unwrap_or_default() {
        let presence = item["presence"].as_str();
        let status = if kind == EntryKind::Friend {
            status_runs(lang, presence)
        } else {
            Vec::new()
        };
        let entry = Entry {
            id: uuid(item["id"].as_str().ok_or("id")?)?,
            name: item["name"].as_str().ok_or("name")?.to_owned(),
            kind,
            status,
        };
        found.push((sort_order(presence), entry));
    }
    if kind == EntryKind::Friend {
        found.sort_by_key(|(order, _)| *order);
    }
    Ok(found.into_iter().map(|(_, entry)| entry).collect())
}

fn listing(found: Vec<Entry>) -> Listing {
    if found.is_empty() {
        Listing::Empty
    } else {
        Listing::Entries(found)
    }
}

/// `FriendsOverlayScreen.refreshLists`: the lists, or what a state that is none shows.
fn view(facts: &Value, lang: &Lang) -> Result<FriendsView, String> {
    let state = facts["state"].as_str().unwrap_or("success");
    let message = |key: &str| {
        Listing::Error(lang.runs(
            &Component::translatable(key).with_colour("gray"),
            &Style::default(),
        ))
    };
    let (friends, pending, requests) = match state {
        "loading" => (Listing::Loading, Listing::Loading, 0),
        "success" => {
            let incoming = entries(&facts["incoming"], EntryKind::Incoming, lang)?;
            let requests = incoming.len();
            let mut both = incoming;
            both.extend(entries(&facts["outgoing"], EntryKind::Outgoing, lang)?);
            let friends = entries(&facts["friends"], EntryKind::Friend, lang)?;
            (listing(friends), listing(both), requests)
        }
        other => {
            let key = match other {
                "upgrade_needed" => "gui.friends.error.upgrade_needed",
                "connection_issue" => "gui.friends.error.connection_issue",
                "temporary_unavailable" => "gui.friends.error.temporary_unavailable",
                "user_may_lack_active_profile" => "gui.friends.error.user_may_lack_active_profile",
                "unauthorized" => "gui.friends.error.unauthorized",
                _ => "gui.friends.error.generic",
            };
            (message(key), message(key), 0)
        }
    };
    Ok(FriendsView {
        friends,
        pending,
        requests,
        profile_name: facts["name"].as_str().unwrap_or("Steve").to_owned(),
        ready: state == "success",
        sending: false,
    })
}

pub(super) fn friends_overlay_case(ctx: &mut GuiContext, row: &Value) -> Result<Value, String> {
    let (font, atlas) = (&ctx.font, &ctx.atlas);
    let int = |key: &str| row[key].as_i64().unwrap_or(0) as i32;
    let lang = Lang::load(&ctx.root);
    let mut window = WindowScale::new(
        int_at(&row["framebuffer"], 0),
        int_at(&row["framebuffer"], 1),
    );
    window.set_gui_scale(window.calculate_scale(int("gui_scale"), false));
    let size = (window.gui_scaled_width(), window.gui_scaled_height());
    let mouse = (int_at(&row["mouse"], 0), int_at(&row["mouse"], 1));
    let mut screen = FriendsOverlay::new(&lang, font, size, view(&row["friends"], &lang)?);
    let blurriness = 5;
    let draw = |screen: &FriendsOverlay, millis: u64| {
        let mut frame = GuiFrame {
            log: Some(Vec::new()),
            millis,
            ..GuiFrame::default()
        };
        screen.draw(&mut frame, (atlas, font), mouse, (blurriness, millis));
        frame
    };
    for step in row["steps"].as_array().ok_or("steps")? {
        match step["do"].as_str().unwrap_or_default() {
            "frame" => {
                draw(&screen, step["ms"].as_u64().unwrap_or(0));
            }
            "click" => {
                draw(&screen, 0);
                let at = (
                    step["x"].as_f64().unwrap_or(0.0),
                    step["y"].as_f64().unwrap_or(0.0),
                );
                screen.clicked(at, font);
            }
            other => return Err(format!("unknown step {other}")),
        }
    }
    let mut frame = draw(&screen, row["ms"].as_u64().unwrap_or(0));
    Ok(json!({
        "label": row["label"],
        "width": window.gui_scaled_width(),
        "height": window.gui_scaled_height(),
        "gui_scale": int("gui_scale"),
        "scroll": 0.0,
        "ops": gui_ops(&frame.log.take().unwrap_or_default()),
    }))
}

fn int_at(array: &Value, index: usize) -> i32 {
    array[index].as_i64().unwrap_or(0) as i32
}
