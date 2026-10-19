use super::*;

struct World {
    survival: bool,
    logs: bool,
    tree_item: bool,
    planks: bool,
    mined: bool,
    crafted: bool,
}

impl Facts for World {
    fn survival(&self) -> bool {
        self.survival
    }
    fn has_logs(&self) -> bool {
        self.logs
    }
    fn has_tree_item(&self) -> bool {
        self.tree_item
    }
    fn has_planks(&self) -> bool {
        self.planks
    }
    fn mined_trees_before(&self) -> bool {
        self.mined
    }
    fn crafted_planks_before(&self) -> bool {
        self.crafted
    }
}

const SURVIVAL: World = World {
    survival: true,
    logs: false,
    tree_item: false,
    planks: false,
    mined: false,
    crafted: false,
};

const MOVING: Keys = Keys {
    forward: true,
    backward: false,
    left: false,
    right: false,
    jump: false,
};

/// Ticks the tutorial and gives everything it asked for.
fn ticks(tutorial: &mut Tutorial, world: &World, count: u32) -> Vec<Command> {
    (0..count).flat_map(|_| tutorial.tick(world)).collect()
}

fn shows(commands: &[Command], icon: TutorialIcon) -> usize {
    commands
        .iter()
        .filter(|command| matches!(command, Command::Show(spec) if spec.icon == icon))
        .count()
}

#[test]
fn the_first_tick_starts_the_step_without_counting() {
    let mut tutorial = Tutorial::new(TutorialStep::Movement);
    // 100 ticks after the start: the start itself is one, and counts for nothing.
    let early = ticks(&mut tutorial, &SURVIVAL, 100);
    assert_eq!(shows(&early, TutorialIcon::MovementKeys), 0);
    let late = tutorial.tick(&SURVIVAL);
    assert_eq!(shows(&late, TutorialIcon::MovementKeys), 1);
}

#[test]
fn moving_and_looking_for_long_enough_ends_the_movement_step() {
    let mut tutorial = Tutorial::new(TutorialStep::Movement);
    // The player idles until the hint is up, then moves.
    let mut all = ticks(&mut tutorial, &SURVIVAL, 101);
    for _ in 0..45 {
        tutorial.on_input(MOVING);
        all.extend(tutorial.tick(&SURVIVAL));
    }
    assert_eq!(shows(&all, TutorialIcon::MovementKeys), 1);
    assert!(all.contains(&Command::Hide(TutorialIcon::MovementKeys)));
    // The look hint comes 20 ticks after the movement was done.
    for _ in 0..60 {
        tutorial.on_mouse((0.5, 0.0));
        all.extend(tutorial.tick(&SURVIVAL));
    }
    assert_eq!(shows(&all, TutorialIcon::Mouse), 1);
    assert_eq!(tutorial.step(), TutorialStep::FindTree);
    assert_eq!(all.last(), Some(&Command::SetStep(TutorialStep::FindTree)));
}

#[test]
fn a_creative_player_ends_the_tutorial_after_the_movement() {
    let mut tutorial = Tutorial::new(TutorialStep::Movement);
    let creative = World {
        survival: false,
        ..SURVIVAL
    };
    ticks(&mut tutorial, &creative, 1);
    let mut all = Vec::new();
    for _ in 0..90 {
        tutorial.on_input(MOVING);
        tutorial.on_mouse((0.0, -2.0));
        all.extend(tutorial.tick(&creative));
    }
    assert_eq!(tutorial.step(), TutorialStep::None);
}

#[test]
fn a_small_turn_is_not_a_turn() {
    let mut tutorial = Tutorial::new(TutorialStep::Movement);
    ticks(&mut tutorial, &SURVIVAL, 1);
    for _ in 0..60 {
        tutorial.on_input(MOVING);
        tutorial.on_mouse((0.01, -0.01));
        tutorial.tick(&SURVIVAL);
    }
    let all = ticks(&mut tutorial, &SURVIVAL, 100);
    assert_eq!(shows(&all, TutorialIcon::Mouse), 1);
    assert_eq!(tutorial.step(), TutorialStep::Movement);
}

