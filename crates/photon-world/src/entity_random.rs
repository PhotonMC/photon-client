//! The random source an entity draws from on the client: its own, which the
//! reference does not seed.

use photon_core::random::RandomSource;

/// A source made when first drawn from (seeded by the clock), which two
/// states never differ by.
#[derive(Debug, Clone, Default)]
pub(crate) struct EntityRandom(Option<RandomSource>);

impl PartialEq for EntityRandom {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for EntityRandom {}

impl EntityRandom {
    pub(crate) fn source(&mut self) -> &mut RandomSource {
        self.0.get_or_insert_with(|| {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_nanos() as i64);
            RandomSource::create(seed)
        })
    }

    /// Fixes the source, so a comparison can be made.
    pub(crate) fn fix_seed(&mut self, seed: i64) {
        self.0 = Some(RandomSource::create(seed));
    }
}
