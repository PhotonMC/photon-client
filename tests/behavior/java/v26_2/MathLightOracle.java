import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import java.util.BitSet;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.DataLayer;
import net.minecraft.world.level.chunk.LightChunk;
import net.minecraft.world.level.chunk.LightChunkGetter;
import net.minecraft.world.level.lighting.LevelLightEngine;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;

class MathLightOracle extends MathKeyOracle {
    static final class LightEngineFixture implements LightChunkGetter {
        final BlockGetter level;

        LightEngineFixture(int minY, int height) {
            this.level = new BlockGetter() {
                @Override
                public int getHeight() {
                    return height;
                }

                @Override
                public int getMinY() {
                    return minY;
                }

                @Override
                public BlockEntity getBlockEntity(BlockPos pos) {
                    return null;
                }

                @Override
                public BlockState getBlockState(BlockPos pos) {
                    return Blocks.AIR.defaultBlockState();
                }

                @Override
                public FluidState getFluidState(BlockPos pos) {
                    return Fluids.EMPTY.defaultFluidState();
                }
            };
        }

        @Override
        public LightChunk getChunkForLighting(int x, int z) {
            return null;
        }

        @Override
        public BlockGetter getLevel() {
            return this.level;
        }
    }

    static BitSet lightBitSet(JsonObject object, String name) {
        var bits = new BitSet();
        for (JsonElement entry : object.getAsJsonArray(name)) {
            bits.set(entry.getAsInt());
        }
        return bits;
    }

    static int queueLightLayer(
        LevelLightEngine engine,
        int chunkX,
        int chunkZ,
        int minLightSection,
        LightLayer layer,
        BitSet mask,
        BitSet emptyMask,
        JsonArray updates
    ) {
        int updateIndex = 0;
        int applied = 0;
        for (int sectionIndex = 0; sectionIndex < engine.getLightSectionCount(); sectionIndex++) {
            boolean hasData = mask.get(sectionIndex);
            boolean hasEmpty = emptyMask.get(sectionIndex);
            if (!hasData && !hasEmpty) {
                continue;
            }

            DataLayer data;
            if (hasData) {
                if (updateIndex >= updates.size()) {
                    throw new IllegalArgumentException("light update data is shorter than its mask");
                }
                data = new DataLayer(hexBytes(updates.get(updateIndex++).getAsString()));
            } else {
                data = new DataLayer();
            }
            engine.queueSectionData(
                layer,
                SectionPos.of(chunkX, minLightSection + sectionIndex, chunkZ),
                data
            );
            applied++;
        }
        return applied;
    }

    static int lightDataValue(
        LevelLightEngine engine, LightLayer layer, int x, int y, int z
    ) {
        SectionPos section = SectionPos.of(
            Math.floorDiv(x, 16), Math.floorDiv(y, 16), Math.floorDiv(z, 16));
        DataLayer data = engine.getLayerListener(layer).getDataLayerData(section);
        if (data == null) {
            return -1;
        }
        return data.get(Math.floorMod(x, 16), Math.floorMod(y, 16), Math.floorMod(z, 16));
    }

    static boolean hasLightData(
        LevelLightEngine engine,
        LightLayer layer,
        int chunkX,
        int chunkZ,
        int minLightSection,
        BitSet mask,
        BitSet emptyMask
    ) {
        for (int sectionIndex = 0; sectionIndex < engine.getLightSectionCount(); sectionIndex++) {
            if ((mask.get(sectionIndex) || emptyMask.get(sectionIndex))
                && engine.getLayerListener(layer).getDataLayerData(
                    SectionPos.of(chunkX, minLightSection + sectionIndex, chunkZ)) != null) {
                return true;
            }
        }
        return false;
    }

    static JsonArray lightQueries(LevelLightEngine engine, JsonArray queryEntries) {
        var queries = new JsonArray();
        for (JsonElement queryEntry : queryEntries) {
            JsonObject query = queryEntry.getAsJsonObject();
            String layerName = query.get("layer").getAsString();
            LightLayer layer = switch (layerName) {
                case "sky" -> LightLayer.SKY;
                case "block" -> LightLayer.BLOCK;
                default -> throw new IllegalArgumentException("Unknown light layer: " + layerName);
            };
            int value = lightDataValue(
                engine,
                layer,
                query.get("x").getAsInt(),
                query.get("y").getAsInt(),
                query.get("z").getAsInt()
            );
            var state = new JsonObject();
            state.addProperty("label", query.get("label").getAsString());
            if (value < 0) {
                state.add("value", JsonNull.INSTANCE);
            } else {
                state.addProperty("value", value);
            }
            queries.add(state);
        }
        return queries;
    }