#[test]
fn the_tree_step_ends_by_looking_at_a_tree_and_the_punch_step_by_breaking_it() {
    let mut tutorial = Tutorial::new(TutorialStep::FindTree);
    ticks(&mut tutorial, &SURVIVAL, 2);
    assert!(tutorial.on_look_at_block(false).is_empty());
    assert_eq!(
        tutorial.on_look_at_block(true),
        vec![Command::SetStep(TutorialStep::PunchTree)]
    );
    // The new step starts with the next tick, which only starts it.
    ticks(&mut tutorial, &SURVIVAL, 2);
    assert!(tutorial.on_destroy_block(true, 0.5).is_empty());
    assert_eq!(
        tutorial.on_destroy_block(true, 1.0),
        vec![Command::SetStep(TutorialStep::OpenInventory)]
    );
}

#[test]
fn a_hint_that_is_up_goes_when_the_step_changes() {
    let mut tutorial = Tutorial::new(TutorialStep::FindTree);
    let all = ticks(&mut tutorial, &SURVIVAL, 6002);
    assert_eq!(shows(&all, TutorialIcon::Tree), 1);
    let commands = tutorial.on_look_at_block(true);
    assert_eq!(
        commands,
        vec![
            Command::Hide(TutorialIcon::Tree),
            Command::SetStep(TutorialStep::PunchTree)
        ]
    );
}

#[test]
fn the_punch_step_counts_the_breaks_that_start_over() {
    let mut tutorial = Tutorial::new(TutorialStep::PunchTree);
    ticks(&mut tutorial, &SURVIVAL, 2);
    for _ in 0..4 {
        tutorial.on_destroy_block(true, -1.0);
    }
    let commands = tutorial.tick(&SURVIVAL);
    assert_eq!(shows(&commands, TutorialIcon::Tree), 1);
    assert_eq!(
        tutorial.on_destroy_block(true, 0.25),
        vec![Command::Progress(TutorialIcon::Tree, 0.25)]
    );
    assert_eq!(
        tutorial.on_destroy_block(false, 0.25),
        vec![Command::Progress(TutorialIcon::Tree, 0.0)]
    );
}

#[test]
fn what_the_player_already_has_skips_a_step_on_its_first_counted_tick() {
    let mut tutorial = Tutorial::new(TutorialStep::PunchTree);
    let logs = World {
        logs: true,
        ..SURVIVAL
    };
    assert!(tutorial.tick(&logs).is_empty());
    assert_eq!(
        tutorial.tick(&logs),
        vec![Command::SetStep(TutorialStep::CraftPlanks)]
    );
    let mut tutorial = Tutorial::new(TutorialStep::CraftPlanks);
    let crafted = World {
        crafted: true,
        ..SURVIVAL
    };
    ticks(&mut tutorial, &crafted, 1);
    assert_eq!(
        tutorial.tick(&crafted),
        vec![Command::SetStep(TutorialStep::None)]
    );
}

#[test]
fn the_inventory_and_item_steps_follow_the_events() {
    let mut tutorial = Tutorial::new(TutorialStep::OpenInventory);
    let all = ticks(&mut tutorial, &SURVIVAL, 601);
    assert_eq!(shows(&all, TutorialIcon::RecipeBook), 1);
    assert_eq!(
        tutorial.on_open_inventory(),
        vec![
            Command::Hide(TutorialIcon::RecipeBook),
            Command::SetStep(TutorialStep::CraftPlanks)
        ]
    );
    ticks(&mut tutorial, &SURVIVAL, 2);
    let log = ItemKind {
        log: true,
        ..ItemKind::default()
    };
    assert!(tutorial.on_get_item(log).is_empty());
    let planks = ItemKind {
        planks: true,
        ..ItemKind::default()
    };
    assert_eq!(
        tutorial.on_get_item(planks),
        vec![Command::SetStep(TutorialStep::None)]
    );
}

#[test]
fn a_player_who_leaves_survival_ends_the_tutorial() {
    let mut tutorial = Tutorial::new(TutorialStep::OpenInventory);
    let creative = World {
        survival: false,
        ..SURVIVAL
    };
    ticks(&mut tutorial, &creative, 1);
    assert_eq!(
        tutorial.tick(&creative),
        vec![Command::SetStep(TutorialStep::None)]
    );
    assert!(ticks(&mut tutorial, &creative, 5).is_empty());
}

#[test]
fn stopping_takes_the_hints_down() {
    let mut tutorial = Tutorial::new(TutorialStep::CraftPlanks);
    ticks(&mut tutorial, &SURVIVAL, 1201);
    assert_eq!(
        tutorial.stop(),
        vec![Command::Hide(TutorialIcon::WoodenPlanks)]
    );
}
