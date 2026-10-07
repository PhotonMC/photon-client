import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;

/** Sky flash, elytra, leash, pick table and light changes in a scene. */
class MovementWorldOracle extends MovementLivingOracle {
    /**
     * The sky colour and the sky light factor the real client level's attribute system gives
     * with the flash time set, with the option that hides the flash on or off: where nothing is
     * loaded (plains, no weather, the clocks at zero).
     */
    static JsonObject skyFlash(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        setField(scene.minecraft(), "level", level);
        scene.options().hideLightningFlash().set(row.get("hide").getAsBoolean());
        level.setSkyFlashTime(row.get("flash").getAsInt());
        level.environmentAttributes().invalidateTickCache();
        var at = new net.minecraft.world.phys.Vec3(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"));
        int color = level.environmentAttributes().getValue(
            net.minecraft.world.attribute.EnvironmentAttributes.SKY_COLOR, at);
        float factor = level.environmentAttributes().getValue(
            net.minecraft.world.attribute.EnvironmentAttributes.SKY_LIGHT_FACTOR, at);
        entry.addProperty("sky_color", color);
        entry.addProperty("sky_light_factor", Oracle.bits(factor));
        return entry;
    }

    /**
     * The real `ElytraAnimationState` of a real remote player ticked through a scenario: whether it
     * glides, its motion and whether it sneaks each tick, and the spread of the wings after each.
     */
    static JsonObject elytraState(final JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var setFlag = net.minecraft.world.entity.Entity.class.getDeclaredMethod("setSharedFlag", int.class, boolean.class);
        setFlag.setAccessible(true);
        {
            var empty = new JsonObject();
            empty.add("blocks", new JsonArray());
            empty.addProperty("chunk_radius", 0);
            var scene = buildScene(empty);
            var player = new net.minecraft.client.player.RemotePlayer(scene.level(),
                new com.mojang.authlib.GameProfile(java.util.UUID.nameUUIDFromBytes(new byte[] {5}), "Glider"));
            var state = new net.minecraft.world.entity.ElytraAnimationState(player);
            var ticks = new JsonArray();
            for (var tickElement : row.getAsJsonArray("ticks")) {
                var tick = tickElement.getAsJsonObject();
                setFlag.invoke(player, 7, tick.get("fall_flying").getAsBoolean());
                player.setPose(tick.get("crouching").getAsBoolean()
                    ? net.minecraft.world.entity.Pose.CROUCHING : net.minecraft.world.entity.Pose.STANDING);
                player.setDeltaMovement(Oracle.f64(tick, "x"), Oracle.f64(tick, "y"), Oracle.f64(tick, "z"));
                state.tick();
                var out = new JsonObject();
                for (var partial : new float[] {0.0F, 0.5F, 1.0F}) {
                    var spread = new JsonArray();
                    spread.add(Oracle.bits(state.getRotX(partial)));
                    spread.add(Oracle.bits(state.getRotY(partial)));
                    spread.add(Oracle.bits(state.getRotZ(partial)));
                    out.add("at_" + partial, spread);
                }
                ticks.add(out);
            }
            entry.add("ticks", ticks);
        }
        return entry;
    }

    /**
     * What a leash needs of an entity: whether it can be leashed, where the line is tied to it
     * (the standing camel's, which moves with its sitting animation), whether it is tied by four
     * lines and where, and the same for a holder.
     */
    static void leashSupport(JsonObject out, net.minecraft.world.entity.Entity entity, String prefix) {
        if (entity instanceof net.minecraft.world.entity.Leashable leashable) {
            out.addProperty(prefix + "leashable", true);
            out.add(prefix + "leash_offset", vectorBits(leashable.getLeashOffset(0.0F)));
            out.addProperty(prefix + "quad", leashable.supportQuadLeash());
            if (leashable.supportQuadLeash()) {
                var offsets = new JsonArray();
                for (var offset : leashable.getQuadLeashOffsets()) {
                    offsets.add(vectorBits(offset));
                }
                out.add(prefix + "quad_offsets", offsets);
            }
        }
        out.addProperty(prefix + "quad_holder", entity.supportQuadLeashAsHolder());
        if (entity.supportQuadLeashAsHolder()) {
            var offsets = new JsonArray();
            for (var offset : entity.getQuadLeashHolderOffsets()) {
                offsets.add(vectorBits(offset));
            }
            out.add(prefix + "quad_holder_offsets", offsets);
        }
    }

    /**
     * Every entity type as a new entity of it answers the crosshair:
     * `isPickable`, `getPickRadius`, `canBePickedFromInside`, and its size in
     * the standing pose, a baby's too where the type has babies. A type that
     * makes no entity here (the player) has no row of answers.
     */
    static JsonObject entityPickTable(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        setField(scene.minecraft(), "level", level);
        setField(scene.listener(), "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // Teams are looked up by entities that build boss bars.
        setField(scene.listener(), "scoreboard", new net.minecraft.world.scores.Scoreboard());
        var types = new JsonArray();
        var registry = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        for (var type : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(type).toString());
            out.addProperty("id", registry.getId(type));
            net.minecraft.world.entity.Entity entity = null;
            try {
                entity = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                    net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            } catch (Throwable failure) {
                var where = failure.getStackTrace().length > 0 ? failure.getStackTrace()[0] : null;
                out.addProperty("error", failure.getClass().getSimpleName() + " at " + where);
            }
            out.addProperty("created", entity != null);
            if (entity != null) {
                out.addProperty("pickable", entity.isPickable());
                out.addProperty("pick_radius", Oracle.bits(entity.getPickRadius()));
                out.addProperty("from_inside", entity.canBePickedFromInside());
                out.addProperty("monster", entity instanceof net.minecraft.world.entity.monster.Monster);
                var size = entity.getDimensions(net.minecraft.world.entity.Pose.STANDING);
                out.addProperty("width", Oracle.bits(size.width()));
                out.addProperty("height", Oracle.bits(size.height()));
                out.addProperty("eye_height", Oracle.bits(size.eyeHeight()));
                // Where each passenger sits (`EntityAttachment.PASSENGER`, not yet turned).
                var seats = new JsonArray();
                for (int seat = 0; seat < 8; seat++) {
                    var point = size.attachments().getNullable(
                        net.minecraft.world.entity.EntityAttachment.PASSENGER, seat, 0.0F);
                    if (point == null) {
                        break;
                    }
                    seats.add(vectorBits(point));
                }
                out.add("passengers", seats);
                leashSupport(out, entity, "");
                if (entity instanceof net.minecraft.world.entity.Mob mob) {
                    mob.setBaby(true);
                    if (mob.isBaby()) {
                        var baby = mob.getDimensions(net.minecraft.world.entity.Pose.STANDING);
                        out.addProperty("baby_width", Oracle.bits(baby.width()));
                        out.addProperty("baby_height", Oracle.bits(baby.height()));
                        out.addProperty("baby_eye_height", Oracle.bits(baby.eyeHeight()));
                        leashSupport(out, entity, "baby_");
                    }
                }
            }
            types.add(out);
        }
        entry.add("types", types);
        return entry;
    }

    /**
     * The client level's own light engine, lit from nothing and then kept
     * current through block updates.
     *
     * The level's light starts the way the server first lights a chunk: every
     * section that holds blocks is marked, every column is switched on, and
     * the light sources are spread. Each event then arrives as a block update
     * from the server would, and the engine runs as the frame loop runs it.
     * After the start and after every event, the block and sky light of a box
     * are reported, as the level's light getters read them.
     */
    static JsonObject lightChanges(JsonObject row) throws Exception {
        var scene = buildScene(row);
        var level = scene.level();
        var extractor = unsafe.allocateInstance(NoRebuilds.class);
        setField(level, "levelExtractor", extractor);
        setField(scene.minecraft(), "levelExtractor", extractor);
        var engine = level.getChunkSource().getLightEngine();
        int radius = row.get("chunk_radius").getAsInt();
        var chunks = new java.util.ArrayList<net.minecraft.world.level.ChunkPos>();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                chunks.add(new net.minecraft.world.level.ChunkPos(chunkX, chunkZ));
            }
        }
        for (var chunkPos : chunks) {
            var chunk = level.getChunk(chunkPos.x(), chunkPos.z());
            var sections = chunk.getSections();
            for (int index = 0; index < sections.length; index++) {
                if (!sections[index].hasOnlyAir()) {
                    engine.updateSectionStatus(net.minecraft.core.SectionPos.of(
                        chunkPos, level.getSectionYFromSectionIndex(index)), false);
                }
            }
        }
        for (var chunkPos : chunks) {
            engine.setLightEnabled(chunkPos, true);
        }
        for (var chunkPos : chunks) {
            engine.propagateLightSources(chunkPos);
        }
        engine.runLightUpdates();

