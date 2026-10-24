import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.Unpooled;
import com.mojang.serialization.Lifecycle;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.client.multiplayer.ClientChunkCache;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.server.Bootstrap;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import sun.misc.Unsafe;

class MathChunkCacheOracle extends MathBoxOracle {
    static long cacheChunkKey(int chunkX, int chunkZ) {
        return ((long)chunkX << 32) ^ (chunkZ & 0xffffffffL);
    }

    static int cacheChunkX(long key) {
        return (int)(key >> 32);
    }

    static int cacheChunkZ(long key) {
        return (int)key;
    }

    static JsonArray cacheChunkPairs(java.util.Collection<Long> keys) {
        var sorted = new ArrayList<Long>(keys);
        sorted.sort((left, right) -> {
            int xCompare = Integer.compare(cacheChunkX(left), cacheChunkX(right));
            return xCompare != 0
                ? xCompare
                : Integer.compare(cacheChunkZ(left), cacheChunkZ(right));
        });
        var result = new JsonArray();
        for (long key : sorted) {
            var pair = new JsonArray();
            pair.add(cacheChunkX(key));
            pair.add(cacheChunkZ(key));
            result.add(pair);
        }
        return result;
    }

    static JsonArray cacheSectionNodes(java.util.Collection<Long> nodes) {
        var sorted = new ArrayList<Long>(nodes);
        sorted.sort(Long::compare);
        var result = new JsonArray();
        for (long node : sorted) {
            result.add(node);
        }
        return result;
    }

