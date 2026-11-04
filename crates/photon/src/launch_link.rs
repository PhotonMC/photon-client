//! `photon open-link photon://launch?code=...`: the website hands the game a one-time launch
//! code, never a token (ADR 0020). The code is spent at the backend named by the environment, off
//! the menu's thread, and the answer decides whether the game joins.

use std::sync::Mutex;
use std::sync::mpsc::{Receiver, channel};

use crate::controls::ConfirmTarget;
use photon_core::binary::Uuid;
use photon_net::backend::{BackendError, Launch, is_same_profile};

/// The link the game was started with, until the menu takes it.
/// How long a link is worth keeping: its launch code lasts a minute at the backend.
const FRESH: std::time::Duration = std::time::Duration::from_secs(60);

/// Tests that use the pending slot take turns, since it is one for the whole process.
#[cfg(test)]
pub(crate) static SLOT_TESTS: Mutex<()> = Mutex::new(());

/// A launch link a game ended holding: being spent, or spent to something the menu must say.
static HANDED_ON: Mutex<Option<LinkTask>> = Mutex::new(None);

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

/// The state of the pending sign-in link, once; a launch link stays pending.
pub(crate) fn take_pending_signin() -> Option<String> {
    let mut pending = PENDING.lock().ok()?;
    if !matches!(pending.as_ref(), Some((Link::SignIn(_), _))) {
        return None;
    }
    let (link, arrived) = pending.take()?;
    drop(pending);
    match link {
        Link::SignIn(state) if arrived.elapsed() <= FRESH => Some(state),
        _ => None,
    }
}

/// Puts a sign-in state back for the menu to take, unless a newer link already waits there.
pub(crate) fn requeue_signin(state: String) {
    if let Ok(mut pending) = PENDING.lock()
        && pending.is_none()
    {
        *pending = Some((Link::SignIn(state), std::time::Instant::now()));
    }
}

/// Keeps `task` for whoever takes it next, unless one is kept already.
pub(crate) fn hand_on(task: LinkTask) {
    if let Ok(mut kept) = HANDED_ON.lock()
        && kept.is_none()
    {
        *kept = Some(task);
    }
}

/// The launch link a game handed on, once: the menu or the next game takes it.
pub(crate) fn take_handed_on() -> Option<LinkTask> {
    HANDED_ON.lock().ok()?.take()
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
    /// Say why the link did nothing, under a heading, both by translation key.
    Say(&'static str, &'static str),
    /// Ask whether to sign the website's browser (the state) in as this player.
    AskSignIn(String, String),
}

/// The heading of what a launch link could not do.
const JOIN_FAILED: &str = "photon.notice.join_failed";

pub(crate) fn in_play(outcome: Outcome) -> InPlay {
    match outcome {
        Outcome::Join(address, name) => InPlay::Ask(address, name),
        Outcome::OtherProfile => InPlay::Say(JOIN_FAILED, "photon.link.other_profile"),
        Outcome::NotSignedIn => InPlay::Say(JOIN_FAILED, "photon.link.not_signed_in"),
        Outcome::Refused => InPlay::Say(JOIN_FAILED, "photon.link.refused"),
        Outcome::Unavailable => InPlay::Say(JOIN_FAILED, "photon.link.unavailable"),
    }
}

/// Where a sign-in link has got to in a game.
pub(crate) enum SignInStep {
    /// Looking up who is signed in, for the question; the browser's state waits.
    Looking(Receiver<Option<String>>, String),
    /// The browser's state, signed in and opened on a thread.
    Working(Receiver<crate::signin_link::Outcome>),
}

/// What a game holds of a link handed over while it is played.
#[derive(Default)]
pub(crate) struct GameLink {
    pub(crate) task: Option<LinkTask>,
    pub(crate) ready: Option<Outcome>,
    pub(crate) signin: Option<SignInStep>,
    /// What a sign-in link has to ask or say, until nothing else is open to be asked over.
    pub(crate) signin_ready: Option<InPlay>,
    /// The state the player agreed to sign in, to be started.
    pub(crate) signin_agreed: Option<String>,
}

impl GameLink {
    /// The sign-in state a game ends holding, asked about or not yet answered, taken out of it.
    /// One the player already answered but that no thread has started on counts too; a running
    /// sign-in does not (its thread still opens the browser).
    pub(crate) fn take_signin_state(&mut self, asked: Option<&ConfirmTarget>) -> Option<String> {
        let held = self.signin_agreed.take();
        let ready = match self.signin_ready.take() {
            Some(InPlay::AskSignIn(state, _)) => Some(state),
            _ => None,
        };
        let looking = match self.signin.take() {
            Some(SignInStep::Looking(_, state)) => Some(state),
            _ => None,
        };
        let asked = match asked {
            Some(ConfirmTarget::SignIn(state)) => Some(state.clone()),
            _ => None,
        };
        asked.or(held).or(ready).or(looking)
    }

