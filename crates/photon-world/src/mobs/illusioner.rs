//! The mirror images of an invisible illusioner (`Illusioner.aiStep` and
//! `getIllusionOffsets`): four offsets that move when it is hit or on the
//! minute and ease from the old ones.

use photon_core::random::RandomSource;

/// Copies an illusioner shows.
pub const COPIES: usize = 4;

type Offset = (f64, f64, f64);

/// `clientSideIllusionTicks` and the two sets of offsets (old and new).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IllusionState {
    ticks: i32,
    offsets: [[Offset; COPIES]; 2],
}

/// The offsets are finite, so the comparison is a total one.
impl Eq for IllusionState {}

impl IllusionState {
    /// `aiStep` of an invisible illusioner. `scatter` is the tick of a hit or
    /// of the minute (the offsets move to new places), `settle` the tick near
    /// the end of the hurt time (they come back to the body).
    pub(crate) fn tick(&mut self, scatter: bool, settle: bool, random: &mut RandomSource) {
        self.ticks = (self.ticks - 1).max(0);
        if scatter {
            self.ticks = 3;
            for i in 0..COPIES {
                self.offsets[0][i] = self.offsets[1][i];
                let spread = |random: &mut RandomSource| {
                    (-6.0 + f64::from(random.next_int_bound(13).unwrap_or(0))) * 0.5
                };
                let x = spread(random);
                let y = f64::from((random.next_int_bound(6).unwrap_or(0) - 4).max(0));
                let z = spread(random);
                self.offsets[1][i] = (x, y, z);
            }
        } else if settle {
            self.ticks = 3;
            for i in 0..COPIES {
                self.offsets[0][i] = self.offsets[1][i];
                self.offsets[1][i] = (0.0, 0.0, 0.0);
            }
        }
    }

    /// `getIllusionOffsets`.
    pub fn offsets(&self, partial_tick: f32) -> [Offset; COPIES] {
        if self.ticks <= 0 {
            return self.offsets[1];
        }
        let scale = (f64::from(self.ticks) - f64::from(partial_tick)) / f64::from(3.0_f32);
        let scale = scale.powf(0.25);
        let mut out = self.offsets[1];
        for (i, slot) in out.iter_mut().enumerate() {
            let (new, old) = (self.offsets[1][i], self.offsets[0][i]);
            *slot = (
                new.0 * (1.0 - scale) + old.0 * scale,
                new.1 * (1.0 - scale) + old.1 * scale,
                new.2 * (1.0 - scale) + old.2 * scale,
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_settle_eases_from_the_old_places_to_the_body() {
        let mut state = IllusionState::default();
        let mut random = RandomSource::create(7);
        state.tick(true, false, &mut random);
        let placed = state.offsets[1];
        state.tick(false, true, &mut random);
        assert_eq!(state.offsets[1], [(0.0, 0.0, 0.0); COPIES]);
        // Just after the settle the copies are still where they were.
        let first = state.offsets(0.0);
        assert_eq!(first, placed);
        for _ in 0..3 {
            state.tick(false, false, &mut random);
        }
        assert_eq!(state.offsets(0.0), [(0.0, 0.0, 0.0); COPIES]);
    }
}
