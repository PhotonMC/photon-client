import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;

class MathEntityStateOracle extends MathEntityMotionOracle {
    static JsonObject clientWorldEntitySharedFlagsResult(JsonObject request) {
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

            var entity = new SharedFlagsFixtureEntity();
            var states = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(sharedFlagsState(
                    operation.get("label").getAsString(), entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity shared flags fixture failed", error);
        }
    }

    static JsonObject sharedFlagsState(
        String label, SharedFlagsFixtureEntity entity
    ) {
        int flags = entity.sharedFlags();
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        state.addProperty("shared_flags", flags);
        state.addProperty("on_fire", entity.isOnFire());
        state.addProperty("shift_key_down", entity.isShiftKeyDown());
        state.addProperty("sprinting", entity.isSprinting());
        state.addProperty("swimming", entity.isSwimming());
        state.addProperty("invisible", entity.isInvisible());
        state.addProperty("glowing", entity.isCurrentlyGlowing());
        state.addProperty("fall_flying", (flags & (1 << 7)) != 0);
        state.addProperty("air_supply", entity.getAirSupply());
        return state;
    }

    static JsonObject clientWorldEntityAirSupplyResult(JsonObject request) {
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

            var entity = new SharedFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(airSupplyState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(airSupplyState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity air supply fixture failed", error);
        }
    }

    static JsonObject airSupplyState(
        String label, boolean accepted, SharedFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("shared_flags", entity.sharedFlags());
        state.addProperty("air_supply", entity.getAirSupply());
        return state;
    }

    static JsonObject clientWorldEntityPoseResult(JsonObject request) {
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

            var entity = new SharedFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityPoseState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityPoseState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity pose fixture failed", error);
        }
    }

    static JsonObject entityPoseState(
        String label, boolean accepted, SharedFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("shared_flags", entity.sharedFlags());
        state.addProperty("pose_id", entity.getPose().id());
        return state;
    }

    static JsonObject clientWorldEntityTicksFrozenResult(JsonObject request) {
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

            var entity = new SharedFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityTicksFrozenState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityTicksFrozenState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity ticks frozen fixture failed", error);
        }
    }

    static JsonObject entityTicksFrozenState(
        String label, boolean accepted, SharedFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("shared_flags", entity.sharedFlags());
        state.addProperty("ticks_frozen", entity.getTicksFrozen());
        state.addProperty("percent_frozen", Oracle.bits(entity.getPercentFrozen()));
        state.addProperty("is_freezing", entity.isFreezing());
        state.addProperty("is_fully_frozen", entity.isFullyFrozen());
        return state;
    }

