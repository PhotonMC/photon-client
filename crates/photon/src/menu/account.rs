//! The Microsoft sign-in of the menu (ADR 0020): a device code is asked for, shown with the
//! address to type it at, and polled for off the menu's thread. A player who cancels leaves
//! nothing stored, even if they finish on their phone later.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};

use photon_game::text::Component;
use photon_gui::screen::notice::{Answer, NoticeScreen};
use photon_net::signin::{self, ClientId, Hosts, SignInError};

use super::input::{ask_event, failed_event};
use super::{Client, Menu, Screen};

/// A sign-in under way: what its thread says, and the flag that stops it saving.
pub(super) struct AccountSignIn {
    steps: Receiver<Step>,
    cancelled: Arc<AtomicBool>,
}

enum Step {
    /// The code to type and where to type it.
    Code { user_code: String, address: String },
    /// How it ended: the player's name, or why not.
    Done(Result<String, SignInError>),
}

impl AccountSignIn {
    pub(super) fn start() -> Self {
        let (sender, steps) = channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&cancelled);
        let _ = std::thread::Builder::new()
            .name("Microsoft Sign-in".to_owned())
            .spawn(move || {
                let done = work(&sender, &flag);
                let _ = sender.send(Step::Done(done));
            });
        Self { steps, cancelled }
    }

    /// The player gave up: whatever the thread gets is dropped, not stored.
    pub(super) fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// The whole sign-in: the code, the wait for the player, then the session and the stored token.
fn work(sender: &Sender<Step>, cancelled: &AtomicBool) -> Result<String, SignInError> {
    let (client, hosts) = (ClientId::from_environment(), Hosts::default());
    let code = signin::request_device_code(&hosts, &client)?;
    let address = code.verification_uri.clone();
    let _ = sender.send(Step::Code {
        user_code: code.user_code.clone(),
        address: address.clone(),
    });
    // The page that takes the code opens by itself; only an `https` address is handed on.
    if address.starts_with("https://") {
        crate::signin_link::open(&address);
    }
    let tokens = signin::poll_for_token(&hosts, &client, &code, &mut std::thread::sleep)?;
    // The token is kept only for a player who is still waiting for it.
    if cancelled.load(Ordering::SeqCst) {
        return Err(SignInError::Declined);
    }
    let host = crate::profile_login::session_host();
    let store = crate::keyring_store::os_store();
    let session = signin::finish_sign_in(&hosts, &tokens, (&host, store.as_ref()))?;
    let name = session.account.name.clone();
    crate::signin_hint::write(&name);
    crate::profile_login::keep(session.account);
    Ok(name)
}

/// The words for a sign-in that did not finish.
fn failure_key(error: &SignInError) -> &'static str {
    match error {
        SignInError::Declined => "photon.account.declined",
        SignInError::Expired => "photon.account.expired",
        SignInError::NotOwned => "photon.account.not_owned",
        SignInError::NoProfile => "photon.account.no_profile",
        SignInError::Unavailable => "photon.account.unavailable",
        _ => "photon.account.failed",
    }
}

/// `template` with each `%s` filled in turn.
fn filled(template: &str, parts: &[&str]) -> String {
    parts.iter().fold(template.to_owned(), |text, part| {
        text.replacen("%s", part, 1)
    })
}

impl Menu {
    /// The question of a player who is not signed in: sign in now, or leave it.
    pub(super) fn offer_sign_in(&mut self, reason_key: &str, size: (i32, i32)) {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let question = NoticeScreen::asking(
            font,
            lang,
            size,
            (
                &Component::translatable("photon.account.heading"),
                &Component::translatable(reason_key),
                &Component::translatable("photon.account.not_now"),
                &Component::translatable("photon.account.sign_in"),
            ),
        );
        self.screen = Screen::Offer(question);
    }

    /// One window event for the offer or the code screen.
    pub(super) fn account_event(&mut self, event: glfw::WindowEvent, size: (i32, i32)) {
        if matches!(self.screen, Screen::Offer(..)) {
            self.offer_event(event, size);
        } else {
            self.code_event(event, size);
        }
    }

    /// The answer to the offer: a no goes home; a yes asks Microsoft for a code.
    fn offer_event(&mut self, event: glfw::WindowEvent, size: (i32, i32)) {
        let mouse = self.mouse;
        let Screen::Offer(screen) = &mut self.screen else {
            return;
        };
        match ask_event(screen, event, mouse) {
            Some(Answer::Back) => self.screen = self.title(size),
            Some(Answer::Yes) => {
                self.account = Some(AccountSignIn::start());
                self.show_linking(size);
            }
            None => {}
        }
    }

    /// The code screen's Cancel or Escape: the thread is told, and the player goes home.
    fn code_event(&mut self, event: glfw::WindowEvent, size: (i32, i32)) {
        let mouse = self.mouse;
        let Screen::Code(screen) = &mut self.screen else {
            return;
        };
        if failed_event(screen, event, mouse) {
            self.cancel_account();
            self.screen = self.title(size);
        }
    }

    /// Stops a sign-in that is under way, so nothing it gets is kept.
    pub(super) fn cancel_account(&mut self) {
        if let Some(account) = self.account.take() {
            account.cancel();
        }
    }

    /// The next step of a Microsoft sign-in, when its thread has one.
    pub(super) fn account_polled(&mut self, size: (i32, i32), client: &mut Client) {
        let Some(account) = self.account.as_ref() else {
            return;
        };
        match account.steps.try_recv() {
            Ok(Step::Code { user_code, address }) => {
                // The player types it on another page, so it is already on the clipboard.
                client.surface.set_clipboard(&user_code);
                self.show_code(&user_code, &address, size);
            }
            Ok(Step::Done(result)) => {
                self.account = None;
                let heading = Component::translatable("photon.account.heading");
                let lang = self.gui.lang();
                let reason = match result {
                    Ok(name) => filled(&lang.text("photon.account.done"), &[&name]),
                    Err(SignInError::Xbox(reason)) => reason,
                    Err(error) => lang.text(failure_key(&error)),
                };
                self.notice_screen(&heading, &Component::literal(reason), size, true);
            }
            Err(_) => {}
        }
    }

    fn show_code(&mut self, user_code: &str, address: &str, size: (i32, i32)) {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let reason = filled(&lang.text("photon.account.enter"), &[address, user_code]);
        self.screen = Screen::Code(NoticeScreen::new(
            font,
            lang,
            size,
            (
                &Component::translatable("photon.account.heading"),
                &Component::literal(reason),
                &Component::translatable("photon.account.cancel"),
            ),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_words_are_filled_in_order() {
        assert_eq!(
            filled("Open %s and enter %s", &["a.example/link", "ABCD"]),
            "Open a.example/link and enter ABCD"
        );
    }

    #[test]
    fn each_failure_has_its_words() {
        assert_eq!(failure_key(&SignInError::Expired), "photon.account.expired");
        assert_eq!(
            failure_key(&SignInError::NotOwned),
            "photon.account.not_owned"
        );
        assert_eq!(
            failure_key(&SignInError::Store("x".into())),
            "photon.account.failed"
        );
    }
}
