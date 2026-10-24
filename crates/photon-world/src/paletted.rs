//! A section's block states stored as the reference stores them
//! (`PalettedContainer`): a palette of the distinct states and, for every
//! position, the palette index packed into as few bits as the palette needs.
//! A section of one state, which most of the sky and the rock are, holds no
//! per-block data at all.

use photon_core::ids::BlockStateId;

/// How many blocks a section holds.
pub(crate) const SECTION_VOLUME: usize = 16 * 16 * 16;

/// The block states of one section, in y, z, x order.
#[derive(Debug, Clone)]
pub(crate) struct PalettedStates {
    palette: Vec<i32>,
    /// Bits per index; zero while the palette holds one state.
    bits: u32,
    /// Indices packed `64 / bits` to a word, never spanning two.
    words: Vec<u64>,
}

impl PalettedStates {
    /// Every state of a section, palettised.
    pub(crate) fn from_values(values: &[i32]) -> Self {
        let mut palette: Vec<i32> = Vec::new();
        let mut indices: Vec<usize> = Vec::with_capacity(values.len());
        for value in values {
            let index = palette
                .iter()
                .position(|known| known == value)
                .unwrap_or_else(|| {
                    palette.push(*value);
                    palette.len() - 1
                });
            indices.push(index);
        }
        if palette.is_empty() {
            palette.push(0);
        }
        let mut states = Self {
            bits: Self::bits_for(palette.len()),
            words: Vec::new(),
            palette,
        };
        states.words = vec![0; states.word_count()];
        for (position, index) in indices.into_iter().enumerate() {
            states.store(position, index);
        }
        states
    }

    /// The bits an index needs for a palette of `size` states.
    fn bits_for(size: usize) -> u32 {
        if size <= 1 {
            0
        } else {
            usize::BITS - (size - 1).leading_zeros()
        }
    }

    fn per_word(&self) -> usize {
        64 / self.bits as usize
    }

    fn word_count(&self) -> usize {
        if self.bits == 0 {
            0
        } else {
            SECTION_VOLUME.div_ceil(self.per_word())
        }
    }

    fn index_at(&self, position: usize) -> usize {
        if self.bits == 0 {
            return 0;
        }
        let per_word = self.per_word();
        let shift = (position % per_word) as u32 * self.bits;
        let mask = (1_u64 << self.bits) - 1;
        ((self.words[position / per_word] >> shift) & mask) as usize
    }

    fn store(&mut self, position: usize, index: usize) {
        if self.bits == 0 {
            return;
        }
        let per_word = self.per_word();
        let shift = (position % per_word) as u32 * self.bits;
        let mask = (1_u64 << self.bits) - 1;
        let word = &mut self.words[position / per_word];
        *word = *word & !(mask << shift) | (index as u64) << shift;
    }

    /// The state at a position.
    pub(crate) fn get(&self, position: usize) -> Option<BlockStateId> {
        (position < SECTION_VOLUME).then(|| BlockStateId(self.palette[self.index_at(position)]))
    }

    /// Whether a state the palette holds satisfies `test`. The palette can
    /// keep a state nothing uses any more, so a `false` is certain and a
    /// `true` is only a hint to look at the positions.
    pub(crate) fn palette_any(&self, test: impl Fn(BlockStateId) -> bool) -> bool {
        self.palette.iter().any(|&state| test(BlockStateId(state)))
    }

    /// Put a state at a position; the state it replaced.
    pub(crate) fn set(&mut self, position: usize, state: BlockStateId) -> Option<BlockStateId> {
        let previous = self.get(position)?;
        let index = if let Some(index) = self.palette.iter().position(|known| *known == state.0) {
            index
        } else {
            self.palette.push(state.0);
            let needed = Self::bits_for(self.palette.len());
            if needed != self.bits {
                self.resize(needed);
            }
            self.palette.len() - 1
        };
        self.store(position, index);
        Some(previous)
    }

    /// Repack every index into words of `bits` bits.
    fn resize(&mut self, bits: u32) {
        let indices: Vec<usize> = (0..SECTION_VOLUME)
            .map(|position| self.index_at(position))
            .collect();
        self.bits = bits;
        self.words = vec![0; self.word_count()];
        for (position, index) in indices.into_iter().enumerate() {
            self.store(position, index);
        }
    }

    /// Every state, in order.
    #[cfg(test)]
    pub(crate) fn to_vec(&self) -> Vec<i32> {
        (0..SECTION_VOLUME)
            .map(|position| self.palette[self.index_at(position)])
            .collect()
    }

    /// How many bytes the states take.
    pub(crate) fn bytes(&self) -> usize {
        self.words.len() * 8 + self.palette.len() * 4
    }
}

impl PartialEq for PalettedStates {
    fn eq(&self, other: &Self) -> bool {
        (0..SECTION_VOLUME).all(|position| self.get(position) == other.get(position))
    }
}

impl Eq for PalettedStates {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_section_of_one_state_holds_no_indices() {
        let states = PalettedStates::from_values(&vec![7; SECTION_VOLUME]);
        assert_eq!(states.bits, 0);
        assert!(states.words.is_empty());
        assert_eq!(states.get(0), Some(BlockStateId(7)));
        assert_eq!(states.get(4095), Some(BlockStateId(7)));
        assert_eq!(states.get(4096), None);
    }

    #[test]
    fn values_come_back_as_they_went_in() {
        let values: Vec<i32> = (0..SECTION_VOLUME)
            .map(|i| ((i * 31 + i / 7) % 23) as i32 * 97)
            .collect();
        let states = PalettedStates::from_values(&values);
        assert_eq!(states.to_vec(), values);
        assert!(states.bytes() < SECTION_VOLUME * 4 / 4, "five bits beat 32");
    }

    #[test]
    fn a_new_state_grows_the_palette_and_repacks() {
        let mut states = PalettedStates::from_values(&vec![0; SECTION_VOLUME]);
        assert_eq!(states.set(10, BlockStateId(5)), Some(BlockStateId(0)));
        assert_eq!(states.bits, 1);
        for value in 1..40 {
            states.set(value, BlockStateId(value as i32 * 3));
        }
        assert_eq!(
            states.get(10),
            Some(BlockStateId(30)),
            "position 10 was overwritten by the loop"
        );
        assert_eq!(states.get(0), Some(BlockStateId(0)));
        assert_eq!(states.get(39), Some(BlockStateId(117)));
        assert_eq!(states.get(41), Some(BlockStateId(0)));
        assert!(states.bits >= 6);
        assert_eq!(states.set(41, BlockStateId(5)), Some(BlockStateId(0)));
        assert_eq!(states.get(41), Some(BlockStateId(5)));
    }
}
