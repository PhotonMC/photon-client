use super::*;

mod direction;
use direction::*;
mod mth;
use mth::*;
mod positions;
use positions::*;
mod vectors;
use vectors::*;
mod player;
use player::*;
mod options;
use options::*;

pub(super) fn math_result(request: &Value) -> Result<Value, String> {
    let operation = string(request, "operation")?;
    match operation {
        "client_world_section_lookup" => client_world_section_lookup_result(request),
        "client_world_chunk_cache" => client_world_chunk_cache_result(request),
        "client_world_block_update" => client_world_block_update_result(request),
        "client_world_light" => client_world_light_result(request),
        "client_world_player_attributes" => client_world_player_attributes_result(request),
        "client_world_player_movement_speed" => client_world_player_movement_speed_result(request),
        "client_world_player_attribute_water" => {
            client_world_player_attribute_water_result(request)
        }
        "client_world_player_sprint_attribute" => {
            client_world_player_sprint_attribute_result(request)
        }
        "client_world_player_sneaking_input" => client_world_player_sneaking_input_result(request),
        "client_world_player_keyboard_input" => client_world_player_keyboard_input_result(request),
        "client_world_player_tick" => client_world_player_tick_result(request),
        "client_world_entity_player_head_rotation" => {
            client_world_entity_player_head_rotation_result(request)
        }
        "client_world_entity_snowball_tick" => client_world_entity_snowball_tick_result(request),
        "client_world_entity_arrow_tick" => client_world_entity_arrow_tick_result(request),
        "client_world_entity_experience_orb_tick" => {
            client_world_entity_experience_orb_tick_result(request)
        }
        "client_world_entity_primed_tnt_tick" => {
            client_world_entity_primed_tnt_tick_result(request)
        }
        "client_world_entity_falling_block_tick" => {
            client_world_entity_falling_block_tick_result(request)
        }
        "client_world_entity_firework_tick" => client_world_entity_firework_tick_result(request),
        "client_world_entity_boat_air_tick" => client_world_entity_boat_air_tick_result(request),
        "client_world_entity_boat_remote_tick" => {
            client_world_entity_boat_remote_tick_result(request)
        }
        "client_world_entity_ticking_eligibility" => {
            client_world_entity_ticking_eligibility_result(request)
        }
        "client_world_entity_transform_history" => {
            client_world_entity_transform_history_result(request)
        }
        "client_world_entity_player_interpolation" => {
            client_world_entity_player_interpolation_result(request)
        }
        "client_world_entity_player_snap" => client_world_entity_player_snap_result(request),
        "client_world_entity_player_motion" => client_world_entity_player_motion_result(request),
        "client_world_entity_teleport" => client_world_entity_teleport_result(request),
        "client_world_entity_passengers" => client_world_entity_passengers_result(request),
        "client_world_entity_passenger_tick" => client_world_entity_passenger_tick_result(request),
        "client_world_entity_passenger_position" => {
            client_world_entity_passenger_position_result(request)
        }
        "client_world_entity_passenger_player_position" => {
            client_world_entity_passenger_player_position_result(request)
        }
        "client_world_entity_passenger_boat_family_position" => {
            client_world_entity_passenger_boat_family_position_result(request)
        }
        "client_world_entity_passenger_rotation" => {
            client_world_entity_passenger_rotation_result(request)
        }
        "client_world_entity_passenger_boat_turn_exemption" => {
            client_world_entity_passenger_boat_turn_exemption_result(request)
        }
        "client_world_entity_passenger_animal_rotation" => {
            client_world_entity_passenger_animal_rotation_result(request)
        }
        "client_world_entity_passenger_animal_position" => {
            client_world_entity_passenger_animal_position_result(request)
        }
        "client_world_entity_passenger_turn" => client_world_entity_passenger_turn_result(request),
        "client_world_player_sprint_trigger" => client_world_player_sprint_trigger_result(request),
        "client_world_entity_shared_flags" => client_world_entity_shared_flags_result(request),
        "client_world_entity_air_supply" => client_world_entity_air_supply_result(request),
        "client_world_entity_pose" => client_world_entity_pose_result(request),
        "client_world_entity_ticks_frozen" => client_world_entity_ticks_frozen_result(request),
        "client_world_entity_custom_name" => client_world_entity_custom_name_result(request),
        "client_world_entity_living_flags" => client_world_entity_living_flags_result(request),
        "client_world_entity_health" => client_world_entity_health_result(request),
        "client_world_entity_mob_flags" => client_world_entity_mob_flags_result(request),
        "client_world_entity_arrow_count" => client_world_entity_arrow_count_result(request),
        "client_world_entity_effect_ambience" => {
            client_world_entity_effect_ambience_result(request)
        }
        "client_world_entity_effect_particles" => {
            client_world_entity_effect_particles_result(request)
        }
        "client_world_entity_sleeping_position" => {
            client_world_entity_sleeping_position_result(request)
        }
        "client_world_entity_stinger_count" => client_world_entity_stinger_count_result(request),
        "random" => random_result(request),
        "random_fork" => random_fork_result(request),
        "random_positional" => random_positional_result(request),
        "aabb_builder" => aabb_builder_result(request),
        "aabb" => aabb_result(request),
        "shape" => shape_result(request),
        "blockpos" => blockpos_result(request),
        "blockpos_packed_helpers" => blockpos_packed_helpers_case(request),
        "blockpos_relative_helpers" => blockpos_relative_helpers_case(request),
        "blockpos_vector_helpers" => blockpos_vector_helpers_case(request),
        "blockpos_geometry_helpers" => blockpos_geometry_helpers_case(request),
        "blockpos_distance_helpers" => blockpos_distance_helpers_case(request),
        "blockpos_integer_helpers" => blockpos_integer_helpers_case(request),
        "blockpos_clamp_location" => blockpos_clamp_location_case(request),
        "blockpos_between_closed" => blockpos_between_closed_case(request),
        "blockpos_between_closed_aabb" => blockpos_between_closed_aabb_case(request),
        "blockpos_between_closed_streams" => blockpos_between_closed_streams_case(request),
        "blockpos_coordinate_helpers" => blockpos_coordinate_helpers_case(request),
        "blockpos_between_corners_direction" => blockpos_between_corners_direction_case(request),
        "blockpos_breadth_first_traversal" => blockpos_breadth_first_traversal_case(request),
        "blockpos_mutable_helpers" => blockpos_mutable_helpers_case(request),
        "axis_cycle_helpers" => axis_cycle_helpers_case(request),
        "blockpos_within_manhattan" => blockpos_within_manhattan_case(request),
        "blockpos_spiral_around" => blockpos_spiral_around_case(request),
        "blockpos_find_closest_match" => blockpos_find_closest_match_case(request),
        "blockpos_neighbor_column" => blockpos_neighbor_column_case(request),
        "blockpos_square_out_south_east" => blockpos_square_out_south_east_case(request),
        "blockpos_random_ranges" => blockpos_random_ranges_case(request),
        "section_relative" => section_relative_result(request),
        "vec3_codec" => vec3_codec_case(request),
        "chunkpos_codec" => chunkpos_codec_case(request),
        "chunkpos_block_coordinates" => chunkpos_block_coordinates_case(request),
        "chunkpos_region_coordinates" => chunkpos_region_coordinates_case(request),
        "chunkpos_block_positions" => chunkpos_block_positions_case(request),
        "chunkpos_representation" => chunkpos_representation_case(request),
        "chunkpos_from_blockpos" => chunkpos_from_blockpos_case(request),
        "chunkpos_distances" => chunkpos_distances_case(request),
        "chunkpos_from_section_node" => chunkpos_from_section_node_case(request),
        "chunkpos_range_closed" => chunkpos_range_closed_case(request),
        "chunkpos_constant" => chunkpos_constant_case(request),
        "chunkpos_packed_accessors" => chunkpos_packed_accessors_case(request),
        "sectionpos_packed_accessors" => sectionpos_packed_accessors_case(request),
        "sectionpos_block_coordinates" => sectionpos_block_coordinates_case(request),
        "sectionpos_relative_to_block" => sectionpos_relative_to_block_case(request),
        "sectionpos_spatial" => sectionpos_spatial_case(request),
        "sectionpos_packed_transformations" => sectionpos_packed_transformations_case(request),
        "sectionpos_as_long" => sectionpos_as_long_case(request),
        "sectionpos_offset" => sectionpos_offset_case(request),
        "sectionpos_ranges" => sectionpos_ranges_case(request),
        "sectionpos_blocks_inside" => sectionpos_blocks_inside_case(request),
        "sectionpos_around_and_at_block_pos" => sectionpos_around_and_at_block_pos_case(request),
        "sectionpos_double_coordinates" => sectionpos_double_coordinates_case(request),
        "sectionpos_of_overloads" => sectionpos_of_overloads_case(request),
        "chunkpos_validity" => chunkpos_validity_case(request),
        "mth_floating_primitives" => mth_floating_primitives_case(request),
        "mth_constants" => mth_constants_case(request),
        "mth_power_helpers" => mth_power_helpers_case(request),
        "mth_frac" => mth_frac_case(request),
        "mth_create_insecure_uuid" => mth_create_insecure_uuid_case(request),
        "mth_murmur_hash3_mixer" => mth_murmur_hash3_mixer_case(request),
        "mth_binary_search" => mth_binary_search_case(request),
        "mth_ray_intersects_aabb" => mth_ray_intersects_aabb_case(request),
        "mth_fast_inv_sqrt" => mth_fast_inv_sqrt_case(request),
        "mth_fast_inv_cube_root" => mth_fast_inv_cube_root_case(request),
        "mth_hsv_to_color" => mth_hsv_to_color_case(request),
        "mth_angle_wrap_difference" => mth_angle_wrap_difference_case(request),
        "mth_angle_approach" => mth_angle_approach_case(request),
        "mth_get_int" => mth_get_int_case(request),
        "mth_get_seed" => mth_get_seed_case(request),
        "mth_atan2" => mth_atan2_case(request),
        "mth_inv_sqrt" => mth_inv_sqrt_case(request),
        "mth_integer_helpers" => mth_integer_helpers_case(request),
        "mth_integer_basics" => mth_integer_basics_case(request),
        "mth_angle_helpers" => mth_angle_helpers_case(request),
        "mth_rounding_helpers" => mth_rounding_helpers_case(request),
        "mth_positive_modulo" => mth_positive_modulo_case(request),
        "mth_abs_max" => mth_abs_max_case(request),
        "mth_equal" => mth_equal_case(request),
        "mth_clamped_lerp" => mth_clamped_lerp_case(request),
        "mth_clamped_map_f64" => mth_clamped_map_f64_case(request),
        "mth_clamped_map_f32" => mth_clamped_map_f32_case(request),
        "mth_map_f64" => mth_map_f64_case(request),
        "mth_map_f32" => mth_map_f32_case(request),
        "mth_wobble_f64" => mth_wobble_f64_case(request),
        "mth_interpolation" => mth_interpolation_case(request),
        "mth_next_random" => mth_next_random_case(request),
        "mth_random_helpers" => mth_random_helpers_case(request),
        "mth_length" => mth_length_case(request),
        "mth_quantize" => mth_quantize_case(request),
        "mth_out_from_origin" => mth_out_from_origin_case(request),
        "mth_mul_and_truncate" => mth_mul_and_truncate_case(request),
        "mth_rotation_around_axis" => mth_rotation_around_axis_case(request),
        "mth_inverse_lerp" => mth_inverse_lerp_case(request),
        "mth_lerp" => mth_lerp_case(request),
        "mth_sign" => mth_sign_case(request),
        "mth_rot_lerp" => mth_rot_lerp_case(request),
        "mth_rot_lerp_rad" => mth_rot_lerp_rad_case(request),
        "mth_triangle_wave" => mth_triangle_wave_case(request),
        "mth_square_f32" => mth_square_f32_case(request),
        "mth_cube_f32" => mth_cube_f32_case(request),
        "mth_square_f64" => mth_square_f64_case(request),
        "mth_square_i32" => mth_square_i32_case(request),
        "mth_square_i64" => mth_square_i64_case(request),
        "vec2" => vec2_case(request),
        "keyboard_input" => keyboard_input_case(request),
        "local_player_input" => local_player_input_case(request),
        "local_player_apply_input" => local_player_apply_input_case(request),
        "local_player_can_start_sprinting" => local_player_can_start_sprinting_case(request),
        "local_player_fluid_falling_adjustment" => {
            local_player_fluid_falling_adjustment_case(request)
        }
        "local_player_climbable_movement" => local_player_climbable_movement_case(request),
        "local_player_friction_influenced_speed" => {
            local_player_friction_influenced_speed_case(request)
        }
        "local_player_modified_friction" => local_player_modified_friction_case(request),
        "local_player_travel_flying" => local_player_travel_flying_case(request),
        "local_player_travel_in_water" => local_player_travel_in_water_case(request),
        "local_player_travel_in_lava" => local_player_travel_in_lava_case(request),
        "local_player_handle_relative_friction" => {
            local_player_handle_relative_friction_case(request)
        }
        "local_player_travel_in_air" => local_player_travel_in_air_case(request),
        "local_player_update_fall_flying_movement" => {
            local_player_update_fall_flying_movement_case(request)
        }
        "local_player_travel_dispatch" => local_player_travel_dispatch_case(request),
        "character_event" => character_event_result(request),
        "preedit_event" => preedit_event_result(request),
        "option_values" => option_values_result(request),
        "sliderable_enum" => sliderable_enum_result(request),
        "cycleable_enum" => cycleable_enum_result(request),
        "alt_enum" => alt_enum_result(request),
        "lazy_enum" => lazy_enum_result(request),
        "clamping_lazy_max_int_range" => clamping_lazy_max_int_range_result(request),
        "mapped_int_range" => mapped_int_range_result(request),
        "mapped_unit_double" => mapped_unit_double_result(request),
        "scroll_wheel" => scroll_wheel_result(request),
        "smooth_double" => smooth_double_result(request),
        "input_with_modifiers" => input_event_result(request),
        "key_mapping" => key_mapping_result(request),
        "toggle_key_mapping" => toggle_key_mapping_result(request),
        "direction" => direction_case(request),
        "direction_axial_rotation" => direction_axial_rotation_case(request),
        "direction_y_rotation" => direction_y_rotation_case(request),
        "direction_data" => direction_data_case(request),
        "direction_y_rot" => direction_y_rot_case(request),
        "direction_axis" => direction_axis_case(request),
        "direction_get" => direction_get_case(request),
        "direction_unit_vectors" => direction_unit_vectors_case(request),
        "direction_step" => direction_step_case(request),
        "direction_axis_orientation" => direction_axis_orientation_case(request),
        "direction_axis_directions" => direction_axis_directions_case(request),
        "direction_axis_direction_values" => direction_axis_direction_values_case(request),
        "direction_axis_choose" => direction_axis_choose_case(request),
        "direction_axis_test" => direction_axis_test_case(request),
        "direction_axis_plane" => direction_axis_plane_case(request),
        "direction_plane_test" => direction_plane_test_case(request),
        "direction_plane_order" => direction_plane_order_case(request),
        "direction_random" => direction_random_case(request),
        "direction_plane_shuffle" => direction_plane_shuffle_case(request),
        "direction_all_shuffle" => direction_all_shuffle_case(request),
        "direction_stream" => direction_stream_case(),
        "direction_to_y_rot" => direction_to_y_rot_case(request),
        "direction_get_y_rot" => direction_get_y_rot_case(request),
        "direction_rotation" => direction_rotation_case(request),
        "direction_name_lookup" => direction_name_lookup_case(request),
        "direction_enum_text" => direction_enum_text_case(request),
        "direction_enum_name" => direction_enum_name_case(request),
        "direction_approximate_nearest" => direction_approximate_nearest_case(request),
        "direction_approximate_nearest_overloads" => {
            direction_approximate_nearest_overloads_case(request)
        }
        "direction_nearest" => direction_nearest_case(request),
        "direction_axis_step_order" => direction_axis_step_order_case(request),
        "direction_is_facing_angle" => direction_is_facing_angle_case(request),
        "coord" => coord_case(request),
        "vec3" => vec3_case(request),
        "vec3_dot" => vec3_dot_case(request),
        "vec3_distance_to_sqr" => vec3_distance_to_sqr_case(request),
        "vec3_distance_to" => vec3_distance_to_case(request),
        "vec3_distance_to_sqr_components" => vec3_distance_to_sqr_components_case(request),
        "vec3_horizontal_distances" => vec3_horizontal_distances_case(request),
        "vec3_closer_than" => vec3_closer_than_case(request),
        "vec3_offset_random" => vec3_offset_random_case(request),
        "vec3_lerp" => vec3_lerp_case(request),
        "vec3_rotate_clockwise_90" => vec3_rotate_clockwise_90_case(request),
        "vec3_is_finite" => vec3_is_finite_case(request),
        "vec3_get" => vec3_get_case(request),
        "vec3_with" => vec3_with_case(request),
        "vec3_relative" => vec3_relative_case(request),
        "vec3_align" => vec3_align_case(request),
        "vec3_projected_on" => vec3_projected_on_case(request),
        "vec3_to_vector3f" => vec3_to_vector3f_case(request),
        "vec3_coordinates" => vec3_coordinates_case(request),
        "vec3_equality" => vec3_equality_case(request),
        "vec3_to_string" => vec3_to_string_case(request),
        "vec3_rotate" => vec3_rotate_case(request),
        "vec3_direction_from_rotation" => vec3_direction_from_rotation_case(request),
        "vec3_local_coordinates" => vec3_local_coordinates_case(request),
        "vec3_rotation" => vec3_rotation_case(request),
        "vec2_rotate" => vec2_rotate_case(request),
        "vec2_dot" => vec2_dot_case(request),
        "vec2_add_vec2" => vec2_add_vec2_case(request),
        "vec2_add_scalar" => vec2_add_scalar_case(request),
        "vec2_distance_to_sqr" => vec2_distance_to_sqr_case(request),
        "vec2_negated" => vec2_negated_case(request),
        "vec2_equality" => vec2_equality_case(request),
        "vec2_constant" => vec2_constant_case(request),
        "vec3_constant" => vec3_constant_case(request),
        "mth_lerp_int" | "mth_lerp_discrete" => mth_lerp_steps_case(request, operation),
        "mth_smoothstep" | "mth_smoothstep_derivative" => {
            mth_smoothstep_pair_case(request, operation)
        }
        _ => scalar_case(request, operation),
    }
}
