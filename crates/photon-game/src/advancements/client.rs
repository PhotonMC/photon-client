//! The client's copy of the advancements (`ClientAdvancements`, `AdvancementTree`,
//! `AdvancementProgress`): the tree the server told of, how far the player is in each, and the tab
//! the screen shows.

use std::collections::HashMap;

use super::{Advancement, UpdateAdvancements};

/// `AdvancementProgress`: each criterion of an advancement, and when it was obtained.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementProgress {
    criteria: HashMap<String, Option<i64>>,
    requirements: Vec<Vec<String>>,
}

impl AdvancementProgress {
    /// `AdvancementProgress.fromNetwork`, before `update`.
    fn from_wire(criteria: &[(String, Option<i64>)]) -> Self {
        Self {
            criteria: criteria.iter().cloned().collect(),
            requirements: Vec::new(),
        }
    }

    /// `update`: the criteria are those the requirements name, each held or empty.
    fn update(&mut self, requirements: &[Vec<String>]) {
        let names: Vec<&String> = requirements.iter().flatten().collect();
        self.criteria.retain(|name, _| names.contains(&name));
        for name in names {
            self.criteria.entry(name.clone()).or_insert(None);
        }
        self.requirements = requirements.to_vec();
    }

    fn criterion_done(&self, name: &str) -> bool {
        self.criteria.get(name).is_some_and(Option::is_some)
    }

    /// `AdvancementRequirements.count`: the groups with a criterion done.
    fn completed_requirements(&self) -> usize {
        self.requirements
            .iter()
            .filter(|group| group.iter().any(|name| self.criterion_done(name)))
            .count()
    }

    /// `isDone`: every group has one done, and there is a group.
    pub fn is_done(&self) -> bool {
        !self.requirements.is_empty() && self.completed_requirements() == self.requirements.len()
    }

    /// `hasProgress`: a criterion is done.
    pub fn has_progress(&self) -> bool {
        self.criteria.values().any(Option::is_some)
    }

    /// `getPercent`.
    pub fn percent(&self) -> f32 {
        if self.criteria.is_empty() {
            return 0.0;
        }
        self.completed_requirements() as f32 / self.requirements.len() as f32
    }

    /// `getProgressText`'s numbers: the groups done and the groups, for an advancement with more
    /// than one (`advancements.progress`).
    pub fn progress_counts(&self) -> Option<(usize, usize)> {
        if self.criteria.is_empty() || self.requirements.len() <= 1 {
            return None;
        }
        Some((self.completed_requirements(), self.requirements.len()))
    }

    /// `getCompletedCriteria` and `getRemainingCriteria`, by name.
    pub fn criteria(&self) -> Vec<(&str, bool)> {
        let mut criteria: Vec<_> = self
            .criteria
            .iter()
            .map(|(name, obtained)| (name.as_str(), obtained.is_some()))
            .collect();
        criteria.sort_unstable();
        criteria
    }

    /// `getFirstProgressDate`: when the first criterion was obtained.
    pub fn first_progress(&self) -> Option<i64> {
        self.criteria.values().flatten().min().copied()
    }
}

/// One advancement in the tree.
#[derive(Debug, Clone)]
struct Node {
    advancement: Advancement,
    parent: Option<String>,
    /// In the order they were inserted; the game's set has no order.
    children: Vec<String>,
}

/// What an update did that others look at.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Updated {
    /// The advancements done by this update whose toast is shown, in the order of the packet's
    /// progress.
    pub toasts: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ClientAdvancements {
    nodes: HashMap<String, Node>,
    /// The roots and the other advancements, in the order they were added (`ObjectLinkedOpenHashSet`).
    roots: Vec<String>,
    tasks: Vec<String>,
    progress: HashMap<String, AdvancementProgress>,
    selected_tab: Option<String>,
}

impl ClientAdvancements {
    /// `ClientAdvancements.update`.
    pub fn update(&mut self, packet: &UpdateAdvancements) -> Updated {
        if packet.reset {
            self.nodes.clear();
            self.roots.clear();
            self.tasks.clear();
            self.progress.clear();
        }
        for id in &packet.removed {
            self.remove(id);
        }
        self.add_all(&packet.added);
        let mut updated = Updated::default();
        for (id, criteria) in &packet.progress {
            let Some(node) = self.nodes.get(id) else {
                continue;
            };
            let mut progress = AdvancementProgress::from_wire(criteria);
            progress.update(&node.advancement.requirements);
            let toast = !packet.reset
                && progress.is_done()
                && packet.show_advancements
                && node
                    .advancement
                    .display
                    .as_ref()
                    .is_some_and(|display| display.show_toast);
            self.progress.insert(id.clone(), progress);
            if toast {
                updated.toasts.push(id.clone());
            }
        }
        updated
    }

