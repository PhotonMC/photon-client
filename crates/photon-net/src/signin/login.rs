//! Staying signed in: the refresh token lives in the operating system's credential store (never in
//! a file the client writes), and a later start exchanges it for a new session.

use super::{ClientId, Hosts, MicrosoftTokens, MinecraftSession, SignInError};
use super::{minecraft_session, refresh};
use crate::account::AccessToken;

/// Where the one secret of a signed-in player is kept. The operating system's keyring is the
/// real one; a test brings its own.
pub trait SecretStore {
    /// The refresh token kept, if there is one.
    fn load(&self) -> Result<Option<AccessToken>, SignInError>;
    fn save(&self, token: &AccessToken) -> Result<(), SignInError>;
    fn clear(&self) -> Result<(), SignInError>;
}

/// Finish a sign-in that just collected Microsoft tokens: the Minecraft session is made first, so
/// a refused account (no game, no profile) keeps nothing, then the refresh token is stored.
pub fn finish_sign_in(
    hosts: &Hosts,
    microsoft: &MicrosoftTokens,
    (session_host, store): (&str, &dyn SecretStore),
) -> Result<MinecraftSession, SignInError> {
    let session = minecraft_session(hosts, microsoft, session_host)?;
    store.save(microsoft.refresh_token())?;
    Ok(session)
}

/// Sign in again from the stored refresh token. `Ok(None)` is nobody signed in: nothing stored,
/// or the token no longer works (then it is dropped, so the next start asks for a new code).
pub fn resume(
    hosts: &Hosts,
    client: &ClientId,
    (session_host, store): (&str, &dyn SecretStore),
) -> Result<Option<MinecraftSession>, SignInError> {
    let Some(stored) = store.load()? else {
        return Ok(None);
    };
    let microsoft = match refresh(hosts, client, &stored) {
        Ok(tokens) => tokens,
        Err(SignInError::Expired) => {
            store.clear()?;
            return Ok(None);
        }
        Err(other) => return Err(other),
    };
    // The service rotates the refresh token: keep the new one before anything else can fail.
    store.save(microsoft.refresh_token())?;
    minecraft_session(hosts, &microsoft, session_host).map(Some)
}

/// Forget the signed-in player.
pub fn sign_out(store: &dyn SecretStore) -> Result<(), SignInError> {
    store.clear()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::super::http::tests::{json_answer, stub_host};
    use super::*;

    /// A store in memory, which also counts how it was used.
    #[derive(Default)]
    struct Memory {
        token: RefCell<Option<String>>,
    }

    impl SecretStore for Memory {
        fn load(&self) -> Result<Option<AccessToken>, SignInError> {
            Ok(self.token.borrow().clone().map(AccessToken::new))
        }

        fn save(&self, token: &AccessToken) -> Result<(), SignInError> {
            *self.token.borrow_mut() = Some(token.reveal().to_owned());
            Ok(())
        }

        fn clear(&self) -> Result<(), SignInError> {
            *self.token.borrow_mut() = None;
            Ok(())
        }
    }

    const TOKEN_BODY: &str = r#"{"access_token":"ACC","refresh_token":"NEWREF","expires_in":3599}"#;
    const CHAIN: [&str; 5] = [
        r#"{"Token":"X","DisplayClaims":{"xui":[{"uhs":"H"}]}}"#,
        r#"{"Token":"S","DisplayClaims":{"xui":[{"uhs":"H"}]}}"#,
        r#"{"access_token":"MC","expires_in":86400}"#,
        r#"{"items":[{"name":"game_minecraft"}]}"#,
        r#"{"id":"0123456789abcdeffedcba9876543210","name":"Alex"}"#,
    ];

    fn hosts(host: &str) -> Hosts {
        Hosts {
            login: host.to_owned(),
            live: host.to_owned(),
            xbox_user: host.to_owned(),
            xsts: host.to_owned(),
            services: host.to_owned(),
        }
    }

    fn client() -> ClientId {
        ClientId::new("client-1".to_owned())
    }

    fn chain_answers(first: Option<String>) -> Vec<String> {
        let mut answers: Vec<String> = first.into_iter().collect();
        answers.extend(CHAIN.iter().map(|body| json_answer("200 OK", body)));
        answers
    }

    #[test]
    fn a_stored_token_is_exchanged_rotated_and_gives_the_session() {
        let (host, seen) = stub_host(chain_answers(Some(json_answer("200 OK", TOKEN_BODY))));
        let store = Memory::default();
        store.save(&AccessToken::new("OLDREF".to_owned())).unwrap();
        let session = resume(&hosts(&host), &client(), ("https://s.example", &store))
            .expect("no error")
            .expect("a session");
        assert_eq!(session.account.name, "Alex");
        assert_eq!(store.token.borrow().as_deref(), Some("NEWREF"));
        assert!(seen.join().unwrap()[0].contains("refresh_token=OLDREF"));
    }

    #[test]
    fn nothing_stored_is_nobody_signed_in_and_asks_nothing() {
        let store = Memory::default();
        // No stub host: a request would fail the test with an unreachable host.
        let result = resume(
            &hosts("http://127.0.0.1:1"),
            &client(),
            ("https://s.example", &store),
        );
        assert_eq!(result.expect("no error"), None);
    }

    #[test]
    fn a_dead_refresh_token_is_dropped() {
        let (host, _) = stub_host(vec![json_answer(
            "400 Bad Request",
            r#"{"error":"invalid_grant"}"#,
        )]);
        let store = Memory::default();
        store.save(&AccessToken::new("OLDREF".to_owned())).unwrap();
        let result = resume(&hosts(&host), &client(), ("https://s.example", &store));
        assert_eq!(result.expect("no error"), None);
        assert_eq!(*store.token.borrow(), None);
    }

    #[test]
    fn a_service_outage_keeps_the_stored_token() {
        let (host, _) = stub_host(vec![json_answer("503 Service Unavailable", "")]);
        let store = Memory::default();
        store.save(&AccessToken::new("OLDREF".to_owned())).unwrap();
        let result = resume(&hosts(&host), &client(), ("https://s.example", &store));
        assert_eq!(result.err(), Some(SignInError::Unavailable));
        assert_eq!(store.token.borrow().as_deref(), Some("OLDREF"));
    }

    #[test]
    fn a_finished_sign_in_stores_the_refresh_token_only_for_an_accepted_account() {
        let (host, _) = stub_host(chain_answers(None));
        let store = Memory::default();
        let microsoft = MicrosoftTokens {
            access: AccessToken::new("ACC".to_owned()),
            ticket_prefix: "d=",
            refresh: AccessToken::new("REF".to_owned()),
            expires_in: std::time::Duration::from_secs(1),
        };
        finish_sign_in(&hosts(&host), &microsoft, ("https://s.example", &store)).expect("session");
        assert_eq!(store.token.borrow().as_deref(), Some("REF"));
        sign_out(&store).unwrap();
        assert_eq!(*store.token.borrow(), None);

        let (host, _) = stub_host(vec![
            json_answer("200 OK", CHAIN[0]),
            json_answer("200 OK", CHAIN[1]),
            json_answer("200 OK", CHAIN[2]),
            json_answer("200 OK", r#"{"items":[]}"#),
        ]);
        let refused = finish_sign_in(&hosts(&host), &microsoft, ("https://s.example", &store));
        assert_eq!(refused.err(), Some(SignInError::NotOwned));
        assert_eq!(*store.token.borrow(), None);
    }
}
