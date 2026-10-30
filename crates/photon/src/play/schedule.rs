//! The play loop as ECS schedules ([ADR 0003](../../../../docs/adr/0003-bevy-ecs.md)): a `Frame`
//! schedule per frame and a `Tick` schedule for each 50 ms tick the frame owes. Both run on the
//! main thread, one system after the other in the order the loop always had (what the client sends
//! and when is observable behaviour). The game's state is still one `Play`, held as a non-send
//! resource until it is split into resources and components of its own.

use bevy_ecs::prelude::{In, IntoScheduleConfigs, NonSend, NonSendMut, Res, ResMut, Resource};
use bevy_ecs::schedule::{Schedule, ScheduleLabel, SingleThreadedExecutor};
use bevy_ecs::system::IntoSystem;
use bevy_ecs::world::World;
use photon_api::{FrameSet, TickSet};

use super::*;

/// One frame of the loop.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Frame;

/// One client tick.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct Tick;

/// What the frame in hand has measured.
#[derive(Resource)]
struct Clock {
    start: Instant,
    /// The frame's length in ticks.
    ticks: f32,
    /// When the ticks the frame owed were done.
    ticked: Instant,
    script_done: bool,
}

/// The keys the tick being run holds.
#[derive(Resource)]
struct TickInput(HeldKeys);

/// Set by the frame that has nothing more to draw (a finished script).
#[derive(Resource)]
struct Stop(bool);

/// The error that ended the loop; the systems after it do not run.
struct Failure(Option<Box<dyn std::error::Error>>);

/// The world a game runs in: its state and the schedules that drive it.
pub(super) fn world(play: Play) -> World {
    let mut world = World::new();
    world.insert_non_send(play);
    world.insert_non_send(Failure(None));
    let now = Instant::now();
    world.insert_resource(Clock {
        start: now,
        ticks: 0.0,
        ticked: now,
        script_done: false,
    });
    world.insert_resource(Stop(false));
    world.add_schedule(tick_schedule());
    world.add_schedule(frame_schedule());
    world
}

/// Whether another frame is due: the window is open, the player has not left and nothing has
/// failed or ended the run.
pub(super) fn continues(world: &World) -> bool {
    let play = world.non_send::<Play>();
    !world.resource::<Stop>().0
        && world.non_send::<Failure>().0.is_none()
        && !play.surface.should_close()
        && !play.controls.leave
}

/// The error that ended the loop, if one did.
pub(super) fn take_failure(world: &mut World) -> Option<Box<dyn std::error::Error>> {
    world.non_send_mut::<Failure>().0.take()
}

/// The game, out of the world it ran in.
pub(super) fn take_play(world: &mut World) -> Play {
    world
        .remove_non_send::<Play>()
        .expect("the world holds the game until it is taken")
}

fn tick_schedule() -> Schedule {
    let mut schedule = Schedule::new(Tick);
    schedule.set_executor(SingleThreadedExecutor::new());
    configure_tick_sets(&mut schedule);
    schedule.add_systems((
        tick_interface.pipe(record).in_set(TickSet::Interface),
        tick_input.pipe(record).in_set(TickSet::Input),
        tick_local_player.pipe(record).in_set(TickSet::LocalPlayer),
        tick_effects.in_set(TickSet::Effects),
        tick_view.pipe(record).in_set(TickSet::View),
    ));
    schedule
}

/// The tick's parts in the order the tick always had, each after the one before it and only
/// while nothing has failed.
fn configure_tick_sets(schedule: &mut Schedule) {
    schedule.configure_sets(
        (
            TickSet::Interface,
            TickSet::Input,
            TickSet::LocalPlayer,
            TickSet::Effects,
            TickSet::View,
        )
            .chain()
            .distributive_run_if(unbroken),
    );
}

/// The frame's parts in the order the loop ran them, each after the one before it and only while
/// nothing has failed or stopped the frame.
fn configure_frame_sets(schedule: &mut Schedule) {
    schedule.configure_sets(
        (
            FrameSet::Begin,
            FrameSet::Events,
            FrameSet::Clock,
            FrameSet::Session,
            FrameSet::Effects,
            FrameSet::Ticks,
            FrameSet::Turn,
            FrameSet::Draw,
            FrameSet::Limit,
        )
            .chain()
            .distributive_run_if(unbroken),
    );
}

