//! The packs the player can choose from (`PackRepository`, `FolderRepositorySource`) and the
//! two lists the Resource Packs screen moves them between (`PackSelectionModel`).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use photon_core::json::JsonValue;

use crate::meta::{Compatibility, PackFormat};
use crate::pack::Pack;
use crate::source::{DirectorySource, Source, ZipSource};

/// `Pack.Position`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Bottom,
}

impl Position {
    fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }
}

/// `PackSelectionConfig`: whether a pack cannot be left out, whether it keeps its place, and
/// where it goes when it is added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionConfig {
    pub required: bool,
    pub fixed_position: bool,
    pub default_position: Position,
}

/// `PackSource`: where a pack came from, which decorates its description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackSource {
    Default,
    BuiltIn,
    Feature,
    World,
    Server,
}

impl PackSource {
    /// The translation key of the source's name, when the description names it
    /// (`pack.nameAndSource`).
    pub fn name_key(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::BuiltIn => Some("pack.source.builtin"),
            Self::Feature => Some("pack.source.feature"),
            Self::World => Some("pack.source.world"),
            Self::Server => Some("pack.source.server"),
        }
    }
}

/// How a pack's title is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Title {
    /// A name as it is (`Component.literal`).
    Literal(String),
    /// A translation key.
    Translatable(String),
}

/// What the lists show of a pack that can be chosen.
#[derive(Debug, Clone, PartialEq)]
pub struct PackEntry {
    pub id: String,
    pub title: Title,
    pub description: JsonValue,
    pub compatibility: Compatibility,
    pub source: PackSource,
    pub config: SelectionConfig,
    /// The bytes of `pack.png`, when the pack has one.
    pub icon: Option<Vec<u8>>,
}

/// `FolderRepositorySource.DISCOVERED_PACK_SELECTION_CONFIG`.
const DISCOVERED: SelectionConfig = SelectionConfig {
    required: false,
    fixed_position: false,
    default_position: Position::Top,
};

/// The entry of an opened pack found in the folder under the name `name`.
fn folder_entry(name: &str, pack: &Pack) -> PackEntry {
    PackEntry {
        id: format!("file/{name}"),
        title: Title::Literal(name.to_owned()),
        description: pack.description().clone(),
        compatibility: pack.compatibility(),
        source: PackSource::Default,
        config: DISCOVERED,
        icon: pack.icon(),
    }
}

