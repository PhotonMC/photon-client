//! `photon open-link photon://launch?code=...`: the website hands the game a one-time launch
//! code, never a token (ADR 0020). The code is spent at the backend named by the environment, off
//! the menu's thread, and the answer decides whether the game joins.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use photon_core::binary::Uuid;
use photon_net::backend::{BackendError, Launch, is_same_profile};

/// The link the game was started with, until the menu takes it.
static PENDING: Mutex<Option<Link>> = Mutex::new(None);

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
    *PENDING.lock().map_err(|_| "the link could not be kept")? = Some(link);
    Ok(())
}

/// The link the game was started with, once: the menu takes it and nothing else sees it again.
pub(crate) fn take_pending() -> Option<Link> {
    PENDING.lock().ok()?.take()
}

/// What the answer to a link makes the menu do.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Join(String),
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
                Outcome::Join(launch.address)
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
    fn a_link_does_not_show_its_value_in_debug_output() {
        let shown = format!("{:?}", Link::SignIn(CODE.to_owned()));
        assert!(!shown.contains(&CODE[..8]), "{shown}");
    }

    #[test]
    fn a_code_joins_only_for_the_profile_it_was_issued_for() {
        let profile = Uuid::new(0x0123_4567_89ab_cdef, 0x0123_4567_89ab_cdef);
        let own = "0123456789abcdef0123456789abcdef";
        let other = "ffffffffffffffffffffffffffffffff";
        assert_eq!(
            decide(Ok(launch(own)), Some(profile)),
            Outcome::Join("play.example.com:25565".to_owned())
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
