//! The fade a section shows in with (`RenderSection.getVisibility`): the time since it first
//! showed over a length, which 26.2 sets each time the section is compiled and 26.3 takes from
//! the option.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use photon_render::section_workers::SectionKey;
use photon_render::vulkan::SectionFade;

/// The squared distance, in blocks, within which a section counts as near the camera.
const NEAR: i64 = 768;

/// Whether a section's centre is near the camera's block (`LevelRenderer.compileSections`).
pub(super) fn near(key: SectionKey, camera_block: [i64; 3]) -> bool {
    [key.0, key.1, key.2]
        .iter()
        .zip(camera_block)
        .map(|(&section, block)| (i64::from(section) * 16 + 8 - block).pow(2))
        .sum::<i64>()
        < NEAR
}

/// How long a section's fade runs. A compile that sets the length (26.2) gives none to a section
/// near the camera and to one that was empty before; otherwise every section takes the option's
/// length.
fn fade_length(
    set_per_compile: bool,
    option: Duration,
    (near, was_empty): (bool, bool),
) -> Duration {
    if set_per_compile && (near || was_empty) {
        Duration::ZERO
    } else {
        option
    }
}

/// Each section's fade, and which sections have shown only as empty.
pub(super) struct Fades {
    shown: HashMap<SectionKey, SectionFade>,
    /// Sections the graph found empty before any mesh: 26.2 stamps no time for them, so they are
    /// fully visible, and what is built in one later does not fade.
    shown_empty: HashSet<SectionKey>,
    /// The chunk fade option.
    option: Duration,
}

impl Fades {
    pub(super) fn new(option: Duration) -> Self {
        Self {
            shown: HashMap::new(),
            shown_empty: HashSet::new(),
            option,
        }
    }

    /// The chunk fade option changed: sections that show for the first time from now on fade in
    /// over this long.
    pub(super) fn set_option(&mut self, length: Duration) {
        self.option = length;
    }

    pub(super) fn clear(&mut self) {
        self.shown.clear();
        self.shown_empty.clear();
    }

    pub(super) fn remove(&mut self, key: SectionKey) {
        self.shown.remove(&key);
        self.shown_empty.remove(&key);
    }

    fn per_compile() -> bool {
        photon_core::versions::rules()
            .render
            .section_fade_is_set_per_compile
    }

    /// A mesh was built for the section: it fades in from the fog the first time it shows, and
    /// its later rebuilds keep that first time.
    pub(super) fn built(&mut self, key: SectionKey, camera_block: [i64; 3]) -> SectionFade {
        let per_compile = Self::per_compile();
        let was_empty = self.shown_empty.remove(&key);
        let length = fade_length(
            per_compile,
            self.option,
            (near(key, camera_block), was_empty),
        );
        let fade = self.shown.entry(key).or_insert_with(|| SectionFade {
            since: Instant::now(),
            length,
        });
        if per_compile {
            fade.length = length;
        }
        *fade
    }

    /// The section holds only air and shows as nothing. 26.3 stamps its time like a mesh's;
    /// 26.2 leaves it unstamped, so it is seen at once.
    pub(super) fn empty(&mut self, key: SectionKey, camera_block: [i64; 3]) -> SectionFade {
        if Self::per_compile() {
            self.shown_empty.insert(key);
            return self
                .shown
                .get(&key)
                .copied()
                .unwrap_or_else(SectionFade::none);
        }
        self.built(key, camera_block)
    }

    /// `LevelRenderer.isSectionCompiledAndVisible`: the section has shown and is at least three
    /// tenths into its fade.
    pub(super) fn compiled_and_visible(&self, key: SectionKey) -> bool {
        self.shown.get(&key).map_or_else(
            || self.shown_empty.contains(&key),
            |fade| fade.visibility(Instant::now()) >= 0.3,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPTION: Duration = Duration::from_millis(750);

    #[test]
    fn a_compile_that_sets_the_length_spares_near_and_previously_empty_sections() {
        assert_eq!(fade_length(true, OPTION, (false, false)), OPTION);
        assert_eq!(fade_length(true, OPTION, (true, false)), Duration::ZERO);
        assert_eq!(fade_length(true, OPTION, (false, true)), Duration::ZERO);
    }

    #[test]
    fn the_option_decides_for_every_section_when_the_compile_does_not_set_it() {
        for case in [(false, false), (true, false), (false, true), (true, true)] {
            assert_eq!(fade_length(false, OPTION, case), OPTION);
        }
    }

    #[test]
    fn a_section_is_near_within_768_squared_blocks_of_its_centre() {
        // The centre of section (0, 4, 0) is (8, 72, 8): 27 blocks away squares to 729, 28 to 784.
        assert!(near((0, 4, 0), [8, 72, 8]));
        assert!(near((0, 4, 0), [8 + 27, 72, 8]));
        assert!(!near((0, 4, 0), [8 + 28, 72, 8]));
    }
}
