import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.Locale;
import java.util.UUID;
import java.util.function.IntPredicate;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

class MathAlgorithmOracle extends MathSupport {
    static JsonArray randomSteps(RandomSource random, JsonArray entries) {
        var events = new JsonArray();
        for (var entry : entries) {
            events.add(randomState(random, entry.getAsJsonObject()));
        }
        return events;
    }

    static JsonObject randomState(RandomSource random, JsonObject step) {
        var state = new JsonObject();
        switch (step.get("op").getAsString()) {
            case "nextInt" -> state.addProperty("value", Integer.toString(random.nextInt()));
            case "nextIntBound" -> state.addProperty("value",
                Integer.toString(random.nextInt(step.get("bound").getAsInt())));
            case "nextLong" -> state.addProperty("value", Long.toString(random.nextLong()));
            case "nextBoolean" -> state.addProperty("value", Boolean.toString(random.nextBoolean()));
            case "nextFloat" -> state.addProperty("bits", Oracle.bits(random.nextFloat()));
            case "nextDouble" -> state.addProperty("bits", Oracle.bits(random.nextDouble()));
            case "nextGaussian" -> state.addProperty("bits", Oracle.bits(random.nextGaussian()));
            case "setSeed" -> {
                random.setSeed(Long.parseLong(step.get("seed").getAsString()));
                state.addProperty("op", "setSeed");
            }
            case "nextIntBetweenInclusive" -> state.addProperty("value", Integer.toString(
                random.nextIntBetweenInclusive(step.get("min").getAsInt(), step.get("max").getAsInt())));
            case "nextIntRange" -> state.addProperty("value", Integer.toString(
                random.nextInt(step.get("origin").getAsInt(), step.get("bound").getAsInt())));
            case "triangleDouble" -> state.addProperty("bits", Oracle.bits(
                random.triangle(f64(step, "mean"), f64(step, "spread"))));
            case "triangleFloat" -> state.addProperty("bits", Oracle.bits(
                random.triangle(Oracle.f32(step, "mean"), Oracle.f32(step, "spread"))));
            case "consumeCount" -> {
                random.consumeCount(step.get("rounds").getAsInt());
                state.addProperty("op", "consumeCount");
            }
            default -> throw new IllegalArgumentException("Unknown random op");
        }
        return state;
    }

    static JsonObject mthCreateInsecureUuidCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(
            Long.parseLong(request.get("seed").getAsString()));
        UUID uuid = Mth.createInsecureUUID(random);
        result.addProperty("most_hex", String.format(Locale.ROOT, "%016x",
            uuid.getMostSignificantBits()));
        result.addProperty("least_hex", String.format(Locale.ROOT, "%016x",
            uuid.getLeastSignificantBits()));
        return result;
    }

    static JsonObject mthMurmurHash3MixerCase(JsonObject request) {
        var result = new JsonObject();
        int hash = Integer.parseInt(request.get("input").getAsString());
        result.addProperty("value", Integer.toString(Mth.murmurHash3Mixer(hash)));
        return result;
    }

    static JsonObject mthBinarySearchCase(JsonObject request) {
        var result = new JsonObject();
        int from = Integer.parseInt(request.get("from").getAsString());
        int to = Integer.parseInt(request.get("to").getAsString());
        int threshold = Integer.parseInt(request.get("threshold").getAsString());
        IntPredicate condition = switch (request.get("condition").getAsString()) {
            case "ge" -> value -> value >= threshold;
            case "gt" -> value -> value > threshold;
            default -> throw new IllegalArgumentException("Unknown Mth binary search condition");
        };
        result.addProperty("value", Integer.toString(Mth.binarySearch(from, to, condition)));
        return result;
    }

    static JsonObject mthRayIntersectsAabbCase(JsonObject request) {
        var result = new JsonObject();
        Vec3 rayStart = vec3(request.getAsJsonObject("ray_start"));
        Vec3 rayDir = vec3(request.getAsJsonObject("ray_dir"));
        AABB aabb = box(request.getAsJsonObject("aabb"));
        result.addProperty("value", Mth.rayIntersectsAABB(rayStart, rayDir, aabb));
        return result;
    }

    static JsonObject mthFastInvSqrtCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.fastInvSqrt(f64(request, "input"))));
        return result;
    }

    static JsonObject mthFastInvCubeRootCase(JsonObject request) {
        var result = new JsonObject();
        result.addProperty("bits", Oracle.bits(Mth.fastInvCubeRoot(
            Oracle.f32(request, "input"))));
        return result;
    }

    static JsonObject mthHsvToColorCase(JsonObject request) {
        var result = new JsonObject();
        try {
            float hue = Oracle.f32(request, "hue");
            float saturation = Oracle.f32(request, "saturation");
            float value = Oracle.f32(request, "value");
            int alpha = Integer.parseInt(request.get("alpha").getAsString());
            int color = switch (request.get("mode").getAsString()) {
                case "rgb" -> Mth.hsvToRgb(hue, saturation, value);
                case "argb" -> Mth.hsvToArgb(hue, saturation, value, alpha);
                default -> throw new IllegalArgumentException("Unknown Mth HSV color mode");
            };
            result.addProperty("value", Integer.toString(color));
        } catch (RuntimeException error) {
            if (error.getClass() != RuntimeException.class) {
                throw error;
            }
            result.addProperty("outcome", "error");
            result.addProperty("error", "hue_out_of_range");
            result.addProperty("java_exception", error.getClass().getName());
        }
        return result;
    }

    static JsonObject mthGetSeedCase(JsonObject request) {
        var result = new JsonObject();
        int x = request.get("x").getAsInt();
        int y = request.get("y").getAsInt();
        int z = request.get("z").getAsInt();
        result.addProperty("value", Long.toString(Mth.getSeed(x, y, z)));
        return result;
    }

    static JsonObject mthInvSqrtCase(JsonObject request) {
        var result = new JsonObject();
        if (request.get("type").getAsString().equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.invSqrt(
                Oracle.f32(request, "input"))));
        } else if (request.get("type").getAsString().equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.invSqrt(
                f64(request, "input"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth invSqrt type");
        }
        return result;
    }

    static JsonObject mthNextRandomCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(
            Long.parseLong(request.get("seed").getAsString()));
        String width = request.get("type").getAsString();
        if (width.equals("i32")) {
            result.addProperty("value", Integer.toString(Mth.nextInt(random,
                Integer.parseInt(request.get("min").getAsString()),
                Integer.parseInt(request.get("max").getAsString()))));
        } else if (width.equals("f32")) {
            result.addProperty("bits", Oracle.bits(Mth.nextFloat(random,
                Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
        } else if (width.equals("f64")) {
            result.addProperty("bits", Oracle.bits(Mth.nextDouble(random,
                f64(request, "min"), f64(request, "max"))));
        } else {
            throw new IllegalArgumentException("Unknown Mth next random type");
        }
        return result;
    }

    static JsonObject mthRandomHelpersCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(
            Long.parseLong(request.get("seed").getAsString()));
        String function = request.get("fn").getAsString();
        if (function.equals("inclusive")) {
            int value = Mth.randomBetweenInclusive(random,
                Integer.parseInt(request.get("min").getAsString()),
                Integer.parseInt(request.get("max").getAsString()));
            result.addProperty("value", Integer.toString(value));
            result.addProperty("follow_up", Integer.toString(random.nextInt()));
        } else if (function.equals("between")) {
            float value = Mth.randomBetween(random,
                Oracle.f32(request, "min"), Oracle.f32(request, "max"));
            result.addProperty("bits", Oracle.bits(value));
            result.addProperty("follow_up_bits", Oracle.bits(random.nextFloat()));
        } else if (function.equals("normal")) {
            float value = Mth.normal(random,
                Oracle.f32(request, "mean"), Oracle.f32(request, "deviation"));
            result.addProperty("bits", Oracle.bits(value));
            result.addProperty("follow_up_bits", Oracle.bits(random.nextGaussian()));
        } else {
            throw new IllegalArgumentException("Unknown Mth random helper");
        }
        return result;
    }

    static JsonObject randomCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        var events = new JsonArray();
        events.add(new JsonObject());
        for (var entry : request.getAsJsonArray("steps")) {
            events.add(randomState(random, entry.getAsJsonObject()));
        }
        result.add("events", events);
        return result;
    }

    static JsonObject randomForkCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        result.add("prefix_events", randomSteps(random, request.getAsJsonArray("prefix_steps")));
        RandomSource child = random.fork();
        result.add("child_events", randomSteps(child, request.getAsJsonArray("child_steps")));
        result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        return result;
    }

    static JsonObject randomPositionalCase(JsonObject request) {
        var result = new JsonObject();
        RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
        PositionalRandomFactory factory = random.forkPositional();
        var factoryEvents = new JsonArray();
        for (var entry : request.getAsJsonArray("factory_steps")) {
            JsonObject step = entry.getAsJsonObject();
            RandomSource child = switch (step.get("op").getAsString()) {
                case "at" -> factory.at(step.get("x").getAsInt(), step.get("y").getAsInt(),
                    step.get("z").getAsInt());
                case "fromHashOf" -> factory.fromHashOf(step.get("name").getAsString());
                case "fromSeed" -> factory.fromSeed(Long.parseLong(step.get("seed").getAsString()));
                default -> throw new IllegalArgumentException("Unknown positional random op");
            };
            var state = new JsonObject();
            state.addProperty("op", step.get("op").getAsString());
            state.add("events", randomSteps(child, step.getAsJsonArray("steps")));
            factoryEvents.add(state);
        }
        result.add("factory_events", factoryEvents);
        result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        return result;
    }
}
