use super::*;

mod packets;
use packets::*;
mod rotation;
use rotation::*;
mod boxes;
use boxes::*;
mod world_index;
use world_index::*;
mod world_collision;
use world_collision::*;
mod border;
use border::*;
mod dimensions;
use dimensions::*;
mod collision;
use collision::*;
mod queries;
use queries::*;

pub(super) fn shape_result(request: &Value) -> Result<Value, String> {
    match string(request, "kind")? {
        "player_abilities_packet" => player_abilities_packet_case(request),
        "player_game_mode_event" => player_game_mode_event_case(request),
        "single" => single_case(request),
        "join" => join_case(request),
        "join_many" => join_many_case(request),
        "single_encompassing" => single_encompassing_case(request),
        "optimize" => optimize_case(request),
        "edges" => edges_case(request),
        "join_coords" => join_coords_case(request),
        "boxes" => boxes_case(request),
        "create_aabb" => create_aabb_case(request),
        "create_coords" => create_coords_case(request),
        "infinity" => infinity_case(request),
        "constants" => constants_case(request),
        "axis_limits" => axis_limits_case(request),
        "axis_limits_at" => axis_limits_at_case(request),
        "equal" => equal_case(request),
        "object_equals" => object_equals_case(request),
        "object_equals_constants" => object_equals_constants_case(),
        "to_string" => to_string_case(request),
        "rotate" => rotate_case(request),
        "rotate_center" => rotate_center_case(request),
        "rotate_join" => rotate_join_case(request),
        "rotate_horizontal_axis" => rotate_horizontal_axis_case(request),
        "rotate_horizontal_axis_center" => rotate_horizontal_axis_center_case(request),
        "rotate_all_axis" => rotate_all_axis_case(request),
        "rotate_all_axis_center" => rotate_all_axis_center_case(request),
        "rotate_horizontal" => rotate_horizontal_case(request),
        "rotate_horizontal_initial" => rotate_horizontal_initial_case(request),
        "rotate_all" => rotate_all_case(request),
        "rotate_all_initial" => rotate_all_initial_case(request),
        "rotate_all_center" => rotate_all_center_case(request),
        "rotate_attach_face" => rotate_attach_face_case(request),
        "rotate_attach_face_initial" => rotate_attach_face_initial_case(request),
        "collide" => collide_case(request),
        "collide_many" => collide_many_case(request),
        "collide_movement" => collide_movement_case(request),
        "entity_movement_state" => entity_movement_state_case(request),
        "entity_movement_state_retention" => entity_movement_state_retention_case(request),
        "entity_collision_shapes" => entity_collision_shapes_case(request),
        "client_world_collision" => client_world_collision_case(request),
        "client_world_indexed_collision" => client_world_indexed_collision_case(request),
        "client_world_happy_ghast_collision" => client_world_happy_ghast_collision_case(request),
        "client_world_entity_section_index" => client_world_entity_section_index_case(request),
        "client_world_stair_neighbor_collision" => {
            client_world_stair_neighbor_collision_case(request)
        }
        "local_player_minor_horizontal_collision" => {
            local_player_minor_horizontal_collision_case(request)
        }
        "client_world_registry_collision" => client_world_registry_collision_case(request),
        "world_border_collision_shape" => world_border_collision_shape_case(request),
        "world_border_collision_shape_moving" => world_border_collision_shape_moving_case(request),
        "client_world_border_state" => client_world_border_state_case(request),
        "world_border_render" => world_border_render_case(request),
        "collect_step_up_heights" => collect_step_up_heights_case(request),
        "select_step_up_movement" => select_step_up_movement_case(request),
        "collect_block_collisions" => collect_block_collisions_case(request),
        "entity_fit" => entity_fit_case(request),
        "living_entity_dimensions_scale" => living_entity_dimensions_scale_case(request),
        "entity_dimensions_refresh" => entity_dimensions_refresh_case(request),
        "player_pose_dimensions" => player_pose_dimensions_case(request),
        "player_pose_selection" => player_pose_selection_case(request),
        "clip" => clip_case(request),
        "clip_join" => clip_join_case(request),
        "closest" => closest_case(request),
        "closest_join" => closest_join_case(request),
        "face" => face_case(request),
        "face_join" => face_join_case(request),
        "block_occludes" => block_occludes_case(request),
        "block_occludes_join" => block_occludes_join_case(request),
        "merged_occludes" => merged_occludes_case(request),
        "face_occludes" => face_occludes_case(request),
        _ => Err("unknown VoxelShape kind".into()),
    }
}
