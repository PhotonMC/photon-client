//! The statuses of the debug screen's entries (`DebugScreenEntryList`): which are shown always,
//! only while F3's overlay is up, or never, the profiles that set them in one go, and the sorted
//! list of those on now.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    AlwaysOn,
    InOverlay,
    Never,
}

impl Status {
    /// `DebugScreenEntryStatus.getSerializedName`.
    pub fn name(self) -> &'static str {
        match self {
            Self::AlwaysOn => "alwaysOn",
            Self::InOverlay => "inOverlay",
            Self::Never => "never",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        [Self::AlwaysOn, Self::InOverlay, Self::Never]
            .into_iter()
            .find(|status| status.name() == name)
    }
}

/// `DebugScreenProfile`: a set of statuses the screen's bottom buttons load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Default,
    Performance,
}

impl Profile {
    pub const ALL: [Self; 2] = [Self::Default, Self::Performance];

    pub fn name(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Performance => "performance",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|profile| profile.name() == name)
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::Default => "debug.options.profile.default",
            Self::Performance => "debug.options.profile.performance",
        }
    }

    /// `DebugScreenEntries.PROFILES`: the entries a profile sets, all others being unset.
    pub fn statuses(self) -> &'static [(&'static str, Status)] {
        match self {
            Self::Default => &DEFAULT_PROFILE,
            Self::Performance => &PERFORMANCE_PROFILE,
        }
    }
}

const DEFAULT_PROFILE: [(&str, Status); 9] = [
    ("3d_crosshair", Status::InOverlay),
    ("game_version", Status::InOverlay),
    ("tps", Status::InOverlay),
    ("fps", Status::InOverlay),
    ("memory", Status::InOverlay),
    ("system_specs", Status::InOverlay),
    ("player_position", Status::InOverlay),
    ("player_section_position", Status::InOverlay),
    ("simple_performance_impactors", Status::InOverlay),
];

const PERFORMANCE_PROFILE: [(&str, Status); 5] = [
    ("tps", Status::InOverlay),
    ("fps", Status::AlwaysOn),
    ("gpu_utilization", Status::InOverlay),
    ("memory", Status::InOverlay),
    ("simple_performance_impactors", Status::InOverlay),
];

/// `DebugEntryCategory`: the groups the options screen lists the entries under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    ScreenText,
    Renderer,
}

impl Category {
    /// The order the groups are listed in.
    pub fn sort_key(self) -> f32 {
        match self {
            Self::ScreenText => 1.0,
            Self::Renderer => 2.0,
        }
    }

    pub fn translation_key(self) -> &'static str {
        match self {
            Self::ScreenText => "debug.options.category.text",
            Self::Renderer => "debug.options.category.renderer",
        }
    }
}

/// One registered entry (`DebugScreenEntries`): its group, and whether reduced debug info allows it.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub id: &'static str,
    pub category: Category,
    pub allowed_when_reduced: bool,
}

const fn text(id: &'static str, allowed_when_reduced: bool) -> Entry {
    Entry {
        id,
        category: Category::ScreenText,
        allowed_when_reduced,
    }
}

/// The entries that draw nothing of their own (`DebugEntryNoop`): the world's debug drawing.
const fn renderer(id: &'static str) -> Entry {
    Entry {
        id,
        category: Category::Renderer,
        allowed_when_reduced: false,
    }
}

