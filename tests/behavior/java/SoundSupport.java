import java.lang.reflect.Field;
import java.nio.file.Files;
import java.util.HashMap;
import net.minecraft.resources.Identifier;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.Map;
import java.nio.file.Path;
import net.minecraft.util.RandomSource;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.client.resources.sounds.Sound;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.client.sounds.WeighedSoundEvents;

/** Registry access, the recording sound instances and the row and dump helpers. */
class SoundSupport {
    static final sun.misc.Unsafe UNSAFE = unsafe();

    static sun.misc.Unsafe unsafe() {
        return Reflect.unsafe();
    }

    static Object get(Object target, Class<?> owner, String name) throws Exception {
        Field field = owner.getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    /** A sound event holder: its place in the registry, or what a packet sent whole says. */
    static void holder(JsonObject out, net.minecraft.core.Holder<net.minecraft.sounds.SoundEvent> holder) {
        var key = holder.unwrapKey();
        if (key.isPresent()) {
            out.addProperty("registered", net.minecraft.core.registries.BuiltInRegistries.SOUND_EVENT.getId(holder.value()));
        } else {
            out.addProperty("direct", holder.value().location().toString());
            holder.value().fixedRange().ifPresent(range -> out.addProperty("range", Oracle.bits(range)));
        }
    }

    /** `SoundManager.prepare` on one text: the registry it makes (nothing, if the file is not valid). */
    static Map<Identifier, WeighedSoundEvents> registryOf(String text, java.util.List<String> fileNames)
        throws Exception {
        var files = new HashMap<Identifier, Resource>();
        for (var file : fileNames) {
            files.put(Identifier.fromNamespaceAndPath("minecraft", file), (Resource) UNSAFE.allocateInstance(Resource.class));
        }
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
        try (var reader = new java.io.StringReader(text)) {
            @SuppressWarnings("unchecked")
            Map<String, net.minecraft.client.resources.sounds.SoundEventRegistration> parsed = net.minecraft.util.GsonHelper.fromJson(
                (com.google.gson.Gson) gson, reader,
                (com.google.gson.reflect.TypeToken<Map<String, net.minecraft.client.resources.sounds.SoundEventRegistration>>) type);
            for (var entry : parsed.entrySet()) {
                handle.invoke(preparations, Identifier.fromNamespaceAndPath("minecraft", entry.getKey()), entry.getValue());
            }
        }
        @SuppressWarnings("unchecked")
        var registry = (Map<Identifier, WeighedSoundEvents>) get(preparations, preparationsClass, "registry");
        return registry;
    }

    static void setField(Object target, Class<?> owner, String name, Object value) throws Exception {
        Field field = owner.getDeclaredField(name);
        field.setAccessible(true);
        field.set(target, value);
    }

    /** A sound instance a scenario describes: a sound at a place, scripted to move and stop if it ticks. */
    static class Instance extends net.minecraft.client.resources.sounds.AbstractSoundInstance {
        final boolean startSilent;
        final boolean canPlay;

        Instance(JsonObject step) {
            super(Identifier.parse(step.get("event").getAsString()),
                net.minecraft.sounds.SoundSource.valueOf(step.get("source").getAsString()),
                RandomSource.create(Long.parseLong(step.get("seed").getAsString())));
            this.volume = Oracle.f32(step, "volume");
            this.pitch = Oracle.f32(step, "pitch");
            this.looping = step.get("looping").getAsBoolean();
            this.delay = step.get("delay").getAsInt();
            this.attenuation = step.get("attenuation").getAsString().equals("none")
                ? net.minecraft.client.resources.sounds.SoundInstance.Attenuation.NONE
                : net.minecraft.client.resources.sounds.SoundInstance.Attenuation.LINEAR;
            this.relative = step.get("relative").getAsBoolean();
            var position = step.getAsJsonArray("pos");
            this.x = position.get(0).getAsDouble();
            this.y = position.get(1).getAsDouble();
            this.z = position.get(2).getAsDouble();
            this.startSilent = step.get("start_silent").getAsBoolean();
            this.canPlay = step.get("can_play").getAsBoolean();
        }

        @Override
        public boolean canStartSilent() {
            return this.startSilent;
        }

        @Override
        public boolean canPlaySound() {
            return this.canPlay;
        }
    }

    /** An instance that ticks: it moves, changes its volume and pitch and stops as its script says. */
    static final class TickingInstance extends Instance
        implements net.minecraft.client.resources.sounds.TickableSoundInstance {
        final JsonObject script;
        private int ticks;
        private boolean stopped;

        TickingInstance(JsonObject step) {
            super(step);
            this.script = step.getAsJsonObject("tickable");
        }

        @Override
        public boolean isStopped() {
            return this.stopped;
        }

        @Override
        public void tick() {
            this.ticks++;
            var moves = this.script.getAsJsonArray("moves");
            if (this.ticks <= moves.size()) {
                var to = moves.get(this.ticks - 1).getAsJsonArray();
                this.x = to.get(0).getAsDouble();
                this.y = to.get(1).getAsDouble();
                this.z = to.get(2).getAsDouble();
                this.volume = (float) this.script.getAsJsonArray("volumes").get(this.ticks - 1).getAsDouble();
                this.pitch = (float) this.script.getAsJsonArray("pitches").get(this.ticks - 1).getAsDouble();
            }
            if (this.ticks >= this.script.get("stop_at").getAsInt()) {
                this.stopped = true;
                this.looping = false;
            }
        }
    }

    /** `sounds.json` text, the files that exist, and what the registry says of the events named. */
    static JsonObject row(JsonObject row) throws Exception {
        String text;
        var files = new HashMap<Identifier, Resource>();
        if (row.has("real")) {
            var assets = Path.of(System.getenv("PHOTON_ASSETS")).getParent().getParent();
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

    static boolean contains(JsonArray names, String name) {
        for (var element : names) {
            if (element.getAsString().equals(name)) {
                return true;
            }
        }
        return false;
    }

    /** An event as registered: its subtitle key, its total weight and what each entry is. */
    static JsonObject dump(String name, WeighedSoundEvents event) throws Exception {
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
