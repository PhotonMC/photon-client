import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Comparator;
import net.minecraft.client.resources.IndexedAssetSource;

/** Test glue for the Java asset-index path mapping; it does not copy original assets. */
public final class ResourceOracle {
    private static final Gson GSON = new Gson();

    private ResourceOracle() {
    }

    public static JsonObject run(JsonObject request) {
        Path assets = null;
        try {
            assets = Files.createTempDirectory("voxel-resource-oracle-");
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
