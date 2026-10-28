use super::*;
use photon_core::text::Component;

/// `ClientboundTestInstanceBlockStatus`: the status line the test instance block shows, and the
/// size of the test's structure when the server knows it.
pub struct TestInstanceBlockStatus {
    pub status: Component,
    pub size: Option<(i32, i32, i32)>,
}

impl Decode for TestInstanceBlockStatus {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let status = read_chat_component(&mut input, "test instance status")?;
        let size = if read_bool(&mut input, "test instance has size")? {
            Some((
                read_varint(&mut input, "test instance size x")?,
                read_varint(&mut input, "test instance size y")?,
                read_varint(&mut input, "test instance size z")?,
            ))
        } else {
            None
        };
        if !input.is_empty() {
            return Err(ConnectionError::Codec(
                "test instance status trailing payload",
            ));
        }
        Ok(Self { status, size })
    }
}
