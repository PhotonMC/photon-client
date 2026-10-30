/// One of the game's registries, read by name and by place. The place is the canonical index of
/// the union list ([ADR 0013](../../../docs/adr/0013-multi-version-content-model.md)), never a
/// version's wire id, so what a feature learns here is the same on every server version.
#[derive(Clone, Copy)]
pub struct Registry {
    len: fn() -> usize,
    name_at: fn(usize) -> Option<&'static str>,
    index_of: fn(&str) -> Option<usize>,
}

impl Registry {
    /// A registry read through three lookups: how many entries, the name at a place, and the
    /// place of a name.
    pub const fn new(
        len: fn() -> usize,
        name_at: fn(usize) -> Option<&'static str>,
        index_of: fn(&str) -> Option<usize>,
    ) -> Self {
        Self {
            len,
            name_at,
            index_of,
        }
    }

    pub fn len(&self) -> usize {
        (self.len)()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The name (`minecraft:stone`) of the entry at `index`.
    pub fn name_at(&self, index: usize) -> Option<&'static str> {
        (self.name_at)(index)
    }

    /// The place of the entry called `name`.
    pub fn index_of(&self, name: &str) -> Option<usize> {
        (self.index_of)(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.index_of(name).is_some()
    }
}

/// The registries a feature can look things up in.
#[derive(Clone, Copy)]
pub struct Registries {
    pub items: Registry,
    pub blocks: Registry,
    pub entity_types: Registry,
    pub sound_events: Registry,
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: [&str; 3] = ["minecraft:a", "minecraft:b", "minecraft:c"];

    fn sample() -> Registry {
        Registry::new(
            || NAMES.len(),
            |index| NAMES.get(index).copied(),
            |name| NAMES.iter().position(|known| *known == name),
        )
    }

    #[test]
    fn a_registry_reads_both_ways() {
        let registry = sample();
        assert_eq!(registry.len(), 3);
        assert_eq!(registry.index_of("minecraft:b"), Some(1));
        assert_eq!(registry.name_at(2), Some("minecraft:c"));
        assert_eq!(registry.name_at(3), None);
        assert!(registry.contains("minecraft:a"));
        assert!(!registry.contains("minecraft:z"));
        assert!(!registry.is_empty());
    }
}
