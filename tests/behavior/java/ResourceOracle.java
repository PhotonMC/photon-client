import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import io.netty.buffer.ByteBufUtil;
import io.netty.buffer.Unpooled;
import io.netty.handler.codec.DecoderException;
import io.netty.handler.codec.EncoderException;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import net.minecraft.network.chat.Component;
import net.minecraft.SharedConstants;
import net.minecraft.DetectedVersion;
import net.minecraft.client.resources.IndexedAssetSource;
import net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks;
import net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.PackLocationInfo;
import net.minecraft.server.packs.PackResources;
import net.minecraft.server.packs.PackType;
import net.minecraft.server.packs.PathPackResources;
import net.minecraft.server.packs.VanillaPackResourcesBuilder;
import net.minecraft.server.packs.metadata.pack.PackFormat;
import net.minecraft.server.packs.repository.KnownPack;
import net.minecraft.server.packs.repository.PackSource;
import net.minecraft.server.packs.repository.PackCompatibility;
import net.minecraft.server.packs.resources.MultiPackResourceManager;
import net.minecraft.server.packs.resources.ReloadInstance;
import net.minecraft.server.packs.resources.ReloadableResourceManager;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.util.Unit;
import net.minecraft.util.InclusiveRange;

/** Test glue for the Java asset-index path mapping; it does not copy original assets. */
public final class ResourceOracle {
    private static final Gson GSON = new Gson();

    private ResourceOracle() {
    }

    public static JsonObject run(JsonObject request) {
        Path assets = null;
        try {
            ensureVersion();
            assets = Files.createTempDirectory("voxel-resource-oracle-");
            if (request.get("operation").getAsString().equals("packs")) {
                return packResult(request, assets);
            }
            if (request.get("operation").getAsString().equals("reload")) {
                return reloadResult(request, assets);
            }
            if (request.get("operation").getAsString().equals("compatibility")) {
                return compatibilityResult(request);
            }
            if (request.get("operation").getAsString().equals("vanilla")) {
                return vanillaResult(request, assets);
            }
            if (request.get("operation").getAsString().equals("known_pack")) {
                return knownPackResult(request);
            }
            if (request.get("operation").getAsString().equals("known_pack_codec")) {
                return knownPackCodecResult(request);
            }
            if (request.get("operation").getAsString().equals("known_pack_list_codec")) {
                return knownPackListCodecResult(request);
            }
            Path indexes = Files.createDirectories(assets.resolve("indexes"));
            Path objects = Files.createDirectories(assets.resolve("objects"));
            String indexName = request.get("index").getAsString();

            JsonObject index = new JsonObject();
            JsonObject entries = new JsonObject();
            for (JsonElement element : request.getAsJsonArray("entries")) {
                JsonObject entry = element.getAsJsonObject();
                JsonObject object = new JsonObject();
                object.addProperty("hash", entry.get("hash").getAsString());
                entries.add(entry.get("path").getAsString(), object);
            }
            index.add("objects", entries);
            Files.writeString(indexes.resolve(indexName + ".json"), GSON.toJson(index), StandardCharsets.UTF_8);

            for (JsonElement element : request.getAsJsonArray("objects")) {
                JsonObject object = element.getAsJsonObject();
                String hash = object.get("hash").getAsString();
                Path directory = Files.createDirectories(objects.resolve(hash.substring(0, 2)));
                Files.write(directory.resolve(hash), decodeHex(object.get("hex").getAsString()));
            }

            Path indexRoot = IndexedAssetSource.createIndexFs(assets, indexName);
            JsonArray queryResults = new JsonArray();
            for (JsonElement element : request.getAsJsonArray("queries")) {
                String query = element.getAsString();
                Path linked = indexRoot.resolve(query);
                boolean exists = Files.exists(linked);
                boolean directory = Files.isDirectory(linked);
                String objectPath = null;
                boolean readable = false;
                String bytesHex = null;
                if (exists && !directory) {
                    try {
                        Path target = linked.toFile().toPath();
                        objectPath = target.getParent().getFileName() + "/" + target.getFileName();
                    } catch (RuntimeException ignored) {
                        // A missing or directory LinkFS entry has no file target.
                    }
                    try {
                        bytesHex = encodeHex(Files.readAllBytes(linked));
                        readable = true;
                    } catch (IOException | RuntimeException ignored) {
                        // The index can expose a link whose object is absent.
                    }
                }

                JsonObject result = new JsonObject();
                result.addProperty("path", query);
                result.addProperty("exists", exists);
                result.addProperty("directory", directory);
                if (objectPath != null) {
                    result.addProperty("object_path", objectPath);
                }
                result.addProperty("readable", readable);
                if (bytesHex != null) {
                    result.addProperty("bytes_hex", bytesHex);
                }
                queryResults.add(result);
            }

            JsonObject result = new JsonObject();
            result.add("queries", queryResults);
            return result;
        } catch (IOException error) {
            throw new IllegalStateException(error);
        } finally {
            deleteTree(assets);
        }
    }