/// The game's own pack (`ClientPackSource.VANILLA_SELECTION_CONFIG`): always selected, at the
/// bottom, named by the game's own texts; `icon` is its `pack.png`.
pub fn vanilla_entry(icon: Option<Vec<u8>>) -> PackEntry {
    let description =
        photon_core::json::parse(r#"{"translate":"resourcePack.vanilla.description"}"#)
            .unwrap_or(JsonValue::Null);
    PackEntry {
        id: "vanilla".to_owned(),
        title: Title::Translatable("resourcePack.vanilla.name".to_owned()),
        description,
        compatibility: Compatibility::Compatible,
        source: PackSource::BuiltIn,
        config: SelectionConfig {
            required: true,
            fixed_position: false,
            default_position: Position::Bottom,
        },
        icon,
    }
}

/// `ClientPackSource.BUILT_IN_SELECTION_CONFIG`: the packs the game ships besides its own.
const BUILT_IN: SelectionConfig = SelectionConfig {
    required: false,
    fixed_position: false,
    default_position: Position::Top,
};

/// `ClientPackSource.SPECIAL_PACK_NAMES`: the built-in packs that have a name of their own.
fn built_in_title(id: &str) -> Title {
    match id {
        "programmer_art" | "high_contrast" => {
            Title::Translatable(format!("resourcePack.{id}.name"))
        }
        _ => Title::Literal(id.to_owned()),
    }
}

/// The built-in packs of `zips` (an id and the zip that holds it), those with a readable
/// `pack.mcmeta` (`BuiltInPackSource.discoverPacksInPath`).
pub fn discover_built_in(zips: &[(String, PathBuf)], current: PackFormat) -> Vec<PackEntry> {
    zips.iter()
        .filter_map(|(id, path)| {
            let source = ZipSource::open(path).ok()?;
            let pack = Pack::open(id.clone(), Box::new(source), current).ok()?;
            Some(PackEntry {
                id: id.clone(),
                title: built_in_title(id),
                description: pack.description().clone(),
                compatibility: pack.compatibility(),
                source: PackSource::BuiltIn,
                config: BUILT_IN,
                icon: pack.icon(),
            })
        })
        .collect()
}

/// Every pack of the folder (`FolderRepositorySource.loadPacks`): a directory or a zip with a
/// readable `pack.mcmeta`. Anything else is passed over.
pub fn discover_folder(folder: &Path, current: PackFormat) -> Vec<PackEntry> {
    let _ = std::fs::create_dir_all(folder);
    let Ok(contents) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    contents
        .filter_map(Result::ok)
        .filter_map(|content| open_in_folder(&content.path(), current))
        .collect()
}

fn open_in_folder(path: &Path, current: PackFormat) -> Option<PackEntry> {
    let name = path.file_name()?.to_str()?.to_owned();
    let source: Box<dyn Source> = if path.is_dir() {
        Box::new(DirectorySource::new(path))
    } else if name.to_lowercase().ends_with(".zip") {
        Box::new(ZipSource::open(path).ok()?)
    } else {
        return None;
    };
    let pack = Pack::open(format!("file/{name}"), source, current).ok()?;
    Some(folder_entry(&name, &pack))
}

/// `Pack.Position.insert`: the place a pack goes in a list among the packs that keep theirs.
fn insert_at(
    list: &mut Vec<String>,
    id: String,
    default: Position,
    config: &dyn Fn(&str) -> SelectionConfig,
    reverse: bool,
) {
    let side = if reverse { default.opposite() } else { default };
    let pinned = |id: &String| {
        let found = config(id);
        found.fixed_position && found.default_position == default
    };
    if side == Position::Bottom {
        let index = list
            .iter()
            .position(|other| !pinned(other))
            .unwrap_or(list.len());
        list.insert(index, id);
    } else {
        let index = list
            .iter()
            .rposition(|other| !pinned(other))
            .map_or(0, |last| last + 1);
        list.insert(index, id);
    }
}

/// `PackRepository`: the packs that can be chosen, and which of them are.
#[derive(Debug, Clone, Default)]
pub struct Repository {
    available: BTreeMap<String, PackEntry>,
    /// The ids in use, the lowest first.
    selected: Vec<String>,
}

impl Repository {
    /// The repository over `entries`, with `selected` chosen (`setSelected`).
    pub fn new(entries: Vec<PackEntry>, selected: &[String]) -> Self {
        let mut repository = Self {
            available: entries.into_iter().map(|e| (e.id.clone(), e)).collect(),
            selected: Vec::new(),
        };
        repository.set_selected(selected);
        repository
    }

    /// `PackRepository.reload`: the packs that exist now, with the ones still there kept.
    pub fn reload(&mut self, entries: Vec<PackEntry>) {
        let kept = self.selected.clone();
        self.available = entries.into_iter().map(|e| (e.id.clone(), e)).collect();
        self.set_selected(&kept);
    }

    /// `PackRepository.setSelected` (`rebuildSelected`): the ids that are present, and every
    /// required pack at its default place.
    pub fn set_selected(&mut self, ids: &[String]) {
        let mut chosen: Vec<String> = ids
            .iter()
            .filter(|id| self.available.contains_key(*id))
            .cloned()
            .collect();
        let config = |id: &str| self.available[id].config;
        let required: Vec<String> = self
            .available
            .values()
            .filter(|e| e.config.required && !chosen.contains(&e.id))
            .map(|e| e.id.clone())
            .collect();
        for id in required {
            let default = config(&id).default_position;
            insert_at(&mut chosen, id, default, &config, false);
        }
        self.selected = chosen;
    }

    pub fn available(&self) -> impl Iterator<Item = &PackEntry> {
        self.available.values()
    }

    pub fn selected_ids(&self) -> &[String] {
        &self.selected
    }

    pub fn get(&self, id: &str) -> Option<&PackEntry> {
        self.available.get(id)
    }
}

/// What `Options.updateResourcePacks` keeps of the selection: the ids of the packs that do not
/// keep their place, and which of them are not compatible.
pub fn options_lists(repository: &Repository) -> (Vec<String>, Vec<String>) {
    let mut packs = Vec::new();
    let mut incompatible = Vec::new();
    for id in repository.selected_ids() {
        let Some(entry) = repository.get(id) else {
            continue;
        };
        if !entry.config.fixed_position {
            packs.push(id.clone());
            if !entry.compatibility.is_compatible() {
                incompatible.push(id.clone());
            }
        }
    }
    (packs, incompatible)
}

/// Which of the two lists an entry is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Selected,
    Unselected,
}

/// `PackSelectionModel`: the selected packs (the highest first) and the others, as the screen
/// shows them.
#[derive(Debug, Clone)]
pub struct SelectionModel {
    repository: Repository,
    selected: Vec<String>,
    unselected: Vec<String>,
}

impl SelectionModel {
    pub fn new(repository: Repository) -> Self {
        let mut model = Self {
            repository,
            selected: Vec::new(),
            unselected: Vec::new(),
        };
        model.take_lists();
        model
    }