    /// `AdvancementTree.remove(Set)`: each id and the advancements under it.
    fn remove(&mut self, id: &str) {
        let Some(node) = self.nodes.get(id) else {
            return;
        };
        for child in node.children.clone() {
            self.remove(&child);
        }
        let Some(node) = self.nodes.remove(id) else {
            return;
        };
        if node.parent.is_none() {
            self.roots.retain(|root| root != id);
        } else {
            self.tasks.retain(|task| task != id);
        }
        // The node is not in the tree, but what the player did in it is forgotten only by a reset.
    }

    /// `AdvancementTree.addAll`: as many as can be inserted, over and over, until none can.
    fn add_all(&mut self, added: &[Advancement]) {
        let mut pending: Vec<&Advancement> = added.iter().collect();
        while !pending.is_empty() {
            let before = pending.len();
            pending.retain(|advancement| !self.try_insert(advancement));
            if pending.len() == before {
                break;
            }
        }
    }

    /// `AdvancementTree.tryInsert`: a child needs its parent to be there already.
    fn try_insert(&mut self, advancement: &Advancement) -> bool {
        if let Some(parent) = &advancement.parent {
            let Some(node) = self.nodes.get_mut(parent) else {
                return false;
            };
            if !node.children.contains(&advancement.id) {
                node.children.push(advancement.id.clone());
            }
            if !self.tasks.contains(&advancement.id) {
                self.tasks.push(advancement.id.clone());
            }
        } else if !self.roots.contains(&advancement.id) {
            self.roots.push(advancement.id.clone());
        }
        self.nodes.insert(
            advancement.id.clone(),
            Node {
                advancement: advancement.clone(),
                parent: advancement.parent.clone(),
                children: Vec::new(),
            },
        );
        true
    }

    /// `setSelectedTab`: the tab the screen shows (the server is told by the screen).
    pub fn select_tab(&mut self, tab: Option<&str>) {
        self.selected_tab = tab.map(str::to_owned);
    }

    pub fn selected_tab(&self) -> Option<&str> {
        self.selected_tab.as_deref()
    }

    /// The roots in the order they were added.
    pub fn roots(&self) -> impl Iterator<Item = &Advancement> {
        self.roots.iter().filter_map(|id| self.get(id))
    }

    /// The advancements that are not roots, in the order they were added.
    pub fn tasks(&self) -> impl Iterator<Item = &Advancement> {
        self.tasks.iter().filter_map(|id| self.get(id))
    }

    pub fn get(&self, id: &str) -> Option<&Advancement> {
        self.nodes.get(id).map(|node| &node.advancement)
    }

    /// The ids of the advancements directly under one, in the order they were added. A node's set
    /// keeps a child that was removed, as the game's does: only the tree forgets it.
    pub fn child_ids(&self, id: &str) -> &[String] {
        self.nodes
            .get(id)
            .map_or(&[], |node| node.children.as_slice())
    }

    /// The advancements directly under one that the tree still has.
    pub fn children(&self, id: &str) -> Vec<&Advancement> {
        self.child_ids(id)
            .iter()
            .filter_map(|child| self.get(child))
            .collect()
    }

    /// `AdvancementNode.root`: the advancement at the top of the one given.
    pub fn root_of(&self, id: &str) -> Option<&Advancement> {
        let mut at = self.nodes.get(id)?;
        while let Some(parent) = at.parent.as_ref().and_then(|parent| self.nodes.get(parent)) {
            at = parent;
        }
        Some(&at.advancement)
    }

    pub fn progress(&self, id: &str) -> Option<&AdvancementProgress> {
        self.progress.get(id)
    }

    /// The advancements the player has progress held for, even those since removed (only a reset
    /// forgets progress), by id.
    pub fn progress_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.progress.keys().map(String::as_str).collect();
        ids.sort_unstable();
        ids
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
