//! The buttons of the command block screens, and where they sit.

use super::*;

/// The buttons, in the order they were added.
pub(super) struct Buttons {
    pub(super) arena: Arena,
    pub(super) output: CycleButton<bool>,
    /// A block's mode, condition and trigger.
    pub(super) extra: Option<Extra>,
    pub(super) done: Widget,
    pub(super) cancel: Widget,
}

pub(super) struct Extra {
    pub(super) mode: CycleButton<CommandBlockMode>,
    pub(super) conditional: CycleButton<bool>,
    pub(super) automatic: CycleButton<bool>,
}

/// The values the buttons start from.
#[derive(Clone, Copy)]
pub(super) struct Values {
    pub(super) track_output: bool,
    pub(super) block: BlockSettings,
}

fn mode_text(mode: &CommandBlockMode) -> Component {
    Component::translatable(match mode {
        CommandBlockMode::Sequence => "advMode.mode.sequence",
        CommandBlockMode::Auto => "advMode.mode.auto",
        CommandBlockMode::Redstone => "advMode.mode.redstone",
    })
}

fn output_text(tracked: &bool) -> Component {
    Component::literal(if *tracked { "O" } else { "X" })
}

fn conditional_text(conditional: &bool) -> Component {
    Component::translatable(if *conditional {
        "advMode.mode.conditional"
    } else {
        "advMode.mode.unconditional"
    })
}

fn automatic_text(automatic: &bool) -> Component {
    Component::translatable(if *automatic {
        "advMode.mode.autoexec.bat"
    } else {
        "advMode.mode.redstoneTriggered"
    })
}

/// The rectangles of the buttons for a screen `size` big.
pub(super) struct Rects {
    pub(super) output: usize,
    pub(super) extra: Option<[usize; 3]>,
    pub(super) done: usize,
    pub(super) cancel: usize,
}

pub(super) fn arena_of((width, height): (i32, i32), kind: CommandBlockKind) -> (Arena, Rects) {
    let mut arena = Arena::default();
    let mut place = |x: i32, y: i32, w: i32| {
        let rect = arena.add(w, 20);
        arena.0[rect].x = x;
        arena.0[rect].y = y;
        rect
    };
    let output = place(width / 2 + 150 - 20, kind.previous_y(), 20);
    let extra = (kind == CommandBlockKind::Block).then(|| {
        [
            place(width / 2 - 50 - 100 - 4, 165, 100),
            place(width / 2 - 50, 165, 100),
            place(width / 2 + 50 + 4, 165, 100),
        ]
    });
    let done = place(width / 2 - 4 - 150, height / 4 + 120 + 12, 150);
    let cancel = place(width / 2 + 4, height / 4 + 120 + 12, 150);
    let rects = Rects {
        output,
        extra,
        done,
        cancel,
    };
    (arena, rects)
}

fn button(rect: usize, active: bool, label: Vec<photon_game::text::Run>) -> Widget {
    Widget {
        rect,
        active,
        kind: Kind::Button { label },
    }
}

impl Buttons {
    /// `init` and `addExtraControls`: the buttons for a screen `size` big.
    pub(super) fn new(
        lang: &Lang,
        size: (i32, i32),
        (kind, values): (CommandBlockKind, Values),
        active: bool,
    ) -> Self {
        let (arena, rects) = arena_of(size, kind);
        let name = |key: &str| Component::translatable(key);
        let runs = |key: &str| lang.runs(&name(key), &Style::default());
        let extra = rects
            .extra
            .map(|rects| Extra::new(lang, rects, values.block));
        let mut buttons = Self {
            output: CycleButton::new(
                lang,
                rects.output,
                name("advMode.trackOutput"),
                vec![true, false],
                output_text,
                &values.track_output,
            )
            .only_value(lang),
            extra,
            done: button(rects.done, active, runs("gui.done")),
            cancel: button(rects.cancel, true, runs("gui.cancel")),
            arena,
        };
        buttons.enable(active);
        buttons
    }

    /// `enableControls`.
    pub(super) fn enable(&mut self, active: bool) {
        self.done.active = active;
        self.output.set_active(active);
        if let Some(extra) = &mut self.extra {
            extra.mode.set_active(active);
            extra.conditional.set_active(active);
            extra.automatic.set_active(active);
        }
    }
}

impl Extra {
    /// The mode, condition and trigger buttons at their rectangles.
    fn new(lang: &Lang, [mode, conditional, automatic]: [usize; 3], block: BlockSettings) -> Self {
        let name = |key: &str| Component::translatable(key);
        Self {
            mode: CycleButton::new(
                lang,
                mode,
                name("advMode.mode"),
                vec![
                    CommandBlockMode::Sequence,
                    CommandBlockMode::Auto,
                    CommandBlockMode::Redstone,
                ],
                mode_text,
                &block.mode,
            )
            .only_value(lang),
            conditional: CycleButton::new(
                lang,
                conditional,
                name("advMode.type"),
                vec![true, false],
                conditional_text,
                &block.conditional,
            )
            .only_value(lang),
            automatic: CycleButton::new(
                lang,
                automatic,
                name("advMode.triggering"),
                vec![true, false],
                automatic_text,
                &block.automatic,
            )
            .only_value(lang),
        }
    }
}
