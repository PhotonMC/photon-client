use super::*;

impl GameMode {
    /// The hands whose stack was used since the last call, in order.
    pub fn take_items_used(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.items_used)
    }
}
