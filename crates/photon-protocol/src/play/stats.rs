use super::*;

pub use photon_data::stat::StatKey;

/// `ClientboundAwardStatsPacket`: the statistics the server counts for the player, in the order
/// they came (a statistic named twice takes the last number). Each key's value is the canonical id
/// of the block, item, entity type or custom statistic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwardStats(pub Vec<(StatKey, i32)>);

impl Decode for AwardStats {
    fn decode(
        payload: &[u8],
        registries: &photon_data::version_registries::VersionRegistries,
    ) -> Result<Self, ConnectionError> {
        let mut input = payload;
        let count = read_varint(&mut input, "award stats count")?;
        let count =
            usize::try_from(count).map_err(|_| ConnectionError::Codec("award stats count"))?;
        let mut stats = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            let stat_type = read_varint(&mut input, "award stats type")?;
            let value = read_varint(&mut input, "award stats value")?;
            let amount = read_varint(&mut input, "award stats amount")?;
            if photon_data::stat::type_name(stat_type).is_none() {
                return Err(ConnectionError::Codec("award stats type id"));
            }
            // The value is an id of the registry the type counts, in the version's numbering; an
            // id that registry does not have is an error, as the reference's registry codec says.
            let value = registries
                .stat_value_to_canonical(stat_type, value)
                .ok_or(ConnectionError::Codec("award stats value id"))?;
            stats.push((StatKey { stat_type, value }, amount));
        }
        if !input.is_empty() {
            return Err(ConnectionError::Codec("award stats trailing payload"));
        }
        Ok(Self(stats))
    }
}
