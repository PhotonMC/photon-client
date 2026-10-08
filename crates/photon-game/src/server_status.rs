//! What a server's status answer says (`ServerStatus` and its codec) and what the server list
//! makes of it (`ServerStatusPinger.handleStatusResponse`, `onPingFailed`).

use photon_core::json::{JsonValue, parse};

use crate::text::{Component, Style};

/// The pinned protocol: a server of another is incompatible.
pub const PROTOCOL_VERSION: i32 = 776;

/// `ServerStatus.Players`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Players {
    pub max: i32,
    pub online: i32,
    /// `NameAndId`: each sampled player's name and id (as the answer writes it).
    pub sample: Vec<(String, String)>,
}

/// `ServerStatus`, its optional fields absent when the answer has none or one that does not read.
#[derive(Debug, Clone, PartialEq)]
pub struct ServerStatus {
    pub description: Component,
    pub players: Option<Players>,
    /// `Version`: its name and protocol.
    pub version: Option<(String, i32)>,
    /// The favicon's PNG bytes.
    pub favicon: Option<Vec<u8>>,
}

fn field<'a>(value: &'a JsonValue, key: &str) -> Option<&'a JsonValue> {
    match value {
        JsonValue::Object(object) => object.get(&key.encode_utf16().collect::<Vec<_>>()),
        _ => None,
    }
}

fn string(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(units) => String::from_utf16(units).ok(),
        _ => None,
    }
}

/// `Codec.INT`: a whole number that fits.
fn int(value: &JsonValue) -> Option<i32> {
    match value {
        JsonValue::Number(text) => text.parse::<f64>().ok().and_then(|number| {
            (number.fract() == 0.0
                && number >= f64::from(i32::MIN)
                && number <= f64::from(i32::MAX))
            .then_some(number as i32)
        }),
        _ => None,
    }
}

impl ServerStatus {
    /// `ServerStatus.CODEC` over the answer's JSON: none when it is not an object.
    pub fn parse(text: &str) -> Option<Self> {
        let root = parse(text).ok()?;
        if !matches!(root, JsonValue::Object(_)) {
            return None;
        }
        let players = field(&root, "players").and_then(|players| {
            let sample = field(players, "sample")
                .and_then(|sample| match sample {
                    JsonValue::Array(entries) => entries
                        .iter()
                        .map(|entry| {
                            Some((string(field(entry, "name")?)?, string(field(entry, "id")?)?))
                        })
                        .collect::<Option<Vec<_>>>(),
                    _ => None,
                })
                .unwrap_or_default();
            Some(Players {
                max: int(field(players, "max")?)?,
                online: int(field(players, "online")?)?,
                sample,
            })
        });
        let version = field(&root, "version").and_then(|version| {
            Some((
                string(field(version, "name")?)?,
                int(field(version, "protocol")?)?,
            ))
        });
        let favicon = field(&root, "favicon").and_then(string).and_then(|text| {
            let base64 = text
                .strip_prefix("data:image/png;base64,")?
                .replace('\n', "");
            photon_core::base64::decode(&base64)
        });
        Some(Self {
            description: field(&root, "description")
                .and_then(Component::from_json)
                .unwrap_or_else(Component::empty),
            players,
            version,
            favicon,
        })
    }
}

/// How a ping ended (`ServerData.State` past `PINGING`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PingState {
    Successful,
    Incompatible,
    Unreachable,
}

/// What the server list shows of a pinged server (`ServerData`'s fields after the ping).
#[derive(Debug, Clone, PartialEq)]
pub struct Pinged {
    pub state: PingState,
    pub motd: Component,
    pub status: Component,
    pub version: Component,
    pub ping: i64,
    pub player_list: Vec<Component>,
}

/// `ServerStatusPinger.formatPlayerCount`.
pub fn player_count(online: i32, max: i32) -> Component {
    Component::translatable_with(
        "multiplayer.status.player_count",
        vec![
            Component::literal(online.to_string()).with_colour("gray"),
            Component::literal(max.to_string()).with_colour("gray"),
        ],
    )
    .with_colour("dark_gray")
}

/// `Util.NIL_UUID` of `MinecraftServer.ANONYMOUS_PLAYER_PROFILE`, as the answer writes it.
const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

impl Pinged {
    /// `handleStatusResponse` and the pong: the description, version, players and the time the
    /// ping took; incompatible for a protocol other than the client's.
    pub fn answered(status: &ServerStatus, ping: i64) -> Self {
        let (version, protocol) = match &status.version {
            Some((name, protocol)) => (Component::literal(name.clone()), *protocol),
            None => (Component::translatable("multiplayer.status.old"), 0),
        };
        let (status_text, player_list) = match &status.players {
            Some(players) => {
                let mut names: Vec<Component> = players
                    .sample
                    .iter()
                    .map(|(name, id)| {
                        if name == "Anonymous Player" && id == NIL_UUID {
                            Component::translatable("multiplayer.status.anonymous_player")
                        } else {
                            Component::literal(name.clone())
                        }
                    })
                    .collect();
                if !names.is_empty() && (names.len() as i32) < players.online {
                    names.push(Component::translatable_with(
                        "multiplayer.status.and_more",
                        vec![Component::literal(
                            (players.online - names.len() as i32).to_string(),
                        )],
                    ));
                }
                (player_count(players.online, players.max), names)
            }
            None => (
                Component::translatable("multiplayer.status.unknown").with_colour("dark_gray"),
                Vec::new(),
            ),
        };
        Self {
            state: if protocol == PROTOCOL_VERSION {
                PingState::Successful
            } else {
                PingState::Incompatible
            },
            motd: status.description.clone(),
            status: status_text,
            version,
            ping,
            player_list,
        }
    }

    /// `onPingFailed` and the list's catch: the red reason, no status. `unknown_host` for a name
    /// the resolver could not find.
    pub fn unreachable(unknown_host: bool) -> Self {
        let key = if unknown_host {
            "multiplayer.status.cannot_resolve"
        } else {
            "multiplayer.status.cannot_connect"
        };
        Self {
            state: PingState::Unreachable,
            motd: Component::translatable(key).with_style(&Style {
                colour: Some(0xFF_0000),
                ..Style::default()
            }),
            status: Component::empty(),
            version: Component::empty(),
            ping: 0,
            player_list: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_answer_reads_with_its_optional_fields() {
        let status = ServerStatus::parse(
            r#"{"description":{"text":"A Minecraft Server"},"players":{"max":20,"online":2,
            "sample":[{"name":"Alex","id":"0-0-0-0-1"}]},"version":{"name":"26.2","protocol":776},
            "favicon":"data:image/png;base64,TWFu"}"#,
        )
        .expect("an object");
        assert_eq!(
            status.players.as_ref().map(|players| players.online),
            Some(2)
        );
        assert_eq!(status.version, Some(("26.2".to_owned(), 776)));
        assert_eq!(status.favicon.as_deref(), Some(&b"Man"[..]));
        let pinged = Pinged::answered(&status, 12);
        assert_eq!(pinged.state, PingState::Successful);
        assert_eq!(pinged.player_list.len(), 2);
    }

    #[test]
    fn a_field_that_does_not_read_is_left_out() {
        let status =
            ServerStatus::parse(r#"{"players":{"max":"x"},"version":7}"#).expect("an object");
        assert_eq!(status.players, None);
        assert_eq!(status.version, None);
        assert_eq!(Pinged::answered(&status, 0).state, PingState::Incompatible);
    }
}
