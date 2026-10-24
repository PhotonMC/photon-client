import com.google.gson.JsonObject;

public final class MathOracle extends MathPlayerInputOracle {
    static JsonObject clientWorldResult(String operation, JsonObject request) {
        if (operation.equals("client_world_section_lookup")) {
            return clientWorldSectionLookupResult(request);
        }
        if (operation.equals("client_world_chunk_cache")) {
            return clientWorldChunkCacheResult(request);
        }
        if (operation.equals("client_world_block_update")) {
            return clientWorldBlockUpdateResult(request);
        }
        if (operation.equals("client_world_light")) {
            return clientWorldLightResult(request);
        }
        if (operation.equals("client_world_player_attributes")) {
            return clientWorldPlayerAttributesResult(request);
        }
        if (operation.equals("client_world_player_movement_speed")) {
            return clientWorldPlayerMovementSpeedResult(request);
        }
        if (operation.equals("client_world_player_attribute_water")) {
            return clientWorldPlayerAttributeWaterResult(request);
        }
        if (operation.equals("client_world_player_sprint_attribute")) {
            return clientWorldPlayerSprintAttributeResult(request);
        }
        if (operation.equals("client_world_player_sneaking_input")) {
            return clientWorldPlayerSneakingInputResult(request);
        }
        if (operation.equals("client_world_player_keyboard_input")) {
            return clientWorldPlayerKeyboardInputResult(request);
        }
        if (operation.equals("client_world_player_tick")) {
            return clientWorldPlayerTickResult(request);
        }
        if (operation.equals("client_world_entity_player_head_rotation")) {
            return clientWorldEntityPlayerHeadRotationResult(request);
        }
        if (operation.equals("client_world_entity_snowball_tick")) {
            return clientWorldEntitySnowballTickResult(request);
        }
        if (operation.equals("client_world_entity_arrow_tick")) {
            return clientWorldEntityArrowTickResult(request);
        }
        if (operation.equals("client_world_entity_experience_orb_tick")) {
            return clientWorldEntityExperienceOrbTickResult(request);
        }
        if (operation.equals("client_world_entity_primed_tnt_tick")) {
            return clientWorldEntityPrimedTntTickResult(request);
        }
        if (operation.equals("client_world_entity_falling_block_tick")) {
            return clientWorldEntityFallingBlockTickResult(request);
        }
        if (operation.equals("client_world_entity_firework_tick")) {
            return clientWorldEntityFireworkTickResult(request);
        }
        if (operation.equals("client_world_entity_boat_air_tick")) {
            return clientWorldEntityBoatAirTickResult(request);
        }
        if (operation.equals("client_world_entity_boat_remote_tick")) {
            return clientWorldEntityBoatRemoteTickResult(request);
        }
        if (operation.equals("client_world_entity_ticking_eligibility")) {
            return clientWorldEntityTickingEligibilityResult(request);
        }
        if (operation.equals("client_world_entity_player_interpolation")) {
            return clientWorldEntityPlayerInterpolationResult(request);
        }
        if (operation.equals("client_world_entity_player_snap")) {
            return clientWorldEntityPlayerSnapResult(request);
        }
        if (operation.equals("client_world_entity_player_motion")) {
            return clientWorldEntityPlayerMotionResult(request);
        }
        if (operation.equals("client_world_entity_teleport")) {
            return clientWorldEntityTeleportResult(request);
        }
        if (operation.equals("client_world_entity_passengers")) {
            return clientWorldEntityPassengersResult(request);
        }
        if (operation.equals("client_world_entity_passenger_position")) {
            return clientWorldEntityPassengerPositionResult(request);
        }
        if (operation.equals("client_world_entity_passenger_player_position")) {
            return clientWorldEntityPassengerPlayerPositionResult(request);
        }
        if (operation.equals("client_world_entity_passenger_boat_family_position")) {
            return clientWorldEntityPassengerBoatFamilyPositionResult(request);
        }
        if (operation.equals("client_world_entity_passenger_rotation")) {
            return clientWorldEntityPassengerRotationResult(request);
        }
        if (operation.equals("client_world_entity_passenger_boat_turn_exemption")) {
            return clientWorldEntityPassengerBoatTurnExemptionResult(request);
        }
        if (operation.equals("client_world_entity_passenger_animal_rotation")) {
            return clientWorldEntityPassengerAnimalRotationResult(request);
        }
        if (operation.equals("client_world_entity_passenger_animal_position")) {
            return clientWorldEntityPassengerAnimalPositionResult(request);
        }
        if (operation.equals("client_world_entity_passenger_turn")) {
            return clientWorldEntityPassengerTurnResult(request);
        }
        if (operation.equals("client_world_player_sprint_trigger")) {
            return clientWorldPlayerSprintTriggerResult(request);
        }
        if (operation.equals("client_world_entity_shared_flags")) {
            return clientWorldEntitySharedFlagsResult(request);
        }
        if (operation.equals("client_world_entity_air_supply")) {
            return clientWorldEntityAirSupplyResult(request);
        }
        if (operation.equals("client_world_entity_pose")) {
            return clientWorldEntityPoseResult(request);
        }
        if (operation.equals("client_world_entity_ticks_frozen")) {
            return clientWorldEntityTicksFrozenResult(request);
        }
        if (operation.equals("client_world_entity_custom_name")) {
            return clientWorldEntityCustomNameResult(request);
        }
        if (operation.equals("client_world_entity_living_flags")) {
            return clientWorldEntityLivingFlagsResult(request);
        }
        if (operation.equals("client_world_entity_health")) {
            return clientWorldEntityHealthResult(request);
        }
        if (operation.equals("client_world_entity_mob_flags")) {
            return clientWorldEntityMobFlagsResult(request);
        }
        if (operation.equals("client_world_entity_arrow_count")) {
            return clientWorldEntityArrowCountResult(request);
        }
        if (operation.equals("client_world_entity_effect_ambience")) {
            return clientWorldEntityEffectAmbienceResult(request);
        }
        if (operation.equals("client_world_entity_effect_particles")) {
            return clientWorldEntityEffectParticlesResult(request);
        }
        if (operation.equals("client_world_entity_sleeping_position")) {
            return clientWorldEntitySleepingPositionResult(request);
        }
        if (operation.equals("client_world_entity_stinger_count")) {
            return clientWorldEntityStingerCountResult(request);
        }
        throw new IllegalArgumentException("Unknown client world operation");
    }

    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        if (operation.startsWith("client_world_")) {
            return clientWorldResult(operation, request);
        }
        return switch (operation) {
            case "vec2" -> vec2Case(request);
            case "keyboard_input" -> keyboardInputCase(request);
            case "local_player_input" -> localPlayerInputCase(request);
            case "local_player_apply_input" -> localPlayerApplyInputCase(request);
            case "local_player_can_start_sprinting" -> localPlayerCanStartSprintingCase(request);
            case "local_player_fluid_falling_adjustment" -> localPlayerFluidFallingAdjustmentCase(request);
            case "local_player_climbable_movement" -> localPlayerClimbableMovementCase(request);
            case "local_player_friction_influenced_speed" -> localPlayerFrictionInfluencedSpeedCase(request);
            case "local_player_modified_friction" -> localPlayerModifiedFrictionResult(request);
            case "local_player_travel_flying" -> localPlayerTravelFlyingResult(request);
            case "local_player_travel_in_water" -> localPlayerTravelInWaterResult(request);
            case "local_player_travel_in_lava" -> localPlayerTravelInLavaResult(request);
            case "local_player_handle_relative_friction" -> localPlayerHandleRelativeFrictionResult(request);
            case "local_player_travel_in_air" -> localPlayerTravelInAirResult(request);
            case "local_player_update_fall_flying_movement" -> localPlayerUpdateFallFlyingMovementResult(request);
            case "local_player_travel_dispatch" -> localPlayerTravelDispatchResult(request);
            case "character_event" -> characterEventCase(request);
            case "preedit_event" -> preeditEventCase(request);
            case "option_values" -> optionValuesCase(request);
            case "sliderable_enum" -> sliderableEnumCase(request);
            case "cycleable_enum" -> cycleableEnumCase(request);
            case "alt_enum" -> altEnumCase(request);
            case "lazy_enum" -> lazyEnumCase(request);
            case "clamping_lazy_max_int_range" -> clampingLazyMaxIntRangeCase(request);
            case "mapped_int_range" -> mappedIntRangeCase(request);
            case "mapped_unit_double" -> mappedUnitDoubleCase(request);
            case "scroll_wheel" -> scrollWheelCase(request);
            case "smooth_double" -> smoothDoubleCase(request);
            case "input_with_modifiers" -> inputWithModifiersCase(request);
            case "key_mapping" -> keyMappingCase(request);
            case "toggle_key_mapping" -> toggleKeyMappingCase(request);
            case "direction" -> directionCase(request);
            case "direction_axial_rotation" -> directionAxialRotationCase(request);
            case "direction_y_rotation" -> directionYRotationCase(request);
            case "direction_data" -> directionDataCase(request);
            case "direction_y_rot" -> directionYRotCase(request);
            case "direction_axis" -> directionAxisCase(request);
            case "direction_get" -> directionGetCase(request);
            case "direction_unit_vectors" -> directionUnitVectorsCase(request);
            case "direction_step" -> directionStepCase(request);
            case "direction_axis_orientation" -> directionAxisOrientationCase(request);
            case "direction_axis_directions" -> directionAxisDirectionsCase(request);
            case "direction_axis_direction_values" -> directionAxisDirectionValuesCase(request);
            case "direction_axis_choose" -> directionAxisChooseCase(request);
            case "direction_axis_test" -> directionAxisTestCase(request);
            case "direction_axis_plane" -> directionAxisPlaneCase(request);
            case "direction_plane_test" -> directionPlaneTestCase(request);
            case "direction_plane_order" -> directionPlaneOrderCase(request);
            case "direction_random" -> directionRandomCase(request);
            case "direction_plane_shuffle" -> directionPlaneShuffleCase(request);
            case "direction_all_shuffle" -> directionAllShuffleCase(request);
            case "direction_stream" -> directionStreamCase(request);
            case "direction_to_y_rot" -> directionToYRotCase(request);
            case "direction_get_y_rot" -> directionGetYRotCase(request);
            case "direction_rotation" -> directionRotationCase(request);
            case "direction_name_lookup" -> directionNameLookupCase(request);
            case "direction_enum_text" -> directionEnumTextCase(request);
            case "direction_enum_name" -> directionEnumNameCase(request);
            case "direction_approximate_nearest" -> directionApproximateNearestCase(request);
            case "direction_approximate_nearest_overloads" -> directionApproximateNearestOverloadsCase(request);
            case "direction_nearest" -> directionNearestCase(request);
            case "direction_axis_step_order" -> directionAxisStepOrderCase(request);
            case "direction_is_facing_angle" -> directionIsFacingAngleCase(request);
            case "coord" -> coordCase(request);
            case "blockpos" -> blockposCase(request);
            case "blockpos_packed_helpers" -> blockposPackedHelpersCase(request);
            case "blockpos_relative_helpers" -> blockposRelativeHelpersCase(request);
            case "blockpos_vector_helpers" -> blockposVectorHelpersCase(request);
            case "blockpos_geometry_helpers" -> blockposGeometryHelpersCase(request);
            case "blockpos_distance_helpers" -> blockposDistanceHelpersCase(request);
            case "blockpos_integer_helpers" -> blockposIntegerHelpersCase(request);
            case "blockpos_clamp_location" -> blockposClampLocationCase(request);
            case "blockpos_between_closed" -> blockposBetweenClosedCase(request);
            case "blockpos_neighbor_column" -> blockposNeighborColumnCase(request);
            case "blockpos_square_out_south_east" -> blockposSquareOutSouthEastCase(request);
            case "blockpos_random_ranges" -> blockposRandomRangesCase(request);
            case "blockpos_between_closed_aabb" -> blockposBetweenClosedAabbCase(request);
            case "blockpos_within_manhattan" -> blockposWithinManhattanCase(request);
            case "blockpos_spiral_around" -> blockposSpiralAroundCase(request);
            case "blockpos_find_closest_match" -> blockposFindClosestMatchCase(request);
            case "blockpos_between_closed_streams" -> blockposBetweenClosedStreamsCase(request);
            case "blockpos_coordinate_helpers" -> blockposCoordinateHelpersCase(request);
            case "blockpos_between_corners_direction" -> blockposBetweenCornersDirectionCase(request);
            case "blockpos_breadth_first_traversal" -> blockposBreadthFirstTraversalCase(request);
            case "blockpos_mutable_helpers" -> blockposMutableHelpersCase(request);
            case "axis_cycle_helpers" -> axisCycleHelpersCase(request);
            case "section_relative" -> sectionRelativeCase(request);
            case "vec3_codec" -> vec3CodecCase(request);
            case "chunkpos_codec" -> chunkposCodecCase(request);
            case "chunkpos_block_coordinates" -> chunkposBlockCoordinatesCase(request);
            case "chunkpos_region_coordinates" -> chunkposRegionCoordinatesCase(request);
            case "chunkpos_block_positions" -> chunkposBlockPositionsCase(request);
            case "chunkpos_representation" -> chunkposRepresentationCase(request);
            case "chunkpos_from_blockpos" -> chunkposFromBlockposCase(request);
            case "chunkpos_distances" -> chunkposDistancesCase(request);
            case "chunkpos_from_section_node" -> chunkposFromSectionNodeCase(request);
            case "chunkpos_range_closed" -> chunkposRangeClosedCase(request);
            case "chunkpos_constant" -> chunkposConstantCase(request);
            case "chunkpos_packed_accessors" -> chunkposPackedAccessorsCase(request);
            case "sectionpos_packed_accessors" -> sectionposPackedAccessorsCase(request);
            case "sectionpos_block_coordinates" -> sectionposBlockCoordinatesCase(request);
            case "sectionpos_relative_to_block" -> sectionposRelativeToBlockCase(request);
            case "sectionpos_spatial" -> sectionposSpatialCase(request);
            case "sectionpos_packed_transformations" -> sectionposPackedTransformationsCase(request);
            case "sectionpos_as_long" -> sectionposAsLongCase(request);
            case "sectionpos_offset" -> sectionposOffsetCase(request);
            case "sectionpos_ranges" -> sectionposRangesCase(request);
            case "sectionpos_blocks_inside" -> sectionposBlocksInsideCase(request);
            case "sectionpos_around_and_at_block_pos" -> sectionposAroundAndAtBlockPosCase(request);
            case "sectionpos_double_coordinates" -> sectionposDoubleCoordinatesCase(request);
            case "sectionpos_of_overloads" -> sectionposOfOverloadsCase(request);
            case "chunkpos_validity" -> chunkposValidityCase(request);
            case "mth_constants" -> mthConstantsCase(request);
            case "mth_floating_primitives" -> mthFloatingPrimitivesCase(request);
            case "mth_power_helpers" -> mthPowerHelpersCase(request);
            case "mth_frac" -> mthFracCase(request);
            case "mth_create_insecure_uuid" -> mthCreateInsecureUuidCase(request);
            case "mth_murmur_hash3_mixer" -> mthMurmurHash3MixerCase(request);
            case "mth_binary_search" -> mthBinarySearchCase(request);
            case "mth_ray_intersects_aabb" -> mthRayIntersectsAabbCase(request);
            case "mth_fast_inv_sqrt" -> mthFastInvSqrtCase(request);
            case "mth_fast_inv_cube_root" -> mthFastInvCubeRootCase(request);
            case "mth_hsv_to_color" -> mthHsvToColorCase(request);
            case "mth_angle_wrap_difference" -> mthAngleWrapDifferenceCase(request);
            case "mth_angle_approach" -> mthAngleApproachCase(request);
            case "mth_get_int" -> mthGetIntCase(request);
            case "mth_get_seed" -> mthGetSeedCase(request);
            case "mth_atan2" -> mthAtan2Case(request);
            case "mth_inv_sqrt" -> mthInvSqrtCase(request);
            case "mth_integer_helpers" -> mthIntegerHelpersCase(request);
            case "mth_integer_basics" -> mthIntegerBasicsCase(request);
            case "mth_angle_helpers" -> mthAngleHelpersCase(request);
            case "mth_rounding_helpers" -> mthRoundingHelpersCase(request);
            case "mth_positive_modulo" -> mthPositiveModuloCase(request);
            case "mth_abs_max" -> mthAbsMaxCase(request);
            case "mth_equal" -> mthEqualCase(request);
            case "mth_clamped_lerp" -> mthClampedLerpCase(request);
            case "mth_next_random" -> mthNextRandomCase(request);
            case "mth_random_helpers" -> mthRandomHelpersCase(request);
            case "mth_length" -> mthLengthCase(request);
            case "mth_quantize" -> mthQuantizeCase(request);
            case "mth_out_from_origin" -> mthOutFromOriginCase(request);
            case "mth_mul_and_truncate" -> mthMulAndTruncateCase(request);
            case "mth_rotation_around_axis" -> mthRotationAroundAxisCase(request);
            case "mth_inverse_lerp" -> mthInverseLerpCase(request);
            case "mth_lerp" -> mthLerpCase(request);
            case "mth_lerp_int" -> mthLerpIntCase(request);
            case "mth_lerp_discrete" -> mthLerpDiscreteCase(request);
            case "mth_smoothstep" -> mthSmoothstepCase(request);
            case "mth_smoothstep_derivative" -> mthSmoothstepDerivativeCase(request);
            case "mth_sign" -> mthSignCase(request);
            case "mth_rot_lerp" -> mthRotLerpCase(request);
            case "mth_rot_lerp_rad" -> mthRotLerpRadCase(request);
            case "mth_triangle_wave" -> mthTriangleWaveCase(request);
            case "mth_square_f32" -> mthSquareF32Case(request);
            case "mth_cube_f32" -> mthCubeF32Case(request);
            case "mth_square_f64" -> mthSquareF64Case(request);
            case "mth_square_i32" -> mthSquareI32Case(request);
            case "mth_square_i64" -> mthSquareI64Case(request);
            case "mth_clamped_map_f64" -> mthClampedMapF64Case(request);
            case "mth_clamped_map_f32" -> mthClampedMapF32Case(request);
            case "mth_map_f64" -> mthMapF64Case(request);
            case "mth_map_f32" -> mthMapF32Case(request);
            case "mth_wobble_f64" -> mthWobbleF64Case(request);
            case "mth_interpolation" -> mthInterpolationCase(request);
            case "vec3" -> vec3Case(request);
            case "vec3_dot" -> vec3DotCase(request);
            case "vec3_distance_to_sqr" -> vec3DistanceToSqrCase(request);
            case "vec3_distance_to" -> vec3DistanceToCase(request);
            case "vec3_distance_to_sqr_components" -> vec3DistanceToSqrComponentsCase(request);
            case "vec3_horizontal_distances" -> vec3HorizontalDistancesCase(request);
            case "vec3_closer_than" -> vec3CloserThanCase(request);
            case "vec3_offset_random" -> vec3OffsetRandomCase(request);
            case "vec3_lerp" -> vec3LerpCase(request);
            case "vec3_rotate_clockwise_90" -> vec3RotateClockwise90Case(request);
            case "vec3_is_finite" -> vec3IsFiniteCase(request);
            case "vec3_get" -> vec3GetCase(request);
            case "vec3_with" -> vec3WithCase(request);
            case "vec3_relative" -> vec3RelativeCase(request);
            case "vec3_align" -> vec3AlignCase(request);
            case "vec3_projected_on" -> vec3ProjectedOnCase(request);
            case "vec3_to_vector3f" -> vec3ToVector3fCase(request);
            case "vec3_coordinates" -> vec3CoordinatesCase(request);
            case "vec3_equality" -> vec3EqualityCase(request);
            case "vec3_to_string" -> vec3ToStringCase(request);
            case "vec3_rotate" -> vec3RotateCase(request);
            case "vec3_direction_from_rotation" -> vec3DirectionFromRotationCase(request);
            case "vec3_local_coordinates" -> vec3LocalCoordinatesCase(request);
            case "vec3_rotation" -> vec3RotationCase(request);
            case "vec2_rotate" -> vec2RotateCase(request);
            case "vec2_dot" -> vec2DotCase(request);
            case "vec2_add_vec2" -> vec2AddVec2Case(request);
            case "vec2_add_scalar" -> vec2AddScalarCase(request);
            case "vec2_distance_to_sqr" -> vec2DistanceToSqrCase(request);
            case "vec2_negated" -> vec2NegatedCase(request);
            case "vec2_equality" -> vec2EqualityCase(request);
            case "vec2_constant" -> vec2ConstantCase(request);
            case "vec3_constant" -> vec3ConstantCase(request);
            case "shape" -> shapeCase(request);
            case "random" -> randomCase(request);
            case "random_fork" -> randomForkCase(request);
            case "random_positional" -> randomPositionalCase(request);
            case "aabb_builder" -> aabbBuilderCase(request);
            case "aabb" -> aabbCase(request);
            default -> ElseCase(request, operation);
        };
    }
}
