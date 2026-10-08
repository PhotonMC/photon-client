use super::*;

/// Apply the non-passenger LocalPlayer.canStartSprinting predicate.
pub fn can_start_local_player_sprinting(state: LocalPlayerSprintGateInput) -> bool {
    !state.sprinting
        && state.forward_impulse > 1.0e-5_f32
        && !state.mobility_restricted
        && (state.enough_food || state.may_fly)
        && (state.flying || !state.shallow_water)
        && !state.slow_due_to_using_item
        && (!state.fall_flying || state.underwater)
        && (!state.moving_slowly || state.underwater)
}

/// Advance Java LocalPlayer's non-passenger sprint trigger window.
pub fn advance_local_player_sprint_trigger(
    state: LocalPlayerSprintTriggerInput,
) -> LocalPlayerSprintTriggerResult {
    let mut sprint_trigger_time = state.sprint_trigger_time;
    if sprint_trigger_time > 0 {
        sprint_trigger_time -= 1;
    }

    if state.previous_input.shift
        || state.conditions.slow_due_to_using_item
        || state.current_input.backward
    {
        sprint_trigger_time = 0;
    }

    let can_start = can_start_local_player_sprinting(LocalPlayerSprintGateInput {
        sprinting: state.sprinting,
        forward_impulse: state.current_move_vector.y,
        mobility_restricted: state.conditions.mobility_restricted,
        enough_food: state.conditions.enough_food,
        may_fly: state.conditions.may_fly,
        shallow_water: state.conditions.shallow_water,
        flying: state.conditions.flying,
        slow_due_to_using_item: state.conditions.slow_due_to_using_item,
        fall_flying: state.conditions.fall_flying,
        underwater: state.conditions.underwater,
        moving_slowly: state.conditions.moving_slowly,
    });
    let mut sprinting = state.sprinting;
    if can_start {
        if state.previous_move_vector.y.partial_cmp(&1.0e-5_f32) != Some(Ordering::Greater) {
            if sprint_trigger_time > 0 {
                sprinting = true;
            } else {
                sprint_trigger_time = state.sprint_window;
            }
        }

        if state.current_input.sprint {
            sprinting = true;
        }
    }

    LocalPlayerSprintTriggerResult {
        sprinting,
        sprint_trigger_time,
    }
}

/// The non-passenger conditions read by LocalPlayer.canStartSprinting.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintGateInput {
    pub sprinting: bool,
    pub forward_impulse: f32,
    pub mobility_restricted: bool,
    pub enough_food: bool,
    pub may_fly: bool,
    pub shallow_water: bool,
    pub flying: bool,
    pub slow_due_to_using_item: bool,
    pub fall_flying: bool,
    pub underwater: bool,
    pub moving_slowly: bool,
}

/// The non-passenger state used by LocalPlayer's sprint double-tap window.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintTriggerConditions {
    pub mobility_restricted: bool,
    pub enough_food: bool,
    pub may_fly: bool,
    pub shallow_water: bool,
    pub flying: bool,
    pub slow_due_to_using_item: bool,
    pub fall_flying: bool,
    pub underwater: bool,
    pub moving_slowly: bool,
}

/// The previous and current input snapshots consumed by one LocalPlayer tick.
#[derive(Debug, Clone, Copy)]
pub struct LocalPlayerSprintTriggerInput {
    pub sprinting: bool,
    pub sprint_trigger_time: i32,
    pub sprint_window: i32,
    pub previous_input: Input,
    pub previous_move_vector: Vec2,
    pub current_input: Input,
    pub current_move_vector: Vec2,
    pub conditions: LocalPlayerSprintTriggerConditions,
}

/// The local sprint state after one LocalPlayer sprint-trigger step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalPlayerSprintTriggerResult {
    pub sprinting: bool,
    pub sprint_trigger_time: i32,
}
