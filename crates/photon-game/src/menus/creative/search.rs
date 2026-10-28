use super::*;
use photon_core::ids::ItemId;

/// The stack a tab declares: the item and the components it carries, as the
/// network carries them. A component naming an entry the registries do not
/// hold is left out.
pub fn table_stack(entry: &CreativeStack, registries: &Registries) -> Stack {
    let item = photon_data::item::registry::index(entry.item)
        .map_or(ItemId(0), |index| ItemId(index as i32));
    let added: Vec<(i32, Vec<u8>)> = entry
        .components
        .iter()
        .filter_map(|component| {
            crate::item::default_components::encode(component, Some(registries))
        })
        .collect();
    let positive = added.iter().map(|(id, _)| *id).collect();
    let patch = crate::item_components::Patch {
        added,
        removed: Vec::new(),
    };
    Stack::from_wire(1, item, positive, Vec::new()).with_patch_bytes(patch.trusted())
}

/// The search tab, which lists every stack the other tabs do.
/// Whether the session's version has the entry's item, `registries.has(..)`: a tab lists the union
/// of every version's items.
pub(super) fn offered(entry: &CreativeStack) -> bool {
    photon_data::version_registries::VersionRegistries::newest().has_item(entry.item)
}

pub(super) fn search_tab() -> &'static photon_data::item::creative::CreativeTab {
    CREATIVE_TABS
        .iter()
        .find(|tab| tab.kind == TabKind::Search)
        .expect("there is a search tab")
}

/// The tabs that are shown, in the reference's order: `CreativeModeTabs.tabs`.
pub fn shown_tabs() -> Vec<usize> {
    (0..CREATIVE_TABS.len())
        .filter(|&tab| CREATIVE_TABS[tab].should_display)
        .collect()
}

/// The search tab's items with what the search reads of each: the lines of
/// its tooltip, lowercased, and its id. `SessionSearchTrees` builds it from
/// the search tab's display items.
pub struct SearchIndex {
    /// The search tab's stacks, by place.
    pub(super) count: usize,
    pub(super) lines: Vec<Vec<String>>,
    /// Each item's namespace and path, lowercased.
    pub(super) ids: Vec<(String, String)>,
}

/// Whether a query finds what has these tooltip lines (lowercased) and these
/// ids (namespace and path, lowercased): a query with a colon names a
/// namespace and a path, anything else is text of a line.
pub fn query_matches(lines: &[String], ids: &[(String, String)], text: &str) -> bool {
    let by_line = |needle: &str| lines.iter().any(|line| line.contains(needle));
    match text.split_once(':') {
        None => by_line(text),
        Some((namespace, path)) => {
            let (namespace, path) = (namespace.trim(), path.trim());
            ids.iter().any(|(id_namespace, id_path)| {
                id_namespace.contains(namespace) && (id_path.contains(path) || by_line(path))
            })
        }
    }
}

/// `creativeTagSearch`, an `IdSearchTree` over each search tab stack's item
/// tags: the places a lowercased query finds, in the tab's order. A query
/// with a colon finds what has a tag whose namespace holds the one part and
/// a tag, not necessarily the same, whose path holds the other; anything else
/// is part of a tag's path. A stack without tags is never found.
pub fn tag_search(tags: &ItemTags, text: &str) -> Vec<u16> {
    let mut of_item: std::collections::HashMap<i32, Vec<(&str, &str)>> =
        std::collections::HashMap::new();
    for (tag, items) in tags.iter() {
        let id = tag.split_once(':').unwrap_or(("minecraft", tag));
        for &item in items {
            of_item.entry(item).or_default().push(id);
        }
    }
    let query = text
        .split_once(':')
        .map(|(namespace, path)| (namespace.trim(), path.trim()));
    search_tab()
        .items
        .iter()
        .enumerate()
        .filter(|(_, entry)| {
            let Some(ids) = photon_data::item::registry::index(entry.item)
                .and_then(|item| of_item.get(&(item as i32)))
            else {
                return false;
            };
            match query {
                None => ids.iter().any(|(_, path)| path.contains(text)),
                Some((namespace, path)) => {
                    ids.iter()
                        .any(|(id_namespace, _)| id_namespace.contains(namespace))
                        && ids.iter().any(|(_, id_path)| id_path.contains(path))
                }
            }
        })
        .map(|(at, _)| at as u16)
        .collect()
}

