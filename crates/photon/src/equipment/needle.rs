//! The compass needles of a frame: where the local player stands and what the level says, and
//! the wobblers that ease each model's needle (`CompassAngleState`).

use photon_core::ids::ItemId;
use photon_net::connection::GlobalBlock;
use photon_render::item_model::{
    CompassTarget, Holder, Wobbler, is_valid_target, rotation_towards, spinning_rotation,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// What a frame knows for the needles: the level and the local player, who is the holder of every
/// stack the GUI and the hand draw.
#[derive(Default)]
pub struct NeedleFrame {
    pub game_time: i64,
    pub dimension: String,
    /// `ClientLevel.getRespawnData`.
    pub respawn: Option<GlobalBlock>,
    /// `Player.getLastDeathLocation`.
    pub last_death: Option<GlobalBlock>,
    pub holder: Option<Holder>,
    /// `tickRateManager().runsNormally()`.
    pub runs_normally: bool,
}

/// One model's needle state: its wobbler for a target, and the spin when there is none.
struct Node {
    wobbler: Wobbler,
    no_target: Wobbler,
}

/// The needles of every compass model, which a draw moves at most once per game tick.
pub struct NeedleStore {
    frame: NeedleFrame,
    nodes: HashMap<(ItemId, CompassTarget), Node>,
    /// `CompassAngleState.random`, unseeded as the reference's is.
    random: photon_core::random::RandomSource,
}

/// The store the frame's caches share.
pub type SharedNeedles = Arc<Mutex<NeedleStore>>;

impl Default for NeedleStore {
    fn default() -> Self {
        Self::new()
    }
}

impl NeedleStore {
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        Self {
            frame: NeedleFrame::default(),
            nodes: HashMap::new(),
            random: photon_core::random::RandomSource::create(seed),
        }
    }

    pub fn set_frame(&mut self, frame: NeedleFrame) {
        self.frame = frame;
    }

    /// `NeedleDirectionHelper.get`: a compass model's needle for a stack, in turns. `lodestone`
    /// is the target the stack's tracker holds.
    pub fn angle(
        &mut self,
        (item, target, wobble): (ItemId, CompassTarget, bool),
        lodestone: Option<(String, [i32; 3])>,
        seed: i32,
    ) -> f32 {
        let Some(holder) = self.frame.holder else {
            return 0.0;
        };
        let aim = self.aim(target, lodestone);
        let node = self.nodes.entry((item, target)).or_insert_with(|| Node {
            wobbler: Wobbler::new(wobble),
            no_target: Wobbler::new(wobble),
        });
        let tick = self.frame.game_time;
        if let Some(block) = aim.filter(|block| is_valid_target(&holder, *block)) {
            let local = self
                .frame
                .runs_normally
                .then_some((&mut node.wobbler, tick));
            return rotation_towards(&holder, block, local);
        }
        if node.no_target.should_update(tick) {
            node.no_target.update(tick, self.random.next_float());
        }
        spinning_rotation(node.no_target.rotation(), seed)
    }

    /// The block a target points at in the holder's dimension (`CompassTarget.get`).
    fn aim(
        &self,
        target: CompassTarget,
        lodestone: Option<(String, [i32; 3])>,
    ) -> Option<[i32; 3]> {
        let (dimension, block) = match target {
            CompassTarget::None => return None,
            CompassTarget::Lodestone => lodestone?,
            CompassTarget::Spawn => {
                let spawn = self.frame.respawn.as_ref()?;
                (spawn.dimension.clone(), spawn.block)
            }
            CompassTarget::Recovery => {
                let death = self.frame.last_death.as_ref()?;
                (death.dimension.clone(), death.block)
            }
        };
        (dimension == self.frame.dimension).then_some(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store(position: [f64; 3], yaw: f32) -> NeedleStore {
        let mut store = NeedleStore::new();
        store.set_frame(NeedleFrame {
            game_time: 10,
            dimension: "minecraft:overworld".to_owned(),
            respawn: Some(GlobalBlock {
                dimension: "minecraft:overworld".to_owned(),
                block: [0, 64, 0],
            }),
            holder: Some(Holder { position, yaw }),
            runs_normally: true,
            ..NeedleFrame::default()
        });
        store
    }

    fn compass() -> ItemId {
        photon_game::item::id_of("minecraft:compass").expect("a compass")
    }

    #[test]
    fn a_needle_eases_once_per_tick_towards_the_spawn() {
        let mut store = store([0.5, 64.0, 20.5], 0.0);
        let at =
            |store: &mut NeedleStore| store.angle((compass(), CompassTarget::Spawn, true), None, 0);
        let first = at(&mut store);
        // The same game tick does not move the needle again.
        assert_eq!(at(&mut store), first);
        store.frame.game_time = 11;
        assert_ne!(at(&mut store), first);
    }

    #[test]
    fn a_target_in_another_dimension_spins_the_needle() {
        let mut store = store([0.5, 64.0, 20.5], 0.0);
        store.frame.respawn = Some(GlobalBlock {
            dimension: "minecraft:the_nether".to_owned(),
            block: [0, 64, 0],
        });
        let angle = store.angle((compass(), CompassTarget::Spawn, true), None, 3);
        assert!((0.0..1.0).contains(&angle));
        // Nothing to hold, nothing to point.
        store.frame.holder = None;
        assert_eq!(
            store.angle((compass(), CompassTarget::Spawn, true), None, 3),
            0.0
        );
    }
}