    static JsonObject clientWorldLightResult(JsonObject request) {
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

            int minY = request.get("dimension_min_y").getAsInt();
            int height = request.get("dimension_height").getAsInt();
            var engine = new LevelLightEngine(new LightEngineFixture(minY, height), true, true);
            int minLightSection = engine.getMinLightSection();
            var events = new JsonArray();
            boolean targetLoaded = false;
            for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                String action = operation.get("action").getAsString();
                var event = new JsonObject();
                event.addProperty("action", action);
                event.addProperty("label", operation.get("label").getAsString());
                event.addProperty("queued_for_unloaded_chunk", false);
                event.addProperty("stored", false);
                event.addProperty("applied_sky_sections", 0);
                event.addProperty("applied_block_sections", 0);

                if (action.equals("apply")) {
                    int chunkX = operation.get("chunk_x").getAsInt();
                    int chunkZ = operation.get("chunk_z").getAsInt();
                    BitSet skyMask = lightBitSet(operation, "sky_mask");
                    BitSet skyEmptyMask = lightBitSet(operation, "sky_empty_mask");
                    BitSet blockMask = lightBitSet(operation, "block_mask");
                    BitSet blockEmptyMask = lightBitSet(operation, "block_empty_mask");
                    int appliedSky = queueLightLayer(
                        engine, chunkX, chunkZ, minLightSection, LightLayer.SKY,
                        skyMask, skyEmptyMask, operation.getAsJsonArray("sky_updates"));
                    int appliedBlock = queueLightLayer(
                        engine, chunkX, chunkZ, minLightSection, LightLayer.BLOCK,
                        blockMask, blockEmptyMask, operation.getAsJsonArray("block_updates"));
                    engine.setLightEnabled(new ChunkPos(chunkX, chunkZ), true);
                    engine.runLightUpdates();
                    event.addProperty("queued_for_unloaded_chunk", !targetLoaded);
                    event.addProperty(
                        "stored",
                        hasLightData(engine, LightLayer.SKY, chunkX, chunkZ,
                            minLightSection, skyMask, skyEmptyMask)
                            || hasLightData(engine, LightLayer.BLOCK, chunkX, chunkZ,
                                minLightSection, blockMask, blockEmptyMask));
                    event.addProperty("applied_sky_sections", appliedSky);
                    event.addProperty("applied_block_sections", appliedBlock);
                } else if (action.equals("load")) {
                    int chunkX = operation.get("chunk_x").getAsInt();
                    int chunkZ = operation.get("chunk_z").getAsInt();
                    int sectionCount = operation.get("section_count").getAsInt();
                    BitSet nonEmptySections = lightBitSet(operation, "non_empty_sections");
                    int minSectionY = Math.floorDiv(minY, 16);
                    for (int sectionIndex = 0; sectionIndex < sectionCount; sectionIndex++) {
                        engine.updateSectionStatus(
                            SectionPos.of(chunkX, minSectionY + sectionIndex, chunkZ),
                            !nonEmptySections.get(sectionIndex)
                        );
                    }
                    engine.runLightUpdates();
                    targetLoaded = true;
                    event.addProperty(
                        "stored",
                        engine.getLayerListener(LightLayer.SKY).getDataLayerData(
                            SectionPos.of(chunkX, minSectionY, chunkZ)) != null
                        || engine.getLayerListener(LightLayer.BLOCK).getDataLayerData(
                            SectionPos.of(chunkX, minSectionY, chunkZ)) != null);
                } else if (action.equals("unload")) {
                    int chunkX = operation.get("chunk_x").getAsInt();
                    int chunkZ = operation.get("chunk_z").getAsInt();
                    ChunkPos chunkPos = new ChunkPos(chunkX, chunkZ);
                    engine.setLightEnabled(chunkPos, false);
                    for (int sectionY = engine.getMinLightSection();
                         sectionY < engine.getMaxLightSection(); sectionY++) {
                        SectionPos sectionPos = SectionPos.of(chunkPos, sectionY);
                        engine.queueSectionData(LightLayer.BLOCK, sectionPos, null);
                        engine.queueSectionData(LightLayer.SKY, sectionPos, null);
                    }
                    int minSectionY = Math.floorDiv(minY, 16);
                    int maxSectionY = Math.floorDiv(minY + height - 1, 16);
                    for (int sectionY = minSectionY; sectionY <= maxSectionY; sectionY++) {
                        engine.updateSectionStatus(
                            SectionPos.of(chunkPos, sectionY), true);
                    }
                    engine.runLightUpdates();
                    targetLoaded = false;
                } else {
                    throw new IllegalArgumentException("Unknown client world light action: " + action);
                }

                JsonArray queryEntries = operation.has("queries")
                    ? operation.getAsJsonArray("queries")
                    : new JsonArray();
                event.add("queries", lightQueries(engine, queryEntries));
                events.add(event);
            }
            var result = new JsonObject();
            result.add("events", events);
            return result;
        } catch (RuntimeException error) {
            throw new IllegalStateException("LevelLightEngine light queue fixture failed", error);
        }
    }
}
