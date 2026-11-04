//! `photon open-link photon://launch?code=...`: the website hands the game a one-time launch
//! code, never a token (ADR 0020). The code is spent at the backend named by the environment, off
//! the menu's thread, and the answer decides whether the game joins.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use photon_core::binary::Uuid;
use photon_net::backend::{BackendError, Launch, is_same_profile};

/// The link the game was started with, until the menu takes it.
/// How long a link is worth keeping: its launch code lasts a minute at the backend.
const FRESH: std::time::Duration = std::time::Duration::from_secs(60);

static PENDING: Mutex<Option<(Link, std::time::Instant)>> = Mutex::new(None);

/// The length of the codes and states: 16 random bytes in hex.
const CODE_LENGTH: usize = 32;

/// What a link asks of the game. Neither form carries a token; the sign-in one carries only the
/// state a browser started with.
#[derive(PartialEq, Eq, Clone)]
pub(crate) enum Link {
    /// Spend this launch code and join.
    Launch(String),
    /// Sign the website's browser in; the state is the browser's.
    SignIn(String),
}

// The code and the state are kept out of `Debug` output, and so out of logs.
impl std::fmt::Debug for Link {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Launch(_) => "Link::Launch(..)",
            Self::SignIn(_) => "Link::SignIn(..)",
        })
    }
}

/// The link's one parameter, when it is `<name>=` and 32 hex digits and nothing else.
fn hex_parameter(query: &str, name: &str) -> Option<String> {
    let value = query.strip_prefix(name)?.strip_prefix('=')?;
    let well_formed =
        value.len() == CODE_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    well_formed.then(|| value.to_ascii_lowercase())
}

/// The link, or nothing when it is anything else: another scheme or host, more or fewer
/// parameters, a malformed value. A `token` parameter is never accepted.
pub(crate) fn parse(link: &str) -> Option<Link> {
    if let Some(query) = link.strip_prefix("photon://launch?") {
        return hex_parameter(query, "code").map(Link::Launch);
    }
    let query = link.strip_prefix("photon://signin?")?;
    hex_parameter(query, "state").map(Link::SignIn)
}

/// Remembers the link the game was started with; a link that is not one is an error.
pub(crate) fn remember(link: &str) -> Result<(), String> {
    let link = parse(link).ok_or("that is not a Photon link")?;
    *PENDING.lock().map_err(|_| "the link could not be kept")? =
        Some((link, std::time::Instant::now()));
    Ok(())
}

/// The link the game was started with, once: the menu takes it and nothing else sees it again.
pub(crate) fn take_pending() -> Option<Link> {
    let (link, arrived) = PENDING.lock().ok()?.take()?;
    if arrived.elapsed() > FRESH {
        log::info!("a link that waited too long was dropped");
        return None;
    }
    Some(link)
}

/// The launch code of the pending link, once; a sign-in link stays pending for the menu.
pub(crate) fn take_pending_launch() -> Option<String> {
    let mut pending = PENDING.lock().ok()?;
    if !matches!(pending.as_ref(), Some((Link::Launch(_), _))) {
        return None;
    }
    let (link, arrived) = pending.take()?;
    drop(pending);
    match link {
        Link::Launch(code) if arrived.elapsed() <= FRESH => Some(code),
        _ => None,
    }
}

/// A link's answer while a game is played: the nowhere-else-handled outcomes, and the question.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InPlay {
    /// Ask whether to leave for this address, shown under this name.
    Ask(String, String),
    /// Say why the link did nothing, by translation key.
    Say(&'static str),
}

pub(crate) fn in_play(outcome: Outcome) -> InPlay {
    match outcome {
        Outcome::Join(address, name) => InPlay::Ask(address, name),
        Outcome::OtherProfile => InPlay::Say("photon.link.other_profile"),
        Outcome::NotSignedIn => InPlay::Say("photon.link.not_signed_in"),
        Outcome::Refused => InPlay::Say("photon.link.refused"),
        Outcome::Unavailable => InPlay::Say("photon.link.unavailable"),
    }
}

/// What a game holds of a link handed over while it is played.
#[derive(Default)]
pub(crate) struct GameLink {
    pub(crate) task: Option<LinkTask>,
    pub(crate) ready: Option<Outcome>,
}

/// What the answer to a link makes the menu do.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// The address to join and the name to show while joining.
    Join(String, String),
    /// The code was issued for another profile than the signed-in one.
    OtherProfile,
    NotSignedIn,
    Refused,
    Unavailable,
}

/// The decision for an exchanged code: joining needs the signed-in profile to be the code's own.
pub(crate) fn decide(answer: Result<Launch, BackendError>, signed_in: Option<Uuid>) -> Outcome {
    match answer {
        Err(BackendError::Refused) => Outcome::Refused,
        Err(_) => Outcome::Unavailable,
        Ok(launch) => match signed_in {
            None => Outcome::NotSignedIn,
            Some(profile) if is_same_profile(&launch.uuid, profile) => {
                let name = if launch.name.is_empty() {
                    launch.address.clone()
                } else {
                    launch.name
                };
                Outcome::Join(launch.address, name)
            }
            Some(_) => Outcome::OtherProfile,
        },
    }
}

