import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.util.Locale;

/** Test glue only: invokes the original JAR; never implements game algorithms. */
public final class Oracle {
    /**
     * The stream records go to.
     *
     * A family that bootstraps the game's registries replaces `System.out`
     * with one that feeds the logger. That family puts it back, but holding
     * the original here means a record can never be swallowed even if one
     * forgets to.
     */
    private static final java.io.PrintStream OUT = System.out;

    /**
     * The version every bootstrapping family installs.
     *
     * `SharedConstants.setVersion` refuses a second, different instance, so
     * the families that bootstrap have to agree on one object rather than each
     * building its own.
     */
    static final net.minecraft.WorldVersion TEST_VERSION =
        net.minecraft.DetectedVersion.createBuiltIn("26.2", "26.2");

    private static net.minecraft.core.RegistryAccess.Frozen vanillaRegistries;

    /**
     * Bring the game up once and load the vanilla data pack the way the
     * server does: the static layer's tags, then the worldgen registries with
     * theirs. Families that read tags go through this, so a tag is never read
     * unbound. The bootstrap redirects the standard streams into the game's
     * logger, which would swallow every record, so they are put back.
     */
    /**
     * Bootstraps the game's built-in registries once. A case that touches game
     * classes calls this itself instead of relying on an earlier case having
     * done it. The bootstrap redirects the standard streams into the game's
     * logger, which would swallow every record, so they are put back.
     */
    static synchronized void bootstrap() {
        var out = System.out;
        var err = System.err;
        // Another family may already have installed the shared version.
        net.minecraft.SharedConstants.setVersion(TEST_VERSION);
        try {
            net.minecraft.server.Bootstrap.bootStrap();
        } finally {
            System.setOut(out);
            System.setErr(err);
        }
    }

    static synchronized net.minecraft.core.RegistryAccess.Frozen vanillaRegistries() {
        if (vanillaRegistries != null) {
            return vanillaRegistries;
        }
        bootstrap();
        var pack = net.minecraft.server.packs.repository.ServerPacksSource.createVanillaPackSource();
        var resources = new net.minecraft.server.packs.resources.MultiPackResourceManager(
            net.minecraft.server.packs.PackType.SERVER_DATA, java.util.List.of(pack));
        var layers = net.minecraft.server.RegistryLayer.createRegistryAccess();
        var staticTags = net.minecraft.tags.TagLoader.loadTagsForExistingRegistries(
            resources, layers.getLayer(net.minecraft.server.RegistryLayer.STATIC));
        var worldgenContext = layers.getAccessForLoading(net.minecraft.server.RegistryLayer.WORLDGEN);
        var worldgenLookups = net.minecraft.tags.TagLoader.buildUpdatedLookups(worldgenContext, staticTags);
        var worldgen = net.minecraft.resources.RegistryDataLoader.load(
            resources, worldgenLookups, net.minecraft.resources.RegistryDataLoader.WORLDGEN_REGISTRIES,
            Runnable::run).join();
        var loaded = layers.replaceFrom(net.minecraft.server.RegistryLayer.WORLDGEN, worldgen);
        staticTags.forEach(net.minecraft.core.Registry.PendingTags::apply);
        vanillaRegistries = loaded.compositeAccess().freeze();
        // Items and other built-in entries get their default components the
        // way the server's resource reload gives them, once the lookups exist.
        net.minecraft.core.registries.BuiltInRegistries.DATA_COMPONENT_INITIALIZERS
            .build(vanillaRegistries)
            .forEach(net.minecraft.core.component.DataComponentInitializers.PendingComponents::apply);
        return vanillaRegistries;
    }

    public static void main(String[] args) throws Exception {
        Locale.setDefault(Locale.ROOT);
        var gson = new Gson();
        try (var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8))) {
            for (String line; (line = input.readLine()) != null;) {
                JsonObject request = JsonParser.parseString(line).getAsJsonObject();
                if (request.get("schema_version").getAsInt() != 1) {
                    throw new IllegalArgumentException("Unsupported scenario schema");
                }
                String handler = switch (request.get("family").getAsString()) {
                    case "wire" -> "WireOracle";
                    case "utf8" -> "Utf8Oracle";
                    case "math" -> "MathOracle";
                    case "timer" -> "TimerOracle";
                    case "binary" -> "BinaryOracle";
                    case "nbt" -> "NbtOracle";
                    case "resources" -> "ResourceOracle";
                    case "window" -> "WindowOracle";
                    case "atlas" -> "AtlasOracle";
                    case "image" -> "ImageOracle";
                    case "model" -> "ModelOracle";
                    case "json" -> "JsonOracle";
                    case "block" -> "BlockOracle";
                    case "movement" -> "MovementOracle";
                    case "entity" -> "EntityOracle";
                    case "gui" -> "GuiOracle";
                    case "sound" -> "SoundOracle";
                    default -> throw new IllegalArgumentException("Unknown scenario family");
                };
                var response = new JsonObject();
                response.addProperty("schema_version", 1);
                response.add("case_id", request.get("case_id"));
                JsonObject result = (JsonObject) Class.forName(handler)
                    .getMethod("run", JsonObject.class).invoke(null, request);
                if (result.has("java_exception")) {
                    var diagnostics = new JsonObject();
                    diagnostics.add("java_exception", result.remove("java_exception"));
                    response.add("diagnostics", diagnostics);
                }
                response.add("result", result);
                OUT.println(gson.toJson(response));
            }
        }
    }

    static float f32(JsonObject object, String name) {
        return Float.intBitsToFloat(Integer.parseUnsignedInt(object.get(name).getAsString(), 16));
    }

    static double f64(JsonObject object, String name) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(object.get(name).getAsString(), 16));
    }

    static String bits(float value) {
        return String.format(Locale.ROOT, "%08x", Float.floatToRawIntBits(value));
    }

    static String bits(double value) {
        return String.format(Locale.ROOT, "%016x", Double.doubleToRawLongBits(value));
    }
}
