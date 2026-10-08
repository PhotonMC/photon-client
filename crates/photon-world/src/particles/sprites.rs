//! The pictures a particle type can take: `SpriteSet`.

use photon_core::random::RandomSource;

/// The textures a particle type lists in its `particles/*.json`, in order.
///
/// A particle holds the place of its picture in the set; the picture itself is the atlas's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteSet {
    names: Vec<String>,
}

impl SpriteSet {
    /// The set of these textures; an empty list is the one missing texture, as the reference binds it.
    pub fn new(mut names: Vec<String>) -> Self {
        if names.is_empty() {
            names.push("missingno".into());
        }
        Self { names }
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// `SpriteSet.get(int, int)`: the picture for an age out of a lifetime, the first at
    /// the start and the last at the end.
    pub fn from_age(&self, age: i32, lifetime: i32) -> usize {
        let last = self.names.len() as i32 - 1;
        (age.wrapping_mul(last) / lifetime) as usize
    }

    /// `SpriteSet.get(RandomSource)`: any of them.
    pub fn from_random(&self, random: &mut RandomSource) -> usize {
        random
            .next_int_bound(self.names.len() as i32)
            .expect("a sprite set is never empty") as usize
    }

    /// `SpriteSet.first`.
    pub fn first(&self) -> usize {
        0
    }
}
