import com.google.gson.JsonArray;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import io.netty.buffer.Unpooled;
import com.mojang.serialization.Lifecycle;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.server.Bootstrap;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainerFactory;

class MathWorldLevelOracle extends MathVectorOracle {
    static JsonObject clientWorldSectionLookupResult(JsonObject request) {
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
            int minSectionY = Math.floorDiv(minY, 16);
            var vanillaBiomes = VanillaRegistries.createLookup().lookupOrThrow(Registries.BIOME);
            var biomeRegistry = new MappedRegistry<net.minecraft.world.level.biome.Biome>(
                Registries.BIOME, Lifecycle.stable());
            vanillaBiomes.listElements().forEach(holder ->
                Registry.register(biomeRegistry, holder.key(), holder.value()));
            biomeRegistry.freeze();
            var registryAccess = new RegistryAccess.ImmutableRegistryAccess(List.of(biomeRegistry));
            var containerFactory = PalettedContainerFactory.create(registryAccess);
            var nonEmptyField = Reflect.field(LevelChunkSection.class, "nonEmptyBlockCount");
            var fluidField = Reflect.field(LevelChunkSection.class, "fluidCount");
            var tracks = new JsonArray();
            for (var trackEntry : request.getAsJsonArray("tracks")) {
                JsonObject track = trackEntry.getAsJsonObject();
                int chunkX = track.get("chunk_x").getAsInt();
                int chunkZ = track.get("chunk_z").getAsInt();
                var sections = new ArrayList<LevelChunkSection>();
                for (var sectionEntry : track.getAsJsonArray("sections")) {
                    LevelChunkSection section = new LevelChunkSection(containerFactory);
                    FriendlyByteBuf buffer = new FriendlyByteBuf(
                        Unpooled.wrappedBuffer(hexBytes(sectionEntry.getAsString())));
                    try {
                        section.read(buffer);
                        if (buffer.isReadable()) {
                            throw new IllegalArgumentException("section payload has trailing bytes");
                        }
                    } finally {
                        buffer.release();
                    }
                    sections.add(section);
                }

                var sectionStates = new JsonArray();
                for (int index = 0; index < sections.size(); index++) {
                    LevelChunkSection section = sections.get(index);
                    int nonEmpty = nonEmptyField.getShort(section);
                    int fluid = fluidField.getShort(section);
                    var state = new JsonObject();
                    state.addProperty("section_y", minSectionY + index);
                    state.addProperty("non_empty_block_count", nonEmpty);
                    state.addProperty("fluid_count", fluid);
                    state.addProperty("has_only_air", section.hasOnlyAir());
                    state.addProperty("has_fluid", section.hasFluid());
                    sectionStates.add(state);
                }

                var queries = new JsonArray();
                for (var queryEntry : track.getAsJsonArray("queries")) {
                    JsonObject query = queryEntry.getAsJsonObject();
                    int x = query.get("x").getAsInt();
                    int y = query.get("y").getAsInt();
                    int z = query.get("z").getAsInt();
                    int querySectionY = Math.floorDiv(y, 16);
                    int sectionIndex = querySectionY - minSectionY;
                    LevelChunkSection section = Math.floorDiv(x, 16) == chunkX
                        && Math.floorDiv(z, 16) == chunkZ
                        && sectionIndex >= 0
                        && sectionIndex < sections.size()
                        ? sections.get(sectionIndex)
                        : null;
                    var state = new JsonObject();
                    state.addProperty("label", query.get("label").getAsString());
                    state.addProperty("section_y", querySectionY);
                    if (section == null) {
                        state.add("block_state_id", JsonNull.INSTANCE);
                        state.add("biome_id", JsonNull.INSTANCE);
                        state.add("section_counts", JsonNull.INSTANCE);
                    } else {
                        int localX = Math.floorMod(x, 16);
                        int localY = Math.floorMod(y, 16);
                        int localZ = Math.floorMod(z, 16);
                        state.addProperty(
                            "block_state_id",
                            Block.getId(section.getBlockState(localX, localY, localZ)));
                        state.addProperty(
                            "biome_id",
                            biomeRegistry.getId(
                                section.getNoiseBiome(localX / 4, localY / 4, localZ / 4).value()));
                        var counts = new JsonArray();
                        counts.add(nonEmptyField.getShort(section));
                        counts.add(fluidField.getShort(section));
                        state.add("section_counts", counts);
                    }
                    queries.add(state);
                }

                var trackState = new JsonObject();
                trackState.addProperty("label", track.get("label").getAsString());
                trackState.add("sections", sectionStates);
                trackState.add("queries", queries);
                tracks.add(trackState);
            }
            var result = new JsonObject();
            result.add("tracks", tracks);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LevelChunkSection.read reflection fixture failed", error);
        }
    }

    static JsonObject clientWorldBlockUpdateResult(JsonObject request) {
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

            int dimensionMinY = request.get("dimension_min_y").getAsInt();
            int minSectionY = Math.floorDiv(dimensionMinY, 16);
            int chunkX = request.get("chunk_x").getAsInt();
            int chunkZ = request.get("chunk_z").getAsInt();
            var vanillaBiomes = VanillaRegistries.createLookup().lookupOrThrow(Registries.BIOME);
            var biomeRegistry = new MappedRegistry<net.minecraft.world.level.biome.Biome>(
                Registries.BIOME, Lifecycle.stable());
            vanillaBiomes.listElements().forEach(holder ->
                Registry.register(biomeRegistry, holder.key(), holder.value()));
            biomeRegistry.freeze();
            var containerFactory = PalettedContainerFactory.create(
                new RegistryAccess.ImmutableRegistryAccess(List.of(biomeRegistry)));
            var nonEmptyField = Reflect.field(LevelChunkSection.class, "nonEmptyBlockCount");
            var fluidField = Reflect.field(LevelChunkSection.class, "fluidCount");
            var sections = new ArrayList<LevelChunkSection>();
            for (var sectionEntry : request.getAsJsonArray("sections")) {
                LevelChunkSection section = new LevelChunkSection(containerFactory);
                FriendlyByteBuf buffer = new FriendlyByteBuf(
                    Unpooled.wrappedBuffer(hexBytes(sectionEntry.getAsString())));
                try {
                    section.read(buffer);
                    if (buffer.isReadable()) {
                        throw new IllegalArgumentException("section payload has trailing bytes");
                    }
                } finally {
                    buffer.release();
                }
                sections.add(section);
            }

            var events = new JsonArray();
            for (var updateEntry : request.getAsJsonArray("updates")) {
                JsonObject update = updateEntry.getAsJsonObject();
                int x = update.get("x").getAsInt();
                int y = update.get("y").getAsInt();
                int z = update.get("z").getAsInt();
                int sectionY = Math.floorDiv(y, 16);
                int sectionIndex = sectionY - minSectionY;
                boolean accepted = Math.floorDiv(x, 16) == chunkX
                    && Math.floorDiv(z, 16) == chunkZ
                    && sectionIndex >= 0
                    && sectionIndex < sections.size();
                var state = new JsonObject();
                state.addProperty("label", update.get("label").getAsString());
                BlockState requestedState = Block.stateById(update.get("state_id").getAsInt());
                if (accepted) {
                    LevelChunkSection section = sections.get(sectionIndex);
                    int localX = Math.floorMod(x, 16);
                    int localY = Math.floorMod(y, 16);
                    int localZ = Math.floorMod(z, 16);
                    boolean wasEmpty = section.hasOnlyAir();
                    BlockState previousState = section.getBlockState(localX, localY, localZ);
                    accepted = !(wasEmpty && requestedState.isAir())
                        && previousState != requestedState;
                }
                state.addProperty("accepted", accepted);
                if (accepted) {
                    LevelChunkSection section = sections.get(sectionIndex);
                    int localX = Math.floorMod(x, 16);
                    int localY = Math.floorMod(y, 16);
                    int localZ = Math.floorMod(z, 16);
                    BlockState previous = section.setBlockState(
                        localX, localY, localZ,
                        requestedState, false);
                    state.addProperty("previous_state_id", Block.getId(previous));
                    state.addProperty(
                        "updated_state_id",
                        Block.getId(section.getBlockState(localX, localY, localZ)));
                    var counts = new JsonArray();
                    counts.add(nonEmptyField.getShort(section));
                    counts.add(fluidField.getShort(section));
                    state.add("section_counts", counts);
                    state.addProperty("has_only_air", section.hasOnlyAir());
                    state.addProperty("has_fluid", section.hasFluid());
                } else {
                    state.add("previous_state_id", JsonNull.INSTANCE);
                    state.add("updated_state_id", JsonNull.INSTANCE);
                    state.add("section_counts", JsonNull.INSTANCE);
                    state.add("has_only_air", JsonNull.INSTANCE);
                    state.add("has_fluid", JsonNull.INSTANCE);
                }

                var queries = new JsonArray();
                for (var queryEntry : update.getAsJsonArray("queries")) {
                    JsonObject query = queryEntry.getAsJsonObject();
                    int queryX = query.get("x").getAsInt();
                    int queryY = query.get("y").getAsInt();
                    int queryZ = query.get("z").getAsInt();
                    int querySectionY = Math.floorDiv(queryY, 16);
                    int querySectionIndex = querySectionY - minSectionY;
                    LevelChunkSection querySection = Math.floorDiv(queryX, 16) == chunkX
                        && Math.floorDiv(queryZ, 16) == chunkZ
                        && querySectionIndex >= 0
                        && querySectionIndex < sections.size()
                        ? sections.get(querySectionIndex)
                        : null;
                    var queryState = new JsonObject();
                    queryState.addProperty("label", query.get("label").getAsString());
                    queryState.addProperty("section_y", querySectionY);
                    if (querySection == null) {
                        queryState.add("block_state_id", JsonNull.INSTANCE);
                        queryState.add("section_counts", JsonNull.INSTANCE);
                    } else {
                        int queryLocalX = Math.floorMod(queryX, 16);
                        int queryLocalY = Math.floorMod(queryY, 16);
                        int queryLocalZ = Math.floorMod(queryZ, 16);
                        queryState.addProperty(
                            "block_state_id",
                            Block.getId(querySection.getBlockState(
                                queryLocalX, queryLocalY, queryLocalZ)));
                        var counts = new JsonArray();
                        counts.add(nonEmptyField.getShort(querySection));
                        counts.add(fluidField.getShort(querySection));
                        queryState.add("section_counts", counts);
                    }
                    queries.add(queryState);
                }
                state.add("queries", queries);
                events.add(state);
            }
            var result = new JsonObject();
            result.add("events", events);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LevelChunkSection.setBlockState reflection fixture failed", error);
        }
    }
}
