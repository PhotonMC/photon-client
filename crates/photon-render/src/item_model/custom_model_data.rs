//! The `custom_model_data` component, as the item model properties read it.

/// The prefix of a `custom_model_data` property, which carries the index after it.
pub const CUSTOM_MODEL_DATA: &str = "minecraft:custom_model_data/";

/// `CustomModelData`: the lists a pack's item definitions pick values from by index. Floats are
/// held as their bits so a state can stay comparable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomModelData {
    pub floats: Vec<u32>,
    pub flags: Vec<bool>,
    pub strings: Vec<String>,
    pub colors: Vec<i32>,
}

impl CustomModelData {
    /// `getFloat`: none past the list.
    pub fn float(&self, index: usize) -> Option<f32> {
        self.floats.get(index).copied().map(f32::from_bits)
    }

    /// `getBoolean`.
    pub fn flag(&self, index: usize) -> Option<bool> {
        self.flags.get(index).copied()
    }

    /// `getString`.
    pub fn string(&self, index: usize) -> Option<&str> {
        self.strings.get(index).map(String::as_str)
    }

    /// `getColor`.
    pub fn color(&self, index: usize) -> Option<i32> {
        self.colors.get(index).copied()
    }
}

/// The index a `custom_model_data` property names.
pub fn custom_index(property: &str) -> Option<usize> {
    property.strip_prefix(CUSTOM_MODEL_DATA)?.parse().ok()
}
