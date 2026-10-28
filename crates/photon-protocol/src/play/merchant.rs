use super::*;
use photon_data::item::registry::AIR_ITEM;

/// `ItemCost.STREAM_CODEC`: an item, a count and the components the payment must carry.
pub(super) fn parse_item_cost(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_data::merchant::ItemCost, ConnectionError> {
    const NAME: &str = "merchant item cost";
    let item = WireItemId::read(input, NAME)?.canonical(registries);
    let count = read_varint(input, NAME)?;
    let mut cursor = photon_data::item_components::Cursor { data: input };
    let components = cursor.count().map_err(|_| ConnectionError::Codec(NAME))?;
    let mut expected = Vec::new();
    for _ in 0..components {
        let id = cursor.varint().map_err(|_| ConnectionError::Codec(NAME))?;
        let start = cursor.data;
        cursor
            .component(id)
            .map_err(|_| ConnectionError::Codec(NAME))?;
        let length = start.len() - cursor.data.len();
        expected.push((id, start[..length].to_vec()));
    }
    *input = cursor.data;
    Ok(photon_data::merchant::ItemCost {
        item,
        count,
        components: expected,
    })
}

/// One offer: what it costs, what it gives, and how much it was used.
fn parse_merchant_offer(
    input: &mut &[u8],
    registries: &VersionRegistries,
) -> Result<photon_data::merchant::MerchantOffer, ConnectionError> {
    const NAME: &str = "merchant offers";
    let cost_a = parse_item_cost(input, registries)?;
    let result = parse_optional_item_stack(input, NAME, registries)?;
    let result = photon_data::item_stack::ClientItemStackState::from_wire(
        result.item_count.unwrap_or(0),
        result.item_registry_id.unwrap_or(AIR_ITEM),
        result.positive_component_type_ids,
        result.negative_component_type_ids,
    )
    .with_patch_bytes(result.patch_bytes);
    let cost_b = if read_bool(input, NAME)? {
        Some(parse_item_cost(input, registries)?)
    } else {
        None
    };
    let out_of_stock = read_bool(input, NAME)?;
    let uses = read_i32(input, NAME)?;
    let max_uses = read_i32(input, NAME)?;
    let xp = read_i32(input, NAME)?;
    let special_price_diff = read_i32(input, NAME)?;
    let price_multiplier = f32::from_bits(read_i32(input, NAME)? as u32);
    let demand = read_i32(input, NAME)?;
    Ok(photon_data::merchant::MerchantOffer {
        cost_a,
        cost_b,
        result,
        uses: if out_of_stock { max_uses } else { uses },
        max_uses,
        xp,
        special_price_diff,
        demand,
        price_multiplier,
    })
}

/// `ClientboundMerchantOffersPacket`: the container, the offers, and the merchant's level,
/// experience and what its screen shows.
pub fn parse_merchant_offers_packet(
    input: &[u8],
    registries: &VersionRegistries,
) -> Result<(i32, photon_data::merchant::MerchantState), ConnectionError> {
    use photon_data::merchant::{MerchantState, Trader};
    const NAME: &str = "merchant offers";
    let mut input = input;
    let input = &mut input;
    let container_id = read_varint(input, NAME)?;
    let count = read_varint(input, NAME)?;
    let mut offers = Vec::new();
    for _ in 0..count.max(0) {
        offers.push(parse_merchant_offer(input, registries)?);
    }
    let level = read_varint(input, NAME)?;
    let xp = read_varint(input, NAME)?;
    let show_progress = read_bool(input, NAME)?;
    let can_restock = read_bool(input, NAME)?;
    Ok((
        container_id,
        MerchantState {
            offers,
            trader: Trader {
                level,
                xp,
                show_progress,
                can_restock,
            },
        },
    ))
}

wrapped_wire_packet!(
    /// `ClientboundMerchantOffersPacket`: the container, and what the trader offers.
    MerchantOffers((i32, photon_data::merchant::MerchantState)) = parse_merchant_offers_packet
);