/// The frame's systems, in the order the loop ran them.
fn frame_schedule() -> Schedule {
    let mut schedule = Schedule::new(Frame);
    schedule.set_executor(SingleThreadedExecutor::new());
    configure_frame_sets(&mut schedule);
    schedule.add_systems((
        begin_frame.in_set(FrameSet::Begin),
        (
            poll_events.pipe(record),
            send_client_information.pipe(record),
            reload_language,
            fit_swapchain.pipe(record),
        )
            .chain()
            .distributive_run_if(unbroken)
            .in_set(FrameSet::Events),
        advance_clock.in_set(FrameSet::Clock),
        (
            receive_session.pipe(record),
            container_tick.pipe(record),
            reset_level.pipe(record),
        )
            .chain()
            .distributive_run_if(unbroken)
            .in_set(FrameSet::Session),
        route_effects.in_set(FrameSet::Effects),
        run_ticks.in_set(FrameSet::Ticks),
        turn_player.in_set(FrameSet::Turn),
        draw_frame.pipe(record).in_set(FrameSet::Draw),
        limit_frame.in_set(FrameSet::Limit),
    ));
    schedule
}

/// Keeps the first error and lets the run end with it.
fn record(In(result): In<Result<()>>, mut failure: NonSendMut<Failure>) {
    if let Err(error) = result {
        failure.0.get_or_insert(error);
    }
}

/// Whether the frame goes on: nothing has failed or stopped it.
fn unbroken(stop: Res<Stop>, failure: NonSend<Failure>) -> bool {
    !stop.0 && failure.0.is_none()
}

fn begin_frame(mut clock: ResMut<Clock>) {
    clock.start = Instant::now();
    // A frame that ended early (nothing to draw yet) leaves its parts behind.
    profile::take_parts();
}

fn poll_events(mut play: NonSendMut<Play>, clock: Res<Clock>) -> Result<()> {
    play.poll_events(clock.start)
}

fn send_client_information(mut play: NonSendMut<Play>) -> Result<()> {
    play.send_client_information()
}

fn reload_language(mut play: NonSendMut<Play>) {
    play.reload_language();
}

fn fit_swapchain(mut play: NonSendMut<Play>) -> Result<()> {
    play.fit_swapchain()
}

fn advance_clock(mut play: NonSendMut<Play>, mut clock: ResMut<Clock>) {
    clock.ticks = play.advance_clock();
}

/// `MouseHandler.handleAccumulatedMovement`, after the ticks and before the frame is drawn.
fn turn_player(mut play: NonSendMut<Play>) {
    play.turn_player();
    // The world screen follows what the server said of the difficulty and the permission.
    let play = &mut *play;
    play.controls.sync_world_facts(&play.session);
}

fn receive_session(mut play: NonSendMut<Play>) -> Result<()> {
    play.receive_session()
}

fn container_tick(mut play: NonSendMut<Play>) -> Result<()> {
    let play = &mut *play;
    play.controls
        .container_tick(&mut play.session, &mut play.surface, &play.screen_assets)
}

fn reset_level(mut play: NonSendMut<Play>) -> Result<()> {
    play.reset_level()
}

fn route_effects(mut play: NonSendMut<Play>) {
    play.route_effects();
}

/// `Minecraft.MAX_TICKS_PER_UPDATE`.
const MAX_TICKS_PER_UPDATE: u32 = 10;

/// The ticks the time since the last frame owes, each with the keys a person or the script holds.
fn run_ticks(world: &mut World) {
    let mut script_done = false;
    let mut ran = 0;
    loop {
        // `Minecraft.MAX_TICKS_PER_UPDATE`: the ticks beyond the cap are dropped, not caught up.
        if ran == MAX_TICKS_PER_UPDATE {
            world.non_send_mut::<Play>().drop_owed_ticks();
            break;
        }
        let held = world.non_send_mut::<Play>().begin_tick(&mut script_done);
        match held {
            Ok(Some(held)) => {
                ran += 1;
                world.insert_resource(TickInput(held));
                world.run_schedule(Tick);
                if world.non_send::<Failure>().0.is_some() {
                    break;
                }
            }
            Ok(None) => break,
            Err(error) => {
                world.non_send_mut::<Failure>().0.get_or_insert(error);
                break;
            }
        }
    }
    let mut clock = world.resource_mut::<Clock>();
    clock.script_done = script_done;
    clock.ticked = Instant::now();
}

