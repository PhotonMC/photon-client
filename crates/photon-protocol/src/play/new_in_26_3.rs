//! Packets 26.3 added: the swing animation, the post effects and the transient block.

use super::*;
use photon_core::ids::EntityId;

/// `ClientboundSwingAnimationPacket`: a living entity swings a hand with an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwingAnimationObservation {
    pub entity_id: EntityId,
    pub off_hand: bool,
    /// `SwingAnimationType`: 0 none, 1 whack, 2 stab.
    pub kind: u8,
    pub duration: i32,
}

pub fn parse_swing_animation_packet(
    input: &[u8],
) -> Result<SwingAnimationObservation, ConnectionError> {
    let mut input = input;
    let entity_id = EntityId(read_varint(&mut input, "swing entity id")?);
    // Both enums are id mappers that fall back to their first value.
    let off_hand = read_varint(&mut input, "swing hand")? == 1;
    let kind = match read_varint(&mut input, "swing type")? {
        1 => 1,
        2 => 2,
        _ => 0,
    };
    let duration = read_varint(&mut input, "swing duration")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("swing animation trailing payload"));
    }
    Ok(SwingAnimationObservation {
        entity_id,
        off_hand,
        kind,
        duration,
    })
}

/// `ClientboundPostEffectsPacket`: the post-processing chains the player has, by name.
pub fn parse_post_effects_packet(input: &[u8]) -> Result<Vec<String>, ConnectionError> {
    let mut input = input;
    let count = read_varint(&mut input, "post effects count")?;
    // Every name takes at least two bytes.
    if count < 0 || count as usize > input.len() / 2 {
        return Err(ConnectionError::Codec("post effects count out of range"));
    }
    let mut effects = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = photon_core::identifier::read_identifier(&mut input)
            .map_err(|_| ConnectionError::Codec("post effects identifier"))?;
        effects.push(name.as_string());
    }
    if !input.is_empty() {
        return Err(ConnectionError::Codec("post effects trailing payload"));
    }
    Ok(effects)
}

/// `ClientboundAddTransientBlockPacket`: a block shown for a moment at a place (the position and
/// the wire id of its state).
pub fn parse_add_transient_block_packet(input: &[u8]) -> Result<(i64, i32), ConnectionError> {
    let mut input = input;
    let position = read_i64(&mut input, "transient block position")?;
    let state = read_varint(&mut input, "transient block state")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("transient block trailing payload"));
    }
    Ok((position, state))
}

wrapped_packet!(
    /// `ClientboundSwingAnimationPacket`.
    SwingAnimation(SwingAnimationObservation) = parse_swing_animation_packet,
    OnError::Ignore("swing_animation")
);

wrapped_packet!(
    /// `ClientboundPostEffectsPacket`.
    PostEffects(Vec<String>) = parse_post_effects_packet,
    OnError::Ignore("post_effects")
);

wrapped_packet!(
    /// `ClientboundAddTransientBlockPacket`.
    AddTransientBlock((i64, i32)) = parse_add_transient_block_packet,
    OnError::Ignore("add_transient_block")
);