    static JsonObject clientWorldChunkCacheResult(JsonObject request) {
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
            var cacheOwner = (ClientChunkCache)unsafe.allocateInstance(ClientChunkCache.class);
            Class<?> storageClass = Class.forName(ClientChunkCache.class.getName() + "$Storage");
            var storageConstructor = storageClass.getDeclaredConstructor(ClientChunkCache.class, int.class);
            storageConstructor.setAccessible(true);
            var calculateStorageRange = Reflect.method(ClientChunkCache.class, "calculateStorageRange", int.class);
            var inRange = storageClass.getDeclaredMethod("inRange", int.class, int.class);
            inRange.setAccessible(true);
            var getIndex = storageClass.getDeclaredMethod("getIndex", int.class, int.class);
            getIndex.setAccessible(true);
            var viewCenterX = storageClass.getDeclaredField("viewCenterX");
            var viewCenterZ = storageClass.getDeclaredField("viewCenterZ");
            viewCenterX.setAccessible(true);
            viewCenterZ.setAccessible(true);

            int serverViewRadius = request.get("server_view_radius").getAsInt();
            int storageRadius = (Integer)calculateStorageRange.invoke(null, serverViewRadius);
            Object storage = storageConstructor.newInstance(cacheOwner, storageRadius);
            int viewCenterXValue = 0;
            int viewCenterZValue = 0;
            int minSectionY = Math.floorDiv(request.get("dimension_min_y").getAsInt(), 16);

            var vanillaBiomes = VanillaRegistries.createLookup().lookupOrThrow(Registries.BIOME);
            var biomeRegistry = new MappedRegistry<net.minecraft.world.level.biome.Biome>(
                Registries.BIOME, Lifecycle.stable());
            vanillaBiomes.listElements().forEach(holder ->
                Registry.register(biomeRegistry, holder.key(), holder.value()));
            biomeRegistry.freeze();
            var containerFactory = PalettedContainerFactory.create(
                new RegistryAccess.ImmutableRegistryAccess(List.of(biomeRegistry)));

            var chunks = new java.util.HashMap<Long, java.util.List<Boolean>>();
            var slots = new java.util.HashMap<Integer, Long>();
            var addedEmptySections = new ArrayList<java.util.TreeSet<Long>>();
            var removedEmptySections = new ArrayList<java.util.TreeSet<Long>>();
            var addedLoadedChunks = new ArrayList<java.util.TreeSet<Long>>();
            var removedLoadedChunks = new ArrayList<java.util.TreeSet<Long>>();
            for (int index = 0; index < 2; index++) {
                addedEmptySections.add(new java.util.TreeSet<>());
                removedEmptySections.add(new java.util.TreeSet<>());
                addedLoadedChunks.add(new java.util.TreeSet<>());
                removedLoadedChunks.add(new java.util.TreeSet<>());
            }
            int updatingSetsIndex = 0;
            var events = new JsonArray();

            for (var operationEntry : request.getAsJsonArray("operations")) {
                JsonObject operation = operationEntry.getAsJsonObject();
                String action = operation.get("action").getAsString();
                boolean accepted;
                if (action.equals("replace")) {
                    int chunkX = operation.get("chunk_x").getAsInt();
                    int chunkZ = operation.get("chunk_z").getAsInt();
                    accepted = (Boolean)inRange.invoke(storage, chunkX, chunkZ);
                    if (accepted) {
                        var sections = new ArrayList<Boolean>();
                        for (var sectionEntry : operation.getAsJsonArray("sections")) {
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
                            sections.add(section.hasOnlyAir());
                        }

                        int slot = (Integer)getIndex.invoke(storage, chunkX, chunkZ);
                        long key = cacheChunkKey(chunkX, chunkZ);
                        Long current = slots.get(slot);
                        if (current != null && current.longValue() == key) {
                            chunks.put(key, sections);
                            var emptySet = addedEmptySections.get(updatingSetsIndex);
                            var nonEmptySet = removedEmptySections.get(updatingSetsIndex);
                            for (int index = 0; index < sections.size(); index++) {
                                long node = SectionPos.asLong(chunkX, minSectionY + index, chunkZ);
                                if (sections.get(index)) {
                                    emptySet.add(node);
                                } else {
                                    nonEmptySet.add(node);
                                }
                            }
                        } else {
                            if (current != null) {
                                var oldSections = chunks.remove(current);
                                removedLoadedChunks.get(updatingSetsIndex).add(current);
                                if (oldSections != null) {
                                    int oldX = cacheChunkX(current);
                                    int oldZ = cacheChunkZ(current);
                                    for (int index = 0; index < oldSections.size(); index++) {
                                        removedEmptySections.get(updatingSetsIndex).add(
                                            SectionPos.asLong(oldX, minSectionY + index, oldZ));
                                    }
                                }
                            }
                            chunks.put(key, sections);
                            slots.put(slot, key);
                            addedLoadedChunks.get(updatingSetsIndex).add(key);
                            for (int index = 0; index < sections.size(); index++) {
                                if (sections.get(index)) {
                                    addedEmptySections.get(updatingSetsIndex).add(
                                        SectionPos.asLong(chunkX, minSectionY + index, chunkZ));
                                }
                            }
                        }
                    }
                } else if (action.equals("drop")) {
                    int chunkX = operation.get("chunk_x").getAsInt();
                    int chunkZ = operation.get("chunk_z").getAsInt();
                    int slot = (Integer)getIndex.invoke(storage, chunkX, chunkZ);
                    long key = cacheChunkKey(chunkX, chunkZ);
                    accepted = (Boolean)inRange.invoke(storage, chunkX, chunkZ)
                        && slots.get(slot) != null
                        && slots.get(slot).longValue() == key;
                    if (accepted) {
                        var oldSections = chunks.remove(key);
                        slots.remove(slot);
                        removedLoadedChunks.get(updatingSetsIndex).add(key);
                        if (oldSections != null) {
                            for (int index = 0; index < oldSections.size(); index++) {
                                removedEmptySections.get(updatingSetsIndex).add(
                                    SectionPos.asLong(chunkX, minSectionY + index, chunkZ));
                            }
                        }
                    }
                } else if (action.equals("center")) {
                    viewCenterXValue = operation.get("chunk_x").getAsInt();
                    viewCenterZValue = operation.get("chunk_z").getAsInt();
                    viewCenterX.setInt(storage, viewCenterXValue);
                    viewCenterZ.setInt(storage, viewCenterZValue);
                    accepted = true;
                } else if (action.equals("radius")) {
                    int newServerViewRadius = operation.get("server_view_radius").getAsInt();
                    int newStorageRadius = (Integer)calculateStorageRange.invoke(null, newServerViewRadius);
                    accepted = true;
                    if (newStorageRadius != storageRadius) {
                        Object newStorage = storageConstructor.newInstance(cacheOwner, newStorageRadius);
                        viewCenterX.setInt(newStorage, viewCenterXValue);
                        viewCenterZ.setInt(newStorage, viewCenterZValue);
                        var retained = new ArrayList<Long>();
                        for (long key : chunks.keySet()) {
                            if ((Boolean)inRange.invoke(newStorage, cacheChunkX(key), cacheChunkZ(key))) {
                                retained.add(key);
                            }
                        }
                        chunks.keySet().removeIf(key -> !retained.contains(key));
                        slots.clear();
                        for (var set : addedEmptySections) {
                            set.clear();
                        }
                        for (var set : removedEmptySections) {
                            set.clear();
                        }
                        for (var set : addedLoadedChunks) {
                            set.clear();
                        }
                        for (var set : removedLoadedChunks) {
                            set.clear();
                        }
                        storage = newStorage;
                        storageRadius = newStorageRadius;
                        for (long key : retained) {
                            int chunkX = cacheChunkX(key);
                            int chunkZ = cacheChunkZ(key);
                            slots.put((Integer)getIndex.invoke(storage, chunkX, chunkZ), key);
                            addedLoadedChunks.get(0).add(key);
                            var sections = chunks.get(key);
                            if (sections != null) {
                                for (int index = 0; index < sections.size(); index++) {
                                    if (sections.get(index)) {
                                        addedEmptySections.get(0).add(
                                            SectionPos.asLong(chunkX, minSectionY + index, chunkZ));
                                    }
                                }
                            }
                        }
                        updatingSetsIndex = 0;
                    }
                } else if (action.equals("section_emptiness")) {
                    int sectionX = operation.get("section_x").getAsInt();
                    int sectionY = operation.get("section_y").getAsInt();
                    int sectionZ = operation.get("section_z").getAsInt();
                    boolean empty = operation.get("empty").getAsBoolean();
                    if ((Boolean)inRange.invoke(storage, sectionX, sectionZ)) {
                        long node = SectionPos.asLong(sectionX, sectionY, sectionZ);
                        if (empty) {
                            addedEmptySections.get(updatingSetsIndex).add(node);
                        } else {
                            removedEmptySections.get(updatingSetsIndex).add(node);
                        }
                    }
                    accepted = true;
                } else if (action.equals("flip")) {
                    updatingSetsIndex = (updatingSetsIndex + 1) % 2;
                    addedEmptySections.get(updatingSetsIndex).clear();
                    removedEmptySections.get(updatingSetsIndex).clear();
                    addedLoadedChunks.get(updatingSetsIndex).clear();
                    removedLoadedChunks.get(updatingSetsIndex).clear();
                    accepted = true;
                } else {
                    throw new IllegalArgumentException("Unknown client chunk cache action: " + action);
                }

                var checks = new JsonArray();
                for (var checkEntry : operation.getAsJsonArray("checks")) {
                    JsonObject check = checkEntry.getAsJsonObject();
                    int chunkX = check.get("x").getAsInt();
                    int chunkZ = check.get("z").getAsInt();
                    var state = new JsonObject();
                    state.addProperty("x", chunkX);
                    state.addProperty("z", chunkZ);
                    state.addProperty("in_range", (Boolean)inRange.invoke(storage, chunkX, chunkZ));
                    state.addProperty("index", (Integer)getIndex.invoke(storage, chunkX, chunkZ));
                    checks.add(state);
                }

                var cache = new JsonObject();
                cache.addProperty("storage_radius", storageRadius);
                cache.addProperty("view_range", storageRadius * 2 + 1);
                cache.addProperty("view_center_x", viewCenterXValue);
                cache.addProperty("view_center_z", viewCenterZValue);
                cache.add("loaded_chunks", cacheChunkPairs(chunks.keySet()));
                cache.add("added_empty_sections", cacheSectionNodes(addedEmptySections.get(updatingSetsIndex)));
                cache.add("removed_empty_sections", cacheSectionNodes(removedEmptySections.get(updatingSetsIndex)));
                cache.add("added_loaded_chunks", cacheChunkPairs(addedLoadedChunks.get(updatingSetsIndex)));
                cache.add("removed_loaded_chunks", cacheChunkPairs(removedLoadedChunks.get(updatingSetsIndex)));
                var event = new JsonObject();
                event.addProperty("action", action);
                event.addProperty("accepted", accepted);
                event.add("cache", cache);
                event.add("checks", checks);
                events.add(event);
            }
            var result = new JsonObject();
            result.add("events", events);
            return result;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "ClientChunkCache.Storage reflection fixture failed", error);
        }
    }
}