fn tick_interface(mut play: NonSendMut<Play>) -> Result<()> {
    play.tick_interface()
}

fn tick_input(mut play: NonSendMut<Play>, input: Res<TickInput>) -> Result<()> {
    play.tick_input(&input.0)
}

fn tick_local_player(mut play: NonSendMut<Play>, input: Res<TickInput>) -> Result<()> {
    play.tick_local_player(&input.0)
}

fn tick_effects(mut play: NonSendMut<Play>) {
    play.tick_effects();
}

fn tick_view(mut play: NonSendMut<Play>) -> Result<()> {
    play.tick_view()
}

fn draw_frame(mut play: NonSendMut<Play>, clock: Res<Clock>, mut stop: ResMut<Stop>) -> Result<()> {
    // A reconfiguration or a transfer has the screen, not the level.
    if let Some(kind) = play.session.switching() {
        play.draw_switch(kind)?;
        return Ok(());
    }
    play.recentre_origin()?;
    // A lost connection has the disconnected screen alone: the level is gone.
    if play.controls.disconnected.is_some() {
        play.draw_disconnected()?;
        return Ok(());
    }
    // A respawn has the loading screen until the level is ready.
    if !play.session.is_loaded() && play.session.level_loading().is_some() {
        play.draw_loading()?;
        return Ok(());
    }
    play.end_loading();
    if let Flow::Stop = play.draw([clock.start, clock.ticked], clock.ticks, clock.script_done)? {
        stop.0 = true;
    }
    Ok(())
}

fn limit_frame(mut play: NonSendMut<Play>) {
    play.limit_frame();
}

#[cfg(test)]
mod tests {
    use bevy_ecs::schedule::SystemSet;

    use super::*;

    #[derive(Resource, Default)]
    struct Log(Vec<String>);

    /// The order the sets run in when their systems are added last to first.
    fn order_of<S: SystemSet + Copy + std::fmt::Debug>(
        configure: fn(&mut Schedule),
        sets: &[S],
    ) -> Vec<String> {
        let mut world = World::new();
        world.insert_non_send(Failure(None));
        world.insert_resource(Stop(false));
        world.init_resource::<Log>();
        let mut schedule = Schedule::new(Frame);
        schedule.set_executor(SingleThreadedExecutor::new());
        configure(&mut schedule);
        for set in sets.iter().rev() {
            let name = format!("{set:?}");
            schedule
                .add_systems((move |mut log: ResMut<Log>| log.0.push(name.clone())).in_set(*set));
        }
        schedule.run(&mut world);
        world.resource::<Log>().0.clone()
    }

    #[test]
    fn the_frame_runs_its_sets_in_the_order_the_loop_always_had() {
        let order = order_of(
            configure_frame_sets,
            &[
                FrameSet::Limit,
                FrameSet::Draw,
                FrameSet::Turn,
                FrameSet::Ticks,
                FrameSet::Effects,
                FrameSet::Session,
                FrameSet::Clock,
                FrameSet::Events,
                FrameSet::Begin,
            ],
        );
        assert_eq!(
            order,
            [
                "Begin", "Events", "Clock", "Session", "Effects", "Ticks", "Turn", "Draw", "Limit"
            ]
        );
    }

    #[test]
    fn the_tick_runs_its_sets_in_the_order_the_tick_always_had() {
        let order = order_of(
            configure_tick_sets,
            &[
                TickSet::View,
                TickSet::Effects,
                TickSet::LocalPlayer,
                TickSet::Input,
                TickSet::Interface,
            ],
        );
        assert_eq!(
            order,
            ["Interface", "Input", "LocalPlayer", "Effects", "View"]
        );
    }

    #[test]
    fn a_failure_stops_the_sets_after_it() {
        let mut world = World::new();
        world.insert_non_send(Failure(Some("failed".into())));
        world.insert_resource(Stop(false));
        world.init_resource::<Log>();
        let mut schedule = Schedule::new(Tick);
        schedule.set_executor(SingleThreadedExecutor::new());
        configure_tick_sets(&mut schedule);
        schedule
            .add_systems((|mut log: ResMut<Log>| log.0.push("ran".into())).in_set(TickSet::Input));
        schedule.run(&mut world);
        assert!(world.resource::<Log>().0.is_empty());
    }
}
