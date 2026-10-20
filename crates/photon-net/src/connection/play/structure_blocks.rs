use super::*;
use photon_game::structure_block::{
    Mirror, Rotation, StructureBlockContents, StructureMode, UpdateType,
};

/// What the structure block's screen sends back, but for the block's position
/// (`ServerboundSetStructureBlockPacket`).
pub struct StructureBlockSettings<'a> {
    pub update: UpdateType,
    pub mode: StructureMode,
    pub name: &'a str,
    pub offset: (i32, i32, i32),
    pub size: (i32, i32, i32),
    pub mirror: Mirror,
    pub rotation: Rotation,
    pub data: &'a str,
    pub flags: StructureFlags,
    pub integrity: f32,
    pub seed: i64,
}

/// The four switches the packet carries as bits.
pub struct StructureFlags {
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
}

impl StructureFlags {
    /// Ignore entities (1), show air (2), show bounding box (4) and strict (8).
    fn bits(&self) -> u8 {
        u8::from(self.ignore_entities)
            | u8::from(self.show_air) << 1
            | u8::from(self.show_bounding_box) << 2
            | u8::from(self.strict) << 3
    }
}

fn write_text(text: &str, output: &mut Vec<u8>) {
    wire::write_varint(text.len() as i32, output);
    output.extend_from_slice(text.as_bytes());
}

impl PlaySession {
    /// `ServerboundSetStructureBlockPacket`: the position, the update and mode by ordinal, the
    /// name, the offset and size as one byte each, mirror and rotation by ordinal, the data, the
    /// integrity as a float, the seed as a varlong and the flags.
    pub fn set_structure_block(
        &mut self,
        pos: (i32, i32, i32),
        settings: &StructureBlockSettings<'_>,
    ) -> Result<(), ConnectionError> {
        self.send(PLAY_SERVERBOUND_SET_STRUCTURE_BLOCK_PACKET_ID, |output| {
            output.extend_from_slice(&pack_block_position(pos).to_be_bytes());
            wire::write_varint(settings.update as i32, output);
            wire::write_varint(settings.mode as i32, output);
            write_text(settings.name, output);
            for each in [settings.offset, settings.size] {
                output.extend_from_slice(&[each.0 as u8, each.1 as u8, each.2 as u8]);
            }
            wire::write_varint(settings.mirror as i32, output);
            wire::write_varint(settings.rotation as i32, output);
            write_text(settings.data, output);
            output.extend_from_slice(&settings.integrity.to_be_bytes());
            wire::write_varlong(settings.seed, output);
            output.push(settings.flags.bits());
            Ok(())
        })
    }

    /// The structure block the player used, whose screen is to open (`Player.openStructureBlock`).
    pub fn take_opened_structure_block(&mut self) -> Option<(i32, i32, i32)> {
        self.local.game_mode_state.take_opened_structure_block()
    }

    /// The structure block at a position with the contents its block entity holds, or `None` when
    /// the block or its entity is not there.
    pub fn structure_block(&self, pos: (i32, i32, i32)) -> Option<StructureBlockContents> {
        let tag = self.world.structure_block_view(pos)?;
        Some(StructureBlockContents::from_tag(tag))
    }
}
