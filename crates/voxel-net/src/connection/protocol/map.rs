use super::*;

/// `ClientboundMapItemDataPacket`: the map's id, scale and lock, the decorations (if sent) and the
/// patch of colours (a width of zero is none).
pub(crate) fn parse_map_item_data_packet(
    input: &[u8],
) -> Result<voxel_world::map::MapItemDataInput, ConnectionError> {
    use voxel_world::map::{MapDecoration, MapItemDataInput, MapPatch};
    let mut input = input;
    let map_id = read_varint(&mut input, "map id")?;
    let scale = read_byte(&mut input, "map scale")? as i8;
    let locked = read_bool(&mut input, "map locked")?;
    let decorations = if read_bool(&mut input, "map decorations present")? {
        let count = read_varint(&mut input, "map decoration count")?;
        let mut decorations = Vec::new();
        for _ in 0..count {
            let type_id = read_varint(&mut input, "map decoration type")?;
            let x = read_byte(&mut input, "map decoration x")? as i8;
            let y = read_byte(&mut input, "map decoration y")? as i8;
            let rot = read_byte(&mut input, "map decoration rot")? as i8;
            let name = if read_bool(&mut input, "map decoration name present")? {
                let before = input;
                voxel_core::nbt::read_any_nbt(&mut input)
                    .map_err(|_| ConnectionError::Codec("map decoration name"))?;
                Some(before[..before.len() - input.len()].to_vec())
            } else {
                None
            };
            decorations.push(MapDecoration {
                type_id,
                x,
                y,
                rot: rot & 15,
                name,
            });
        }
        Some(decorations)
    } else {
        None
    };
    let width = read_byte(&mut input, "map patch width")?;
    let patch = if width > 0 {
        let height = read_byte(&mut input, "map patch height")?;
        let start_x = read_byte(&mut input, "map patch x")?;
        let start_y = read_byte(&mut input, "map patch y")?;
        let length = read_count(&mut input, "map patch colour count")?;
        let colors = input
            .get(..length)
            .ok_or(ConnectionError::Codec("map patch colours"))?
            .to_vec();
        input = &input[length..];
        Some(MapPatch {
            start_x: usize::from(start_x),
            start_y: usize::from(start_y),
            width: usize::from(width),
            height: usize::from(height),
            colors,
        })
    } else {
        None
    };
    if !input.is_empty() {
        return Err(ConnectionError::Codec("map item data trailing payload"));
    }
    Ok(MapItemDataInput {
        map_id,
        scale,
        locked,
        decorations,
        patch,
    })
}
