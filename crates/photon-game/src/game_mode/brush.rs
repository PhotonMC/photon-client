//! `BrushItem`: the use that starts on a block, and what each tick of it does on a client.

use super::mining::{block, is_air};
use super::{Aim, Context, UseResult};
use photon_core::direction::Direction;
use photon_core::ids::BlockStateId;
use photon_data::particle_values::{LevelParticles, ParticleData};
use photon_world::local_player::{LocalSound, LocalSoundSource};

/// `BrushItem.getUseDuration`.
const USE_DURATION: i32 = 200;

/// `BrushItem.useOn`: aiming at a block starts the use, and the click is consumed without a swing.
pub(super) fn use_on(ctx: &mut Context<'_>, hand: u8) -> UseResult {
    ctx.player.start_using_item(hand != super::MAIN_HAND);
    UseResult::Success { swing: false }
}

/// `BrushItem.onUseTick` for the item in use: on every tenth tick, from the fifth, the dust and the
/// sound of the aimed block; a look that is no longer on a block lets go of the brush.
pub(super) fn use_tick(ctx: &mut Context<'_>, (aim, main_arm_left): (Aim, bool)) {
    if !ctx.player.is_using_item() || !ctx.held_is_brush() {
        return;
    }
    let elapsed = ctx.player.used_ticks() + 1;
    if USE_DURATION - elapsed < 0 {
        ctx.player.release_using_item();
        return;
    }
    let Aim::Block {
        x,
        y,
        z,
        face,
        location,
        ..
    } = aim
    else {
        ctx.player.release_using_item();
        return;
    };
    if elapsed % 10 != 5 {
        return;
    }
    let pos = (x, y, z);
    let state = ctx.state_at(pos);
    let direction = Direction::from_3d_data_value(i32::from(face));
    if block(state).spawn_terrain_particles && !is_air(state) {
        dust(ctx, state, (direction, location), main_arm_left);
    }
    // `BrushableBlock.getBrushSound`.
    let event = match block(state).index() {
        photon_data::block::ids::SUSPICIOUS_SAND => {
            concat!("minecraft:", "item.brush.brushing.sand")
        }
        photon_data::block::ids::SUSPICIOUS_GRAVEL => {
            concat!("minecraft:", "item.brush.brushing.gravel")
        }
        _ => concat!("minecraft:", "item.brush.brushing.generic"),
    };
    let at = (f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5);
    ctx.world.push_sound(LocalSound::at(
        event,
        LocalSoundSource::Blocks,
        (1.0, 1.0),
        at,
    ));
}

/// `BrushItem.spawnDustParticles`: seven to eleven specks of the block, swept along the face the
/// brush is on, the way the brushing arm goes.
fn dust(
    ctx: &mut Context<'_>,
    state: BlockStateId,
    (direction, location): (Direction, [f64; 3]),
    main_arm_left: bool,
) {
    let Some(type_id) = photon_data::particle_type::id("block") else {
        return;
    };
    // `getUsedItemHand() == MAIN_HAND ? getMainArm() : getMainArm().getOpposite()`.
    let right = main_arm_left == ctx.player.used_item_is_off_hand();
    let flip = if right { 1.0 } else { -1.0 };
    let (y_rot, x_rot) = ctx.player.rotation();
    let view = photon_core::math::Vec3::view_vector(x_rot, y_rot);
    let (xd, zd) = match direction {
        Direction::Down | Direction::Up => (view.z, -view.x),
        Direction::North => (1.0, -0.1),
        Direction::South => (-1.0, 0.1),
        Direction::West => (-0.1, -1.0),
        Direction::East => (0.1, 1.0),
    };
    let random = ctx.world.level_random();
    let count = random.next_int_range(7, 12).unwrap_or(7);
    let mut specks = Vec::new();
    for _ in 0..count {
        let speed_x = xd * flip * 3.0 * random.next_double();
        let speed_z = zd * flip * 3.0 * random.next_double();
        specks.push(LevelParticles {
            type_id,
            data: ParticleData::Block(state),
            override_limiter: false,
            always_show: false,
            at: (
                location[0]
                    - if direction == Direction::West {
                        f64::from(1.0E-6_f32)
                    } else {
                        0.0
                    },
                location[1],
                location[2]
                    - if direction == Direction::North {
                        f64::from(1.0E-6_f32)
                    } else {
                        0.0
                    },
            ),
            spread: (speed_x as f32, 0.0, speed_z as f32),
            max_speed: (1.0, 1.0, 1.0),
            count: 0,
            randomization: Default::default(),
        });
    }
    ctx.player.push_particles(specks);
}

