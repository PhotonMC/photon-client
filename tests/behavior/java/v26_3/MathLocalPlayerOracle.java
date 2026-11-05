import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathLocalPlayerOracle extends MathAttributeFixtures {
    static JsonObject localPlayerModifiedFrictionResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, 
                "computeModifiedFriction", float.class, float.class);
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                float friction = (float)method.invoke(
                    null, Oracle.f32(input, "friction"), Oracle.f32(input, "modifier"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.addProperty("friction", Oracle.bits(friction));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.computeModifiedFriction reflection failed", error);
        }
    }

    static JsonObject localPlayerTravelFlyingResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, 
                "travelFlying", Vec3.class, float.class, float.class, float.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureTravelFlying(
                    input.get("in_water").getAsBoolean(),
                    input.get("in_lava").getAsBoolean(),
                    Oracle.f32(input, "y_rot"),
                    vec3(input.getAsJsonObject("delta")));
                method.invoke(
                    player,
                    vec3(input.getAsJsonObject("input")),
                    Oracle.f32(input, "water_speed"),
                    Oracle.f32(input, "lava_speed"),
                    Oracle.f32(input, "air_speed"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LivingEntity.travelFlying reflection failed", error);
        }
    }

    static JsonObject localPlayerTravelInWaterResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, 
                "travelInWater", Vec3.class, double.class, boolean.class, double.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureTravelInWater(
                    input.get("sprinting").getAsBoolean(),
                    Oracle.f32(input, "water_slow_down"),
                    Oracle.f32(input, "water_efficiency"),
                    input.get("on_ground").getAsBoolean(),
                    Oracle.f32(input, "speed"),
                    input.get("dolphins_grace").getAsBoolean(),
                    input.get("horizontal_collision").getAsBoolean(),
                    input.get("on_climbable").getAsBoolean(),
                    input.get("free_after_jump").getAsBoolean(),
                    Oracle.f32(input, "y_rot"),
                    f64(input, "current_y"),
                    vec3(input.getAsJsonObject("delta")));
                method.invoke(
                    player,
                    vec3(input.getAsJsonObject("input")),
                    f64(input, "base_gravity"),
                    input.get("is_falling").getAsBoolean(),
                    f64(input, "old_y"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LivingEntity.travelInWater reflection failed", error);
        }
    }

    static JsonObject localPlayerTravelInLavaResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, 
                "travelInLava", Vec3.class, double.class, boolean.class, double.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureTravelInLava(
                    input.get("sprinting").getAsBoolean(),
                    input.get("shallow_lava").getAsBoolean(),
                    input.get("horizontal_collision").getAsBoolean(),
                    input.get("free_after_jump").getAsBoolean(),
                    Oracle.f32(input, "y_rot"),
                    f64(input, "current_y"),
                    vec3(input.getAsJsonObject("delta")));
                method.invoke(
                    player,
                    vec3(input.getAsJsonObject("input")),
                    f64(input, "base_gravity"),
                    input.get("is_falling").getAsBoolean(),
                    f64(input, "old_y"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LivingEntity.travelInLava reflection failed", error);
        }
    }

    static JsonObject localPlayerHandleRelativeFrictionResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, 
                "handleRelativeFrictionAndCalculateMovement", Vec3.class, float.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureHandleRelativeFriction(
                    input.get("on_ground").getAsBoolean(),
                    Oracle.f32(input, "speed"),
                    Oracle.f32(input, "flying_speed"),
                    input.get("flying").getAsBoolean(),
                    input.get("sprinting").getAsBoolean(),
                    input.get("passenger").getAsBoolean(),
                    input.get("on_climbable").getAsBoolean(),
                    input.get("scaffolding").getAsBoolean(),
                    input.get("suppress_sliding_down_ladder").getAsBoolean(),
                    input.get("horizontal_collision").getAsBoolean(),
                    input.get("jumping").getAsBoolean(),
                    Oracle.f32(input, "y_rot"),
                    vec3(input.getAsJsonObject("delta")));
                Vec3 movement = (Vec3)method.invoke(
                    player,
                    vec3(input.getAsJsonObject("input")),
                    Oracle.f32(input, "block_friction"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(movement));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.handleRelativeFrictionAndCalculateMovement reflection failed", error);
        }
    }

    static JsonObject localPlayerTravelInAirResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, "travelInAir", Vec3.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureTravelInAir(
                    input.get("on_ground").getAsBoolean(),
                    input.get("below_block").getAsString(),
                    Oracle.f32(input, "friction_modifier"),
                    Oracle.f32(input, "speed"),
                    Oracle.f32(input, "flying_speed"),
                    input.get("flying").getAsBoolean(),
                    input.get("sprinting").getAsBoolean(),
                    input.get("passenger").getAsBoolean(),
                    input.get("on_climbable").getAsBoolean(),
                    input.get("scaffolding").getAsBoolean(),
                    input.get("suppress_sliding_down_ladder").getAsBoolean(),
                    input.get("horizontal_collision").getAsBoolean(),
                    input.get("jumping").getAsBoolean(),
                    input.get("levitation_amplifier").getAsInt(),
                    f64(input, "gravity"),
                    input.get("should_discard_friction").getAsBoolean(),
                    Oracle.f32(input, "air_drag_modifier"),
                    input.get("omnidirectional_air_mover").getAsBoolean(),
                    input.get("client_side").getAsBoolean(),
                    input.get("has_chunk_at").getAsBoolean(),
                    input.get("min_y").getAsInt(),
                    Oracle.f32(input, "y_rot"),
                    f64(input, "current_y"),
                    vec3(input.getAsJsonObject("delta")));
                method.invoke(player, vec3(input.getAsJsonObject("input")));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LivingEntity.travelInAir reflection failed", error);
        }
    }

    static JsonObject localPlayerUpdateFallFlyingMovementResult(JsonObject request) {
        try {
            var method = Reflect.method(LivingEntity.class, "updateFallFlyingMovement", Vec3.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureFallFlyingMovement(
                    vec3(input.getAsJsonObject("look")),
                    Oracle.f32(input, "x_rot"),
                    f64(input, "gravity"));
                Vec3 movement = (Vec3)method.invoke(
                    player,
                    vec3(input.getAsJsonObject("delta")));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(movement));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.updateFallFlyingMovement reflection failed", error);
        }
    }

    static JsonObject localPlayerTravelDispatchResult(JsonObject request) {
        try {
            var method = LivingEntity.class.getMethod("travel", Vec3.class);
            var states = new JsonArray();
            Unsafe unsafe = unsafeInstance();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureTravelDispatch(input);
                method.invoke(player, vec3(input.getAsJsonObject("input")));
                String path = player.travelPath == null ? "air" : player.travelPath;
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.addProperty(
                    "mode",
                    path.equals("water") ? "water"
                        : path.equals("lava") ? "lava"
                        : path.startsWith("fall_flying") ? "fall_flying"
                        : "air");
                state.addProperty("fall_flying_climbable_fallback", path.equals("fall_flying_climbable"));
                state.add("movement", snapshot(player.getDeltaMovement()));
                states.add(state);
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LivingEntity.travel dispatch reflection failed", error);
        }
    }

    static JsonObject localPlayerInputCase(JsonObject request) {
        var result = new JsonObject();
        try {
            var output = System.out;
            var errorOutput = System.err;
            try {
                SharedConstants.setVersion(TEST_VERSION);
                Bootstrap.bootStrap();
            } finally {
                System.setOut(output);
                System.setErr(errorOutput);
            }
            Unsafe unsafe = unsafeInstance();
            var method = Reflect.method(LocalPlayer.class, "modifyInput", Vec2.class);
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.usingItem = input.get("using_item").getAsBoolean();
                player.passenger = input.get("passenger").getAsBoolean();
                player.movingSlowly = input.get("moving_slowly").getAsBoolean();
                player.sneakingMovementFactor = Oracle.f32(input, "sneaking_movement_factor");
                player.setItemUseSpeed(Oracle.f32(input, "item_use_speed_multiplier"));
                Vec2 original = new Vec2(Oracle.f32(input, "x"), Oracle.f32(input, "y"));
                Vec2 modified = (Vec2)method.invoke(player, original);
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("input", snapshot(original));
                state.add("modified", snapshot(modified));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LocalPlayer.modifyInput reflection failed", error);
        }
        return result;
    }

    static JsonObject localPlayerApplyInputCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = LocalPlayer.class.getMethod("applyInput");
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureApplyInput(
                    Oracle.f32(input, "move_x"),
                    Oracle.f32(input, "move_y"),
                    input.get("jump").getAsBoolean(),
                    Oracle.f32(input, "xxa"),
                    Oracle.f32(input, "zza"),
                    input.get("jumping").getAsBoolean(),
                    Oracle.f32(input, "x_bob"),
                    Oracle.f32(input, "y_bob"),
                    Oracle.f32(input, "x_bob_o"),
                    Oracle.f32(input, "y_bob_o"),
                    Oracle.f32(input, "x_rot"),
                    Oracle.f32(input, "y_rot"),
                    input.get("controlled_camera").getAsBoolean());
                method.invoke(player);
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(new Vec2(player.xxa, player.zza)));
                state.addProperty("jumping", player.isJumping());
                state.addProperty("x_bob", Oracle.bits(player.xBob));
                state.addProperty("y_bob", Oracle.bits(player.yBob));
                state.addProperty("x_bob_o", Oracle.bits(player.xBobO));
                state.addProperty("y_bob_o", Oracle.bits(player.yBobO));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LocalPlayer.applyInput reflection failed", error);
        }
        return result;
    }

    static JsonObject localPlayerCanStartSprintingCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = Reflect.method(LocalPlayer.class, "canStartSprinting");
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.configureSprintGate(
                    Oracle.f32(input, "move_y"),
                    input.get("sprinting").getAsBoolean(),
                    input.get("mobility_restricted").getAsBoolean(),
                    input.get("enough_food").getAsBoolean(),
                    input.get("mayfly").getAsBoolean(),
                    input.get("shallow_water").getAsBoolean(),
                    input.get("flying").getAsBoolean(),
                    input.get("using_item").getAsBoolean(),
                    input.get("fall_flying").getAsBoolean(),
                    input.get("underwater").getAsBoolean(),
                    input.get("moving_slowly").getAsBoolean());
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.addProperty("can_start", (boolean)method.invoke(player));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("LocalPlayer.canStartSprinting reflection failed", error);
        }
        return result;
    }

    static JsonObject localPlayerFluidFallingAdjustmentCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = LivingEntity.class.getMethod(
                "getFluidFallingAdjustedMovement", double.class, boolean.class, Vec3.class);
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.sprinting = input.get("sprinting").getAsBoolean();
                Vec3 movement = new Vec3(
                    f64(input, "x"), f64(input, "y"), f64(input, "z"));
                Vec3 adjusted = (Vec3)method.invoke(
                    player,
                    f64(input, "base_gravity"),
                    input.get("is_falling").getAsBoolean(),
                    movement);
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(adjusted));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.getFluidFallingAdjustedMovement reflection failed", error);
        }
        return result;
    }

    static JsonObject localPlayerClimbableMovementCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = Reflect.method(LivingEntity.class, "handleOnClimbable", Vec3.class);
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.onClimbableState = input.get("on_climbable").getAsBoolean();
                player.scaffolding = input.get("scaffolding").getAsBoolean();
                player.suppressSlidingDownLadder =
                    input.get("suppress_sliding_down_ladder").getAsBoolean();
                Vec3 movement = new Vec3(
                    f64(input, "x"), f64(input, "y"), f64(input, "z"));
                Vec3 adjusted = (Vec3)method.invoke(player, movement);
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.add("movement", snapshot(adjusted));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.handleOnClimbable reflection failed", error);
        }
        return result;
    }

    static JsonObject localPlayerFrictionInfluencedSpeedCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = Reflect.method(LivingEntity.class, 
                "getFrictionInfluencedSpeed", float.class);
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayerInputFixture player =
                    (LocalPlayerInputFixture)unsafe.allocateInstance(LocalPlayerInputFixture.class);
                player.onGroundState = input.get("on_ground").getAsBoolean();
                player.configuredSpeed = Oracle.f32(input, "speed");
                player.passenger = input.get("passenger").getAsBoolean();
                player.sprinting = input.get("sprinting").getAsBoolean();
                player.abilities = new Abilities();
                player.abilities.flying = input.get("flying").getAsBoolean();
                player.abilities.setFlyingSpeed(Oracle.f32(input, "flying_speed"));
                var abilitiesField = Reflect.field(net.minecraft.world.entity.player.Player.class, "abilities");
                unsafe.putObject(player, unsafe.objectFieldOffset(abilitiesField), player.abilities);
                float speed = (float)method.invoke(
                    player, Oracle.f32(input, "block_friction"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.addProperty("speed", Oracle.bits(speed));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LivingEntity.getFrictionInfluencedSpeed reflection failed", error);
        }
        return result;
    }
}