    private static JsonObject packResult(JsonObject request, Path root) throws IOException {
        List<PackResources> packs = createPacks(
            request.getAsJsonArray("packs"), Files.createDirectories(root.resolve("packs"))
        );

        try (var manager = new MultiPackResourceManager(PackType.CLIENT_RESOURCES, packs)) {
            JsonObject result = new JsonObject();
            result.add("queries", resourceQueries(manager, request.getAsJsonArray("queries")));
            return result;
        }
    }

    private static List<PackResources> createPacks(JsonArray packElements, Path packsRoot) throws IOException {
        List<PackResources> packs = new ArrayList<>();
        for (JsonElement packElement : packElements) {
            JsonObject pack = packElement.getAsJsonObject();
            String id = pack.get("id").getAsString();
            Path packRoot = Files.createDirectories(packsRoot.resolve(id));
            for (JsonElement resourceElement : pack.getAsJsonArray("resources")) {
                JsonObject resource = resourceElement.getAsJsonObject();
                Identifier location = Identifier.parse(resource.get("id").getAsString());
                Path file = packRoot.resolve(PackType.CLIENT_RESOURCES.getDirectory())
                    .resolve(location.getNamespace()).resolve(location.getPath());
                Files.createDirectories(file.getParent());
                Files.write(file, decodeHex(resource.get("hex").getAsString()));
            }
            packs.add(new PathPackResources(
                new PackLocationInfo(id, Component.literal(id), PackSource.DEFAULT, Optional.empty()), packRoot
            ));
        }
        return packs;
    }

    private static JsonObject reloadResult(JsonObject request, Path root) throws IOException {
        try (var manager = new ReloadableResourceManager(PackType.CLIENT_RESOURCES)) {
            JsonObject result = new JsonObject();
            result.add("initial", managerSnapshot(manager, request.getAsJsonArray("initial_queries")));
            JsonArray roundResults = new JsonArray();
            Path reloadRoot = Files.createDirectories(root.resolve("reload-packs"));
            int roundIndex = 0;
            for (JsonElement roundElement : request.getAsJsonArray("rounds")) {
                JsonObject round = roundElement.getAsJsonObject();
                Path roundRoot = Files.createDirectories(reloadRoot.resolve(Integer.toString(roundIndex)));
                List<PackResources> packs = createPacks(round.getAsJsonArray("packs"), roundRoot);
                ReloadInstance reload = manager.createReload(
                    Runnable::run,
                    Runnable::run,
                    CompletableFuture.completedFuture(Unit.INSTANCE),
                    packs
                );
                reload.done().join();
                JsonObject snapshot = managerSnapshot(manager, round.getAsJsonArray("queries"));
                snapshot.addProperty("round", roundIndex);
                roundResults.add(snapshot);
                roundIndex++;
            }
            result.add("rounds", roundResults);
            return result;
        }
    }

    private static JsonObject managerSnapshot(ResourceManager manager, JsonArray queryElements) throws IOException {
        JsonArray packs = new JsonArray();
        manager.listPacks().forEach(pack -> packs.add(pack.packId()));
        JsonArray namespaces = new JsonArray();
        manager.getNamespaces().stream().sorted().forEach(namespaces::add);
        JsonObject result = new JsonObject();
        result.add("packs", packs);
        result.add("namespaces", namespaces);
        result.add("queries", resourceQueries(manager, queryElements));
        return result;
    }

