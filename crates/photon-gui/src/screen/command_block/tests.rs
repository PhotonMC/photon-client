use super::*;
use crate::widgets::edit_box::Clipboard;

struct Held(String);

impl Clipboard for Held {
    fn get(&mut self) -> String {
        self.0.clone()
    }

    fn set(&mut self, text: &str) {
        self.0 = text.to_owned();
    }
}

fn font() -> Option<Arc<Font>> {
    Font::load(std::path::Path::new(
        &std::env::var("PHOTON_ASSETS").unwrap_or_default(),
    ))
    .map(Arc::new)
}

fn contents(command: &str) -> CommandBlockContents {
    CommandBlockContents {
        command: command.to_owned(),
        ..CommandBlockContents::default()
    }
}

fn press(screen: &mut CommandBlockScreen, lang: &Lang, key: glfw::Key) -> CommandBlockAction {
    screen.key_pressed(lang, key, (false, false), &mut Held(String::new()))
}

#[test]
fn enter_sends_what_the_block_data_filled_in() {
    let Some(font) = font() else { return };
    let lang = Lang::default();
    let mut screen = CommandBlockScreen::new((&font, &lang), &contents(""), (427, 240));
    screen.update_gui(&lang, (CommandBlockMode::Auto, true), &contents("say hi"));
    screen.char_typed('!');
    let CommandBlockAction::Done(edit) = press(&mut screen, &lang, glfw::Key::Enter) else {
        panic!("Enter is Done");
    };
    assert_eq!(edit.command, "say hi!");
    assert_eq!(edit.mode, CommandBlockMode::Auto);
    assert!(edit.conditional && edit.track_output && !edit.automatic);
}

#[test]
fn escape_closes_without_sending() {
    let Some(font) = font() else { return };
    let lang = Lang::default();
    let mut screen = CommandBlockScreen::new((&font, &lang), &contents("say hi"), (427, 240));
    assert!(matches!(
        press(&mut screen, &lang, glfw::Key::Escape),
        CommandBlockAction::Close
    ));
}

#[test]
fn the_buttons_sleep_until_the_data_arrives() {
    let Some(font) = font() else { return };
    let lang = Lang::default();
    let mut screen = CommandBlockScreen::new((&font, &lang), &contents(""), (427, 240));
    // Tab from the command box skips the previous output (empty) and the sleeping buttons: only
    // Cancel can take the focus.
    press(&mut screen, &lang, glfw::Key::Tab);
    assert!(matches!(
        press(&mut screen, &lang, glfw::Key::Enter),
        CommandBlockAction::Close
    ));
}
