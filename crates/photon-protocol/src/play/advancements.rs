use super::*;
pub use photon_game::advancements::UpdateAdvancements;
use photon_game::advancements::{Advancement, AdvancementType, DisplayInfo, Progress};

/// A count the packet's lists and maps start with: not negative, and no more than there are bytes
/// left to hold that many entries.
fn read_count(input: &mut &[u8], name: &'static str) -> Result<usize, ConnectionError> {
    let count =
        usize::try_from(read_varint(input, name)?).map_err(|_| ConnectionError::Codec(name))?;
    if count > input.len() {
        return Err(ConnectionError::Codec(name));
    }
    Ok(count)
}

fn read_utf(input: &mut &[u8], name: &'static str) -> Result<String, ConnectionError> {
    read_chat_string(input, 32767, name)
}

/// `DisplayInfo.fromNetwork`.
fn read_display(input: &mut &[u8]) -> Result<DisplayInfo, ConnectionError> {
    let title = read_chat_component(input, "advancement title")?;
    let description = read_chat_component(input, "advancement description")?;
    // `ItemStackTemplate.STREAM_CODEC`: the item, its count and its components.
    let item = ItemId(read_varint(input, "advancement icon")?);
    let count = read_varint(input, "advancement icon count")?;
    let patch = photon_game::item_components::read_patch(input)
        .map_err(|_| ConnectionError::Codec("advancement icon components"))?;
    let frame = AdvancementType::from_ordinal(read_varint(input, "advancement frame")?)
        .ok_or(ConnectionError::Codec("advancement frame"))?;
    let flags = read_i32(input, "advancement flags")?;
    let background = if flags & 1 != 0 {
        Some(read_identifier_string(input, "advancement background")?)
    } else {
        None
    };
    Ok(DisplayInfo {
        title,
        description,
        icon: (item, count, patch),
        frame,
        show_toast: flags & 2 != 0,
        hidden: flags & 4 != 0,
        background,
        x: read_f32(input, "advancement x")?,
        y: read_f32(input, "advancement y")?,
    })
}

/// `AdvancementHolder.STREAM_CODEC`.
fn read_advancement(input: &mut &[u8]) -> Result<Advancement, ConnectionError> {
    let id = read_identifier_string(input, "advancement id")?;
    let parent = read_bool(input, "advancement parent")?
        .then(|| read_identifier_string(input, "advancement parent id"))
        .transpose()?;
    let display = read_bool(input, "advancement display")?
        .then(|| read_display(input))
        .transpose()?;
    let groups = read_count(input, "advancement requirements")?;
    let mut requirements = Vec::with_capacity(groups);
    for _ in 0..groups {
        let size = read_count(input, "advancement requirement")?;
        let mut group = Vec::with_capacity(size);
        for _ in 0..size {
            group.push(read_utf(input, "advancement criterion")?);
        }
        requirements.push(group);
    }
    Ok(Advancement {
        id,
        parent,
        display,
        requirements,
        sends_telemetry_event: read_bool(input, "advancement telemetry")?,
    })
}

/// `AdvancementProgress.fromNetwork`: each criterion with when it was obtained, if it was.
fn read_progress(input: &mut &[u8]) -> Result<Progress, ConnectionError> {
    let count = read_count(input, "advancement progress")?;
    let mut criteria = Vec::with_capacity(count);
    for _ in 0..count {
        let name = read_utf(input, "criterion name")?;
        let obtained = read_bool(input, "criterion obtained")?
            .then(|| read_i64(input, "criterion time"))
            .transpose()?;
        criteria.push((name, obtained));
    }
    Ok(criteria)
}

impl Decode for UpdateAdvancements {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let reset = read_bool(&mut input, "advancements reset")?;
        let added_count = read_count(&mut input, "advancements added")?;
        let mut added = Vec::with_capacity(added_count);
        for _ in 0..added_count {
            added.push(read_advancement(&mut input)?);
        }
        let removed_count = read_count(&mut input, "advancements removed")?;
        let mut removed = Vec::with_capacity(removed_count);
        for _ in 0..removed_count {
            // A set: an id named twice counts once, where it first came.
            let id = read_identifier_string(&mut input, "advancement removed")?;
            if !removed.contains(&id) {
                removed.push(id);
            }
        }
        let progress_count = read_count(&mut input, "advancements progress")?;
        let mut progress = Vec::with_capacity(progress_count);
        for _ in 0..progress_count {
            // A map: an id named twice takes the last progress.
            let id = read_identifier_string(&mut input, "advancement progress id")?;
            let criteria = read_progress(&mut input)?;
            match progress.iter_mut().find(|(known, _)| *known == id) {
                Some((_, held)) => *held = criteria,
                None => progress.push((id, criteria)),
            }
        }
        let show_advancements = read_bool(&mut input, "show advancements")?;
        Ok(Self {
            reset,
            added,
            removed,
            progress,
            show_advancements,
        })
    }
}

/// `ClientboundSelectAdvancementsTabPacket`: the tab the server opens, or none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectAdvancementsTab(pub Option<String>);

impl Decode for SelectAdvancementsTab {
    fn decode(payload: &[u8]) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let tab = read_bool(&mut input, "advancements tab")?
            .then(|| read_identifier_string(&mut input, "advancements tab id"))
            .transpose()?;
        Ok(Self(tab))
    }
}
