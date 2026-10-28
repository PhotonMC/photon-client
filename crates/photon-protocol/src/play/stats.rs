use super::*;

pub use photon_data::stat::StatKey;

/// `ClientboundAwardStatsPacket`: the statistics the server counts for the player, in the order
/// they came (a statistic named twice takes the last number).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AwardStats(pub Vec<(StatKey, i32)>);

impl Decode for AwardStats {
    fn decode(
        payload: &[u8],
        _registries: &photon_data::version_registries::VersionRegistries,
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
            stats.push((StatKey { stat_type, value }, amount));
        }
        if !input.is_empty() {
            return Err(ConnectionError::Codec("award stats trailing payload"));
        }
        Ok(Self(stats))
    }
}
