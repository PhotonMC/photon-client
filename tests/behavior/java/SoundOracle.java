import java.nio.file.Files;
import java.util.HashMap;
import net.minecraft.resources.Identifier;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.Map;
import java.nio.file.Path;
import net.minecraft.client.resources.sounds.Sound;
import net.minecraft.client.sounds.SoundManager;

/**
 * The reference's own sound registry: `sounds.json` parsed and registered by `SoundManager`'s own
 * parser and registration (a text given by the scenario, or the game's real one), then dumped,
 * and picked from with seeded random sources.
 */
public final class SoundOracle extends SoundSupport {
    SoundOracle() {
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
    static JsonObject decode(JsonObject request) throws Exception {
        var assets = Path.of(System.getenv("PHOTON_ASSETS")).getParent().getParent();
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
    static JsonObject damageEffects() {
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

    static JsonObject blockSounds() throws Exception {
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
    static JsonObject soundEvents() {
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
    static JsonObject packets(JsonObject request) {
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

    /**
     * Scenarios for the reference's own `SoundEngine`, with the sound device replaced by a channel that writes
     * its calls down: sounds played, stopped, ticked and finishing, and the calls each made of its channel.
     */
    static JsonObject engine(JsonObject request) throws Exception {
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
}