    static JsonObject clientWorldEntityCustomNameResult(JsonObject request) {
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

            var entity = new SharedFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityCustomNameState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityCustomNameState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity custom name fixture failed", error);
        }
    }

    static JsonObject entityCustomNameState(
        String label, boolean accepted, SharedFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("custom_name_present", entity.hasCustomName());
        state.addProperty("custom_name_visible", entity.isCustomNameVisible());
        state.addProperty("should_show_name", entity.shouldShowName());
        return state;
    }

    static JsonObject clientWorldEntityLivingFlagsResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityLivingFlagsState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityLivingFlagsState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity living flags fixture failed", error);
        }
    }

    static JsonObject entityLivingFlagsState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("living_flags", entity.livingFlags());
        state.addProperty("is_using_item", entity.isUsingItem());
        state.addProperty("used_item_is_off_hand", entity.getUsedItemHand()
            == net.minecraft.world.InteractionHand.OFF_HAND);
        state.addProperty("is_auto_spin_attack", entity.isAutoSpinAttack());
        return state;
    }

    static JsonObject clientWorldEntityHealthResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityHealthState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity health fixture failed", error);
        }
    }

    static JsonObject entityHealthState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("living_flags", entity.livingFlags());
        state.addProperty("health_bits", Oracle.bits(entity.getHealth()));
        return state;
    }

    static JsonObject clientWorldEntityMobFlagsResult(JsonObject request) {
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

            var entity = new MobFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityMobFlagsState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityMobFlagsState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity mob flags fixture failed", error);
        }
    }

    static JsonObject entityMobFlagsState(
        String label, boolean accepted, MobFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("mob_flags", entity.mobFlags());
        state.addProperty("is_no_ai", entity.isNoAi());
        state.addProperty("is_left_handed", entity.isLeftHanded());
        state.addProperty("is_aggressive", entity.isAggressive());
        return state;
    }

    static JsonObject clientWorldEntityArrowCountResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityArrowCountState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityArrowCountState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity arrow count fixture failed", error);
        }
    }

    static JsonObject entityArrowCountState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("arrow_count", entity.getArrowCount());
        state.addProperty("living_flags", entity.livingFlags());
        return state;
    }

    static JsonObject clientWorldEntityEffectAmbienceResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityEffectAmbienceState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityEffectAmbienceState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity effect ambience fixture failed", error);
        }
    }

    static JsonObject entityEffectAmbienceState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("living_flags", entity.livingFlags());
        state.addProperty("effect_ambience", entity.effectAmbience());
        return state;
    }

    static JsonObject clientWorldEntityEffectParticlesResult(JsonObject request) {
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

            if (request.has("spawn")) {
                return effectParticleSpawns(request.getAsJsonObject("spawn"));
            }
            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityEffectParticlesState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(
                        operation.get("metadata_hex").getAsString(),
                        RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY)));
                states.add(entityEffectParticlesState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity effect particles fixture failed", error);
        }
    }

    /**
     * `LivingEntity.tickEffects` on a client, ticked with a seeded source: the particles it adds,
     * in order, with the tick each came on.
     */
    static JsonObject effectParticleSpawns(JsonObject spawn) {
        var entity = new EffectParticlesFixtureEntity();
        entity.setPos(1.25, 80.0, -35.5);
        var particles = new java.util.ArrayList<net.minecraft.core.particles.ParticleOptions>();
        for (JsonElement element : spawn.getAsJsonArray("particles")) {
            var described = element.getAsJsonObject();
            var type = BuiltInRegistries.PARTICLE_TYPE.byId(described.get("type_id").getAsInt());
            particles.add(described.has("color")
                ? net.minecraft.core.particles.ColorParticleOption.create(
                    net.minecraft.core.particles.ParticleTypes.ENTITY_EFFECT, described.get("color").getAsInt())
                : (net.minecraft.core.particles.SimpleParticleType) type);
        }
        entity.setEffectParticles(particles, spawn.get("ambient").getAsBoolean());
        entity.setInvisible(spawn.get("invisible").getAsBoolean());
        entity.getRandom().setSeed(spawn.get("seed").getAsLong());
        ParticleRecordingLevel.ADDED.clear();
        var added = new JsonArray();
        for (int tick = 0; tick < spawn.get("ticks").getAsInt(); tick++) {
            entity.tickEffectsOnce();
            for (var call : ParticleRecordingLevel.ADDED) {
                var particle = (net.minecraft.core.particles.ParticleOptions) call[0];
                var described = new JsonObject();
                described.addProperty("tick", tick);
                described.addProperty("type_id", BuiltInRegistries.PARTICLE_TYPE.getId(particle.getType()));
                for (int index = 1; index < 7; index++) {
                    described.addProperty(AXES[index - 1], Oracle.bits((double) call[index]));
                }
                added.add(described);
            }
            ParticleRecordingLevel.ADDED.clear();
        }
        var result = new JsonObject();
        result.add("added", added);
        return result;
    }

    private static final String[] AXES = {"x", "y", "z", "xd", "yd", "zd"};

    static JsonObject entityEffectParticlesState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("living_flags", entity.livingFlags());
        state.addProperty("effect_ambience", entity.effectAmbience());
        var particles = new JsonArray();
        for (var particle : entity.effectParticles()) {
            var described = new JsonObject();
            described.addProperty("type_id",
                net.minecraft.core.registries.BuiltInRegistries.PARTICLE_TYPE.getId(particle.getType()));
            if (particle instanceof net.minecraft.core.particles.ColorParticleOption) {
                try {
                    described.addProperty("color", Reflect.field(
                        net.minecraft.core.particles.ColorParticleOption.class, "color").getInt(particle));
                } catch (ReflectiveOperationException error) {
                    throw new IllegalStateException(error);
                }
            }
            particles.add(described);
        }
        state.add("effect_particles", particles);
        return state;
    }

    static JsonObject clientWorldEntitySleepingPositionResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entitySleepingPositionState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entitySleepingPositionState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity sleeping position fixture failed", error);
        }
    }

    static JsonObject entitySleepingPositionState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("living_flags", entity.livingFlags());
        entity.getSleepingPos().ifPresentOrElse(
            position -> state.add("sleeping_pos", snapshot(position)),
            () -> state.add("sleeping_pos", JsonNull.INSTANCE));
        state.addProperty("is_sleeping", entity.isSleeping());
        return state;
    }

    static JsonObject clientWorldEntityStingerCountResult(JsonObject request) {
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

            var entity = new LivingFlagsFixtureEntity();
            var states = new JsonArray();
            states.add(entityStingerCountState("initial", true, entity));
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                entity.getEntityData().assignValues(
                    decodeEntityMetadata(operation.get("metadata_hex").getAsString()));
                states.add(entityStingerCountState(
                    operation.get("label").getAsString(), true, entity));
            }
            var result = new JsonObject();
            result.add("states", states);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("Entity stinger count fixture failed", error);
        }
    }

    static JsonObject entityStingerCountState(
        String label, boolean accepted, LivingFlagsFixtureEntity entity
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        state.addProperty("stinger_count", entity.getStingerCount());
        state.addProperty("living_flags", entity.livingFlags());
        return state;
    }
}
