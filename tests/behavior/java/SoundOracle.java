import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.lang.reflect.Field;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;
import net.minecraft.client.resources.sounds.Sound;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.client.sounds.WeighedSoundEvents;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.util.RandomSource;

/**
 * The reference's own sound registry: `sounds.json` parsed and registered by `SoundManager`'s own
 * parser and registration (a text given by the scenario, or the game's real one), then dumped,
 * and picked from with seeded random sources.
 */
public final class SoundOracle {
    private static final sun.misc.Unsafe UNSAFE = unsafe();

    private SoundOracle() {
    }

    private static sun.misc.Unsafe unsafe() {
        try {
            Field field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (sun.misc.Unsafe) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }

    public static JsonObject run(JsonObject request) {
        try {
            Oracle.vanillaRegistries();
            var rows = new JsonArray();
            for (var element : request.getAsJsonArray("rows")) {
                rows.add(row(element.getAsJsonObject()));
            }
            var result = new JsonObject();
            result.add("rows", rows);
            return result;
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    private static Object get(Object target, Class<?> owner, String name) throws Exception {
        Field field = owner.getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    /** `sounds.json` text, the files that exist, and what the registry says of the events named. */
    private static JsonObject row(JsonObject row) throws Exception {
        String text;
        var files = new HashMap<Identifier, Resource>();
        if (row.has("real")) {
            var assets = Path.of(System.getenv("VOXEL_ASSETS")).getParent().getParent();
            var index = com.google.gson.JsonParser.parseString(Files.readString(assets.resolve("indexes/32.json")))
                .getAsJsonObject().getAsJsonObject("objects");
            var hash = index.getAsJsonObject("minecraft/sounds.json").get("hash").getAsString();
            text = Files.readString(assets.resolve("objects").resolve(hash.substring(0, 2)).resolve(hash));
            for (var entry : index.entrySet()) {
                if (entry.getKey().startsWith("minecraft/sounds/") && entry.getKey().endsWith(".ogg")) {
                    files.put(Identifier.fromNamespaceAndPath("minecraft", entry.getKey().substring("minecraft/".length())),
                        (Resource) UNSAFE.allocateInstance(Resource.class));
                }
            }
        } else {
            text = row.get("json").getAsString();
            for (var file : row.getAsJsonArray("files")) {
                files.put(Identifier.fromNamespaceAndPath("minecraft", file.getAsString()),
                    (Resource) UNSAFE.allocateInstance(Resource.class));
            }
        }

        // `SoundManager.prepare`: the file parsed as one, every event registered; a file with anything
        // wrong in it registers nothing.
        var preparationsClass = Class.forName("net.minecraft.client.sounds.SoundManager$Preparations");
        var constructor = preparationsClass.getDeclaredConstructor();
        constructor.setAccessible(true);
        var preparations = constructor.newInstance();
        var cache = preparationsClass.getDeclaredField("soundCache");
        cache.setAccessible(true);
        cache.set(preparations, files);
        var handle = preparationsClass.getDeclaredMethod("handleRegistration", Identifier.class,
            net.minecraft.client.resources.sounds.SoundEventRegistration.class);
        handle.setAccessible(true);
        var gson = get(null, SoundManager.class, "GSON");
        var type = get(null, SoundManager.class, "SOUND_EVENT_REGISTRATION_TYPE");
        boolean valid = true;
        Map<String, net.minecraft.client.resources.sounds.SoundEventRegistration> parsed = null;
        try (var reader = new java.io.StringReader(text)) {
            parsed = net.minecraft.util.GsonHelper.fromJson((com.google.gson.Gson) gson, reader,
                (com.google.gson.reflect.TypeToken<Map<String, net.minecraft.client.resources.sounds.SoundEventRegistration>>) type);
        } catch (RuntimeException error) {
            valid = false;
        }
        if (valid) {
            for (var entry : parsed.entrySet()) {
                handle.invoke(preparations, Identifier.fromNamespaceAndPath("minecraft", entry.getKey()), entry.getValue());
            }
        }
        @SuppressWarnings("unchecked")
        var registry = (Map<Identifier, WeighedSoundEvents>) get(preparations, preparationsClass, "registry");

        var out = new JsonObject();
        out.addProperty("label", row.get("label").getAsString());
        out.addProperty("valid", valid);
        var events = new JsonArray();
        var names = new java.util.TreeSet<String>();
        registry.keySet().forEach(location -> names.add(location.toString()));
        // The whole registry of a small file; of the real one, the events the scenario names.
        for (String name : names) {
            if (row.has("events") && !contains(row.getAsJsonArray("events"), name)) {
                continue;
            }
            events.add(dump(name, registry.get(Identifier.parse(name))));
        }
        out.add("events", events);
        var picks = new JsonArray();
        if (row.has("picks")) {
            for (var element : row.getAsJsonArray("picks")) {
                var pick = element.getAsJsonObject();
                var event = registry.get(Identifier.parse(pick.get("event").getAsString()));
                var random = RandomSource.create(Long.parseLong(pick.get("seed").getAsString()));
                var sequence = new JsonArray();
                for (int i = 0; i < pick.get("count").getAsInt(); i++) {
                    Sound sound = event.getSound(random);
                    var described = new JsonObject();
                    described.addProperty("location", sound.getLocation().toString());
                    described.addProperty("volume", Oracle.bits(sound.getVolume().sample(random)));
                    described.addProperty("pitch", Oracle.bits(sound.getPitch().sample(random)));
                    described.addProperty("stream", sound.shouldStream());
                    described.addProperty("preload", sound.shouldPreload());
                    described.addProperty("attenuation_distance", sound.getAttenuationDistance());
                    sequence.add(described);
                }
                var one = new JsonObject();
                one.addProperty("event", pick.get("event").getAsString());
                one.addProperty("seed", pick.get("seed").getAsString());
                one.add("sequence", sequence);
                picks.add(one);
            }
        }
        out.add("picks", picks);
        return out;
    }

    private static boolean contains(JsonArray names, String name) {
        for (var element : names) {
            if (element.getAsString().equals(name)) {
                return true;
            }
        }
        return false;
    }

    /** An event as registered: its subtitle key, its total weight and what each entry is. */
    private static JsonObject dump(String name, WeighedSoundEvents event) throws Exception {
        var out = new JsonObject();
        out.addProperty("name", name);
        var subtitle = event.getSubtitle();
        out.addProperty("subtitle", subtitle == null ? null
            : ((net.minecraft.network.chat.contents.TranslatableContents) subtitle.getContents()).getKey());
        out.addProperty("weight", event.getWeight());
        var entries = new JsonArray();
        @SuppressWarnings("unchecked")
        var list = (java.util.List<net.minecraft.client.sounds.Weighted<Sound>>) get(event, WeighedSoundEvents.class, "list");
        for (var weighted : list) {
            var entry = new JsonObject();
            if (weighted instanceof Sound sound) {
                entry.addProperty("kind", "file");
                entry.addProperty("name", sound.getLocation().toString());
                entry.addProperty("weight", sound.getWeight());
                entry.addProperty("volume", Oracle.bits(sound.getVolume().sample(RandomSource.create(0L))));
                entry.addProperty("pitch", Oracle.bits(sound.getPitch().sample(RandomSource.create(0L))));
                entry.addProperty("stream", sound.shouldStream());
                entry.addProperty("preload", sound.shouldPreload());
                entry.addProperty("attenuation_distance", sound.getAttenuationDistance());
            } else {
                entry.addProperty("kind", "event");
                entry.addProperty("weight", weighted.getWeight());
            }
            entries.add(entry);
        }
        out.add("entries", entries);
        return out;
    }
}