/// `updateVisibleTags`: the item tags a tag search names, read from the text
/// as typed (not lowercased), whose tooltips list them. A colon splits a
/// namespace from a path, both trimmed; anything else is part of the path.
pub fn visible_tags(tags: &ItemTags, text: &str) -> Vec<String> {
    let query = text
        .split_once(':')
        .map(|(namespace, path)| (namespace.trim(), path.trim()));
    tags.iter()
        .map(|(tag, _)| tag)
        .filter(|tag| {
            let (namespace, path) = tag.split_once(':').unwrap_or(("minecraft", tag));
            match query {
                None => path.contains(text),
                Some((wanted_namespace, wanted_path)) => {
                    namespace.contains(wanted_namespace) && path.contains(wanted_path)
                }
            }
        })
        .map(str::to_owned)
        .collect()
}

/// The search trees of a session: built from the language the first time a
/// search asks, which takes a few milliseconds.
pub struct SearchTrees {
    pub(super) lang: std::sync::Arc<crate::text::Lang>,
    pub(super) registries: std::sync::Arc<Registries>,
    pub(super) index: std::sync::OnceLock<SearchIndex>,
}

impl std::fmt::Debug for SearchTrees {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SearchTrees")
    }
}

/// `CreativeModeTab.contains`: whether a stack is one the tab gives the
/// search tab, by item and component patch.
pub fn tab_contains(tab: usize, stack: &Stack, registries: &Registries) -> bool {
    let entry = &CREATIVE_TABS[tab];
    let candidates = if entry.search_items.is_empty() {
        entry.items
    } else {
        entry.search_items
    };
    // A stack made here has no patch bytes: the empty patch is two zeros.
    let patch: &[u8] = if stack.patch_bytes().is_empty() {
        &[0, 0]
    } else {
        stack.patch_bytes()
    };
    let item = usize::try_from(stack.item_registry_id().0).ok();
    candidates.iter().any(|candidate| {
        photon_data::item::registry::index(candidate.item) == item
            && table_stack(candidate, registries).patch_bytes() == patch
    })
}

/// `CreativeModeTabs.getDefaultTab`.
pub fn default_tab() -> usize {
    CREATIVE_TABS
        .iter()
        .position(|tab| tab.name == DEFAULT_TAB)
        .expect("the default tab is in the table")
}

impl SearchIndex {
    pub fn build(lang: &crate::text::Lang, registries: &Registries) -> Self {
        let items = search_tab().items;
        let mut lines = Vec::with_capacity(items.len());
        let mut ids = Vec::with_capacity(items.len());
        for entry in items {
            let stack = table_stack(entry, registries);
            lines.push(
                crate::tooltip::search_lines(&stack, lang, registries)
                    .into_iter()
                    .map(|line| line.to_lowercase())
                    .collect(),
            );
            let name = entry.item;
            let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
            ids.push((namespace.to_lowercase(), path.to_lowercase()));
        }
        Self {
            count: items.len(),
            lines,
            ids,
        }
    }

    /// `SearchTree.search`: the places in the search tab of the stacks a
    /// query finds, in its order. A query with a colon names a namespace and
    /// a path; anything else is read as text of the tooltip.
    pub fn search(&self, text: &str) -> Vec<u16> {
        (0..self.count)
            .filter(|&at| query_matches(&self.lines[at], std::slice::from_ref(&self.ids[at]), text))
            .map(|at| at as u16)
            .collect()
    }
}

impl SearchTrees {
    pub fn new(
        lang: std::sync::Arc<crate::text::Lang>,
        registries: std::sync::Arc<Registries>,
    ) -> Self {
        Self {
            lang,
            registries,
            index: std::sync::OnceLock::new(),
        }
    }

    /// The registries the trees were built with.
    pub fn registries(&self) -> &std::sync::Arc<Registries> {
        &self.registries
    }

    pub fn index(&self) -> &SearchIndex {
        self.index
            .get_or_init(|| SearchIndex::build(&self.lang, &self.registries))
    }
}
