use super::*;

/// `ItemCost.STREAM_CODEC`: an item, a count and the components the payment must carry.
pub(super) fn parse_item_cost(
    input: &mut &[u8],
) -> Result<photon_game::menus::merchant::ItemCost, ConnectionError> {
    const NAME: &str = "merchant item cost";
    let item = read_varint(input, NAME)?;
    let count = read_varint(input, NAME)?;
    let mut cursor = photon_game::item_components::Cursor { data: input };
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
    Ok(photon_game::menus::merchant::ItemCost {
        item,
        count,
        components: expected,
    })
}

/// `ClientboundMerchantOffersPacket`: the container, the offers, and the merchant's level,
/// experience and what its screen shows.
pub fn parse_merchant_offers_packet(
    input: &[u8],
) -> Result<(i32, photon_game::menus::merchant::MerchantState), ConnectionError> {
    use photon_game::menus::merchant::{MerchantOffer, MerchantState, Trader};
    const NAME: &str = "merchant offers";
    let mut input = input;
    let input = &mut input;
    let container_id = read_varint(input, NAME)?;
    let count = read_varint(input, NAME)?;
    let mut offers = Vec::new();
    for _ in 0..count.max(0) {
        let cost_a = parse_item_cost(input)?;
        let result = parse_optional_item_stack(input, NAME)?;
        let result = photon_world::ClientItemStackState::from_wire(
            result.item_count.unwrap_or(0),
            result.item_registry_id.unwrap_or(0),
            result.positive_component_type_ids,
            result.negative_component_type_ids,
        )
        .with_patch_bytes(result.patch_bytes);
        let cost_b = if read_bool(input, NAME)? {
            Some(parse_item_cost(input)?)
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
        offers.push(MerchantOffer {
            cost_a,
            cost_b,
            result,
            uses: if out_of_stock { max_uses } else { uses },
            max_uses,
            xp,
            special_price_diff,
            demand,
            price_multiplier,
        });
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

wrapped_packet!(
    /// `ClientboundMerchantOffersPacket`: the container, and what the trader offers.
    MerchantOffers((i32, photon_game::menus::merchant::MerchantState)) = parse_merchant_offers_packet
);
