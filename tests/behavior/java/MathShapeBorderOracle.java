import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.client.renderer.WorldBorderRenderer;
import net.minecraft.client.renderer.state.level.WorldBorderRenderState;
import net.minecraft.util.Mth;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathShapeBorderOracle extends MathSectionOracle {
    static JsonObject shapeworldBorderCollisionShapeCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
            border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
            border.setSize(f64(input, "size"));
            states.add(snapshot(border.getCollisionShape()));
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeworldBorderCollisionShapeMovingCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
            border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
            border.lerpSizeBetween(
                f64(input, "from"),
                f64(input, "to"),
                input.get("duration").getAsLong(),
                0L
            );
            var partialTicks = input.getAsJsonArray("partial_ticks");
            int tickCount = input.get("tick_count").getAsInt();
            var track = new JsonArray();
            for (int tick = 0; tick <= tickCount; tick++) {
                var samples = new JsonArray();
                for (var partialEntry : partialTicks) {
                    float partialTick = Float.intBitsToFloat(
                        Integer.parseUnsignedInt(partialEntry.getAsString(), 16));
                    var sample = new JsonObject();
                    sample.addProperty("partial_tick", Oracle.bits(partialTick));
                    sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                    sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                    sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                    sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                    samples.add(sample);
                }
                var state = new JsonObject();
                state.addProperty("size", Oracle.bits(border.getSize()));
                state.addProperty("lerp_time", border.getLerpTime());
                state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                state.addProperty("status", border.getStatus().name());
                state.add("samples", samples);
                state.add("collision_shape", snapshot(border.getCollisionShape()));
                track.add(state);
                if (tick < tickCount) {
                    border.tick();
                }
            }
            states.add(track);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeclientWorldBorderStateCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var tracks = new JsonArray();
        for (var trackEntry : request.getAsJsonArray("inputs")) {
            WorldBorder border = new WorldBorder();
            long gameTime = 0L;
            var states = new JsonArray();
            for (var eventEntry : trackEntry.getAsJsonObject().getAsJsonArray("events")) {
                JsonObject event = eventEntry.getAsJsonObject();
                String eventKind = event.get("op").getAsString();
                switch (eventKind) {
                    case "initialize" -> {
                        border.setCenter(f64(event, "center_x"), f64(event, "center_z"));
                        long lerpTime = event.get("lerp_time").getAsLong();
                        if (lerpTime > 0L) {
                            border.lerpSizeBetween(
                                f64(event, "old_size"),
                                f64(event, "new_size"),
                                lerpTime,
                                gameTime
                            );
                        } else {
                            border.setSize(f64(event, "new_size"));
                        }
                        border.setAbsoluteMaxSize(event.get("absolute_max_size").getAsInt());
                        border.setWarningBlocks(event.get("warning_blocks").getAsInt());
                        border.setWarningTime(event.get("warning_time").getAsInt());
                    }
                    case "center" -> border.setCenter(
                        f64(event, "center_x"), f64(event, "center_z"));
                    case "lerp" -> border.lerpSizeBetween(
                        f64(event, "old_size"),
                        f64(event, "new_size"),
                        event.get("lerp_time").getAsLong(),
                        gameTime
                    );
                    case "size" -> border.setSize(f64(event, "size"));
                    case "warning_blocks" -> border.setWarningBlocks(event.get("value").getAsInt());
                    case "warning_time" -> border.setWarningTime(event.get("value").getAsInt());
                    case "tick" -> {
                        border.tick();
                        gameTime++;
                    }
                    default -> throw new IllegalArgumentException("Unknown world border event");
                }
                var samples = new JsonArray();
                for (float partialTick : new float[] {0.0F, 0.5F, 1.0F}) {
                    var sample = new JsonObject();
                    sample.addProperty("partial_tick", Oracle.bits(partialTick));
                    sample.addProperty("min_x", Oracle.bits(border.getMinX(partialTick)));
                    sample.addProperty("max_x", Oracle.bits(border.getMaxX(partialTick)));
                    sample.addProperty("min_z", Oracle.bits(border.getMinZ(partialTick)));
                    sample.addProperty("max_z", Oracle.bits(border.getMaxZ(partialTick)));
                    samples.add(sample);
                }
                var state = new JsonObject();
                state.addProperty("event", eventKind);
                state.addProperty("game_time", gameTime);
                state.addProperty("center_x", Oracle.bits(border.getCenterX()));
                state.addProperty("center_z", Oracle.bits(border.getCenterZ()));
                state.addProperty("absolute_max_size", border.getAbsoluteMaxSize());
                state.addProperty("warning_blocks", border.getWarningBlocks());
                state.addProperty("warning_time", border.getWarningTime());
                state.addProperty("size", Oracle.bits(border.getSize()));
                state.addProperty("lerp_time", border.getLerpTime());
                state.addProperty("lerp_target", Oracle.bits(border.getLerpTarget()));
                state.addProperty("status", border.getStatus().name());
                state.add("samples", samples);
                state.add("collision_shape", snapshot(border.getCollisionShape()));
                states.add(state);
            }
            tracks.add(states);
        }
        result.add("tracks", tracks);
        return result;
    }

    /**
     * {@code WorldBorderRenderer.extract} and {@code closestBorder}, called
     * on the real renderer, plus the quad geometry {@code
     * rebuildWorldBorderBuffer} lays out. The renderer is allocated without
     * its constructor (which eagerly creates a GPU vertex buffer the
     * headless oracle has no device for); {@code extract} never touches
     * that field, so the real method still runs. The quad math is
     * transcribed from the private {@code rebuildWorldBorderBuffer} using
     * the same real {@code Mth} calls, since that method's own last step
     * uploads to the same GPU buffer.
     */
    static JsonObject shapeworldBorderRenderCase(JsonObject request) {
        var result = new JsonObject();
        SharedConstants.setVersion(TEST_VERSION);
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(input, "center_x"), f64(input, "center_z"));
            border.setAbsoluteMaxSize(input.get("absolute_max_size").getAsInt());
            if (input.has("from")) {
                border.lerpSizeBetween(
                    f64(input, "from"), f64(input, "to"), input.get("duration").getAsLong(), 0L);
                int ticks = input.get("ticks").getAsInt();
                for (int tick = 0; tick < ticks; tick++) {
                    border.tick();
                }
            } else {
                border.setSize(f64(input, "size"));
            }
            float partialTick = Oracle.f32(input, "partial_tick");
            Vec3 cameraPos = new Vec3(f64(input, "camera_x"), f64(input, "camera_y"), f64(input, "camera_z"));
            double renderDistance = f64(input, "render_distance");
            float halfHeightY = Oracle.f32(input, "half_height_y");
            long timeMillis = input.get("time_millis").getAsLong();

            WorldBorderRenderState state = new WorldBorderRenderState();
            WorldBorderRenderer renderer;
            try {
                Unsafe unsafe = MathSupport.unsafeInstance();
                renderer = (WorldBorderRenderer)unsafe.allocateInstance(WorldBorderRenderer.class);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate a border renderer without its GPU buffer", error);
            }
            renderer.extract(border, partialTick, cameraPos, renderDistance, state);

            var closest = new JsonArray();
            for (var side : state.closestBorder(cameraPos.x, cameraPos.z)) {
                var entrySide = new JsonObject();
                entrySide.addProperty("direction", side.direction().name());
                entrySide.addProperty("distance", Oracle.bits(side.distance()));
                closest.add(entrySide);
            }

            var out = new JsonObject();
            out.addProperty("alpha", Oracle.bits(state.alpha));
            out.addProperty("tint_red", Oracle.bits(net.minecraft.util.ARGB.red(state.tint) / 255.0F));
            out.addProperty("tint_green", Oracle.bits(net.minecraft.util.ARGB.green(state.tint) / 255.0F));
            out.addProperty("tint_blue", Oracle.bits(net.minecraft.util.ARGB.blue(state.tint) / 255.0F));
            out.add("closest", closest);
            out.add(
                "quads",
                state.alpha <= 0.0
                    ? new JsonArray()
                    : worldBorderQuads(state, cameraPos, renderDistance, halfHeightY, timeMillis));
            states.add(out);
        }
        result.add("states", states);
        return result;
    }

    private static JsonArray worldBorderQuads(
        WorldBorderRenderState state, Vec3 cameraPos, double renderDistance, float halfHeightY, long timeMillis
    ) {
        double cameraX = cameraPos.x;
        double cameraY = cameraPos.y;
        double cameraZ = cameraPos.z;
        double borderMinX = state.minX;
        double borderMaxX = state.maxX;
        double borderMinZ = state.minZ;
        double borderMaxZ = state.maxZ;
        double minZ = Math.max(Mth.floor(cameraZ - renderDistance), borderMinZ);
        double maxZ = Math.min(Mth.ceil(cameraZ + renderDistance), borderMaxZ);
        float u0z = (Mth.floor(minZ) & 1) * 0.5F;
        float u1z = (float)(maxZ - minZ) / 2.0F;
        double minX = Math.max(Mth.floor(cameraX - renderDistance), borderMinX);
        double maxX = Math.min(Mth.ceil(cameraX + renderDistance), borderMaxX);
        float u0x = (Mth.floor(minX) & 1) * 0.5F;
        float u1x = (float)(maxX - minX) / 2.0F;
        float offset = (float)(timeMillis % 3000L) / 3000.0F;
        float v0 = (float)(-Mth.frac(cameraY * 0.5));
        float v1 = v0 + halfHeightY;
        float y0 = -halfHeightY;
        float y1 = halfHeightY;

        var quads = new JsonArray();
        for (var side : state.closestBorder(cameraX, cameraZ)) {
            if (!(side.distance() < renderDistance)) {
                continue;
            }
            float[][] corners = switch (side.direction()) {
                case SOUTH -> new float[][] {
                    {(float)minX, y0, (float)borderMaxZ, u0x, v1},
                    {(float)maxX, y0, (float)borderMaxZ, u1x + u0x, v1},
                    {(float)maxX, y1, (float)borderMaxZ, u1x + u0x, v0},
                    {(float)minX, y1, (float)borderMaxZ, u0x, v0},
                };
                case WEST -> new float[][] {
                    {(float)minX, y0, (float)minZ, u0z, v1},
                    {(float)minX, y0, (float)maxZ, u1z + u0z, v1},
                    {(float)minX, y1, (float)maxZ, u1z + u0z, v0},
                    {(float)minX, y1, (float)minZ, u0z, v0},
                };
                case NORTH -> new float[][] {
                    {(float)maxX, y0, (float)minZ, u0x, v1},
                    {(float)minX, y0, (float)minZ, u1x + u0x, v1},
                    {(float)minX, y1, (float)minZ, u1x + u0x, v0},
                    {(float)maxX, y1, (float)minZ, u0x, v0},
                };
                default -> new float[][] {
                    {(float)borderMaxX, y0, (float)maxZ, u0z, v1},
                    {(float)borderMaxX, y0, (float)minZ, u1z + u0z, v1},
                    {(float)borderMaxX, y1, (float)minZ, u1z + u0z, v0},
                    {(float)borderMaxX, y1, (float)maxZ, u0z, v0},
                };
            };
            var quad = new JsonObject();
            quad.addProperty("side", side.direction().name());
            var cornersJson = new JsonArray();
            for (float[] corner : corners) {
                var c = new JsonObject();
                c.addProperty("x", Oracle.bits(corner[0]));
                c.addProperty("y", Oracle.bits(corner[1]));
                c.addProperty("z", Oracle.bits(corner[2]));
                c.addProperty("u", Oracle.bits(corner[3] + offset));
                c.addProperty("v", Oracle.bits(corner[4] + offset));
                cornersJson.add(c);
            }
            quad.add("corners", cornersJson);
            quads.add(quad);
        }
        return quads;
    }
}
