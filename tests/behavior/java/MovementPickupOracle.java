import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The item pickup animation: `ItemPickupParticle` and where its group draws it. */
class MovementPickupOracle extends MovementParticleOracle {
    /**
     * An entity taken from a place flies to a taker that moves each tick; where it is drawn,
     * from the camera, at each partial tick after each tick.
     */
    static JsonObject itemPickup(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var targets = row.getAsJsonArray("targets");
        var target = (net.minecraft.world.entity.Entity) unsafe.allocateInstance(
            net.minecraft.world.entity.decoration.ArmorStand.class);
        placeTarget(target, targets.get(0).getAsJsonObject());
        var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
        state.x = Oracle.f64(row, "x");
        state.y = Oracle.f64(row, "y");
        state.z = Oracle.f64(row, "z");
        var particle = new net.minecraft.client.particle.ItemPickupParticle(
            scene.level(), state, target, net.minecraft.world.phys.Vec3.ZERO);
        var camera = new net.minecraft.client.Camera();
        setField(camera, "position", new net.minecraft.world.phys.Vec3(
            Oracle.f64(row, "camera_x"), Oracle.f64(row, "camera_y"), Oracle.f64(row, "camera_z")));
        var instance = Class.forName("net.minecraft.client.particle.ItemPickupParticleGroup$ParticleInstance");
        var fromParticle = instance.getDeclaredMethod("fromParticle",
            net.minecraft.client.particle.ItemPickupParticle.class, net.minecraft.client.Camera.class, float.class);
        fromParticle.setAccessible(true);
        var partials = row.getAsJsonArray("partials");
        var states = new JsonArray();
        for (int tick = 0; tick < targets.size(); tick++) {
            if (tick > 0) {
                placeTarget(target, targets.get(tick).getAsJsonObject());
                particle.tick();
            }
            var described = new JsonObject();
            described.addProperty("removed", !particle.isAlive());
            var drawn = new JsonArray();
            for (var partial : partials) {
                var made = fromParticle.invoke(null, particle, camera, Float.intBitsToFloat(Integer.parseUnsignedInt(partial.getAsString(), 16)));
                var offsets = new JsonArray();
                for (var name : new String[] {"xOffset", "yOffset", "zOffset"}) {
                    offsets.add(Oracle.bits((double) getField(made, name)));
                }
                drawn.add(offsets);
            }
            described.add("drawn", drawn);
            states.add(described);
        }
        entry.add("states", states);
        return entry;
    }

    private static void placeTarget(net.minecraft.world.entity.Entity target, JsonObject at) throws Exception {
        setField(target, "position", new net.minecraft.world.phys.Vec3(
            Oracle.f64(at, "x"), Oracle.f64(at, "y"), Oracle.f64(at, "z")));
        setField(target, "eyeHeight", Oracle.f32(at, "eye"));
    }
}