/// Every registered entry, in the order `DebugScreenEntries` registers them.
pub const ENTRIES: [Entry; 45] = [
    text("game_version", true),
    text("fps", true),
    text("tps", true),
    text("memory", true),
    text("detailed_memory", true),
    text("system_specs", true),
    text("looking_at_block_state", false),
    text("looking_at_block_tags", false),
    text("looking_at_fluid_state", false),
    text("looking_at_fluid_tags", false),
    text("looking_at_entity", false),
    text("looking_at_entity_tags", false),
    text("chunk_render_stats", true),
    text("chunk_generation_stats", false),
    text("entity_render_stats", true),
    text("particle_render_stats", false),
    text("chunk_source_stats", true),
    text("player_position", false),
    text("player_section_position", true),
    text("light_levels", false),
    text("heightmap", false),
    text("biome", false),
    text("local_difficulty", false),
    text("day_count", false),
    text("entity_spawn_counts", false),
    text("sound_mood", false),
    text("sound_cache", true),
    text("post_effect", false),
    renderer("entity_hitboxes"),
    renderer("chunk_borders"),
    renderer("3d_crosshair"),
    renderer("chunk_section_paths"),
    text("gpu_utilization", true),
    text("simple_performance_impactors", true),
    renderer("chunk_section_octree"),
    renderer("visualize_water_levels"),
    renderer("visualize_heightmap"),
    renderer("visualize_collision_boxes"),
    renderer("visualize_entity_supporting_blocks"),
    renderer("visualize_block_light_levels"),
    renderer("visualize_sky_light_levels"),
    renderer("visualize_solid_faces"),
    renderer("visualize_chunks_on_server"),
    renderer("visualize_sky_light_sections"),
    renderer("chunk_section_visibility"),
];

/// `DebugScreenEntries.getEntry`.
pub fn entry(id: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|entry| entry.id == id)
}

/// `DebugScreenEntry.isAllowed`.
fn allowed(id: &str, reduced_debug_info: bool) -> bool {
    entry(id).is_some_and(|entry| !reduced_debug_info || entry.allowed_when_reduced)
}

/// The entries in the order the options screen lists them (`OptionList.COMPARATOR`): by group, then
/// by id.
pub fn listed_entries() -> Vec<&'static Entry> {
    let mut all: Vec<&Entry> = ENTRIES.iter().collect();
    all.sort_by(|a, b| {
        a.category
            .sort_key()
            .total_cmp(&b.category.sort_key())
            .then(a.id.cmp(b.id))
    });
    all
}

#[derive(Debug, Clone)]
pub struct EntryList {
    /// Every status that was set; an entry absent here has none (which is not the same as never).
    statuses: BTreeMap<String, Status>,
    /// The profile the statuses still are, until one is changed.
    profile: Option<Profile>,
    overlay_visible: bool,
    /// Something was saved by the reference's rules (every change of a status writes the file).
    changed: bool,
}

impl Default for EntryList {
    fn default() -> Self {
        let mut list = Self {
            statuses: BTreeMap::new(),
            profile: None,
            overlay_visible: false,
            changed: false,
        };
        list.load_profile(Profile::Default);
        list
    }
}

impl EntryList {
    /// `DebugScreenEntryList.load` from a saved profile name or custom statuses; none of them (a
    /// file that said neither) leaves nothing set.
    pub fn from_saved(profile: Option<Profile>, custom: Vec<(String, Status)>) -> Self {
        let mut list = Self::default();
        if let Some(profile) = profile {
            list.load_profile(profile);
        } else {
            list.statuses = custom.into_iter().collect();
            list.profile = None;
        }
        list
    }

    /// What `save` writes: the profile, or else every status set.
    pub fn saved(&self) -> (Option<Profile>, Vec<(&str, Status)>) {
        match self.profile {
            Some(profile) => (Some(profile), Vec::new()),
            None => (
                None,
                self.statuses
                    .iter()
                    .map(|(id, status)| (id.as_str(), *status))
                    .collect(),
            ),
        }
    }

    /// Whether a status changed since the last call: the file is to be written.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// `DebugScreenEntryList.loadProfile`.
    pub fn load_profile(&mut self, profile: Profile) {
        self.profile = Some(profile);
        self.statuses = profile
            .statuses()
            .iter()
            .map(|(id, status)| ((*id).to_owned(), *status))
            .collect();
    }

    /// `isUsingProfile`.
    pub fn is_using_profile(&self, profile: Profile) -> bool {
        self.profile == Some(profile)
    }

    /// `getStatus`: an entry nothing was set for is never shown.
    pub fn status(&self, id: &str) -> Status {
        self.statuses.get(id).copied().unwrap_or(Status::Never)
    }

