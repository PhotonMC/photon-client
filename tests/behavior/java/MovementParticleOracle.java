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
    static JsonObject particleTypes(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var types = new JsonArray();
        for (var type : net.minecraft.core.registries.BuiltInRegistries.PARTICLE_TYPE) {
            var described = new JsonObject();
            described.addProperty("name",
                net.minecraft.core.registries.BuiltInRegistries.PARTICLE_TYPE.getKey(type).toString());
            described.addProperty("override_limiter", type.getOverrideLimiter());
            described.addProperty("simple", type instanceof net.minecraft.core.particles.SimpleParticleType);
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
     * which is told how far off it was.
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
        described.addProperty("sprite", sprites.indexOf(getField(particle, "sprite")));
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
}