/// A link being spent.
pub(crate) struct LinkTask {
    receiver: Receiver<Outcome>,
}

impl LinkTask {
    /// Starts spending a launch code. The code is single-use.
    pub(crate) fn start(code: String) -> Self {
        let (sender, receiver) = channel();
        let _ = std::thread::Builder::new()
            .name("Launch Link".to_owned())
            .spawn(move || {
                let answer = crate::links_config::backend()
                    .unwrap_or(Err(BackendError::Unavailable))
                    .and_then(|backend| backend.exchange(&code));
                let signed_in = crate::profile_login::identity()
                    .ok()
                    .and_then(|(_, account)| account)
                    .map(|account| account.profile_id);
                let _ = sender.send(decide(answer, signed_in));
            });
        Self { receiver }
    }

    /// The decision, when it has come.
    pub(crate) fn poll(&self) -> Option<Outcome> {
        self.receiver.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: &str = "00112233445566778899aabbccddeeff";

    fn launch(uuid: &str) -> Launch {
        Launch {
            address: "play.example.com:25565".to_owned(),
            uuid: uuid.to_owned(),
            name: "Example".to_owned(),
        }
    }

    #[test]
    fn only_a_launch_or_sign_in_link_with_one_well_formed_value_parses() {
        assert_eq!(
            parse(&format!("photon://launch?code={CODE}")),
            Some(Link::Launch(CODE.to_owned()))
        );
        let upper = CODE.to_ascii_uppercase();
        assert_eq!(
            parse(&format!("photon://launch?code={upper}")),
            Some(Link::Launch(CODE.to_owned()))
        );
        assert_eq!(
            parse(&format!("photon://signin?state={CODE}")),
            Some(Link::SignIn(CODE.to_owned()))
        );
        for link in [
            "".to_owned(),
            "https://launch?code=".to_owned() + CODE,
            "photon://other?code=".to_owned() + CODE,
            "photon://launch?token=".to_owned() + CODE,
            "photon://signin?token=".to_owned() + CODE,
            // A name belongs to its own host: a state cannot launch, a code cannot sign in.
            "photon://launch?state=".to_owned() + CODE,
            "photon://signin?code=".to_owned() + CODE,
            format!("photon://launch?code={CODE}&token=abc"),
            format!("photon://launch?token=abc&code={CODE}"),
            format!("photon://signin?state={CODE}&code={CODE}"),
            format!("photon://signin?code={CODE}&state={CODE}"),
            "photon://launch?code=abc".to_owned(),
            "photon://signin?state=abc".to_owned(),
            format!("photon://signin?state={}g", &CODE[..31]),
        ] {
            assert_eq!(parse(&link), None, "{link}");
        }
    }

    #[test]
    fn a_game_takes_a_launch_link_and_leaves_a_sign_in_link() {
        remember(&format!("photon://signin?state={CODE}")).expect("remembered");
        assert_eq!(take_pending_launch(), None);
        assert!(take_pending().is_some());
        remember(&format!("photon://launch?code={CODE}")).expect("remembered");
        assert_eq!(take_pending_launch(), Some(CODE.to_owned()));
        assert_eq!(take_pending_launch(), None);
    }

    #[test]
    fn a_game_asks_about_a_join_and_says_the_rest() {
        assert_eq!(
            in_play(Outcome::Join("a:1".into(), "A".into())),
            InPlay::Ask("a:1".into(), "A".into())
        );
        assert_eq!(
            in_play(Outcome::Refused),
            InPlay::Say("photon.link.refused")
        );
    }

    #[test]
    fn a_link_does_not_show_its_value_in_debug_output() {
        let shown = format!("{:?}", Link::SignIn(CODE.to_owned()));
        assert!(!shown.contains(&CODE[..8]), "{shown}");
    }

    #[test]
    fn a_backend_without_a_name_falls_back_to_the_address() {
        let profile = Uuid::new(0x0123_4567_89ab_cdef, 0x0123_4567_89ab_cdef);
        let mut unnamed = launch("0123456789abcdef0123456789abcdef");
        unnamed.name.clear();
        assert_eq!(
            decide(Ok(unnamed), Some(profile)),
            Outcome::Join(
                "play.example.com:25565".to_owned(),
                "play.example.com:25565".to_owned()
            )
        );
    }

    #[test]
    fn a_code_joins_only_for_the_profile_it_was_issued_for() {
        let profile = Uuid::new(0x0123_4567_89ab_cdef, 0x0123_4567_89ab_cdef);
        let own = "0123456789abcdef0123456789abcdef";
        let other = "ffffffffffffffffffffffffffffffff";
        assert_eq!(
            decide(Ok(launch(own)), Some(profile)),
            Outcome::Join("play.example.com:25565".to_owned(), "Example".to_owned())
        );
        assert_eq!(
            decide(Ok(launch(other)), Some(profile)),
            Outcome::OtherProfile
        );
        assert_eq!(decide(Ok(launch(own)), None), Outcome::NotSignedIn);
        assert_eq!(decide(Err(BackendError::Refused), None), Outcome::Refused);
        assert_eq!(
            decide(Err(BackendError::Insecure), None),
            Outcome::Unavailable
        );
    }
}
