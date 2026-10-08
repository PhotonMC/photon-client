import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * Particles: the registry's types, and a particle of the reference ticked in a
 * scene.
 *
 * <p>A particle draws what it starts as (its life, size, colour, motion) from
 * a random source it makes itself, seeded from the clock. The scenario fixes
 * that seed: the particle is built again until the seed it got is the one
 * asked for.
 */
class MovementParticleOracle extends MovementMenuOracle {
    private static final long SEED_MULTIPLIER = 25214903917L;
    private static final long SEED_MASK = 281474976710655L;
    private static final long UNIQUIFIER_STEP = 1181783497276652981L;
    /** The step's inverse modulo 2^64: what the uniquifier is set from, so that its next step lands where wanted. */
    private static final long UNIQUIFIER_STEP_INVERSE = inverse(UNIQUIFIER_STEP);
    private static final int MOST_DRAWS = 128;

    private static long inverse(long odd) {
        long result = odd;
        for (int i = 0; i < 6; i++) {
            result *= 2 - odd * result;
        }
        return result;
    }

    /** The registry of particle types, in the order of its ids. */
    static JsonObject particleTypes(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var types = new JsonArray();
        // The types whose options are a block state, by what the field of `ParticleTypes` is declared as.
        var carryingBlocks = new java.util.HashSet<Object>();
        for (var field : net.minecraft.core.particles.ParticleTypes.class.getDeclaredFields()) {
            if (field.getGenericType() instanceof java.lang.reflect.ParameterizedType generic
                && generic.getActualTypeArguments()[0] == net.minecraft.core.particles.BlockParticleOption.class) {
                carryingBlocks.add(field.get(null));
            }
        }
        for (var type : net.minecraft.core.registries.BuiltInRegistries.PARTICLE_TYPE) {
            var described = new JsonObject();
            described.addProperty("name",
                net.minecraft.core.registries.BuiltInRegistries.PARTICLE_TYPE.getKey(type).toString());
            described.addProperty("override_limiter", type.getOverrideLimiter());
            described.addProperty("simple", type instanceof net.minecraft.core.particles.SimpleParticleType);
            described.addProperty("block_state", carryingBlocks.contains(type));
            types.add(described);
        }
        entry.add("types", types);
        return entry;
    }

    /**
     * Whether `particle`'s random source is the one `RandomSource.create(wanted)` makes, after
     * some draws: the draws since are walked forward from the wanted seed.
     */
    private static boolean seededWith(net.minecraft.client.particle.Particle particle, long wanted) throws Exception {
        var random = getField(particle, "random");
        var seedField = random.getClass().getDeclaredField("seed");
        seedField.setAccessible(true);
        long now = ((java.util.concurrent.atomic.AtomicLong) seedField.get(random)).get();
        long walked = (wanted ^ SEED_MULTIPLIER) & SEED_MASK;
        for (int draws = 0; draws <= MOST_DRAWS; draws++) {
            if (walked == now) {
                return true;
            }
            walked = (walked * SEED_MULTIPLIER + 11L) & SEED_MASK;
        }
        return false;
    }

    /** The clock reading `particle`'s random source was seeded with, from the seed it holds now. */
    private static long clockOf(net.minecraft.client.particle.Particle particle, long uniquifier, long near) throws Exception {
        var random = getField(particle, "random");
        var seedField = random.getClass().getDeclaredField("seed");
        seedField.setAccessible(true);
        long walked = ((java.util.concurrent.atomic.AtomicLong) seedField.get(random)).get();
        long inverseMultiplier = inverse(SEED_MULTIPLIER);
        for (int draws = 0; draws <= MOST_DRAWS; draws++) {
            long clock = ((walked ^ SEED_MULTIPLIER) ^ uniquifier) & SEED_MASK;
            long distance = ((clock - near) << 16) >> 16;
            if (Math.abs(distance) < (1L << 24)) {
                return near + distance;
            }
            walked = ((walked - 11L) * inverseMultiplier) & SEED_MASK;
        }
        return near;
    }