    private static JsonArray resourceQueries(ResourceManager manager, JsonArray queryElements) throws IOException {
        JsonArray queryResults = new JsonArray();
        for (JsonElement queryElement : queryElements) {
            String query = queryElement.getAsString();
            Identifier location = Identifier.parse(query);
            Optional<net.minecraft.server.packs.resources.Resource> resource = manager.getResource(location);
            JsonObject result = new JsonObject();
            result.addProperty("id", query);
            result.addProperty("exists", resource.isPresent());
            JsonArray stack = new JsonArray();
            for (var layered : manager.getResourceStack(location)) {
                stack.add(layered.sourcePackId());
            }
            result.add("stack", stack);
            if (resource.isPresent()) {
                var selected = resource.get();
                result.addProperty("source", selected.sourcePackId());
                try (var input = selected.open()) {
                    result.addProperty("bytes_hex", encodeHex(input.readAllBytes()));
                }
            }
            queryResults.add(result);
        }
        return queryResults;
    }

    private static JsonObject compatibilityResult(JsonObject request) {
        PackFormat game = PackFormat.of(SharedConstants.RESOURCE_PACK_FORMAT_MAJOR, SharedConstants.RESOURCE_PACK_FORMAT_MINOR);
        JsonArray probeResults = new JsonArray();
        for (JsonElement probeElement : request.getAsJsonArray("probes")) {
            JsonObject probe = probeElement.getAsJsonObject();
            PackFormat min = PackFormat.of(probe.get("min_major").getAsInt(), probe.get("min_minor").getAsInt());
            PackFormat max = PackFormat.of(probe.get("max_major").getAsInt(), probe.get("max_minor").getAsInt());
            PackCompatibility compatibility = PackCompatibility.forVersion(new InclusiveRange<>(min, max), game);
            JsonObject result = new JsonObject();
            result.addProperty("compatibility", compatibility.name());
            result.addProperty("is_compatible", compatibility.isCompatible());
            probeResults.add(result);
        }

        JsonObject result = new JsonObject();
        result.addProperty("game_major", game.major());
        result.addProperty("game_minor", game.minor());
        result.add("probes", probeResults);
        return result;
    }

    private static JsonObject vanillaResult(JsonObject request, Path root) throws IOException {
        Path rootsRoot = Files.createDirectories(root.resolve("vanilla-roots"));
        VanillaPackResourcesBuilder builder = new VanillaPackResourcesBuilder();
        int rootIndex = 0;
        for (JsonElement rootElement : request.getAsJsonArray("roots")) {
            JsonObject rootValue = rootElement.getAsJsonObject();
            Path assetRoot = Files.createDirectories(rootsRoot.resolve(Integer.toString(rootIndex)));
            for (JsonElement resourceElement : rootValue.getAsJsonArray("resources")) {
                JsonObject resource = resourceElement.getAsJsonObject();
                Identifier location = Identifier.parse(resource.get("id").getAsString());
                Path file = assetRoot.resolve(location.getNamespace()).resolve(location.getPath());
                Files.createDirectories(file.getParent());
                Files.write(file, decodeHex(resource.get("hex").getAsString()));
            }
            builder.pushAssetPath(PackType.CLIENT_RESOURCES, assetRoot);
            rootIndex++;
        }
        for (JsonElement namespace : request.getAsJsonArray("namespaces")) {
            builder.exposeNamespace(namespace.getAsString());
        }

        PackResources vanilla = builder.build(
            new PackLocationInfo("vanilla", Component.literal("vanilla"), PackSource.BUILT_IN, Optional.empty())
        );
        try (var manager = new MultiPackResourceManager(PackType.CLIENT_RESOURCES, List.of(vanilla))) {
            JsonObject result = new JsonObject();
            JsonArray namespaces = new JsonArray();
            manager.getNamespaces().stream().sorted().forEach(namespaces::add);
            result.add("namespaces", namespaces);
            result.add("queries", resourceQueries(manager, request.getAsJsonArray("queries")));
            return result;
        }
    }

