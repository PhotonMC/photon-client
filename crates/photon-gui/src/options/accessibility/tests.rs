use super::*;

fn assets() -> Option<(Font, Lang)> {
    let root = std::path::PathBuf::from(std::env::var("PHOTON_ASSETS").ok()?);
    Some((Font::load(&root)?, Lang::load(&root)))
}

fn screen(font: &Font, lang: &Lang) -> AccessibilityScreen {
    AccessibilityScreen::new(
        font,
        lang,
        (427, 240),
        true,
        (&Options::default(), false, false),
    )
}

#[test]
fn the_footer_and_the_controls_link_say_what_was_pressed() {
    let Some((font, lang)) = assets() else { return };
    let mut screen = screen(&font, &lang);
    // The Controls link is the right of the first row; the guide and Done make up the footer.
    let controls = screen.clicked(&lang, (300.0, 45.0), false);
    assert_eq!(
        (controls.controls, controls.guide, controls.done),
        (true, false, false)
    );
    let guide = screen.clicked(&lang, (134.0, 223.0), false);
    assert_eq!(
        (guide.controls, guide.guide, guide.done),
        (false, true, false)
    );
    let done = screen.clicked(&lang, (293.0, 223.0), false);
    assert_eq!((done.controls, done.guide, done.done), (false, false, true));
}

#[test]
fn enter_on_a_focused_footer_button_presses_it() {
    let Some((font, lang)) = assets() else { return };
    let mut screen = screen(&font, &lang);
    // Shift+Tab goes from nothing to Done, then to the guide.
    screen.tab(false);
    let done = screen.key_pressed(&lang, SliderKey::Select, false);
    assert!(done.done && !done.guide);
    screen.tab(false);
    let guide = screen.key_pressed(&lang, SliderKey::Select, false);
    assert!(guide.guide && !guide.done);
}