        var box = row.getAsJsonObject("box");
        var snapshots = new JsonArray();
        snapshots.add(lightSnapshot(engine, box));
        for (var element : row.get("events").getAsJsonArray()) {
            var event = element.getAsJsonObject();
            level.setServerVerifiedBlockState(
                new BlockPos(event.get("x").getAsInt(), event.get("y").getAsInt(), event.get("z").getAsInt()),
                state(event), 19);
            engine.runLightUpdates();
            snapshots.add(lightSnapshot(engine, box));
        }
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "lit");
        entry.add("snapshots", snapshots);
        return entry;
    }

    /** Block then sky light of every position in the box, y, then z, then x, as hex digits. */
    static JsonObject lightSnapshot(
        net.minecraft.world.level.lighting.LevelLightEngine engine, JsonObject box) {
        var out = new JsonObject();
        for (var layer : net.minecraft.world.level.LightLayer.values()) {
            var listener = engine.getLayerListener(layer);
            var values = new StringBuilder();
            for (int y = 0; y < box.get("size_y").getAsInt(); y++) {
                for (int z = 0; z < box.get("size_z").getAsInt(); z++) {
                    for (int x = 0; x < box.get("size_x").getAsInt(); x++) {
                        values.append(Integer.toHexString(listener.getLightValue(new BlockPos(
                            box.get("x").getAsInt() + x, box.get("y").getAsInt() + y,
                            box.get("z").getAsInt() + z))));
                    }
                }
            }
            out.addProperty(layer == net.minecraft.world.level.LightLayer.BLOCK ? "block" : "sky",
                values.toString());
        }
        return out;
    }
}
