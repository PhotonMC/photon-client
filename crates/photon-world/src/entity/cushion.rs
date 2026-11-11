use super::*;

impl ClientEntity {
    /// `Cushion.getColor`: the id of the dye colour the cushion has (white until the server says).
    pub fn cushion_colour(&self) -> i32 {
        self.metadata()
            .and_then(crate::metadata::cushion_colour_from_metadata)
            .unwrap_or(0)
    }
}
