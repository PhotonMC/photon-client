//! The tutorial (`Tutorial` and its step instances): which hint is up, and what moves it on. It
//! knows nothing of the engine: the game tells it what happened and asks it for the toasts to
//! show, hide or fill, and for the step to keep in the options.

mod steps;

use crate::toasts::TutorialIcon;
use photon_game::options::TutorialStep;
use steps::Instance;

/// What the game answers about the player and the world when a step asks.
pub trait Facts {
    /// `Tutorial.isSurvival`.
    fn survival(&self) -> bool;
    /// The inventory holds a log (`ItemTags.LOGS`).
    fn has_logs(&self) -> bool;
    /// The inventory holds an item that completes the tree step
    /// (`ItemTags.COMPLETES_FIND_TREE_TUTORIAL`).
    fn has_tree_item(&self) -> bool;
    /// The inventory holds planks (`ItemTags.PLANKS`).
    fn has_planks(&self) -> bool;
    /// The statistics say a tree block was mined before.
    fn mined_trees_before(&self) -> bool;
    /// The statistics say planks were crafted before.
    fn crafted_planks_before(&self) -> bool;
}

/// The words of a tutorial toast: a translation key and the keybinds its arguments name (the
/// part after `key.`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Words {
    pub key: &'static str,
    pub keybinds: &'static [&'static str],
}

/// A toast to put up (`new TutorialToast(..)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToastSpec {
    pub icon: TutorialIcon,
    pub title: Words,
    pub message: Option<Words>,
    pub progressable: bool,
}

/// What the game does for the tutorial; the toasts are named by their icon.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Show(ToastSpec),
    Hide(TutorialIcon),
    /// `TutorialToast.updateProgress`.
    Progress(TutorialIcon, f32),
    /// `Tutorial.setStep`: the options keep the step.
    SetStep(TutorialStep),
}

/// The keys a tick of input held (`ClientInput.keyPresses`).
#[derive(Debug, Clone, Copy, Default)]
pub struct Keys {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
}

/// What an item stack is to the tutorial.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItemKind {
    pub log: bool,
    pub tree_item: bool,
    pub planks: bool,
}

pub struct Tutorial {
    instance: Option<Instance>,
    step: TutorialStep,
}

impl Tutorial {
    /// A tutorial at the step the options keep, started by the first tick in a level.
    pub fn new(step: TutorialStep) -> Self {
        Self {
            instance: None,
            step,
        }
    }

    /// The step the tutorial is at.
    pub fn step(&self) -> TutorialStep {
        self.step
    }

    /// `Tutorial.tick` in a level: the first one only starts the step.
    pub fn tick(&mut self, facts: &dyn Facts) -> Vec<Command> {
        let Some(instance) = self.instance.as_mut() else {
            self.instance = Some(Instance::new(self.step));
            return Vec::new();
        };
        let mut out = Out::default();
        instance.tick(facts, &mut out);
        self.finish(out)
    }

    /// `Tutorial.stop`: the level is gone.
    pub fn stop(&mut self) -> Vec<Command> {
        let mut out = Out::default();
        if let Some(mut instance) = self.instance.take() {
            instance.clear(&mut out);
        }
        out.commands
    }

    pub fn on_input(&mut self, keys: Keys) {
        if let Some(instance) = self.instance.as_mut() {
            instance.on_input(keys);
        }
    }

    pub fn on_mouse(&mut self, (xd, yd): (f64, f64)) {
        if let Some(instance) = self.instance.as_mut() {
            instance.on_mouse(xd, yd);
        }
    }

    /// `onLookAt`: the player looks at a block; `completes_tree` says it is one the tree step
    /// asks for.
    pub fn on_look_at_block(&mut self, completes_tree: bool) -> Vec<Command> {
        self.event(|instance, out| instance.on_look_at_block(completes_tree, out))
    }

    /// `onDestroyBlock`: how far a block is broken (0 to 1, or -1 for a break that stopped).
    pub fn on_destroy_block(&mut self, log: bool, percent: f32) -> Vec<Command> {
        self.event(|instance, out| instance.on_destroy_block(log, percent, out))
    }

    pub fn on_open_inventory(&mut self) -> Vec<Command> {
        self.event(|instance, _| {
            instance.on_open_inventory();
        })
    }

    pub fn on_get_item(&mut self, item: ItemKind) -> Vec<Command> {
        self.event(|instance, _| {
            instance.on_get_item(item);
        })
    }

    fn event(&mut self, run: impl FnOnce(&mut Instance, &mut Out)) -> Vec<Command> {
        let Some(instance) = self.instance.as_mut() else {
            return Vec::new();
        };
        let mut out = Out::default();
        run(instance, &mut out);
        self.finish(out)
    }

    /// `Tutorial.setStep`: the step is kept, the old instance clears its toasts and the new one
    /// starts waiting for the next tick.
    fn finish(&mut self, mut out: Out) -> Vec<Command> {
        let Some(instance) = self.instance.as_mut() else {
            return out.commands;
        };
        if let Some(next) = instance.take_next() {
            instance.clear(&mut out);
            self.step = next;
            self.instance = Some(Instance::new(next));
            out.commands.push(Command::SetStep(next));
        }
        out.commands
    }
}

/// The commands a call gave.
#[derive(Default)]
pub(crate) struct Out {
    pub(crate) commands: Vec<Command>,
}

#[cfg(test)]
mod tests;
