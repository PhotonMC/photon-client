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
        &photon_core::content_root::assets().unwrap_or_default(),
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
    assert!(edit.track_output);
    assert_eq!(
        edit.block,
        Some(BlockSettings {
            mode: CommandBlockMode::Auto,
            conditional: true,
            automatic: false,
        })
    );
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
    // Tab from the command box goes to the last output's box, then skips the sleeping buttons:
    // only Cancel can take the focus after it.
    press(&mut screen, &lang, glfw::Key::Tab);
    press(&mut screen, &lang, glfw::Key::Tab);
    assert!(matches!(
        press(&mut screen, &lang, glfw::Key::Enter),
        CommandBlockAction::Close
    ));
}

#[test]
fn the_minecart_screen_is_awake_with_its_command_and_sends_no_block_settings() {
    let Some(font) = font() else { return };
    let lang = Lang::default();
    let mut screen = CommandBlockScreen::minecart((&font, &lang), &contents("say hi"), (427, 240));
    let CommandBlockAction::Done(edit) = press(&mut screen, &lang, glfw::Key::Enter) else {
        panic!("Enter is Done");
    };
    assert_eq!(edit.command, "say hi");
    assert!(edit.track_output);
    assert_eq!(edit.block, None);
}

#[test]
fn the_minecart_screen_tabs_over_five_widgets() {
    let Some(font) = font() else { return };
    let lang = Lang::default();
    let mut screen = CommandBlockScreen::minecart((&font, &lang), &contents("x"), (427, 240));
    let mut seen = Vec::new();
    for _ in 0..5 {
        press(&mut screen, &lang, glfw::Key::Tab);
        seen.push(screen.focus);
    }
    assert_eq!(
        seen,
        [
            Focus::Previous,
            Focus::Output,
            Focus::Done,
            Focus::Cancel,
            Focus::Command
        ]
    );
}
