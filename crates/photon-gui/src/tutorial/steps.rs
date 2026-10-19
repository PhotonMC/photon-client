//! The step instances: one for each step of the tutorial.

use super::{Command, Facts, ItemKind, Keys, Out, ToastSpec, Words};
use crate::toasts::TutorialIcon;
use photon_game::options::TutorialStep;

const MOVE: ToastSpec = ToastSpec {
    icon: TutorialIcon::MovementKeys,
    title: Words {
        key: "tutorial.move.title",
        keybinds: &["forward", "left", "back", "right"],
    },
    message: Some(Words {
        key: "tutorial.move.description",
        keybinds: &["jump"],
    }),
    progressable: true,
};

const LOOK: ToastSpec = ToastSpec {
    icon: TutorialIcon::Mouse,
    title: Words {
        key: "tutorial.look.title",
        keybinds: &[],
    },
    message: Some(Words {
        key: "tutorial.look.description",
        keybinds: &[],
    }),
    progressable: true,
};

const FIND_TREE: ToastSpec = ToastSpec {
    icon: TutorialIcon::Tree,
    title: Words {
        key: "tutorial.find_tree.title",
        keybinds: &[],
    },
    message: Some(Words {
        key: "tutorial.find_tree.description",
        keybinds: &[],
    }),
    progressable: false,
};

const PUNCH_TREE: ToastSpec = ToastSpec {
    icon: TutorialIcon::Tree,
    title: Words {
        key: "tutorial.punch_tree.title",
        keybinds: &[],
    },
    message: Some(Words {
        key: "tutorial.punch_tree.description",
        keybinds: &["attack"],
    }),
    progressable: true,
};

const OPEN_INVENTORY: ToastSpec = ToastSpec {
    icon: TutorialIcon::RecipeBook,
    title: Words {
        key: "tutorial.open_inventory.title",
        keybinds: &[],
    },
    message: Some(Words {
        key: "tutorial.open_inventory.description",
        keybinds: &["inventory"],
    }),
    progressable: false,
};

const CRAFT_PLANKS: ToastSpec = ToastSpec {
    icon: TutorialIcon::WoodenPlanks,
    title: Words {
        key: "tutorial.craft_planks.title",
        keybinds: &[],
    },
    message: Some(Words {
        key: "tutorial.craft_planks.description",
        keybinds: &[],
    }),
    progressable: false,
};

/// A step that waits and then puts a hint up: the ticks, and whether the hint is up.
#[derive(Default)]
pub(super) struct Timed {
    time: u32,
    shown: bool,
}

impl Timed {
    /// Puts the hint up once `delay` ticks have passed (`timeWaiting >= HINT_DELAY`).
    fn hint(&mut self, delay: u32, spec: ToastSpec, out: &mut Out) {
        if self.time >= delay && !self.shown {
            self.shown = true;
            out.commands.push(Command::Show(spec));
        }
    }

    fn clear(&mut self, icon: TutorialIcon, out: &mut Out) {
        if std::mem::take(&mut self.shown) {
            out.commands.push(Command::Hide(icon));
        }
    }
}

/// `MovementTutorialStepInstance`.
#[derive(Default)]
pub(super) struct Movement {
    waiting: u32,
    moved_ticks: u32,
    looked_ticks: u32,
    moved: bool,
    turned: bool,
    move_done: Option<u32>,
    look_done: Option<u32>,
    move_toast: bool,
    look_toast: bool,
}

impl Movement {
    fn tick(&mut self, survival: bool, out: &mut Out) -> Option<TutorialStep> {
        self.waiting += 1;
        self.moved_ticks += u32::from(std::mem::take(&mut self.moved));
        self.looked_ticks += u32::from(std::mem::take(&mut self.turned));
        if self.move_done.is_none() && self.moved_ticks > 40 {
            hide(&mut self.move_toast, TutorialIcon::MovementKeys, out);
            self.move_done = Some(self.waiting);
        }
        if self.look_done.is_none() && self.looked_ticks > 40 {
            hide(&mut self.look_toast, TutorialIcon::Mouse, out);
            self.look_done = Some(self.waiting);
        }
        let next = (self.move_done.is_some() && self.look_done.is_some()).then_some(if survival {
            TutorialStep::FindTree
        } else {
            TutorialStep::None
        });
        self.progress(out);
        self.hints(out);
        next
    }

    fn progress(&self, out: &mut Out) {
        if self.move_toast {
            let value = self.moved_ticks as f32 / 40.0;
            out.commands
                .push(Command::Progress(TutorialIcon::MovementKeys, value));
        }
        if self.look_toast {
            let value = self.looked_ticks as f32 / 40.0;
            out.commands
                .push(Command::Progress(TutorialIcon::Mouse, value));
        }
    }

    /// The movement hint after 100 ticks, the look hint 20 ticks after the movement is done.
    fn hints(&mut self, out: &mut Out) {
        if self.waiting < 100 {
            return;
        }
        match (self.move_done, self.look_done) {
            (None, _) if !self.move_toast => {
                self.move_toast = true;
                out.commands.push(Command::Show(MOVE));
            }
            (Some(done), None) if self.waiting - done >= 20 && !self.look_toast => {
                self.look_toast = true;
                out.commands.push(Command::Show(LOOK));
            }
            _ => {}
        }
    }

    fn clear(&mut self, out: &mut Out) {
        hide(&mut self.move_toast, TutorialIcon::MovementKeys, out);
        hide(&mut self.look_toast, TutorialIcon::Mouse, out);
    }
}

fn hide(shown: &mut bool, icon: TutorialIcon, out: &mut Out) {
    if std::mem::take(shown) {
        out.commands.push(Command::Hide(icon));
    }
}

