//! The question a server's mod raises: which server asks, which mod (by id and a short form of
//! its hash) and what it may do, with Allow and Deny. It is Photon's own notice screen over the
//! dimmed game, like the questions of Photon's links; the game and its connection go on behind
//! it. One question is up at a time and the rest wait their turn.

use super::*;
use photon_game::text::{Component, Lang};
use photon_gui::screen::notice::NoticeScreen;
use photon_net::mods::Question as ModQuestion;

/// How many characters of the hash the question shows.
const HASH_SHOWN: usize = 12;

/// The heading and the words of the question about `question`, asked by `server`.
pub(super) fn consent_texts(lang: &Lang, server: &str, question: &ModQuestion) -> (String, String) {
    let shown: String = question.hash.chars().take(HASH_SHOWN).collect();
    let caps = question
        .caps
        .iter()
        .map(|cap| {
            let key = format!("photon.mods.cap.{cap}");
            let words = lang.text(&key);
            if words == key { cap.clone() } else { words }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let ask = lang
        .text("photon.mods.ask")
        .replacen("%s", server, 1)
        .replacen("%s", &question.id, 1)
        .replacen("%s", &shown, 1)
        .replacen("%s", &caps, 1);
    (lang.text("photon.mods.heading"), ask)
}

impl Controls {
    /// Opens the question about `question` when nothing else is up over the game: the mouse is let
    /// go, as for any screen. Returns whether it opened.
    pub(crate) fn ask_about_mod(
        &mut self,
        (server, question): (&str, &ModQuestion),
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) -> bool {
        if self.question_open() || self.any_screen_open() || self.pack_prompt.is_some() {
            return false;
        }
        let lang = &assets.lang;
        let (heading, reason) = consent_texts(lang, server, question);
        let text = |key: &str| Component::literal(lang.text(key));
        let mut screen = NoticeScreen::asking(
            &assets.creative.font,
            lang,
            Self::gui_size(surface),
            (
                &Component::literal(heading),
                &Component::literal(reason),
                &text("photon.mods.deny"),
                &text("photon.mods.allow"),
            ),
        )
        .over_game();
        // Opens by itself over whatever the player was doing: wakes after a second.
        screen.set_delay(20);
        self.link_confirm = Some((screen.into(), ConfirmTarget::Mod(question.id.clone())));
        if let Some(chat) = self.chat.take() {
            self.leave_chat(&chat, ExitReason::Interrupted);
        }
        self.captured = false;
        self.release_keys();
        surface.set_cursor_captured(false);
        true
    }

    /// The player's answer: kept for the game to hand to the mods, and the mouse goes back to the
    /// game unless something else holds it.
    pub(super) fn answered_mod(&mut self, id: &str, granted: bool, surface: &mut WindowSurface) {
        self.game_link.mod_answers.push((id.to_owned(), granted));
        if !self.any_screen_open() && self.disconnected.is_none() {
            self.captured = true;
            surface.set_cursor_captured(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn question() -> ModQuestion {
        ModQuestion {
            id: "hud".to_owned(),
            hash: "0123456789abcdef0123".to_owned(),
            caps: vec!["fs.mod_data".to_owned()],
        }
    }

    fn lang_with(pairs: &[(&str, &str)]) -> Lang {
        Lang::new(
            pairs
                .iter()
                .map(|(key, text)| ((*key).to_owned(), (*text).to_owned())),
        )
    }

    #[test]
    fn the_text_holds_the_server_the_mod_a_short_hash_and_the_capabilities() {
        let lang = lang_with(&[
            ("photon.mods.ask", "%s / %s / %s / %s"),
            ("photon.mods.cap.fs.mod_data", "files"),
        ]);
        let (_, words) = consent_texts(&lang, "s:1", &question());
        assert_eq!(words, "s:1 / hud / 0123456789ab / files");
    }

    #[test]
    fn a_capability_without_words_shows_its_id() {
        let lang = lang_with(&[("photon.mods.ask", "%s %s %s %s")]);
        let mut asked = question();
        asked.caps = vec!["x.y".to_owned(), "fs.mod_data".to_owned()];
        let (_, words) = consent_texts(&lang, "s", &asked);
        assert!(words.ends_with("x.y, fs.mod_data"));
    }
}
