//! What waits for a player's own Microsoft sign-in, and what it becomes once they are signed in.

use photon_core::binary::Uuid;
use photon_net::backend::Launch;

use crate::launch_link::{Outcome, decide};

/// A link that could not go on because nobody was signed in.
pub(super) enum Resume {
    /// A website sign-in: the browser's state waits to be asked about.
    Website(String),
    /// A launch whose code is spent: the server it named waits to be joined.
    Launch(Launch),
}

/// What the menu does when the sign-in is over.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum After {
    /// Ask whether the website's browser may be signed in, with this state.
    Ask(String),
    /// The decision about the launch, as if the link had just been answered.
    Link(Outcome),
}

/// The next step for what waited, given the profile that is signed in now.
pub(super) fn after_sign_in(resume: Resume, signed_in: Option<Uuid>) -> After {
    match resume {
        Resume::Website(state) => After::Ask(state),
        Resume::Launch(launch) => After::Link(decide(Ok(launch), signed_in)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use photon_net::account::parse_profile_id;

    const OWN: &str = "0123456789abcdeffedcba9876543210";

    fn launch(uuid: &str) -> Launch {
        Launch {
            address: "h:1".to_owned(),
            uuid: uuid.to_owned(),
            name: "G".to_owned(),
        }
    }

    #[test]
    fn a_website_sign_in_asks_its_question_whoever_signed_in() {
        let after = after_sign_in(Resume::Website("s".to_owned()), None);
        assert_eq!(after, After::Ask("s".to_owned()));
    }

    #[test]
    fn a_launch_joins_for_its_own_profile() {
        let profile = parse_profile_id(OWN).unwrap();
        let after = after_sign_in(Resume::Launch(launch(OWN)), Some(profile));
        assert_eq!(
            after,
            After::Link(Outcome::Join("h:1".to_owned(), "G".to_owned()))
        );
    }

    #[test]
    fn a_launch_for_another_profile_is_refused() {
        let profile = parse_profile_id(OWN).unwrap();
        let other = "ffffffffffffffffffffffffffffffff";
        let after = after_sign_in(Resume::Launch(launch(other)), Some(profile));
        assert_eq!(after, After::Link(Outcome::OtherProfile));
    }
}