    /**
     * A particle made by `build`, whose own random source was seeded with `wanted`. The
     * seed is the clock's xor a counter that is set here, so the counter is set to cancel the
     * clock's reading the particle is expected to see; a reading that is off makes another try,
     * which is told how far off it was. Nothing made (a type with no provider) is nothing.
     */
    static net.minecraft.client.particle.Particle seededParticle(
        long wanted, java.util.concurrent.Callable<net.minecraft.client.particle.Particle> build) throws Exception {
        var uniquifierField = net.minecraft.world.level.levelgen.RandomSupport.class.getDeclaredField("SEED_UNIQUIFIER");
        uniquifierField.setAccessible(true);
        var uniquifier = (java.util.concurrent.atomic.AtomicLong) uniquifierField.get(null);
        long gap = 300;
        for (int attempt = 0; attempt < 2_000_000; attempt++) {
            long before = System.nanoTime();
            long expected = before + gap;
            long stepped = wanted ^ expected;
            uniquifier.set(stepped * UNIQUIFIER_STEP_INVERSE);
            var particle = build.call();
            if (particle == null) {
                return null;
            }
            if (seededWith(particle, wanted)) {
                return particle;
            }
            long seen = clockOf(particle, stepped, expected);
            gap = Math.max(1, seen - before);
        }
        throw new IllegalStateException("no particle seeded with " + wanted);
    }

    /** A sprite set of `count` sprites that are told apart by their place in it. */
    static final class SpriteSetProbe {
        final Object set;
        final java.util.List<net.minecraft.client.renderer.texture.TextureAtlasSprite> sprites = new java.util.ArrayList<>();

        SpriteSetProbe(int count) throws Exception {
            var type = Class.forName("net.minecraft.client.particle.ParticleResources$MutableSpriteSet");
            var constructor = type.getDeclaredConstructor();
            constructor.setAccessible(true);
            set = constructor.newInstance();
            for (int i = 0; i < count; i++) {
                sprites.add((net.minecraft.client.renderer.texture.TextureAtlasSprite)
                    unsafe.allocateInstance(net.minecraft.client.renderer.texture.TextureAtlasSprite.class));
            }
            var rebind = type.getDeclaredMethod("rebind", java.util.List.class);
            rebind.setAccessible(true);
            rebind.invoke(set, sprites);
        }

        int indexOf(Object sprite) {
            for (int i = 0; i < sprites.size(); i++) {
                if (sprites.get(i) == sprite) {
                    return i;
                }
            }
            throw new IllegalStateException("a sprite not of the set");
        }
    }

    private static net.minecraft.client.particle.Particle makeParticle(
        String kind, SpriteSetProbe sprites, net.minecraft.client.multiplayer.ClientLevel level,
        double[] at, net.minecraft.util.RandomSource engineRandom) {
        var spriteSet = (net.minecraft.client.particle.SpriteSet) sprites.set;
        return switch (kind) {
            case "smoke" -> new net.minecraft.client.particle.SmokeParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.SMOKE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "flame" -> new net.minecraft.client.particle.FlameParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.FLAME, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "small_flame" -> new net.minecraft.client.particle.FlameParticle.SmallFlameProvider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.SMALL_FLAME, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "large_smoke" -> new net.minecraft.client.particle.LargeSmokeParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.LARGE_SMOKE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "white_smoke" -> new net.minecraft.client.particle.WhiteSmokeParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.WHITE_SMOKE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "ash" -> new net.minecraft.client.particle.AshParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.ASH, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "white_ash" -> new net.minecraft.client.particle.WhiteAshParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.WHITE_ASH, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "dust_plume" -> new net.minecraft.client.particle.DustPlumeParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.DUST_PLUME, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "soul" -> new net.minecraft.client.particle.SoulParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.SOUL, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "sculk_soul" -> new net.minecraft.client.particle.SoulParticle.EmissiveProvider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.SCULK_SOUL, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "campfire_cosy_smoke" -> new net.minecraft.client.particle.CampfireSmokeParticle.CosyProvider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.CAMPFIRE_COSY_SMOKE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "campfire_signal_smoke" -> new net.minecraft.client.particle.CampfireSmokeParticle.SignalProvider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.CAMPFIRE_SIGNAL_SMOKE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "cloud" -> new net.minecraft.client.particle.PlayerCloudParticle.Provider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.CLOUD, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            case "sneeze" -> new net.minecraft.client.particle.PlayerCloudParticle.SneezeProvider(spriteSet).createParticle(
                net.minecraft.core.particles.ParticleTypes.SNEEZE, level, at[0], at[1], at[2], at[3], at[4], at[5], engineRandom);
            default -> throw new IllegalArgumentException("Unknown particle " + kind);
        };
    }

