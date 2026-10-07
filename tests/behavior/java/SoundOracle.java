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
            switch (request.get("operation").getAsString()) {
                case "sound_events" -> {
                    return soundEvents();
                }
                case "packets" -> {
                    return packets(request);
                }
                case "decode" -> {
                    return decode(request);
                }
                case "engine" -> {
                    return engine(request);
                }
                case "block_sounds" -> {
                    return blockSounds();
                }
                case "damage_effects" -> {
                    return damageEffects();
                }
                default -> {
                }
            }
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

    /**
     * Sound files decoded by the reference's own `JOrbisAudioStream` and `ChunkedSampleByteBuf`: the
     * format and how many samples came out. The samples themselves are written next to the assets
     * (`reference-pcm/<label>.pcm`, 16-bit little-endian, never tracked) for the Rust decoder to be
     * held against, one sample at a time, within a least significant bit: the two decoders are not the
     * same code, and a float calculation done in another order rounds differently now and then.
     */
    private static JsonObject decode(JsonObject request) throws Exception {
        var assets = Path.of(System.getenv("VOXEL_ASSETS")).getParent().getParent();
        var index = com.google.gson.JsonParser.parseString(Files.readString(assets.resolve("indexes/32.json")))
            .getAsJsonObject().getAsJsonObject("objects");
        var reference = assets.getParent().resolve("reference-pcm");
        Files.createDirectories(reference);
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var hash = index.getAsJsonObject(row.get("object").getAsString()).get("hash").getAsString();
            var data = Files.readAllBytes(assets.resolve("objects").resolve(hash.substring(0, 2)).resolve(hash));
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            try (var stream = new net.minecraft.client.sounds.JOrbisAudioStream(new java.io.ByteArrayInputStream(data))) {
                var format = stream.getFormat();
                var pcm = stream.readAll();
                var bytes = new byte[pcm.remaining()];
                pcm.get(bytes);
                Files.write(reference.resolve(row.get("label").getAsString() + ".pcm"), bytes);
                out.addProperty("channels", format.getChannels());
                out.addProperty("rate", (int) format.getSampleRate());
                out.addProperty("samples", bytes.length / 2);
                // Itself, sample for sample, within a least significant bit.
                out.addProperty("within_one_lsb", true);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The sound type of every block: the types the reference names (`SoundType`'s own constants), then each
     * block's type (its default state's) and the states of it that have another.
     */
    /**
     * What each damage type does to the one it hurts (`DamageType.effects`): the effect, and the
     * sound the player makes for it (`DamageEffects.sound`), then the same for each effect.
     */
    private static JsonObject damageEffects() {
        var rows = new JsonArray();
        for (var effect : net.minecraft.world.damagesource.DamageEffects.values()) {
            var out = new JsonObject();
            out.addProperty("label", "effect:" + effect.getSerializedName());
            out.addProperty("kind", "effect");
            out.addProperty("name", effect.getSerializedName());
            out.addProperty("sound", effect.sound().location().toString());
            rows.add(out);
        }
        var registry = Oracle.vanillaRegistries().lookupOrThrow(net.minecraft.core.registries.Registries.DAMAGE_TYPE);
        var types = new java.util.TreeMap<String, String>();
        registry.listElements().forEach(holder ->
            types.put(holder.key().identifier().toString(), holder.value().effects().getSerializedName()));
        for (var entry : types.entrySet()) {
            var out = new JsonObject();
            out.addProperty("label", "type:" + entry.getKey());
            out.addProperty("kind", "type");
            out.addProperty("name", entry.getKey());
            out.addProperty("effect", entry.getValue());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static JsonObject blockSounds() throws Exception {
        var types = new java.util.LinkedHashMap<net.minecraft.world.level.block.SoundType, String>();
        var rows = new JsonArray();
        for (var field : net.minecraft.world.level.block.SoundType.class.getFields()) {
            if (java.lang.reflect.Modifier.isStatic(field.getModifiers())
                && field.getType() == net.minecraft.world.level.block.SoundType.class) {
                var type = (net.minecraft.world.level.block.SoundType) field.get(null);
                types.putIfAbsent(type, field.getName());
            }
        }
        for (var entry : types.entrySet()) {
            var type = entry.getKey();
            var out = new JsonObject();
            out.addProperty("label", "type:" + entry.getValue());
            out.addProperty("kind", "type");
            out.addProperty("name", entry.getValue());
            out.addProperty("volume", Oracle.bits(type.getVolume()));
            out.addProperty("pitch", Oracle.bits(type.getPitch()));
            out.addProperty("break", type.getBreakSound().location().toString());
            out.addProperty("step", type.getStepSound().location().toString());
            out.addProperty("place", type.getPlaceSound().location().toString());
            out.addProperty("hit", type.getHitSound().location().toString());
            out.addProperty("fall", type.getFallSound().location().toString());
            rows.add(out);
        }
        for (var block : net.minecraft.core.registries.BuiltInRegistries.BLOCK) {
            var normal = block.defaultBlockState().getSoundType();
            var out = new JsonObject();
            var name = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(block).toString();
            out.addProperty("label", "block:" + name);
            out.addProperty("kind", "block");
            out.addProperty("name", name);
            out.addProperty("type", types.get(normal));
            var overrides = new JsonArray();
            for (var state : block.getStateDefinition().getPossibleStates()) {
                if (state.getSoundType() != normal) {
                    var one = new JsonArray();
                    one.add(net.minecraft.world.level.block.Block.getId(state));
                    one.add(types.get(state.getSoundType()));
                    overrides.add(one);
                }
            }
            out.add("overrides", overrides);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The built-in sound event registry in id order. */
    private static JsonObject soundEvents() {
        var registry = net.minecraft.core.registries.BuiltInRegistries.SOUND_EVENT;
        var rows = new JsonArray();
        for (var event : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(event).toString());
            out.addProperty("id", registry.getId(event));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The three sound packets decoded by the reference's own codecs from bytes a scenario gives. */
    private static JsonObject packets(JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var hex = row.get("hex").getAsString();
            var bytes = new byte[hex.length() / 2];
            for (int i = 0; i < bytes.length; i++) {
                bytes[i] = (byte) Integer.parseInt(hex.substring(2 * i, 2 * i + 2), 16);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                io.netty.buffer.Unpooled.wrappedBuffer(bytes), (net.minecraft.core.RegistryAccess) Oracle.vanillaRegistries());
            try {
                switch (row.get("kind").getAsString()) {
                    case "sound" -> {
                        var packet = net.minecraft.network.protocol.game.ClientboundSoundPacket.STREAM_CODEC.decode(buffer);
                        holder(out, packet.getSound());
                        out.addProperty("source", packet.getSource().ordinal());
                        out.addProperty("x", Oracle.bits(packet.getX()));
                        out.addProperty("y", Oracle.bits(packet.getY()));
                        out.addProperty("z", Oracle.bits(packet.getZ()));
                        out.addProperty("volume", Oracle.bits(packet.getVolume()));
                        out.addProperty("pitch", Oracle.bits(packet.getPitch()));
                        out.addProperty("seed", Long.toString(packet.getSeed()));
                    }
                    case "sound_entity" -> {
                        var packet = net.minecraft.network.protocol.game.ClientboundSoundEntityPacket.STREAM_CODEC.decode(buffer);
                        holder(out, packet.getSound());
                        out.addProperty("source", packet.getSource().ordinal());
                        out.addProperty("entity", packet.getId());
                        out.addProperty("volume", Oracle.bits(packet.getVolume()));
                        out.addProperty("pitch", Oracle.bits(packet.getPitch()));
                        out.addProperty("seed", Long.toString(packet.getSeed()));
                    }
                    default -> {
                        var packet = net.minecraft.network.protocol.game.ClientboundStopSoundPacket.STREAM_CODEC.decode(buffer);
                        if (packet.getName() != null) {
                            out.addProperty("name", packet.getName().toString());
                        }
                        if (packet.getSource() != null) {
                            out.addProperty("source", packet.getSource().ordinal());
                        }
                    }
                }
                out.addProperty("trailing", buffer.readableBytes());
            } catch (RuntimeException error) {
                out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.addProperty("error", true);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A sound event holder: its place in the registry, or what a packet sent whole says. */
    private static void holder(JsonObject out, net.minecraft.core.Holder<net.minecraft.sounds.SoundEvent> holder) {
        var key = holder.unwrapKey();
        if (key.isPresent()) {
            out.addProperty("registered", net.minecraft.core.registries.BuiltInRegistries.SOUND_EVENT.getId(holder.value()));
        } else {
            out.addProperty("direct", holder.value().location().toString());
            holder.value().fixedRange().ifPresent(range -> out.addProperty("range", Oracle.bits(range)));
        }
    }

    private static Object get(Object target, Class<?> owner, String name) throws Exception {
        Field field = owner.getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    /** `SoundManager.prepare` on one text: the registry it makes (nothing, if the file is not valid). */
    private static Map<Identifier, WeighedSoundEvents> registryOf(String text, java.util.List<String> fileNames)
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
        private final JsonObject script;
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

    private static void setField(Object target, Class<?> owner, String name, Object value) throws Exception {
        Field field = owner.getDeclaredField(name);
        field.setAccessible(true);
        field.set(target, value);
    }

    /**
     * Scenarios for the reference's own `SoundEngine`, with the sound device replaced by a channel that writes
     * its calls down: sounds played, stopped, ticked and finishing, and the calls each made of its channel.
     */
    private static JsonObject engine(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            com.mojang.blaze3d.audio.Channel.reset();
            var files = new java.util.ArrayList<String>();
            for (var file : row.getAsJsonArray("files")) {
                files.add(file.getAsString());
            }
            var registry = registryOf(row.get("json").getAsString(), files);
            var manager = (SoundManager) UNSAFE.allocateInstance(SoundManager.class);
            setField(manager, SoundManager.class, "registry", registry);

            // The volume sliders.
            var options = (net.minecraft.client.Options) UNSAFE.allocateInstance(net.minecraft.client.Options.class);
            var volumes = new java.util.EnumMap<net.minecraft.sounds.SoundSource, net.minecraft.client.OptionInstance<Double>>(
                net.minecraft.sounds.SoundSource.class);
            for (var source : net.minecraft.sounds.SoundSource.values()) {
                @SuppressWarnings("unchecked")
                var option = (net.minecraft.client.OptionInstance<Double>) UNSAFE.allocateInstance(
                    net.minecraft.client.OptionInstance.class);
                setField(option, net.minecraft.client.OptionInstance.class, "value",
                    row.getAsJsonObject("volumes").has(source.name())
                        ? row.getAsJsonObject("volumes").get(source.name()).getAsDouble() : 1.0);
                volumes.put(source, option);
            }
            setField(options, net.minecraft.client.Options.class, "soundSourceVolumes", volumes);

            // The device: pools of channels, and buffers that are never loaded.
            var library = new com.mojang.blaze3d.audio.Library();
            library.setPools(row.get("static_channels").getAsInt(), row.get("streaming_channels").getAsInt());
            var buffers = new net.minecraft.client.sounds.SoundBufferLibrary(null) {
                @Override
                public java.util.concurrent.CompletableFuture<com.mojang.blaze3d.audio.SoundBuffer> getCompleteBuffer(
                    final Identifier location) {
                    com.mojang.blaze3d.audio.Channel.LOG.add("buffer " + location);
                    return java.util.concurrent.CompletableFuture.completedFuture(null);
                }

                @Override
                public java.util.concurrent.CompletableFuture<net.minecraft.client.sounds.AudioStream> getStream(
                    final Identifier location, final boolean looping) {
                    com.mojang.blaze3d.audio.Channel.LOG.add("stream " + location + " looping " + looping);
                    return java.util.concurrent.CompletableFuture.completedFuture(null);
                }
            };

            var engine = UNSAFE.allocateInstance(net.minecraft.client.sounds.SoundEngine.class);
            var engineClass = net.minecraft.client.sounds.SoundEngine.class;
            setField(engine, engineClass, "soundManager", manager);
            setField(engine, engineClass, "options", options);
            setField(engine, engineClass, "loaded", true);
            setField(engine, engineClass, "library", library);
            setField(engine, engineClass, "soundBuffers", buffers);
            setField(engine, engineClass, "executor", new net.minecraft.client.sounds.SoundEngineExecutor());
            var access = new net.minecraft.client.sounds.ChannelAccess(library, Runnable::run);
            setField(engine, engineClass, "channelAccess", access);
            setField(engine, engineClass, "instanceToChannel", new HashMap<>());
            setField(engine, engineClass, "instanceBySource", com.google.common.collect.HashMultimap.create());
            var gains = new it.unimi.dsi.fastutil.objects.Object2FloatOpenHashMap<net.minecraft.sounds.SoundSource>();
            gains.defaultReturnValue(1.0F);
            setField(engine, engineClass, "gainBySource", gains);
            setField(engine, engineClass, "tickingSounds", new java.util.ArrayList<>());
            setField(engine, engineClass, "queuedSounds", new HashMap<>());
            setField(engine, engineClass, "soundDeleteTime", new HashMap<>());
            setField(engine, engineClass, "listeners", new java.util.ArrayList<>());
            setField(engine, engineClass, "queuedTickableSounds", new java.util.ArrayList<>());
            setField(engine, engineClass, "preloadQueue", new java.util.ArrayList<>());

            var instances = new HashMap<Integer, Instance>();
            var steps = new JsonArray();
            int logged = 0;
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                var result = new JsonObject();
                switch (step.get("op").getAsString()) {
                    case "play", "play_delayed", "queue_ticking" -> {
                        var instance = step.get("tickable").isJsonNull() ? new Instance(step) : new TickingInstance(step);
                        instances.put(step.get("id").getAsInt(), instance);
                        switch (step.get("op").getAsString()) {
                            case "play" -> result.addProperty("result", engine.getClass()
                                .getMethod("play", net.minecraft.client.resources.sounds.SoundInstance.class)
                                .invoke(engine, instance).toString());
                            case "play_delayed" -> engine.getClass().getMethod("playDelayed",
                                net.minecraft.client.resources.sounds.SoundInstance.class, int.class)
                                .invoke(engine, instance, step.get("delay_ticks").getAsInt());
                            default -> engine.getClass().getMethod("queueTickingSound",
                                net.minecraft.client.resources.sounds.TickableSoundInstance.class).invoke(engine, instance);
                        }
                    }
                    case "stop" -> engine.getClass().getMethod("stop", net.minecraft.client.resources.sounds.SoundInstance.class)
                        .invoke(engine, instances.get(step.get("id").getAsInt()));
                    case "stop_named" -> engine.getClass().getMethod("stop", Identifier.class, net.minecraft.sounds.SoundSource.class)
                        .invoke(engine, step.get("event").isJsonNull() ? null : Identifier.parse(step.get("event").getAsString()),
                            step.get("source").isJsonNull() ? null
                                : net.minecraft.sounds.SoundSource.valueOf(step.get("source").getAsString()));
                    case "tick" -> {
                        var method = engineClass.getDeclaredMethod(step.get("paused").getAsBoolean()
                            ? "tickMusicWhenPaused" : "tickInGameSound");
                        method.setAccessible(true);
                        method.invoke(engine);
                        access.scheduleTick();
                    }
                    case "finish" -> {
                        @SuppressWarnings("unchecked")
                        var handles = (Map<Object, Object>) get(engine, engineClass, "instanceToChannel");
                        var handle = handles.get(instances.get(step.get("id").getAsInt()));
                        if (handle != null) {
                            var channel = (com.mojang.blaze3d.audio.Channel) get(handle, handle.getClass(), "channel");
                            if (channel != null) {
                                channel.finish();
                            }
                        }
                    }
                    case "category_volume" -> engine.getClass().getMethod("updateCategoryVolume",
                        net.minecraft.sounds.SoundSource.class, float.class)
                        .invoke(engine, net.minecraft.sounds.SoundSource.valueOf(step.get("source").getAsString()),
                            Oracle.f32(step, "value"));
                    case "slider" -> {
                        var source = net.minecraft.sounds.SoundSource.valueOf(step.get("source").getAsString());
                        setField(volumes.get(source), net.minecraft.client.OptionInstance.class, "value",
                            step.get("value").getAsDouble());
                        engine.getClass().getMethod("refreshCategoryVolume", net.minecraft.sounds.SoundSource.class)
                            .invoke(engine, source);
                    }
                    case "pause_all_except" -> {
                        var sources = new java.util.ArrayList<net.minecraft.sounds.SoundSource>();
                        for (var name : step.getAsJsonArray("sources")) {
                            sources.add(net.minecraft.sounds.SoundSource.valueOf(name.getAsString()));
                        }
                        engine.getClass().getMethod("pauseAllExcept", net.minecraft.sounds.SoundSource[].class)
                            .invoke(engine, (Object) sources.toArray(new net.minecraft.sounds.SoundSource[0]));
                    }
                    case "resume" -> engine.getClass().getMethod("resume").invoke(engine);
                    case "active" -> result.addProperty("active", (boolean) engine.getClass()
                        .getMethod("isActive", net.minecraft.client.resources.sounds.SoundInstance.class)
                        .invoke(engine, instances.get(step.get("id").getAsInt())));
                    default -> throw new IllegalArgumentException("Unknown engine step");
                }
                // The reference walks its channels in the order of their instances' identity hashes, which
                // changes from one run to the next: what each channel is asked is kept, in order, and the
                // channels are put in their own order. A line that names no channel belongs to the one before.
                var byChannel = new java.util.TreeMap<Integer, java.util.List<String>>();
                int current = -1;
                for (; logged < com.mojang.blaze3d.audio.Channel.LOG.size(); logged++) {
                    var line = com.mojang.blaze3d.audio.Channel.LOG.get(logged);
                    if (line.startsWith("channel")) {
                        current = Integer.parseInt(line.substring(7, line.indexOf(' ')));
                    }
                    byChannel.computeIfAbsent(current, key -> new java.util.ArrayList<>()).add(line);
                }
                var calls = new JsonArray();
                byChannel.values().forEach(lines -> lines.forEach(calls::add));
                result.add("calls", calls);
                steps.add(result);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
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