    /// `setStatus`: the profile is no longer the profile, and the file is to be saved.
    pub fn set_status(&mut self, id: &str, status: Status) {
        self.profile = None;
        self.statuses.insert(id.to_owned(), status);
        self.changed = true;
    }

    pub fn is_overlay_visible(&self) -> bool {
        self.overlay_visible
    }

    /// `toggleDebugOverlay`.
    pub fn toggle_overlay(&mut self) {
        self.overlay_visible = !self.overlay_visible;
    }

    pub fn set_overlay_visible(&mut self, visible: bool) {
        self.overlay_visible = visible;
    }

    /// `toggleStatus`: an entry that is on goes off, one that is off comes on (in the overlay only
    /// while the overlay is up), and one nothing was set for comes on always. Whether it is on now.
    pub fn toggle_status(&mut self, id: &str) -> bool {
        let (now, on) = match self.statuses.get(id).copied() {
            Some(Status::AlwaysOn) => (Status::Never, false),
            Some(Status::InOverlay) if self.overlay_visible => (Status::Never, false),
            Some(Status::InOverlay) => (Status::AlwaysOn, true),
            Some(Status::Never) if self.overlay_visible => (Status::InOverlay, true),
            Some(Status::Never) | None => (Status::AlwaysOn, true),
        };
        self.set_status(id, now);
        on
    }

    /// `getCurrentlyEnabled`: the ids shown now, sorted by their paths as `Identifier`'s natural
    /// order does.
    pub fn enabled(&self, reduced_debug_info: bool) -> Vec<&'static str> {
        let mut shown: Vec<&'static str> = ENTRIES
            .iter()
            .filter(|entry| {
                let status = self.statuses.get(entry.id).copied();
                (status == Some(Status::AlwaysOn)
                    || self.overlay_visible && status == Some(Status::InOverlay))
                    && allowed(entry.id, reduced_debug_info)
            })
            .map(|entry| entry.id)
            .collect();
        shown.sort_unstable();
        shown
    }

    /// `showDebugScreen`: the overlay is up or an entry is on.
    pub fn any_shown(&self, reduced_debug_info: bool) -> bool {
        self.overlay_visible || !self.enabled(reduced_debug_info).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_profile_shows_only_in_the_overlay_and_sorted() {
        let mut list = EntryList::default();
        assert!(list.enabled(false).is_empty());
        list.toggle_overlay();
        assert_eq!(
            list.enabled(false),
            [
                "3d_crosshair",
                "fps",
                "game_version",
                "memory",
                "player_position",
                "player_section_position",
                "simple_performance_impactors",
                "system_specs",
                "tps"
            ]
        );
    }

    #[test]
    fn toggling_an_entry_follows_the_overlay() {
        let mut list = EntryList::default();
        assert!(list.toggle_status("chunk_borders"));
        assert_eq!(list.enabled(false), ["chunk_borders"]);
        assert!(!list.toggle_status("chunk_borders"));
        list.toggle_overlay();
        assert!(list.toggle_status("chunk_borders"));
        assert!(list.enabled(false).contains(&"chunk_borders"));
        list.toggle_overlay();
        assert!(list.enabled(false).is_empty());
        assert!(list.enabled(true).is_empty());
    }

    #[test]
    fn reduced_debug_info_drops_the_position_and_the_crosshair() {
        let mut list = EntryList::default();
        list.toggle_overlay();
        assert_eq!(
            list.enabled(true),
            [
                "fps",
                "game_version",
                "memory",
                "player_section_position",
                "simple_performance_impactors",
                "system_specs",
                "tps"
            ]
        );
        // The chunk borders and the hitboxes are not allowed with reduced debug info.
        list.toggle_status("chunk_borders");
        list.toggle_status("entity_hitboxes");
        assert!(!list.enabled(true).contains(&"chunk_borders"));
        assert!(!list.enabled(true).contains(&"entity_hitboxes"));
        assert!(list.enabled(false).contains(&"entity_hitboxes"));
    }
}
