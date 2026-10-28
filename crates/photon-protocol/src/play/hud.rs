use super::*;

/// `ClientboundBossEventPacket(RegistryFriendlyByteBuf)`.
pub(super) fn parse_boss_event(input: &mut &[u8]) -> Result<HudEvent, ConnectionError> {
    let id = wire::take::<16>(input)
        .map(u128::from_be_bytes)
        .ok_or(ConnectionError::Codec("boss event id"))?;
    let operation = match read_varint(input, "boss event operation")? {
        0 => BossOperation::Add {
            name: read_chat_component(input, "boss bar name")?,
            progress: read_f32(input, "boss bar progress")?,
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
            flags: read_byte(input, "boss bar flags")?,
        },
        1 => BossOperation::Remove,
        2 => BossOperation::Progress(read_f32(input, "boss bar progress")?),
        3 => BossOperation::Name(read_chat_component(input, "boss bar name")?),
        4 => BossOperation::Style {
            color: boss_ordinal(input, 7, "boss bar color")?,
            overlay: boss_ordinal(input, 5, "boss bar overlay")?,
        },
        5 => BossOperation::Properties(read_byte(input, "boss bar flags")?),
        _ => return Err(ConnectionError::Codec("boss event operation")),
    };
    Ok(HudEvent::Boss { id, operation })
}

/// `ClientboundTrackedWaypointPacket.STREAM_CODEC`: the operation, then the
/// waypoint (`TrackedWaypoint.read`).
pub fn parse_waypoint_packet(
    input: &[u8],
) -> Result<
    (
        photon_core::waypoint::WaypointOperation,
        photon_core::waypoint::TrackedWaypoint,
    ),
    ConnectionError,
> {
    use photon_core::waypoint::{
        TrackedWaypoint, WaypointIcon, WaypointId, WaypointKind, WaypointOperation,
    };
    let mut input = input;
    let input = &mut input;
    // `ByIdMap.continuous` with `WRAP`: an id past the end comes round.
    let operation = match read_varint(input, "waypoint operation")?.rem_euclid(3) {
        0 => WaypointOperation::Track,
        1 => WaypointOperation::Untrack,
        _ => WaypointOperation::Update,
    };
    let id = if read_bool(input, "waypoint id side")? {
        WaypointId::Uuid(
            wire::take::<16>(input)
                .map(u128::from_be_bytes)
                .ok_or(ConnectionError::Codec("waypoint uuid"))?,
        )
    } else {
        WaypointId::Name(read_chat_string(input, 32767, "waypoint name")?)
    };
    let style = read_identifier_string(input, "waypoint style")?;
    let colour = if read_bool(input, "waypoint colour present")? {
        let red = i32::from(read_byte(input, "waypoint red")?);
        let green = i32::from(read_byte(input, "waypoint green")?);
        let blue = i32::from(read_byte(input, "waypoint blue")?);
        Some(red << 16 | green << 8 | blue)
    } else {
        None
    };
    // `readEnum`: the ordinal, which must exist.
    let kind = match read_varint(input, "waypoint type")? {
        0 => WaypointKind::Empty,
        1 => WaypointKind::Position([
            read_varint(input, "waypoint x")?,
            read_varint(input, "waypoint y")?,
            read_varint(input, "waypoint z")?,
        ]),
        2 => WaypointKind::Chunk([
            read_varint(input, "waypoint chunk x")?,
            read_varint(input, "waypoint chunk z")?,
        ]),
        3 => WaypointKind::Azimuth(read_f32(input, "waypoint angle")?),
        _ => return Err(ConnectionError::Codec("waypoint type")),
    };
    let icon = WaypointIcon { style, colour };
    Ok((operation, TrackedWaypoint { id, icon, kind }))
}

/// An enum on the wire: its ordinal as a varint, below `count`.
pub(super) fn boss_ordinal(
    input: &mut &[u8],
    count: i32,
    name: &'static str,
) -> Result<u8, ConnectionError> {
    let ordinal = read_varint(input, name)?;
    if (0..count).contains(&ordinal) {
        Ok(ordinal as u8)
    } else {
        Err(ConnectionError::Codec(name))
    }
}

/// The title, action bar and tab list packets, by id.
pub fn parse_hud_packet(id: i32, input: &[u8]) -> Result<Option<HudEvent>, ConnectionError> {
    let mut input = input;
    let input = &mut input;
    Ok(Some(match id {
        PLAY_CLIENTBOUND_SET_TITLE_TEXT_PACKET_ID => {
            HudEvent::Title(read_chat_component(input, "title")?)
        }
        PLAY_CLIENTBOUND_SET_SUBTITLE_TEXT_PACKET_ID => {
            HudEvent::Subtitle(read_chat_component(input, "subtitle")?)
        }
        PLAY_CLIENTBOUND_SET_ACTION_BAR_TEXT_PACKET_ID => {
            HudEvent::ActionBar(read_chat_component(input, "action bar")?)
        }
        PLAY_CLIENTBOUND_SET_TITLES_ANIMATION_PACKET_ID => HudEvent::Times {
            fade_in: read_i32(input, "title fade in")?,
            stay: read_i32(input, "title stay")?,
            fade_out: read_i32(input, "title fade out")?,
        },
        PLAY_CLIENTBOUND_CLEAR_TITLES_PACKET_ID => HudEvent::Clear {
            reset_times: read_bool(input, "clear titles reset")?,
        },
        PLAY_CLIENTBOUND_TAB_LIST_PACKET_ID => HudEvent::TabList {
            header: read_chat_component(input, "tab list header")?,
            footer: read_chat_component(input, "tab list footer")?,
        },
        PLAY_CLIENTBOUND_BOSS_EVENT_PACKET_ID => parse_boss_event(input)?,
        _ => return Ok(None),
    }))
}

/// What the title packets, the action bar and the tab list ask the HUD to show.
#[derive(Debug, Clone, PartialEq)]
pub enum HudEvent {
    Title(photon_core::text::Component),
    Subtitle(photon_core::text::Component),
    /// `ClientboundSetTitlesAnimationPacket`, in ticks.
    Times {
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    },
    Clear {
        reset_times: bool,
    },
    ActionBar(photon_core::text::Component),
    TabList {
        header: photon_core::text::Component,
        footer: photon_core::text::Component,
    },
    /// `ClientboundBossEventPacket`.
    Boss {
        id: u128,
        operation: BossOperation,
    },
}

/// What a boss event packet does to the bar `id`.
#[derive(Debug, Clone, PartialEq)]
pub enum BossOperation {
    Add {
        name: photon_core::text::Component,
        progress: f32,
        /// `BossBarColor` ordinal: pink, blue, red, green, yellow, purple, white.
        color: u8,
        /// `BossBarOverlay` ordinal: progress, then notched 6, 10, 12, 20.
        overlay: u8,
        flags: u8,
    },
    Remove,
    Progress(f32),
    Name(photon_core::text::Component),
    Style {
        color: u8,
        overlay: u8,
    },
    Properties(u8),
}

wrapped_packet!(
    /// `ClientboundTrackedWaypointPacket`: the operation and the waypoint it is done to.
    Waypoint(
        (
            photon_core::waypoint::WaypointOperation,
            photon_core::waypoint::TrackedWaypoint,
        )
    ) = parse_waypoint_packet,
    OnError::Warn("waypoint")
);