#[cfg(test)]
mod tests {
    use super::super::{Aim, Context, GameMode, Keys, Miner};
    use photon_world::input::Input;
    use photon_world::{
        ClientPlayerGameMode, ClientWorld, DimensionBounds, SectionInput, local_player::LocalPlayer,
    };

    /// A brush held on a block (`suspicious_sand`) for `ticks` ticks with the use key down, the
    /// look on the block unless `look_away`; returns the sounds, the particles and whether the
    /// use is still on.
    fn brushing(
        block_name: &str,
        ticks: usize,
        look_away: bool,
    ) -> (Vec<photon_world::local_player::LocalSound>, usize, bool) {
        let index = photon_data::block::registry::block_index(block_name).expect(block_name);
        let mut blocks = vec![0; 4_096];
        blocks[256 + 16 + 1] = photon_data::block::registry::BLOCKS[index]
            .default_state_id()
            .0;
        let biomes = vec![0; 64];
        let mut world = ClientWorld::new(DimensionBounds::new(0, 16));
        assert!(world.replace_chunk(
            0,
            0,
            [SectionInput {
                non_empty_block_count: 1,
                fluid_count: 0,
                block_states: &blocks,
                biomes: &biomes,
            }],
        ));
        let mut player = LocalPlayer::new(
            photon_core::ids::EntityId(1),
            photon_core::math::Vec3::new(1.5, 1.0, 3.5),
            180.0,
            35.0,
            ClientPlayerGameMode::Survival,
            true,
        );
        world.set_local_player_id(player.entity_id());
        let held = crate::item::id_of("minecraft:brush").expect("an item");
        let stack = photon_world::ClientItemStackState::new(1, held);
        world.apply_local_player_container_slot(0, 0, 36, Some(&stack));
        player.set_main_hand_item(held);
        let mut cooldowns = crate::item::cooldowns::ItemCooldowns::default();
        let mut game_mode = GameMode::default();
        let on_block = Aim::Block {
            x: 1,
            y: 1,
            z: 1,
            face: 1,
            location: [1.5, 1.0, 1.5],
            inside: false,
        };
        let (mut sounds, mut particles) = (Vec::new(), 0);
        for tick in 0..ticks {
            let mut ctx = Context {
                miner: &Miner::of(&world, 0),
                world: &mut world,
                player: &mut player,
                mode: ClientPlayerGameMode::Survival,
                selected_slot: 0,
                cooldowns: &mut cooldowns,
                instruments: &[],
                spin_attack: &[],
                prevents_armor_change: &[],
                difficulty: 2,
                game_master: false,
                profile_id: None,
                creaking_active: false,
            };
            let keys = Keys {
                use_clicks: u32::from(tick == 0),
                use_held: true,
                ..Default::default()
            };
            let aim = if look_away && tick > 0 {
                Aim::Miss
            } else {
                on_block
            };
            game_mode.keys_tick(&mut ctx, keys, aim);
            let none = Input::new((false, false, false, false), false, false, false);
            player.tick(&world, none).expect("a tick");
            sounds.extend(world.take_sounds());
            particles += player.take_particles().len();
        }
        (sounds, particles, player.is_using_item())
    }

    /// The first burst comes on the fifth tick of use: seven to eleven specks and the sand's
    /// brushing sound; nothing before it.
    #[test]
    fn suspicious_sand_brushes_with_dust_and_its_sound() {
        let (sounds, particles, using) = brushing("minecraft:suspicious_sand", 4, false);
        assert!(using && particles == 0 && sounds.is_empty());
        let (sounds, particles, using) = brushing("minecraft:suspicious_sand", 6, false);
        assert!(using);
        assert!((7..=11).contains(&particles), "{particles}");
        assert_eq!(sounds.len(), 1);
        assert_eq!(sounds[0].event, "minecraft:item.brush.brushing.sand");
    }

    /// A look that leaves the block lets go of the brush.
    #[test]
    fn looking_away_lets_go_of_the_brush() {
        let (_, _, using) = brushing("minecraft:suspicious_sand", 3, true);
        assert!(!using);
    }
}
