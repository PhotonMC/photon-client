use super::*;
use photon_game::game_mode::Aim;
use photon_gui::tutorial::{Command, Facts, ItemKind, Keys, Words};

const FIND_TREE_TAG: &str = "minecraft:completes_find_tree_tutorial";

/// What the tutorial asks of the session.
struct SessionFacts<'a>(&'a PlaySession);

impl Facts for SessionFacts<'_> {
    fn survival(&self) -> bool {
        self.0.game_mode() == photon_world::ClientPlayerGameMode::Survival
    }

    fn has_logs(&self) -> bool {
        self.0.inventory_has_item_in_tag("minecraft:logs")
    }

    fn has_tree_item(&self) -> bool {
        self.0.inventory_has_item_in_tag(FIND_TREE_TAG)
    }

    fn has_planks(&self) -> bool {
        self.0.inventory_has_item_in_tag("minecraft:planks")
    }

    fn mined_trees_before(&self) -> bool {
        self.0.mined_block_in_tag(FIND_TREE_TAG)
    }

    fn crafted_planks_before(&self) -> bool {
        self.0.crafted_item_in_tag("minecraft:planks")
    }
}

impl Play {
    /// The tutorial's share of one client tick before the player ticks, in `Minecraft.tick`'s
    /// order: the block looked at, what the keys did to blocks, and the input of the player.
    pub(super) fn tutorial_before_tick(&mut self, aim: &Aim, held: &HeldKeys) {
        if self.session.player().is_none() {
            let commands = self.tutorial.stop();
            self.run_tutorial(commands);
            return;
        }
        if let Aim::Block { x, y, z, .. } = *aim {
            let registry = self.session.world().block_state_registry();
            let completes = self
                .session
                .world()
                .block_state_at(x, y, z)
                .is_some_and(|state| registry.is_state_in_tag(state, FIND_TREE_TAG));
            let commands = self.tutorial.on_look_at_block(completes);
            self.run_tutorial(commands);
        }
        for (log, percent) in self.session.take_destroyed_blocks("minecraft:logs") {
            let commands = self.tutorial.on_destroy_block(log, percent);
            self.run_tutorial(commands);
        }
        for turn in std::mem::take(&mut self.controls.turning.turned) {
            self.tutorial.on_mouse(turn);
        }
        if std::mem::take(&mut self.controls.turning.inventory_opened) {
            let commands = self.tutorial.on_open_inventory();
            self.run_tutorial(commands);
        }
        self.tutorial.on_input(Keys {
            forward: held.forward,
            backward: held.backward,
            left: held.left,
            right: held.right,
            jump: held.jump,
        });
    }

    /// `Tutorial.tick`, which `Minecraft.tick` runs after the level's entities ticked.
    pub(super) fn tutorial_tick(&mut self) {
        if self.session.player().is_some() {
            let commands = self.tutorial.tick(&SessionFacts(&self.session));
            self.run_tutorial(commands);
        }
    }

    /// `Tutorial.onGetItem` for the stacks the server put in the inventory.
    pub(super) fn tutorial_items(&mut self) {
        for item in self.session.take_received_items() {
            let session = &self.session;
            let kind = ItemKind {
                log: session.item_in_tag(item, "minecraft:logs"),
                tree_item: session.item_in_tag(item, FIND_TREE_TAG),
                planks: session.item_in_tag(item, "minecraft:planks"),
            };
            let commands = self.tutorial.on_get_item(kind);
            self.run_tutorial(commands);
        }
    }

    fn run_tutorial(&mut self, commands: Vec<Command>) {
        for command in commands {
            match command {
                Command::Show(spec) => {
                    let title = self.tutorial_words(&spec.title);
                    let message = spec
                        .message
                        .as_ref()
                        .map(|words| self.tutorial_words(words));
                    let time = self.controls.options.notification_display_time;
                    self.gui.add_tutorial_toast(
                        spec.icon,
                        (&title, message.as_ref()),
                        (spec.progressable, time),
                    );
                }
                Command::Hide(icon) => self.gui.tutorial_toast(icon, None),
                Command::Progress(icon, progress) => self.gui.tutorial_toast(icon, Some(progress)),
                Command::SetStep(step) => {
                    self.controls.options.tutorial_step = step;
                    let _ = self.controls.options.save(&crate::controls::options_path());
                }
            }
        }
    }

    /// The words with their keybinds shown as the keys, in bold.
    fn tutorial_words(&self, words: &Words) -> photon_game::text::Component {
        let options = &self.controls.options;
        let keys = words
            .keybinds
            .iter()
            .map(|name| {
                let mut key = options.key_binding(&format!("key.{name}")).map_or_else(
                    || photon_game::text::Component::literal(""),
                    |key| {
                        photon_game::options::key_display_name(
                            key,
                            crate::controls::keybinds::system_key_name(key),
                            &self.gui.lang,
                        )
                    },
                );
                key.style.bold = Some(true);
                key
            })
            .collect::<Vec<_>>();
        if keys.is_empty() {
            photon_game::text::Component::translatable(words.key)
        } else {
            photon_game::text::Component::translatable_with(words.key, keys)
        }
    }
}
