/// Who registered something. Everything a feature adds is tagged with its owner, so turning the
/// feature off takes all of it away again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Owner(&'static str);

impl Owner {
    /// The owner called `name`, which is the feature's id (two owners with one name are one).
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn name(self) -> &'static str {
        self.0
    }
}

/// Most distinct names [`Owner::dynamic`] keeps; the names live as long as the process.
const MAX_DYNAMIC: usize = 512;

impl Owner {
    /// The owner called `name` for a feature whose name is only known while running (a mod a
    /// server delivers). The same name gives the same owner. `None` when too many names exist.
    pub fn dynamic(name: &str) -> Option<Self> {
        use std::collections::HashSet;
        use std::sync::{Mutex, OnceLock};
        static NAMES: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
        let mut names = NAMES
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(known) = names.get(name) {
            return Some(Self(known));
        }
        if names.len() >= MAX_DYNAMIC {
            return None;
        }
        let kept: &'static str = Box::leak(name.to_owned().into_boxed_str());
        names.insert(kept);
        Some(Self(kept))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dynamic_name_is_one_owner_however_often_it_is_asked_for() {
        let (a, b) = (Owner::dynamic("tycoon"), Owner::dynamic("tycoon"));
        assert_eq!(a, b);
        assert_eq!(a.map(Owner::name), Some("tycoon"));
        assert_ne!(a, Owner::dynamic("planes"));
    }
}