    /** What a particle is, as bits, with the sizes and lights it would be drawn with at each partial tick. */
    private static JsonObject describeParticle(
        net.minecraft.client.particle.Particle particle, SpriteSetProbe sprites, JsonArray partials) throws Exception {
        var described = new JsonObject();
        var quad = (net.minecraft.client.particle.SingleQuadParticle) particle;
        described.addProperty("age", (int) getField(particle, "age"));
        described.addProperty("lifetime", particle.getLifetime());
        for (var name : new String[] {"x", "y", "z", "xo", "yo", "zo", "xd", "yd", "zd"}) {
            described.addProperty(name, Oracle.bits((double) getField(particle, name)));
        }
        described.addProperty("on_ground", (boolean) getField(particle, "onGround"));
        described.addProperty("removed", !particle.isAlive());
        described.addProperty("width", Oracle.bits((float) getField(particle, "bbWidth")));
        described.addProperty("height", Oracle.bits((float) getField(particle, "bbHeight")));
        var box = particle.getBoundingBox();
        for (var pair : new Object[][] {{"min_x", box.minX}, {"min_y", box.minY}, {"min_z", box.minZ},
            {"max_x", box.maxX}, {"max_y", box.maxY}, {"max_z", box.maxZ}}) {
            described.addProperty((String) pair[0], Oracle.bits((double) pair[1]));
        }
        for (var name : new String[] {"quadSize", "rCol", "gCol", "bCol", "alpha", "roll", "oRoll", "gravity", "friction"}) {
            described.addProperty(name, Oracle.bits((float) getField(particle, name)));
        }
        if (sprites != null) {
            described.addProperty("sprite", sprites.indexOf(getField(particle, "sprite")));
        }
        var drawn = new JsonArray();
        var lightMethod = net.minecraft.client.particle.Particle.class.getDeclaredMethod("getLightCoords", float.class);
        lightMethod.setAccessible(true);
        for (var element : partials) {
            float partial = Float.intBitsToFloat(Integer.parseUnsignedInt(element.getAsString(), 16));
            var at = new JsonObject();
            at.addProperty("quad_size", Oracle.bits(quad.getQuadSize(partial)));
            at.addProperty("light", (int) lightMethod.invoke(particle, partial));
            drawn.add(at);
        }
        described.add("drawn", drawn);
        return described;
    }