/// `PunchTreeTutorialStepInstance`.
#[derive(Default)]
pub(super) struct Punch {
    timed: Timed,
    resets: u32,
}

pub(super) enum Step {
    Movement(Movement),
    FindTree(Timed),
    PunchTree(Punch),
    OpenInventory(Timed),
    CraftPlanks(Timed),
    Done,
}

/// A step and the step it asked to go to.
pub(crate) struct Instance {
    step: Step,
    next: Option<TutorialStep>,
}

impl Instance {
    pub(super) fn new(step: TutorialStep) -> Self {
        let step = match step {
            TutorialStep::Movement => Step::Movement(Movement::default()),
            TutorialStep::FindTree => Step::FindTree(Timed::default()),
            TutorialStep::PunchTree => Step::PunchTree(Punch::default()),
            TutorialStep::OpenInventory => Step::OpenInventory(Timed::default()),
            TutorialStep::CraftPlanks => Step::CraftPlanks(Timed::default()),
            TutorialStep::None => Step::Done,
        };
        Self { step, next: None }
    }

    pub(super) fn take_next(&mut self) -> Option<TutorialStep> {
        self.next.take()
    }

    pub(super) fn tick(&mut self, facts: &dyn Facts, out: &mut Out) {
        self.next = match &mut self.step {
            Step::Movement(movement) => movement.tick(facts.survival(), out),
            Step::Done => None,
            _ if !facts.survival() => Some(TutorialStep::None),
            Step::FindTree(timed) => find_tree_tick(timed, facts, out),
            Step::PunchTree(punch) => punch_tick(punch, facts, out),
            Step::OpenInventory(timed) => {
                timed.time += 1;
                timed.hint(600, OPEN_INVENTORY, out);
                None
            }
            Step::CraftPlanks(timed) => craft_tick(timed, facts, out),
        };
    }

    pub(super) fn clear(&mut self, out: &mut Out) {
        match &mut self.step {
            Step::Movement(movement) => movement.clear(out),
            Step::FindTree(timed) => timed.clear(TutorialIcon::Tree, out),
            Step::PunchTree(punch) => punch.timed.clear(TutorialIcon::Tree, out),
            Step::OpenInventory(timed) => timed.clear(TutorialIcon::RecipeBook, out),
            Step::CraftPlanks(timed) => timed.clear(TutorialIcon::WoodenPlanks, out),
            Step::Done => {}
        }
    }

    pub(super) fn on_input(&mut self, keys: Keys) {
        if let Step::Movement(movement) = &mut self.step {
            movement.moved |= keys.forward || keys.backward || keys.left || keys.right || keys.jump;
        }
    }

    pub(super) fn on_mouse(&mut self, xd: f64, yd: f64) {
        if let Step::Movement(movement) = &mut self.step {
            movement.turned |= xd.abs() > 0.01 || yd.abs() > 0.01;
        }
    }

    pub(super) fn on_look_at_block(&mut self, completes_tree: bool, _: &mut Out) {
        if matches!(self.step, Step::FindTree(_)) && completes_tree {
            self.next = Some(TutorialStep::PunchTree);
        }
    }

    pub(super) fn on_destroy_block(&mut self, log: bool, percent: f32, out: &mut Out) {
        let Step::PunchTree(punch) = &mut self.step else {
            return;
        };
        let icon = TutorialIcon::Tree;
        if log && percent > 0.0 {
            if punch.timed.shown {
                out.commands.push(Command::Progress(icon, percent));
            }
            if percent >= 1.0 {
                self.next = Some(TutorialStep::OpenInventory);
            }
        } else if punch.timed.shown {
            out.commands.push(Command::Progress(icon, 0.0));
        } else if log {
            punch.resets += 1;
        }
    }

    pub(super) fn on_open_inventory(&mut self) {
        if matches!(self.step, Step::OpenInventory(_)) {
            self.next = Some(TutorialStep::CraftPlanks);
        }
    }

    pub(super) fn on_get_item(&mut self, item: ItemKind) {
        match self.step {
            Step::FindTree(_) if item.tree_item => self.next = Some(TutorialStep::CraftPlanks),
            Step::PunchTree(_) if item.log => self.next = Some(TutorialStep::CraftPlanks),
            Step::CraftPlanks(_) if item.planks => self.next = Some(TutorialStep::None),
            _ => {}
        }
    }
}

fn find_tree_tick(timed: &mut Timed, facts: &dyn Facts, out: &mut Out) -> Option<TutorialStep> {
    timed.time += 1;
    if timed.time == 1 && (facts.has_tree_item() || facts.mined_trees_before()) {
        return Some(TutorialStep::CraftPlanks);
    }
    timed.hint(6000, FIND_TREE, out);
    None
}

fn punch_tick(punch: &mut Punch, facts: &dyn Facts, out: &mut Out) -> Option<TutorialStep> {
    punch.timed.time += 1;
    if punch.timed.time == 1 && (facts.has_logs() || facts.mined_trees_before()) {
        return Some(TutorialStep::CraftPlanks);
    }
    if (punch.timed.time >= 600 || punch.resets > 3) && !punch.timed.shown {
        punch.timed.shown = true;
        out.commands.push(Command::Show(PUNCH_TREE));
    }
    None
}

fn craft_tick(timed: &mut Timed, facts: &dyn Facts, out: &mut Out) -> Option<TutorialStep> {
    timed.time += 1;
    if timed.time == 1 && (facts.has_planks() || facts.crafted_planks_before()) {
        return Some(TutorialStep::None);
    }
    timed.hint(1200, CRAFT_PLANKS, out);
    None
}