    fn take_lists(&mut self) {
        self.selected = self
            .repository
            .selected_ids()
            .iter()
            .rev()
            .cloned()
            .collect();
        self.unselected = self
            .repository
            .available()
            .map(|entry| entry.id.clone())
            .filter(|id| !self.selected.contains(id))
            .collect();
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn entry(&self, id: &str) -> Option<&PackEntry> {
        self.repository.get(id)
    }

    /// The ids of a list, in the order the screen shows them.
    pub fn list(&self, side: Side) -> &[String] {
        match side {
            Side::Selected => &self.selected,
            Side::Unselected => &self.unselected,
        }
    }

    fn list_mut(&mut self, side: Side) -> &mut Vec<String> {
        match side {
            Side::Selected => &mut self.selected,
            Side::Unselected => &mut self.unselected,
        }
    }

    fn config(&self, id: &str) -> SelectionConfig {
        self.repository.get(id).map_or(DISCOVERED, |e| e.config)
    }

    /// `canUnselect`: a selected pack that is not required.
    pub fn can_unselect(&self, id: &str) -> bool {
        self.selected.iter().any(|other| other == id) && !self.config(id).required
    }

    /// `canSelect`: a pack not selected.
    pub fn can_select(&self, id: &str) -> bool {
        !self.selected.iter().any(|other| other == id)
    }

    /// `toggleSelection`: the pack changes list and goes to its default place in the other.
    pub fn toggle(&mut self, id: &str) {
        let from = if self.can_select(id) {
            Side::Unselected
        } else {
            Side::Selected
        };
        let to = if from == Side::Selected {
            Side::Unselected
        } else {
            Side::Selected
        };
        self.list_mut(from).retain(|other| other != id);
        let default = self.config(id).default_position;
        let configs: BTreeMap<String, SelectionConfig> = self
            .repository
            .available()
            .map(|e| (e.id.clone(), e.config))
            .collect();
        let lookup = |id: &str| configs.get(id).copied().unwrap_or(DISCOVERED);
        insert_at(self.list_mut(to), id.to_owned(), default, &lookup, true);
        self.update_repository();
    }

    fn update_repository(&mut self) {
        let ids: Vec<String> = self.selected.iter().rev().cloned().collect();
        self.repository.set_selected(&ids);
    }

    fn position_of(&self, id: &str) -> Option<(Side, usize)> {
        [Side::Selected, Side::Unselected]
            .into_iter()
            .find_map(|side| {
                self.list(side)
                    .iter()
                    .position(|other| other == id)
                    .map(|at| (side, at))
            })
    }

    /// `canMoveUp`: not the first of its list, and the one above keeps no place of its own.
    pub fn can_move_up(&self, id: &str) -> bool {
        let Some((side, at)) = self.position_of(id) else {
            return false;
        };
        at > 0 && !self.config(&self.list(side)[at - 1]).fixed_position
    }

    /// `canMoveDown`.
    pub fn can_move_down(&self, id: &str) -> bool {
        let Some((side, at)) = self.position_of(id) else {
            return false;
        };
        let list = self.list(side);
        at + 1 < list.len() && !self.config(&list[at + 1]).fixed_position
    }

    pub fn move_up(&mut self, id: &str) {
        self.shift(id, -1);
    }

    pub fn move_down(&mut self, id: &str) {
        self.shift(id, 1);
    }

    /// `move`: the pack swaps with its neighbour in its list.
    fn shift(&mut self, id: &str, by: isize) {
        let Some((side, at)) = self.position_of(id) else {
            return;
        };
        let list = self.list_mut(side);
        let to = at.saturating_add_signed(by);
        if to < list.len() {
            let moved = list.remove(at);
            list.insert(to, moved);
        }
    }

    /// `findNewPacks`: the packs of the folder now, with the selected ones that are still there
    /// kept in their order.
    pub fn find_new_packs(&mut self, entries: Vec<PackEntry>) {
        self.repository.reload(entries);
        self.selected.retain(|id| self.repository.get(id).is_some());
        self.unselected = self
            .repository
            .available()
            .map(|entry| entry.id.clone())
            .filter(|id| !self.selected.contains(id))
            .collect();
    }

    /// `commit`: the repository with the selection as the lists leave it.
    pub fn commit(&mut self) -> &Repository {
        self.update_repository();
        &self.repository
    }
}

/// The folder the packs are in (`Minecraft.getResourcePackDirectory`).
pub fn pack_directory(game_directory: &Path) -> PathBuf {
    game_directory.join("resourcepacks")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, config: SelectionConfig, compatibility: Compatibility) -> PackEntry {
        PackEntry {
            id: id.to_owned(),
            title: Title::Literal(id.to_owned()),
            description: JsonValue::Null,
            compatibility,
            source: PackSource::Default,
            config,
            icon: None,
        }
    }