    /** A particle of the reference from its provider, seeded, then ticked in a scene. */
    static JsonObject particleTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var sprites = new SpriteSetProbe(row.get("sprites").getAsInt());
        var at = new double[] {Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f64(row, "xa"), Oracle.f64(row, "ya"), Oracle.f64(row, "za")};
        // Made again with each try, so that every try draws what the first would have.
        var particle = seededParticle(row.get("seed").getAsLong(),
            () -> makeParticle(row.get("particle").getAsString(), sprites, level, at,
                net.minecraft.util.RandomSource.create(row.get("engine_seed").getAsLong())));
        var partials = row.getAsJsonArray("partials");
        var states = new JsonArray();
        states.add(describeParticle(particle, sprites, partials));
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            particle.tick();
            states.add(describeParticle(particle, sprites, partials));
        }
        entry.add("states", states);
        return entry;
    }

    /**
     * An engine whose particles are each made with the next seed of a list: a try that did not
     * get it is taken out of the queue again, and a type that makes nothing takes no seed.
     */
    static final class SeededEngine extends net.minecraft.client.particle.ParticleEngine {
        private final java.util.ArrayDeque<Long> seeds = new java.util.ArrayDeque<>();

        SeededEngine(net.minecraft.client.multiplayer.ClientLevel level,
            net.minecraft.client.particle.ParticleResources resources) {
            super(level, resources);
        }

        @Override
        public net.minecraft.client.particle.Particle createParticle(
            net.minecraft.core.particles.ParticleOptions options, double x, double y, double z,
            double xa, double ya, double za) {
            try {
                @SuppressWarnings("unchecked")
                var queue = (java.util.Queue<net.minecraft.client.particle.Particle>) getField(this, "particlesToAdd");
                // What a try that missed drew from the engine's random source is given back.
                var engineSeedField = getField(this, "random").getClass().getDeclaredField("seed");
                engineSeedField.setAccessible(true);
                var engineSeed = (java.util.concurrent.atomic.AtomicLong) engineSeedField.get(getField(this, "random"));
                long engineSeedBefore = engineSeed.get();
                var particle = seededParticle(seeds.isEmpty() ? 0L : seeds.peekFirst(), () -> {
                    engineSeed.set(engineSeedBefore);
                    var made = super.createParticle(options, x, y, z, xa, ya, za);
                    if (made != null && !seededWith(made, seeds.isEmpty() ? 0L : seeds.peekFirst())) {
                        queue.remove(made);
                    }
                    return made;
                });
                if (particle != null) {
                    seeds.removeFirst();
                }
                return particle;
            } catch (Exception error) {
                throw new IllegalStateException(error);
            }
        }
    }

    private static net.minecraft.core.particles.ParticleOptions simpleType(String name) {
        return (net.minecraft.core.particles.SimpleParticleType) net.minecraft.core.registries.BuiltInRegistries
            .PARTICLE_TYPE.getValue(net.minecraft.resources.Identifier.parse("minecraft:" + name));
    }

    private static final long FNV_OFFSET = 0xcbf29ce484222325L;
    private static final long FNV_PRIME = 0x100000001b3L;

    private static long fold(long hash, long value) {
        return (hash ^ value) * FNV_PRIME;
    }

    /**
     * The real engine, with the real providers, asked for particles by the level and by the
     * packet: how many wait and how many are in the group after each tick, the first few
     * particles as they are and a digest of all of them.
     */
    static JsonObject particleEngine(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var listener = scene.listener();
        var minecraft = scene.minecraft();
        setField(minecraft, "packetProcessor", new net.minecraft.network.PacketProcessor(Thread.currentThread()));
        setField(listener, "level", level);
        setField(listener, "minecraft", minecraft);
        var listenerRandom = net.minecraft.util.RandomSource.createThreadSafe();
        listenerRandom.setSeed(row.get("listener_seed").getAsLong());
        setField(listener, "random", listenerRandom);
        ((net.minecraft.util.RandomSource) getField(level, "random")).setSeed(row.get("level_seed").getAsLong());
        scene.options().particles().set(net.minecraft.server.level.ParticleStatus.valueOf(row.get("status").getAsString()));

        var camera = new net.minecraft.client.Camera();
        setField(camera, "position", new net.minecraft.world.phys.Vec3(
            Oracle.f64(row, "camera_x"), Oracle.f64(row, "camera_y"), Oracle.f64(row, "camera_z")));
        var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(renderer, "mainCamera", camera);
        setField(minecraft, "gameRenderer", renderer);

        var sprites = new SpriteSetProbe(row.get("sprites").getAsInt());
        var resources = new net.minecraft.client.particle.ParticleResources();
        @SuppressWarnings("unchecked")
        var sets = (java.util.Map<?, ?>) getField(resources, "spriteSets");
        var rebind = Class.forName("net.minecraft.client.particle.ParticleResources$MutableSpriteSet")
            .getDeclaredMethod("rebind", java.util.List.class);
        rebind.setAccessible(true);
        for (var set : sets.values()) {
            rebind.invoke(set, sprites.sprites);
        }
        var engine = new SeededEngine(level, resources);
        setField(engine, "random", net.minecraft.util.RandomSource.create(row.get("engine_seed").getAsLong()));
        long base = row.get("particle_seed_base").getAsLong();
        long step = row.get("particle_seed_step").getAsLong();
        for (int index = 0; index < row.get("particle_seeds").getAsInt(); index++) {
            engine.seeds.add(base + step * index);
        }
        setField(minecraft, "particleEngine", engine);

        var partials = row.getAsJsonArray("partials");
        @SuppressWarnings("unchecked")
        var pending = (java.util.Queue<net.minecraft.client.particle.Particle>) getField(engine, "particlesToAdd");
        var ticks = new JsonArray();
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            for (var element : row.getAsJsonArray("events")) {
                var event = element.getAsJsonObject();
                if (event.get("tick").getAsInt() != tick) {
                    continue;
                }
                var options = simpleType(event.get("particle").getAsString());
                switch (event.get("type").getAsString()) {
                    case "add" -> level.addParticle(options, event.get("override").getAsBoolean(),
                        event.get("always_show").getAsBoolean(), Oracle.f64(event, "x"), Oracle.f64(event, "y"),
                        Oracle.f64(event, "z"), Oracle.f64(event, "xa"), Oracle.f64(event, "ya"), Oracle.f64(event, "za"));
                    case "packet" -> listener.handleParticleEvent(
                        new net.minecraft.network.protocol.game.ClientboundLevelParticlesPacket(options,
                            event.get("override").getAsBoolean(), event.get("always_show").getAsBoolean(),
                            Oracle.f64(event, "x"), Oracle.f64(event, "y"), Oracle.f64(event, "z"),
                            Oracle.f32(event, "x_dist"), Oracle.f32(event, "y_dist"), Oracle.f32(event, "z_dist"),
                            Oracle.f32(event, "max_speed"), event.get("count").getAsInt()));
                    default -> throw new IllegalArgumentException("Unknown particle event " + event);
                }
            }
            var counted = new JsonObject();
            counted.addProperty("pending", pending.size());
            engine.tick();
            counted.addProperty("particles", groupParticles(engine).size());
            ticks.add(counted);
        }
        entry.add("ticks", ticks);
        var inGroup = groupParticles(engine);
        long digest = FNV_OFFSET;
        var first = new JsonArray();
        int limit = row.get("describe").getAsInt();
        for (var particle : inGroup) {
            for (var name : new String[] {"x", "y", "z", "xd", "yd", "zd"}) {
                digest = fold(digest, Double.doubleToRawLongBits((double) getField(particle, name)));
            }
            digest = fold(digest, (int) getField(particle, "age"));
            digest = fold(digest, particle.getLifetime());
            digest = fold(digest, sprites.indexOf(getField(particle, "sprite")));
            if (first.size() < limit) {
                first.add(describeParticle(particle, sprites, partials));
            }
        }
        entry.add("first", first);
        entry.addProperty("digest", Long.toHexString(digest));
        return entry;
    }

    /** The particles of the engine's group, in the order they tick. */
    private static java.util.List<net.minecraft.client.particle.Particle> groupParticles(
        net.minecraft.client.particle.ParticleEngine engine) throws Exception {
        var all = new java.util.ArrayList<net.minecraft.client.particle.Particle>();
        for (var group : ((java.util.Map<?, ?>) getField(engine, "particles")).values()) {
            for (var particle : (java.util.Queue<?>) getField(group, "particles")) {
                all.add((net.minecraft.client.particle.Particle) particle);
            }
        }
        return all;
    }

    /** Keeps each vertex a particle quad writes: where, its texture coordinate, colour and light. */
    static final class QuadCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final JsonArray vertices = new JsonArray();
        private JsonObject current;

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            this.current = new JsonObject();
            this.current.addProperty("x", Oracle.bits(x));
            this.current.addProperty("y", Oracle.bits(y));
            this.current.addProperty("z", Oracle.bits(z));
            this.vertices.add(this.current);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            this.current.addProperty("color", color);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            this.current.addProperty("u", Oracle.bits(u));
            this.current.addProperty("v", Oracle.bits(v));
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            this.current.addProperty("light", u | v << 16);
            return this;
        }

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /**
     * A particle of the reference, ticked, then extracted for a camera and written as the quad
     * the renderer writes: the four vertices of its layer's buffer. The sprites are told apart by
     * the texture coordinates the scenario gives them.
     */
    static JsonObject particleQuads(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var uvs = row.getAsJsonArray("sprite_uvs");
        var sprites = new SpriteSetProbe(uvs.size());
        for (int i = 0; i < uvs.size(); i++) {
            var coordinates = uvs.get(i).getAsJsonArray();
            var sprite = sprites.sprites.get(i);
            int at = 0;
            for (var name : new String[] {"u0", "u1", "v0", "v1"}) {
                float value = Float.intBitsToFloat(Integer.parseUnsignedInt(coordinates.get(at++).getAsString(), 16));
                setField(sprite, name, value);
            }
        }
        var at = new double[] {Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f64(row, "xa"), Oracle.f64(row, "ya"), Oracle.f64(row, "za")};
        var particle = seededParticle(row.get("seed").getAsLong(),
            () -> makeParticle(row.get("particle").getAsString(), sprites, level, at,
                net.minecraft.util.RandomSource.create(row.get("engine_seed").getAsLong())));
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            particle.tick();
        }
        var camera = new net.minecraft.client.Camera();
        var setPosition = net.minecraft.client.Camera.class.getDeclaredMethod("setPosition", net.minecraft.world.phys.Vec3.class);
        setPosition.setAccessible(true);
        setPosition.invoke(camera, new net.minecraft.world.phys.Vec3(
            Oracle.f64(row, "camera_x"), Oracle.f64(row, "camera_y"), Oracle.f64(row, "camera_z")));
        var setRotation = net.minecraft.client.Camera.class.getDeclaredMethod("setRotation", float.class, float.class);
        setRotation.setAccessible(true);
        setRotation.invoke(camera, Oracle.f32(row, "y_rot"), Oracle.f32(row, "x_rot"));
        if (row.has("roll")) {
            setField(particle, "oRoll", Oracle.f32(row, "o_roll"));
            setField(particle, "roll", Oracle.f32(row, "roll"));
        }
        var state = new net.minecraft.client.renderer.state.level.QuadParticleRenderState();
        ((net.minecraft.client.particle.SingleQuadParticle) particle).extract(state, camera, Oracle.f32(row, "partial"));
        var capture = new QuadCapture();
        for (var layer : state.layers()) {
            state.buildLayer(layer, capture);
        }
        entry.add("vertices", capture.vertices);
        return entry;
    }

    /** An engine that keeps the particles it is given instead of keeping them to tick. */
    static final class RecordingEngine extends net.minecraft.client.particle.ParticleEngine {
        final java.util.List<net.minecraft.client.particle.Particle> added = new java.util.ArrayList<>();

        RecordingEngine(net.minecraft.client.multiplayer.ClientLevel level) {
            super(level, new net.minecraft.client.particle.ParticleResources());
        }

        @Override
        public void add(net.minecraft.client.particle.Particle particle) {
            this.added.add(particle);
        }
    }

    private static JsonObject describeTerrain(net.minecraft.client.particle.Particle particle, JsonArray partials) throws Exception {
        var described = describeParticle(particle, null, partials);
        var uvs = new JsonArray();
        for (var name : new String[] {"getU0", "getU1", "getV0", "getV1"}) {
            var method = net.minecraft.client.particle.SingleQuadParticle.class.getDeclaredMethod(name);
            method.setAccessible(true);
            uvs.add(Oracle.bits((float) method.invoke(particle)));
        }
        described.add("uvs", uvs);
        return described;
    }

    private static JsonObject at(net.minecraft.client.particle.Particle particle) throws Exception {
        var out = new JsonObject();
        for (var name : new String[] {"x", "y", "z"}) {
            out.addProperty(name, Oracle.bits((double) getField(particle, name)));
        }
        return out;
    }

    /**
     * What a block's terrain particles are: one made by a provider and ticked, the particles of
     * a block breaking (where each is), and of one being broken (where it is and how big).
     */
    static JsonObject particleTerrain(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var minecraft = scene.minecraft();
        setField(minecraft, "level", level);
        var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) unsafe.allocateInstance(BlankSprite.class);
        var uv = row.getAsJsonArray("sprite_uv");
        int at = 0;
        for (var name : new String[] {"u0", "u1", "v0", "v1"}) {
            setField(sprite, name, Float.intBitsToFloat(Integer.parseUnsignedInt(uv.get(at++).getAsString(), 16)));
        }
        BlankModels.sprite = sprite;
        BlankModelManager.models = (net.minecraft.client.renderer.block.BlockStateModelSet) unsafe.allocateInstance(BlankModels.class);
        setField(minecraft, "modelManager", unsafe.allocateInstance(BlankModelManager.class));
        setField(minecraft, "blockColors", net.minecraft.client.color.block.BlockColors.createDefault());
        var engine = new RecordingEngine(level);
        setField(minecraft, "particleEngine", engine);
        var state = state(row.getAsJsonObject("state"));
        var pos = new net.minecraft.core.BlockPos(row.get("pos_x").getAsInt(), row.get("pos_y").getAsInt(), row.get("pos_z").getAsInt());
        var partials = row.getAsJsonArray("partials");
        switch (row.get("mode").getAsString()) {
            case "single" -> {
                var options = new net.minecraft.core.particles.BlockParticleOption(
                    net.minecraft.core.particles.ParticleTypes.BLOCK, state);
                var coordinates = new double[] {Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
                    Oracle.f64(row, "xa"), Oracle.f64(row, "ya"), Oracle.f64(row, "za")};
                var provider = row.get("provider").getAsString();
                var particle = seededParticle(row.get("seed").getAsLong(), () -> {
                    var random = net.minecraft.util.RandomSource.create(row.get("engine_seed").getAsLong());
                    return switch (provider) {
                        case "block" -> new net.minecraft.client.particle.TerrainParticle.Provider().createParticle(
                            options, level, coordinates[0], coordinates[1], coordinates[2], coordinates[3], coordinates[4], coordinates[5], random);
                        case "crumbling" -> new net.minecraft.client.particle.TerrainParticle.CrumblingProvider().createParticle(
                            options, level, coordinates[0], coordinates[1], coordinates[2], coordinates[3], coordinates[4], coordinates[5], random);
                        case "dust_pillar" -> new net.minecraft.client.particle.TerrainParticle.DustPillarProvider().createParticle(
                            options, level, coordinates[0], coordinates[1], coordinates[2], coordinates[3], coordinates[4], coordinates[5], random);
                        default -> throw new IllegalArgumentException("Unknown provider " + provider);
                    };
                });
                entry.addProperty("made", particle != null);
                var states = new JsonArray();
                if (particle != null) {
                    states.add(describeTerrain(particle, partials));
                    for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
                        particle.tick();
                        states.add(describeTerrain(particle, partials));
                    }
                }
                entry.add("states", states);
            }
            case "destroy" -> {
                level.addDestroyBlockEffect(pos, state);
                var made = new JsonArray();
                for (var particle : engine.added) {
                    made.add(at(particle));
                }
                entry.add("particles", made);
            }
            case "breaking" -> {
                ((net.minecraft.util.RandomSource) getField(level, "random")).setSeed(row.get("level_seed").getAsLong());
                level.addBreakingBlockEffect(pos, net.minecraft.core.Direction.valueOf(row.get("direction").getAsString()));
                var made = new JsonArray();
                for (var particle : engine.added) {
                    var out = at(particle);
                    out.addProperty("width", Oracle.bits((float) getField(particle, "bbWidth")));
                    out.addProperty("height", Oracle.bits((float) getField(particle, "bbHeight")));
                    made.add(out);
                }
                entry.add("particles", made);
            }
            default -> throw new IllegalArgumentException("Unknown terrain mode");
        }
        return entry;
    }
}
