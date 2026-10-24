import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The client state the fog and the lightmap read of the local player's effects and eyes. */
class EntityFogOracle extends EntityBlockOracle {
    private static net.minecraft.core.Holder<net.minecraft.world.effect.MobEffect> effectNamed(final String name) {
        return net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT.get(
            net.minecraft.resources.Identifier.parse(name)).orElseThrow();
    }

    private static Object blendField(final Object blend, final String name) throws Exception {
        var field = Reflect.field(blend.getClass(), name);
        return field.get(blend);
    }

    /**
     * What the real `MobEffectInstance` does as it is ticked on a client: the duration, and the
     * blend state's two factors after each tick, for an effect that blends or starts blended.
     */
    static JsonObject mobEffectBlend(final JsonObject request) throws Exception {
        Game26_2.vanillaRegistries();
        var instanceBlend = Reflect.field(net.minecraft.world.effect.MobEffectInstance.class, "blendState");
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var instance = new net.minecraft.world.effect.MobEffectInstance(
                effectNamed(row.get("effect").getAsString()), row.get("duration").getAsInt(),
                row.get("amplifier").getAsInt());
            if (!row.get("blend").getAsBoolean()) {
                instance.skipBlending();
            }
            var blend = instanceBlend.get(instance);
            var steps = new JsonArray();
            for (int tick = 0; tick < row.get("ticks").getAsInt(); tick++) {
                instance.tickClient();
                var step = new JsonObject();
                step.addProperty("duration", instance.getDuration());
                step.addProperty("factor", Oracle.bits((float) blendField(blend, "factor")));
                step.addProperty("previous", Oracle.bits((float) blendField(blend, "factorPreviousFrame")));
                step.addProperty("ends_within_200", instance.endsWithin(200));
                steps.add(step);
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

    /** A player with one effect, to ask the render code what it makes of it. */
    static final class EffectFixture extends net.minecraft.client.player.RemotePlayer {
        net.minecraft.world.effect.MobEffectInstance effect;

        EffectFixture() {
            super(null, null);
        }

        @Override
        public net.minecraft.world.effect.MobEffectInstance getEffect(
            final net.minecraft.core.Holder<net.minecraft.world.effect.MobEffect> holder) {
            return this.effect;
        }
    }

    /** A local player whose eyes are in water or not, to ask `getWaterVision` of. */
    static final class WaterFixture extends net.minecraft.client.player.LocalPlayer {
        boolean eyesInWater;

        WaterFixture() {
            super(null, null, null, null, null, net.minecraft.world.entity.player.Input.EMPTY, false, null);
        }

        @Override
        public boolean isEyeInFluid(final net.minecraft.tags.TagKey<net.minecraft.world.level.material.Fluid> fluid) {
            return this.eyesInWater;
        }
    }

    /**
     * `GameRenderer.nightVisionScale` of a night vision effect with some time left, and
     * `LocalPlayer.getWaterVision` after some time under water or none.
     */
    static JsonObject viewScales(final JsonObject request) throws Exception {
        Game26_2.vanillaRegistries();
        var unsafe = (sun.misc.Unsafe) UNSAFE;
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            var row = element.getAsJsonObject();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            if (row.get("kind").getAsString().equals("night_vision")) {
                var fixture = (EffectFixture) unsafe.allocateInstance(EffectFixture.class);
                fixture.effect = new net.minecraft.world.effect.MobEffectInstance(
                    net.minecraft.world.effect.MobEffects.NIGHT_VISION, row.get("duration").getAsInt());
                out.addProperty("scale", Oracle.bits(
                    net.minecraft.client.renderer.GameRenderer.nightVisionScale(fixture, Oracle.f32(row, "partial"))));
            } else {
                var fixture = (WaterFixture) unsafe.allocateInstance(WaterFixture.class);
                fixture.eyesInWater = row.get("eyes_in_water").getAsBoolean();
                var time = Reflect.field(net.minecraft.client.player.LocalPlayer.class, "waterVisionTime");
                time.setInt(fixture, row.get("time").getAsInt());
                out.addProperty("scale", Oracle.bits(fixture.getWaterVision()));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
