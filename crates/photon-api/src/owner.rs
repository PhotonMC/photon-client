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
