//! The places in the entity textures that downloaded skins are drawn from. A skin keeps its
//! place for as long as a frame needs it; a place is given up only when no frame since the
//! current one has used it, so a draw that was already built never sees another skin.

/// A table of places, each holding the name of the skin it was filled with.
pub(super) struct Slots {
    held: Vec<Option<Held>>,
}

struct Held {
    name: String,
    used: u64,
}

impl Slots {
    pub(super) fn new(count: usize) -> Self {
        Self {
            held: (0..count).map(|_| None).collect(),
        }
    }

    /// The place of the skin `name` in `frame`, and whether the skin was put there just now (and
    /// its pixels are to be written). None where every place is in use in this frame.
    pub(super) fn take(&mut self, name: &str, frame: u64) -> Option<(usize, bool)> {
        if let Some(index) = self
            .held
            .iter()
            .position(|held| held.as_ref().is_some_and(|held| held.name == name))
        {
            if let Some(held) = self.held[index].as_mut() {
                held.used = frame;
            }
            return Some((index, false));
        }
        let free = self.held.iter().position(Option::is_none);
        let index = free.or_else(|| {
            // The place least recently used, if that was not in this frame.
            self.held
                .iter()
                .enumerate()
                .filter_map(|(index, held)| held.as_ref().map(|held| (index, held.used)))
                .filter(|(_, used)| *used < frame)
                .min_by_key(|(_, used)| *used)
                .map(|(index, _)| index)
        })?;
        self.held[index] = Some(Held {
            name: name.to_owned(),
            used: frame,
        });
        Some((index, true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skin_keeps_its_place_and_is_written_once() {
        let mut slots = Slots::new(2);
        assert_eq!(slots.take("a", 1), Some((0, true)));
        assert_eq!(slots.take("a", 2), Some((0, false)));
        assert_eq!(slots.take("b", 2), Some((1, true)));
    }

    #[test]
    fn the_place_least_recently_used_is_given_up_first() {
        let mut slots = Slots::new(2);
        slots.take("a", 1);
        slots.take("b", 2);
        slots.take("a", 3);
        assert_eq!(slots.take("c", 4), Some((1, true)));
    }

    #[test]
    fn a_place_used_in_this_frame_is_not_given_up() {
        let mut slots = Slots::new(2);
        slots.take("a", 5);
        slots.take("b", 5);
        assert_eq!(slots.take("c", 5), None);
        assert_eq!(slots.take("c", 6), Some((0, true)));
    }
}