    private static JsonObject knownPackResult(JsonObject request) {
        JsonArray packs = new JsonArray();
        for (JsonElement element : request.getAsJsonArray("probes")) {
            JsonObject probe = element.getAsJsonObject();
            KnownPack pack;
            if (probe.has("kind") && probe.get("kind").getAsString().equals("vanilla")) {
                pack = KnownPack.vanilla(probe.get("id").getAsString());
            } else {
                pack = new KnownPack(
                    probe.get("namespace").getAsString(),
                    probe.get("id").getAsString(),
                    probe.get("version").getAsString()
                );
            }
            JsonObject result = new JsonObject();
            result.addProperty("namespace", pack.namespace());
            result.addProperty("id", pack.id());
            result.addProperty("version", pack.version());
            result.addProperty("is_vanilla", pack.isVanilla());
            result.addProperty("string", pack.toString());
            packs.add(result);
        }

        JsonObject result = new JsonObject();
        result.add("packs", packs);
        return result;
    }

    private static JsonObject knownPackCodecResult(JsonObject request) {
        String operation = request.get("mode").getAsString();
        boolean decode = switch (operation) {
            case "decode" -> true;
            case "encode" -> false;
            default -> throw new IllegalArgumentException("Unknown KnownPack codec operation");
        };
        byte[] prefix = decodeHex(request.get("prefix_hex").getAsString());
        byte[] input = decode ? decodeHex(request.get("hex").getAsString()) : null;
        KnownPack pack = decode ? null : new KnownPack(
            request.get("namespace").getAsString(),
            request.get("id").getAsString(),
            request.get("version").getAsString()
        );

        var buffer = Unpooled.buffer();
        var result = new JsonObject();
        try {
            buffer.writeBytes(prefix);
            if (decode) {
                buffer.writeBytes(input);
                buffer.readerIndex(prefix.length);
                pack = KnownPack.STREAM_CODEC.decode(buffer);
                result.addProperty("namespace", pack.namespace());
                result.addProperty("id", pack.id());
                result.addProperty("version", pack.version());
                result.addProperty("string", pack.toString());
            } else {
                KnownPack.STREAM_CODEC.encode(buffer, pack);
            }
            result.addProperty("outcome", "value");
        } catch (RuntimeException error) {
            String category;
            if (decode && error instanceof DecoderException) {
                category = "decoder_error";
            } else if (!decode && error instanceof EncoderException) {
                category = "encoder_error";
            } else if (decode && error instanceof IndexOutOfBoundsException) {
                category = "buffer_underflow";
            } else if (decode && error.getClass() == RuntimeException.class) {
                category = "too_big";
            } else {
                throw error;
            }
            result.addProperty("outcome", "error");
            result.addProperty("error", category);
            result.addProperty("java_exception", error.getClass().getName());
        }
        if (decode) {
            result.addProperty("consumed", buffer.readerIndex() - prefix.length);
            result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
        } else {
            result.addProperty("written", buffer.writerIndex() - prefix.length);
            result.addProperty("hex", ByteBufUtil.hexDump(buffer));
        }
        buffer.release();
        return result;
    }

