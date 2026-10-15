use super::*;

impl PlaySession {
    /// `handleOpenSignEditor`: the screen opens over a sign the client has; any other block is
    /// ignored with a warning.
    pub(super) fn on_open_sign_editor(&mut self, pos: photon_core::binary::BlockPos, front: bool) {
        let pos = (pos.x, pos.y, pos.z);
        if self.world.sign_view(pos).is_some() {
            self.sign_editor = Some((pos, front));
        } else {
            self.warn(format!(
                "ignoring openTextEdit on a block that is no sign at {pos:?}"
            ));
        }
    }

    /// The sign whose screen the server asked to open, once.
    pub fn take_sign_editor(&mut self) -> Option<((i32, i32, i32), bool)> {
        self.sign_editor.take()
    }

    /// `ServerboundSignUpdatePacket`: the lines the sign's screen closed with.
    pub fn sign_update(
        &mut self,
        pos: (i32, i32, i32),
        front: bool,
        lines: &[String; 4],
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SIGN_UPDATE_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            output.push(u8::from(front));
            for line in lines {
                wire::write_varint(line.len() as i32, output);
                output.extend_from_slice(line.as_bytes());
            }
            Ok(())
        })
    }
}
