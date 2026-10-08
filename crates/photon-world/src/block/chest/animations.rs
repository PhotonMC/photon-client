use super::*;

/// `ChestLidController`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ChestLid {
    pub(super) should_be_open: bool,
    pub(super) openness: f32,
    pub(super) old_openness: f32,
}

/// `ShulkerBoxBlockEntity`'s lid: how far the box is open, which a block event starts and the
/// tick carries on at a tenth a tick.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct ShulkerLid {
    pub(super) status: ShulkerStatus,
    pub(super) progress: f32,
    pub(super) progress_old: f32,
}

/// `ShulkerBoxBlockEntity.AnimationStatus`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ShulkerStatus {
    #[default]
    Closed,
    Opening,
    Opened,
    Closing,
}

/// `BellBlockEntity`: the shake a ring starts and the tick wears away.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct BellShake {
    pub(super) ticks: i32,
    pub(super) shaking: bool,
    pub(super) direction: Option<Direction>,
}

/// `SkullBlockEntity`'s animation: counts the ticks a powered dragon or piglin head has been on.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SkullAnimation {
    pub(super) ticks: i32,
    pub(super) animating: bool,
}

/// The lids the server has opened, by block.
#[derive(Debug, Clone, Default)]
pub(crate) struct BlockAnimations {
    pub(crate) chests: HashMap<(i32, i32, i32), ChestLid>,
    pub(crate) shulkers: HashMap<(i32, i32, i32), ShulkerLid>,
    pub(crate) bells: HashMap<(i32, i32, i32), BellShake>,
    pub(crate) skulls: HashMap<(i32, i32, i32), SkullAnimation>,
}

impl PartialEq for BlockAnimations {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for BlockAnimations {}

impl ChestLid {
    /// `tickLid`.
    pub(super) fn tick(&mut self) {
        self.old_openness = self.openness;
        if !self.should_be_open && self.openness > 0.0 {
            self.openness = (self.openness - 0.1).max(0.0);
        } else if self.should_be_open && self.openness < 1.0 {
            self.openness = (self.openness + 0.1).min(1.0);
        }
    }

    /// `getOpenness`: `Mth.lerp(a, old, now)`.
    pub(super) fn openness(&self, partial: f32) -> f32 {
        self.old_openness + partial * (self.openness - self.old_openness)
    }
}

impl ShulkerLid {
    /// `updateAnimation` (without the neighbour updates and the pushing of entities).
    pub(super) fn tick(&mut self) {
        self.progress_old = self.progress;
        match self.status {
            ShulkerStatus::Closed => self.progress = 0.0,
            ShulkerStatus::Opening => {
                self.progress += 0.1;
                if self.progress >= 1.0 {
                    self.status = ShulkerStatus::Opened;
                    self.progress = 1.0;
                }
            }
            ShulkerStatus::Opened => self.progress = 1.0,
            ShulkerStatus::Closing => {
                self.progress -= 0.1;
                if self.progress <= 0.0 {
                    self.status = ShulkerStatus::Closed;
                    self.progress = 0.0;
                }
            }
        }
    }

    /// `triggerEvent` for event 1: a count of one opens the box, none shuts it.
    pub(super) fn trigger(&mut self, count: u8) {
        if count == 0 {
            self.status = ShulkerStatus::Closing;
        }
        if count == 1 {
            self.status = ShulkerStatus::Opening;
        }
    }

    /// `getProgress`: `Mth.lerp(a, old, now)`.
    pub(super) fn progress(&self, partial: f32) -> f32 {
        self.progress_old + partial * (self.progress - self.progress_old)
    }
}

impl BellShake {
    /// `tick`, without the raiders' resonance.
    pub(super) fn tick(&mut self) {
        if self.shaking {
            self.ticks += 1;
        }
        if self.ticks >= 50 {
            self.shaking = false;
            self.ticks = 0;
        }
    }

    /// `triggerEvent` for event 1: the direction of the blow, in `Direction.from3DDataValue`.
    pub(super) fn ring(&mut self, direction: u8) {
        self.direction = Some(Direction::from_3d_data_value(i32::from(direction)));
        self.ticks = 0;
        self.shaking = true;
    }
}

impl SkullAnimation {
    /// `SkullBlockEntity.animation`.
    pub(super) fn tick(&mut self, powered: bool) {
        self.animating = powered;
        if powered {
            self.ticks += 1;
        }
    }