    private static JsonObject knownPackListCodecResult(JsonObject request) {
        String packet = request.get("packet").getAsString();
        boolean serverbound = switch (packet) {
            case "clientbound" -> false;
            case "serverbound" -> true;
            default -> throw new IllegalArgumentException("Unknown KnownPack list packet");
        };
        String operation = request.get("mode").getAsString();
        boolean decode = switch (operation) {
            case "decode" -> true;
            case "encode" -> false;
            default -> throw new IllegalArgumentException("Unknown KnownPack list codec operation");
        };
        byte[] prefix = decodeHex(request.get("prefix_hex").getAsString());
        byte[] input = decode ? decodeHex(request.get("hex").getAsString()) : null;
        List<KnownPack> packs = decode ? null : knownPacksFromJson(request.getAsJsonArray("packs"));

        var buffer = Unpooled.buffer();
        var result = new JsonObject();
        try {
            buffer.writeBytes(prefix);
            if (decode) {
                buffer.writeBytes(input);
                buffer.readerIndex(prefix.length);
                packs = serverbound
                    ? ServerboundSelectKnownPacks.STREAM_CODEC.decode(buffer).knownPacks()
                    : ClientboundSelectKnownPacks.STREAM_CODEC.decode(buffer).knownPacks();
                result.add("packs", knownPackArray(packs));
            } else if (serverbound) {
                ServerboundSelectKnownPacks.STREAM_CODEC.encode(buffer, new ServerboundSelectKnownPacks(packs));
            } else {
                ClientboundSelectKnownPacks.STREAM_CODEC.encode(buffer, new ClientboundSelectKnownPacks(packs));
            }
            result.addProperty("outcome", "value");
        } catch (RuntimeException error) {
            String category;
            if (decode && error instanceof DecoderException) {
                category = "decoder_error";
            } else if (!decode && error instanceof EncoderException) {
                category = "encoder_error";
            } else if (decode && error instanceof IndexOutOfBoundsException) {
                category = "buffer_underflow";
            } else if (decode && error instanceof IllegalArgumentException) {
                category = "invalid_count";
            } else if (decode && error.getClass() == RuntimeException.class) {
                category = "too_big";
            } else {
                throw error;
            }
            result.addProperty("outcome", "error");
            result.addProperty("error", category);
            result.addProperty("java_exception", error.getClass().getName());
        }
        if (decode) {
            result.addProperty("consumed", buffer.readerIndex() - prefix.length);
            result.addProperty("remaining_hex", ByteBufUtil.hexDump(buffer));
        } else {
            result.addProperty("written", buffer.writerIndex() - prefix.length);
            result.addProperty("hex", ByteBufUtil.hexDump(buffer));
        }
        buffer.release();
        return result;
    }

    private static List<KnownPack> knownPacksFromJson(JsonArray elements) {
        List<KnownPack> packs = new ArrayList<>(elements.size());
        for (JsonElement element : elements) {
            JsonObject pack = element.getAsJsonObject();
            packs.add(new KnownPack(
                pack.get("namespace").getAsString(),
                pack.get("id").getAsString(),
                pack.get("version").getAsString()
            ));
        }
        return packs;
    }

    private static JsonArray knownPackArray(List<KnownPack> packs) {
        JsonArray result = new JsonArray();
        for (KnownPack pack : packs) {
            JsonObject value = new JsonObject();
            value.addProperty("namespace", pack.namespace());
            value.addProperty("id", pack.id());
            value.addProperty("version", pack.version());
            value.addProperty("string", pack.toString());
            result.add(value);
        }
        return result;
    }

    private static void ensureVersion() {
        try {
            SharedConstants.getCurrentVersion();
        } catch (IllegalStateException error) {
            SharedConstants.setVersion(DetectedVersion.createBuiltIn("26.2", "26.2"));
        }
    }

    private static byte[] decodeHex(String value) {
        if ((value.length() & 1) != 0) {
            throw new IllegalArgumentException("hex must contain complete bytes");
        }
        byte[] result = new byte[value.length() / 2];
        for (int index = 0; index < result.length; index++) {
            int high = Character.digit(value.charAt(index * 2), 16);
            int low = Character.digit(value.charAt(index * 2 + 1), 16);
            if (high < 0 || low < 0) {
                throw new IllegalArgumentException("invalid hex byte");
            }
            result[index] = (byte)((high << 4) | low);
        }
        return result;
    }

    private static String encodeHex(byte[] value) {
        StringBuilder result = new StringBuilder(value.length * 2);
        for (byte element : value) {
            result.append(String.format("%02x", element & 0xff));
        }
        return result.toString();
    }

    private static void deleteTree(Path root) {
        if (root == null) {
            return;
        }
        try (var paths = Files.walk(root)) {
            paths.sorted(Comparator.reverseOrder()).forEach(path -> {
                try {
                    Files.deleteIfExists(path);
                } catch (IOException ignored) {
                    // Temporary test data is best-effort cleanup only.
                }
            });
        } catch (IOException ignored) {
            // Temporary test data is best-effort cleanup only.
        }
    }
}