    /// What a launch link still has to say when the game ends, as a task the menu polls: the one
    /// being spent, else a decision that is not a join (a join is joined, see `take_ready_join`).
    pub(crate) fn take_launch_task(&mut self) -> Option<LinkTask> {
        let spent = self
            .ready
            .take_if(|outcome| !matches!(outcome, Outcome::Join(..)));
        self.task.take().or(spent.map(LinkTask::decided))
    }

    /// The join a spent code was waiting to ask about, taken out of it.
    pub(crate) fn take_ready_join(&mut self) -> Option<(String, String)> {
        match self.ready.take() {
            Some(Outcome::Join(address, name)) => Some((address, name)),
            _ => None,
        }
    }
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

    /// A task whose decision is already made, to be handed on as it is.
    pub(crate) fn decided(outcome: Outcome) -> Self {
        let (sender, receiver) = channel();
        let _ = sender.send(outcome);
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

    #[test]
    fn a_game_hands_on_the_sign_in_state_nearest_to_the_player() {
        let mut held = GameLink::default();
        assert_eq!(held.take_signin_state(None), None);
        held.signin = Some(SignInStep::Looking(channel().1, "looking".to_owned()));
        held.signin_ready = Some(InPlay::AskSignIn("ready".to_owned(), "N".to_owned()));
        let asked = ConfirmTarget::SignIn("asked".to_owned());
        assert_eq!(
            held.take_signin_state(Some(&asked)).as_deref(),
            Some("asked")
        );
        // Everything it held is taken with it, so a second hand-over finds nothing.
        assert_eq!(held.take_signin_state(None), None);
        held.signin_ready = Some(InPlay::AskSignIn("ready".to_owned(), "N".to_owned()));
        assert_eq!(held.take_signin_state(None).as_deref(), Some("ready"));
        // A running sign-in finishes on its thread; only a notice would be lost.
        held.signin = Some(SignInStep::Working(channel().1));
        assert_eq!(held.take_signin_state(None), None);
    }

    #[test]
    fn a_game_hands_on_the_launch_task_or_a_decision_that_is_not_a_join() {
        let mut held = GameLink::default();
        assert!(held.take_launch_task().is_none());
        held.ready = Some(Outcome::Join("h:1".to_owned(), "G".to_owned()));
        assert!(held.take_launch_task().is_none());
        assert!(held.ready.is_some());
        held.ready = Some(Outcome::Refused);
        let task = held.take_launch_task().expect("task");
        assert_eq!(task.poll(), Some(Outcome::Refused));
        assert!(held.ready.is_none());
        held.task = Some(LinkTask::decided(Outcome::Unavailable));
        held.ready = Some(Outcome::NotSignedIn);
        let task = held.take_launch_task().expect("task");
        assert_eq!(task.poll(), Some(Outcome::Unavailable));
    }

    #[test]
    fn only_a_spent_codes_join_is_taken_as_a_join() {
        let mut held = GameLink {
            ready: Some(Outcome::Refused),
            ..GameLink::default()
        };
        assert_eq!(held.take_ready_join(), None);
        held.ready = Some(Outcome::Join("h:1".to_owned(), "G".to_owned()));
        assert_eq!(
            held.take_ready_join(),
            Some(("h:1".to_owned(), "G".to_owned()))
        );
        assert!(held.ready.is_none());
    }

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
        let _turn = SLOT_TESTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        remember(&format!("photon://signin?state={CODE}")).expect("remembered");
        assert_eq!(take_pending_launch(), None);
        assert!(take_pending().is_some());
        remember(&format!("photon://launch?code={CODE}")).expect("remembered");
        assert_eq!(take_pending_launch(), Some(CODE.to_owned()));
        assert_eq!(take_pending_launch(), None);
    }

    #[test]
    fn a_game_takes_a_sign_in_link_and_leaves_a_launch_link() {
        let _turn = SLOT_TESTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        remember(&format!("photon://launch?code={CODE}")).expect("remembered");
        assert_eq!(take_pending_signin(), None);
        assert!(take_pending().is_some());
        remember(&format!("photon://signin?state={CODE}")).expect("remembered");
        assert_eq!(take_pending_signin(), Some(CODE.to_owned()));
        assert_eq!(take_pending_signin(), None);
    }

    #[test]
    fn a_game_asks_about_a_join_and_says_the_rest() {
        assert_eq!(
            in_play(Outcome::Join("a:1".into(), "A".into())),
            InPlay::Ask("a:1".into(), "A".into())
        );
        assert_eq!(
            in_play(Outcome::Refused),
            InPlay::Say(JOIN_FAILED, "photon.link.refused")
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
