use super::*;

pub(crate) fn parse_open_screen_packet(
    input: &[u8],
) -> Result<OpenScreenObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "open screen container id")?;
    let menu_type_id = read_varint(&mut input, "open screen menu type")?;
    let title_input = input;
    let title = photon_core::nbt::read_any_nbt(&mut input)
        .map_err(|_| ConnectionError::Codec("open screen title"))?;
    if title.is_none() {
        return Err(ConnectionError::Codec("open screen title"));
    }
    let title_length = title_input.len() - input.len();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("open screen trailing payload"));
    }
    Ok(OpenScreenObservation {
        container_id,
        menu_type_id,
        title_nbt: title_input[..title_length].to_vec(),
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_close_packet(
    input: &[u8],
) -> Result<ContainerCloseObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container close id")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container close trailing payload"));
    }
    Ok(ContainerCloseObservation {
        container_id,
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_set_data_packet(
    input: &[u8],
) -> Result<ContainerDataObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container data id")?;
    let id = read_i16(&mut input, "container data slot")?;
    let value = read_i16(&mut input, "container data value")?;
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container data trailing payload"));
    }
    Ok(ContainerDataObservation {
        container_id,
        id,
        value,
    })
}

/// Wire-level fields decoded from one clientbound open-screen packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenScreenObservation {
    pub container_id: i32,
    pub menu_type_id: i32,
    pub title_nbt: Vec<u8>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container-close packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerCloseObservation {
    pub container_id: i32,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container slot update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetSlotObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub slot: i16,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound cursor item update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorItemObservation {
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one direct local-player inventory update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayerInventoryObservation {
    pub slot: i32,
    pub item_count: Option<i32>,
    pub item_registry_id: Option<i32>,
    pub positive_component_type_ids: Vec<i32>,
    pub negative_component_type_ids: Vec<i32>,
    pub item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// Wire-level fields decoded from one clientbound container content update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerSetContentObservation {
    pub container_id: i32,
    pub state_id: i32,
    pub item_count: usize,
    pub items: Vec<Option<ClientItemStackState>>,
    pub hotbar_slot_zero_item_count: Option<i32>,
    pub hotbar_slot_zero_item_registry_id: Option<i32>,
    pub hotbar_slot_zero_positive_component_type_ids: Vec<i32>,
    pub hotbar_slot_zero_negative_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_item_count: Option<i32>,
    pub main_inventory_slot_zero_item_registry_id: Option<i32>,
    pub main_inventory_slot_zero_positive_component_type_ids: Vec<i32>,
    pub main_inventory_slot_zero_negative_component_type_ids: Vec<i32>,
    pub carried_item_count: Option<i32>,
    pub carried_item_registry_id: Option<i32>,
    pub carried_positive_component_type_ids: Vec<i32>,
    pub carried_negative_component_type_ids: Vec<i32>,
    pub carried_item: Option<ClientItemStackState>,
    pub applied_to_world: bool,
}

/// One clientbound container-data packet: a menu's `ContainerData` value
/// (a furnace's burn time, an enchanting table's cost).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerDataObservation {
    pub container_id: i32,
    pub id: i16,
    pub value: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedItemStackObservation {
    pub(super) item_count: Option<i32>,
    pub(super) item_registry_id: Option<i32>,
    pub(super) positive_component_type_ids: Vec<i32>,
    pub(super) negative_component_type_ids: Vec<i32>,
    pub(super) patch_bytes: Vec<u8>,
}

pub(super) fn parse_optional_item_stack(
    input: &mut &[u8],
    name: &'static str,
) -> Result<ParsedItemStackObservation, ConnectionError> {
    let item_count = read_varint(input, name)?;
    if item_count <= 0 {
        return Ok(ParsedItemStackObservation {
            item_count: None,
            item_registry_id: None,
            positive_component_type_ids: Vec::new(),
            negative_component_type_ids: Vec::new(),
            patch_bytes: Vec::new(),
        });
    }
    let item_registry_id = read_varint(input, name)?;
    let patch_start = *input;
    let patch = photon_game::item_components::read_patch(input)
        .map_err(|_| ConnectionError::Codec(name))?;
    let positive_component_type_ids = patch.added.iter().map(|(id, _)| *id).collect();
    let negative_component_type_ids = patch.removed;
    let patch_bytes = patch_start[..patch_start.len() - input.len()].to_vec();
    Ok(ParsedItemStackObservation {
        item_count: Some(item_count),
        item_registry_id: Some(item_registry_id),
        positive_component_type_ids,
        negative_component_type_ids,
        patch_bytes,
    })
}

/// An item stack as an entity's data holds it (`ItemStack.OPTIONAL_STREAM_CODEC`
/// at the start of `value`): none when the field is empty or cannot be read.
pub(crate) fn parse_entity_item_stack(value: &[u8]) -> Option<ClientItemStackState> {
    let mut input = value;
    parse_optional_item_stack(&mut input, "entity item stack")
        .ok()?
        .client_state()
}

pub(crate) fn parse_container_set_content_packet(
    input: &[u8],
) -> Result<ContainerSetContentObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container content container id")?;
    let state_id = read_varint(&mut input, "container content state id")?;
    let item_count = read_count(&mut input, "container content item count")?;
    let mut hotbar_slot_zero_item_count = None;
    let mut hotbar_slot_zero_item_registry_id = None;
    let mut hotbar_slot_zero_positive_component_type_ids = Vec::new();
    let mut hotbar_slot_zero_negative_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_item_count = None;
    let mut main_inventory_slot_zero_item_registry_id = None;
    let mut main_inventory_slot_zero_positive_component_type_ids = Vec::new();
    let mut main_inventory_slot_zero_negative_component_type_ids = Vec::new();
    let mut items = Vec::with_capacity(item_count);
    for slot in 0..item_count {
        let item = parse_optional_item_stack(&mut input, "container content item stack")?;
        let client_state = item.client_state();
        if slot == 36 {
            hotbar_slot_zero_item_count = item.item_count;
            hotbar_slot_zero_item_registry_id = item.item_registry_id;
            hotbar_slot_zero_positive_component_type_ids = item.positive_component_type_ids.clone();
            hotbar_slot_zero_negative_component_type_ids = item.negative_component_type_ids.clone();
        } else if slot == 9 {
            main_inventory_slot_zero_item_count = item.item_count;
            main_inventory_slot_zero_item_registry_id = item.item_registry_id;
            main_inventory_slot_zero_positive_component_type_ids =
                item.positive_component_type_ids.clone();
            main_inventory_slot_zero_negative_component_type_ids =
                item.negative_component_type_ids.clone();
        }
        items.push(client_state);
    }
    let carried_item = parse_optional_item_stack(&mut input, "container content carried item")?;
    let carried_client_state = carried_item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container content trailing payload"));
    }
    Ok(ContainerSetContentObservation {
        container_id,
        state_id,
        item_count,
        items,
        hotbar_slot_zero_item_count,
        hotbar_slot_zero_item_registry_id,
        hotbar_slot_zero_positive_component_type_ids,
        hotbar_slot_zero_negative_component_type_ids,
        main_inventory_slot_zero_item_count,
        main_inventory_slot_zero_item_registry_id,
        main_inventory_slot_zero_positive_component_type_ids,
        main_inventory_slot_zero_negative_component_type_ids,
        carried_item_count: carried_item.item_count,
        carried_item_registry_id: carried_item.item_registry_id,
        carried_positive_component_type_ids: carried_item.positive_component_type_ids,
        carried_negative_component_type_ids: carried_item.negative_component_type_ids,
        carried_item: carried_client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_container_set_slot_packet(
    input: &[u8],
) -> Result<ContainerSetSlotObservation, ConnectionError> {
    let mut input = input;
    let container_id = read_varint(&mut input, "container slot container id")?;
    let state_id = read_varint(&mut input, "container slot state id")?;
    let slot = read_i16(&mut input, "container slot index")?;
    let item = parse_optional_item_stack(&mut input, "container slot item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("container slot trailing payload"));
    }
    Ok(ContainerSetSlotObservation {
        container_id,
        state_id,
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_cursor_item_packet(
    input: &[u8],
) -> Result<CursorItemObservation, ConnectionError> {
    let mut input = input;
    let item = parse_optional_item_stack(&mut input, "cursor item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("cursor item trailing payload"));
    }
    Ok(CursorItemObservation {
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

pub(crate) fn parse_set_player_inventory_packet(
    input: &[u8],
) -> Result<SetPlayerInventoryObservation, ConnectionError> {
    let mut input = input;
    let slot = read_varint(&mut input, "player inventory slot")?;
    let item = parse_optional_item_stack(&mut input, "player inventory item stack")?;
    let client_state = item.client_state();
    if !input.is_empty() {
        return Err(ConnectionError::Codec("player inventory trailing payload"));
    }
    Ok(SetPlayerInventoryObservation {
        slot,
        item_count: item.item_count,
        item_registry_id: item.item_registry_id,
        positive_component_type_ids: item.positive_component_type_ids,
        negative_component_type_ids: item.negative_component_type_ids,
        item: client_state,
        applied_to_world: false,
    })
}

impl ParsedItemStackObservation {
    pub(super) fn client_state(&self) -> Option<ClientItemStackState> {
        Some(
            ClientItemStackState::from_wire(
                self.item_count?,
                self.item_registry_id?,
                self.positive_component_type_ids.clone(),
                self.negative_component_type_ids.clone(),
            )
            .with_patch_bytes(self.patch_bytes.clone()),
        )
    }
}
