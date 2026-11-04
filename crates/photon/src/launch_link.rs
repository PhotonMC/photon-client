//! `photon open-link photon://launch?code=...`: the website hands the game a one-time launch
//! code, never a token (ADR 0020). The code is spent at the backend named by the environment, off
//! the menu's thread, and the answer decides whether the game joins.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use photon_core::binary::Uuid;
use photon_net::backend::{Backend, BackendError, Launch, is_same_profile};

/// The code of the link the game was started with, until the menu takes it.
static PENDING: Mutex<Option<String>> = Mutex::new(None);

/// The length of the codes the backend issues: 16 random bytes in hex.
const CODE_LENGTH: usize = 32;

/// The launch code of a link, or nothing when the link is anything else: another scheme or host,
/// more or fewer parameters, a malformed code. A `token` parameter is never accepted.
pub(crate) fn parse(link: &str) -> Option<String> {
    let query = link.strip_prefix("photon://launch?")?;
    let code = query.strip_prefix("code=")?;
    let well_formed =
        code.len() == CODE_LENGTH && code.bytes().all(|byte| byte.is_ascii_hexdigit());
    well_formed.then(|| code.to_ascii_lowercase())
}

/// Remembers the link the game was started with; a link that is not one is an error.
pub(crate) fn remember(link: &str) -> Result<(), String> {
    let code = parse(link).ok_or("that is not a Photon launch link")?;
    *PENDING.lock().map_err(|_| "the link could not be kept")? = Some(code);
    Ok(())
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
    /// Takes the remembered link and starts spending its code; nothing when there is none. The
    /// code is single-use, so it leaves the process with this call.
    pub(crate) fn start() -> Option<Self> {
        let code = PENDING.lock().ok()?.take()?;
        let (sender, receiver) = channel();
        let _ = std::thread::Builder::new()
            .name("Launch Link".to_owned())
            .spawn(move || {
                let answer = Backend::from_environment()
                    .unwrap_or(Err(BackendError::Unavailable))
                    .and_then(|backend| backend.exchange(&code));
                let signed_in = crate::profile_login::identity()
                    .ok()
                    .and_then(|(_, account)| account)
                    .map(|account| account.profile_id);
                let _ = sender.send(decide(answer, signed_in));
            });
        Some(Self { receiver })
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
    fn only_a_launch_link_with_a_well_formed_code_parses() {
        assert_eq!(
            parse(&format!("photon://launch?code={CODE}")),
            Some(CODE.to_owned())
        );
        let upper = CODE.to_ascii_uppercase();
        assert_eq!(
            parse(&format!("photon://launch?code={upper}")),
            Some(CODE.to_owned())
        );
        for link in [
            "".to_owned(),
            "https://launch?code=".to_owned() + CODE,
            "photon://other?code=".to_owned() + CODE,
            "photon://launch?token=".to_owned() + CODE,
            format!("photon://launch?code={CODE}&token=abc"),
            format!("photon://launch?token=abc&code={CODE}"),
            "photon://launch?code=abc".to_owned(),
            format!("photon://launch?code={}g", &CODE[..31]),
        ] {
            assert_eq!(parse(&link), None, "{link}");
        }
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
