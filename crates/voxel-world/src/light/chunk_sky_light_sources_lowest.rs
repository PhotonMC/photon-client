use super::*;

impl ChunkSkyLightSources {
    /// Every column open to below the world, as a new chunk starts.
    pub fn new(level_min_y: i32) -> Self {
        let min_y = level_min_y - 1;
        Self {
            min_y,
            heights: vec![min_y; 256].into_boxed_slice(),
        }
    }

    pub(super) fn index(x: i32, z: i32) -> usize {
        (x + z * 16) as usize
    }

    /// `fillFrom`: from the chunk's sections, the highest first. `section`
    /// answers a section's block states (y, z, x order, 4096 of them) by
    /// index, or nothing if it holds only air.
    pub(crate) fn fill_from<'a>(
        &mut self,
        section_count: usize,
        section: impl Fn(usize) -> Option<&'a crate::paletted::PalettedStates>,
    ) {
        let sections: Vec<Option<&crate::paletted::PalettedStates>> =
            (0..section_count).map(&section).collect();
        let Some(highest) = sections.iter().rposition(Option::is_some) else {
            self.heights.fill(self.min_y);
            return;
        };
        for z in 0..16 {
            for x in 0..16 {
                let found = self.find_lowest_source_y(&sections, highest, x, z);
                self.heights[Self::index(x, z)] = found.max(self.min_y);
            }
        }
    }

    pub(super) fn find_lowest_source_y(
        &self,
        sections: &[Option<&crate::paletted::PalettedStates>],
        top: usize,
        x: i32,
        z: i32,
    ) -> i32 {
        let level_min_y = self.min_y + 1;
        let section_min = |index: usize| level_min_y + index as i32 * 16;
        let mut top_y = section_min(top) + 16;
        let mut top_state = 0;
        for index in (0..=top).rev() {
            match &sections[index] {
                None => {
                    top_state = 0;
                    top_y = section_min(index);
                }
                Some(states) => {
                    for y in (0..16).rev() {
                        let bottom_state = states.get((y * 256 + z * 16 + x) as usize).unwrap_or(0);
                        if is_edge_occluded(top_state, bottom_state) {
                            return top_y;
                        }
                        top_state = bottom_state;
                        top_y -= 1;
                    }
                }
            }
        }
        self.min_y
    }

    /// `update` after the block at local `x`, `z` and height `y` changed.
    /// `state` answers the chunk's block at a local position and height.
    pub fn update(&mut self, state: impl Fn(i32, i32, i32) -> i32, x: i32, y: i32, z: i32) -> bool {
        let index = Self::index(x, z);
        let current = self.heights[index];
        if y + 1 < current {
            return false;
        }
        let top = state(x, y + 1, z);
        let middle = state(x, y, z);
        if self.update_edge(
            &state,
            index,
            current,
            (x, y + 1, z),
            top,
            (x, y, z),
            middle,
        ) {
            return true;
        }
        let bottom = state(x, y - 1, z);
        self.update_edge(
            &state,
            index,
            current,
            (x, y, z),
            middle,
            (x, y - 1, z),
            bottom,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "the reference method's parameters, in its order"
    )]
    pub(super) fn update_edge(
        &mut self,
        state: &impl Fn(i32, i32, i32) -> i32,
        index: usize,
        old_top_edge_y: i32,
        top_pos: (i32, i32, i32),
        top_state: i32,
        bottom_pos: (i32, i32, i32),
        bottom_state: i32,
    ) -> bool {
        let checked = top_pos.1;
        if is_edge_occluded(top_state, bottom_state) {
            if checked > old_top_edge_y {
                self.heights[index] = checked;
                return true;
            }
        } else if checked == old_top_edge_y {
            self.heights[index] = self.find_lowest_source_below(state, bottom_pos, bottom_state);
            return true;
        }
        false
    }

    pub(super) fn find_lowest_source_below(
        &self,
        state: &impl Fn(i32, i32, i32) -> i32,
        start: (i32, i32, i32),
        start_state: i32,
    ) -> i32 {
        let (x, mut top_y, z) = start;
        let mut top_state = start_state;
        while top_y > self.min_y {
            let bottom_state = state(x, top_y - 1, z);
            if is_edge_occluded(top_state, bottom_state) {
                return top_y;
            }
            top_state = bottom_state;
            top_y -= 1;
        }
        self.min_y
    }

    /// `getLowestSourceY`: below the world reads as negative infinity.
    pub fn lowest_source_y(&self, x: i32, z: i32) -> i32 {
        self.extend(self.heights[Self::index(x, z)])
    }

    pub(super) fn highest_lowest_source_y(&self) -> i32 {
        self.extend(self.heights.iter().copied().max().unwrap_or(self.min_y))
    }

    pub(super) fn extend(&self, value: i32) -> i32 {
        if value == self.min_y { i32::MIN } else { value }
    }
}