    /// A required pack that also keeps its place, as a server's pack does.
    fn vanilla() -> PackEntry {
        let config = SelectionConfig {
            required: true,
            fixed_position: true,
            default_position: Position::Bottom,
        };
        entry("vanilla", config, Compatibility::Compatible)
    }

    fn model(extra: &[&str]) -> SelectionModel {
        let mut entries = vec![vanilla()];
        entries.extend(
            extra
                .iter()
                .map(|id| entry(id, DISCOVERED, Compatibility::Compatible)),
        );
        SelectionModel::new(Repository::new(entries, &[]))
    }

    fn ids(model: &SelectionModel, side: Side) -> Vec<&str> {
        model.list(side).iter().map(String::as_str).collect()
    }

    #[test]
    fn the_games_own_pack_is_required_at_the_bottom_and_listed_in_the_options() {
        let own = vanilla_entry(None);
        assert!(own.config.required && !own.config.fixed_position);
        assert_eq!(own.config.default_position, Position::Bottom);
        let model = SelectionModel::new(Repository::new(vec![own], &[]));
        assert_eq!(ids(&model, Side::Selected), ["vanilla"]);
        assert_eq!(options_lists(model.repository()).0, ["vanilla"]);
    }

    #[test]
    fn the_required_pack_is_selected_and_the_others_wait() {
        let model = model(&["file/a", "file/b"]);
        assert_eq!(ids(&model, Side::Selected), ["vanilla"]);
        assert_eq!(ids(&model, Side::Unselected), ["file/a", "file/b"]);
        assert!(!model.can_unselect("vanilla"));
        assert!(model.can_select("file/a"));
    }

    #[test]
    fn a_selected_pack_goes_over_the_ones_below_it() {
        let mut model = model(&["file/a", "file/b"]);
        model.toggle("file/a");
        model.toggle("file/b");
        assert_eq!(ids(&model, Side::Selected), ["file/b", "file/a", "vanilla"]);
        assert_eq!(ids(&model, Side::Unselected), Vec::<&str>::new());
        // The lowest keeps off the fixed pack under it.
        assert!(!model.can_move_down("file/a"));
        assert!(model.can_move_up("file/a"));
        assert!(!model.can_move_up("file/b"));
        model.move_up("file/a");
        assert_eq!(ids(&model, Side::Selected), ["file/a", "file/b", "vanilla"]);
    }

    #[test]
    fn commit_gives_the_lowest_first_and_options_skip_the_fixed_pack() {
        let mut model = model(&["file/a", "file/b"]);
        model.toggle("file/a");
        model.toggle("file/b");
        model.move_up("file/a");
        let repository = model.commit().clone();
        assert_eq!(repository.selected_ids(), ["vanilla", "file/b", "file/a"]);
        let (packs, incompatible) = options_lists(&repository);
        assert_eq!(packs, ["file/b", "file/a"]);
        assert!(incompatible.is_empty());
    }

    #[test]
    fn an_unselected_pack_returns_to_the_top_of_the_other_list() {
        let mut model = model(&["file/a", "file/b"]);
        model.toggle("file/a");
        model.toggle("file/a");
        assert_eq!(ids(&model, Side::Selected), ["vanilla"]);
        assert_eq!(ids(&model, Side::Unselected), ["file/a", "file/b"]);
    }

    #[test]
    fn an_incompatible_pack_is_listed_in_the_options() {
        let old = entry("file/old", DISCOVERED, Compatibility::TooOld);
        let repository = Repository::new(vec![vanilla(), old], &["file/old".to_owned()]);
        let (packs, incompatible) = options_lists(&repository);
        assert_eq!(packs, ["file/old"]);
        assert_eq!(incompatible, ["file/old"]);
        assert_eq!(repository.selected_ids(), ["vanilla", "file/old"]);
    }

    #[test]
    fn a_pack_the_folder_lost_leaves_the_selection() {
        let mut model = model(&["file/a"]);
        model.toggle("file/a");
        model.find_new_packs(vec![vanilla()]);
        assert_eq!(ids(&model, Side::Selected), ["vanilla"]);
        assert!(ids(&model, Side::Unselected).is_empty());
    }

    #[test]
    fn the_folder_lists_directories_and_zips_with_metadata() {
        let dir = std::env::temp_dir().join(format!("photon-packs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let folder = dir.join("loose");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join("pack.mcmeta"),
            r#"{"pack":{"description":"Hi","min_format":88,"max_format":88}}"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("empty")).unwrap();
        std::fs::write(dir.join("notes.txt"), "x").unwrap();
        let found = discover_folder(&dir, PackFormat::of(88, 0));
        std::fs::remove_dir_all(&dir).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "file/loose");
        assert_eq!(found[0].compatibility, Compatibility::Compatible);
    }
}