    /// `getAnimation`.
    pub(super) fn animation(&self, partial: f32) -> f32 {
        if self.animating {
            self.ticks as f32 + partial
        } else {
            self.ticks as f32
        }
    }
}

impl BlockAnimations {
    pub(crate) fn tick(&mut self) {
        for lid in self.chests.values_mut() {
            lid.tick();
        }
        for lid in self.shulkers.values_mut() {
            lid.tick();
        }
        for bell in self.bells.values_mut() {
            bell.tick();
        }
    }

    pub(crate) fn remove(&mut self, pos: &(i32, i32, i32)) {
        self.chests.remove(pos);
        self.shulkers.remove(pos);
        self.bells.remove(pos);
        self.skulls.remove(pos);
    }
}

#[cfg(test)]
mod tests {
    use photon_core::direction::Direction;

    use super::{BellShake, ChestLid, ShulkerLid, SkullAnimation};

    #[test]
    fn skull_counts_only_while_powered() {
        let mut skull = SkullAnimation::default();
        skull.tick(true);
        skull.tick(true);
        assert_eq!(skull.animation(0.5), 2.5);
        skull.tick(false);
        assert_eq!(skull.animation(0.5), 2.0);
    }

    #[test]
    fn banner_layers_come_from_the_patterns_list() {
        use photon_core::nbt::{Field, FieldValue};
        let layer = |pattern: &str, color: &str| {
            FieldValue::Compound(vec![
                Field {
                    key: "pattern".to_owned(),
                    value: FieldValue::Str(pattern.to_owned()),
                },
                Field {
                    key: "color".to_owned(),
                    value: FieldValue::Str(color.to_owned()),
                },
            ])
        };
        let mut tag = Vec::new();
        photon_core::nbt::write_nbt(
            Some(&[Field {
                key: "patterns".to_owned(),
                value: FieldValue::List(vec![
                    layer("minecraft:cross", "red"),
                    FieldValue::Compound(Vec::new()),
                ]),
            }]),
            &mut tag,
        );
        assert_eq!(
            super::banner_patterns(&tag),
            [("minecraft:cross".to_owned(), "red".to_owned())]
        );
        assert!(super::banner_patterns(&[]).is_empty());
    }

    #[test]
    fn bell_swings_for_fifty_ticks_from_the_blow() {
        let mut bell = BellShake::default();
        bell.tick();
        assert!(!bell.shaking && bell.ticks == 0);
        bell.ring(2);
        assert_eq!(bell.direction, Some(Direction::North));
        for _ in 0..49 {
            bell.tick();
        }
        assert!(bell.shaking && bell.ticks == 49);
        bell.tick();
        assert!(!bell.shaking && bell.ticks == 0);
    }

    #[test]
    fn shulker_lid_opens_then_stays_open_and_closes() {
        let mut lid = ShulkerLid::default();
        lid.trigger(1);
        for _ in 0..9 {
            lid.tick();
        }
        assert!(lid.progress < 1.0);
        lid.tick();
        lid.tick();
        assert_eq!(lid.progress, 1.0);
        // Another opener changes nothing; the last one leaving closes it.
        lid.trigger(2);
        lid.tick();
        assert_eq!(lid.progress, 1.0);
        lid.trigger(0);
        for _ in 0..11 {
            lid.tick();
        }
        assert_eq!(lid.progress, 0.0);
    }

    #[test]
    fn lid_opens_a_tenth_a_tick_and_closes_the_same_way() {
        let mut lid = ChestLid {
            should_be_open: true,
            ..ChestLid::default()
        };
        for _ in 0..4 {
            lid.tick();
        }
        assert!((lid.openness - 0.4).abs() < 1e-6);
        assert!((lid.openness(0.5) - 0.35).abs() < 1e-6);
        lid.should_be_open = false;
        lid.tick();
        assert!((lid.openness - 0.3).abs() < 1e-6);
        for _ in 0..10 {
            lid.tick();
        }
        assert_eq!(lid.openness, 0.0);
    }
}
