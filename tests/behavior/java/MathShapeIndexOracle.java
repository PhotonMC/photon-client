import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.SharedConstants;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.entity.TransientEntitySectionManager;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

class MathShapeIndexOracle extends MathShapeDimensionOracle {
    static JsonObject shapeclientWorldIndexedCollisionCase(JsonObject request) {
        var result = new JsonObject();
        var output = System.out;
        var errorOutput = System.err;
        try {
            SharedConstants.setVersion(TEST_VERSION);
            Bootstrap.bootStrap();
        } finally {
            System.setOut(output);
            System.setErr(errorOutput);
        }
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            AABB testArea = box(input.getAsJsonObject("test_area"));
            var entityStorage = new TransientEntitySectionManager<Entity>(
                Entity.class, new NoopEntityCallbacks());
            var entitiesById = new java.util.HashMap<Integer, Entity>();
            for (var entityEntry : input.getAsJsonArray("entities")) {
                JsonObject entityInput = entityEntry.getAsJsonObject();
                int entityId = entityInput.get("entity_id").getAsInt();
                int typeId = entityInput.get("type_id").getAsInt();
                JsonObject positionInput = entityInput.getAsJsonObject("position");
                Vec3 position = new Vec3(
                    f64(positionInput, "x"),
                    f64(positionInput, "y"),
                    f64(positionInput, "z"));
                boolean spectator = entityInput.has("spectator")
                    && entityInput.get("spectator").getAsBoolean();
                boolean alive = !entityInput.has("alive")
                    || entityInput.get("alive").getAsBoolean();
                boolean onClimbable = entityInput.has("on_climbable")
                    && entityInput.get("on_climbable").getAsBoolean();
                boolean flying = entityInput.has("flying")
                    && entityInput.get("flying").getAsBoolean();
                JsonObject climbableContext = entityInput.has("climbable_context")
                    ? entityInput.getAsJsonObject("climbable_context")
                    : null;
                boolean collidable = entityInput.has("can_be_collided_with")
                    && entityInput.get("can_be_collided_with").getAsBoolean();
                boolean sameVehicle = entityInput.has("same_vehicle")
                    && entityInput.get("same_vehicle").getAsBoolean();
                Entity entity = CollisionFixtureEntity.createIndexed(
                    entityId,
                    typeId,
                    position,
                    spectator,
                    alive,
                    onClimbable,
                    flying,
                    collidable,
                    sameVehicle,
                    climbableContext);
                entityStorage.addEntity(entity);
                entitiesById.put(entityId, entity);
            }
            Integer sourceId = input.has("source_entity_id")
                && !input.get("source_entity_id").isJsonNull()
                ? input.get("source_entity_id").getAsInt()
                : null;
            Entity source = sourceId == null ? null : entitiesById.get(sourceId);
            JsonObject borderInput = input.getAsJsonObject("border");
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
            border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
            border.setSize(f64(borderInput, "size"));
            CollisionGetter collision = collisionFixture(
                input.getAsJsonArray("placements"),
                new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                border);
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("entity_count", entityStorage.count());
            state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
            state.addProperty("no_collision", collision.noCollision(source, testArea));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeclientWorldEntitySectionIndexCase(JsonObject request) {
        var result = new JsonObject();
        var output = System.out;
        var errorOutput = System.err;
        try {
            SharedConstants.setVersion(TEST_VERSION);
            Bootstrap.bootStrap();
        } finally {
            System.setOut(output);
            System.setErr(errorOutput);
        }
        var tracks = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            var entityStorage = new TransientEntitySectionManager<Entity>(
                Entity.class, new NoopEntityCallbacks());
            var entitiesById = new java.util.HashMap<Integer, Entity>();
            var entityTypesById = new java.util.HashMap<Integer, Integer>();
            for (var entityEntry : input.getAsJsonArray("entities")) {
                JsonObject entityInput = entityEntry.getAsJsonObject();
                int entityId = entityInput.get("entity_id").getAsInt();
                int typeId = entityInput.get("type_id").getAsInt();
                Vec3 position = vec3(entityInput.getAsJsonObject("position"));
                boolean spectator = entityInput.has("spectator")
                    && entityInput.get("spectator").getAsBoolean();
                boolean alive = !entityInput.has("alive")
                    || entityInput.get("alive").getAsBoolean();
                boolean collidable = entityInput.has("can_be_collided_with")
                    && entityInput.get("can_be_collided_with").getAsBoolean();
                boolean sameVehicle = entityInput.has("same_vehicle")
                    && entityInput.get("same_vehicle").getAsBoolean();
                Entity entity = CollisionFixtureEntity.createIndexed(
                    entityId,
                    typeId,
                    position,
                    spectator,
                    alive,
                    false,
                    false,
                    collidable,
                    sameVehicle,
                    null);
                entityStorage.addEntity(entity);
                entitiesById.put(entityId, entity);
                entityTypesById.put(entityId, typeId);
            }
            Integer sourceId = input.has("source_entity_id")
                && !input.get("source_entity_id").isJsonNull()
                ? input.get("source_entity_id").getAsInt()
                : null;
            JsonObject borderInput = input.getAsJsonObject("border");
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
            border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
            border.setSize(f64(borderInput, "size"));
            CollisionGetter collision = collisionFixture(
                input.getAsJsonArray("placements"),
                new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                border);
            var states = new JsonArray();
            for (var eventEntry : input.getAsJsonArray("events")) {
                JsonObject event = eventEntry.getAsJsonObject();
                String eventKind = event.get("op").getAsString();
                switch (eventKind) {
                    case "move" -> {
                        int entityId = event.get("entity_id").getAsInt();
                        Entity entity = entitiesById.get(entityId);
                        if (entity == null) {
                            throw new IllegalArgumentException("Cannot move missing entity " + entityId);
                        }
                        Vec3 position = vec3(event.getAsJsonObject("position"));
                        entity.setPosRaw(position.x, position.y, position.z);
                        int typeId = entityTypesById.get(entityId);
                        var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
                        entity.setBoundingBox(entityType.getDimensions().makeBoundingBox(position));
                    }
                    case "remove" -> {
                        int entityId = event.get("entity_id").getAsInt();
                        Entity entity = entitiesById.remove(entityId);
                        entityTypesById.remove(entityId);
                        if (entity != null) {
                            entity.setRemoved(Entity.RemovalReason.DISCARDED);
                        }
                    }
                    case "add" -> {
                        JsonObject entityInput = event.getAsJsonObject("entity");
                        int entityId = entityInput.get("entity_id").getAsInt();
                        Entity previous = entitiesById.remove(entityId);
                        entityTypesById.remove(entityId);
                        if (previous != null) {
                            previous.setRemoved(Entity.RemovalReason.DISCARDED);
                        }
                        int typeId = entityInput.get("type_id").getAsInt();
                        Vec3 position = vec3(entityInput.getAsJsonObject("position"));
                        boolean spectator = entityInput.has("spectator")
                            && entityInput.get("spectator").getAsBoolean();
                        boolean alive = !entityInput.has("alive")
                            || entityInput.get("alive").getAsBoolean();
                        boolean collidable = entityInput.has("can_be_collided_with")
                            && entityInput.get("can_be_collided_with").getAsBoolean();
                        boolean sameVehicle = entityInput.has("same_vehicle")
                            && entityInput.get("same_vehicle").getAsBoolean();
                        Entity entity = CollisionFixtureEntity.createIndexed(
                            entityId,
                            typeId,
                            position,
                            spectator,
                            alive,
                            false,
                            false,
                            collidable,
                            sameVehicle,
                            null);
                        entityStorage.addEntity(entity);
                        entitiesById.put(entityId, entity);
                        entityTypesById.put(entityId, typeId);
                    }
                    case "query" -> {
                        AABB testArea = box(event.getAsJsonObject("test_area"));
                        Entity source = sourceId == null ? null : entitiesById.get(sourceId);
                        String[] stats = entityStorage.gatherStats().split(",");
                        var state = new JsonObject();
                        state.addProperty("label", event.get("label").getAsString());
                        state.addProperty("entity_count", entityStorage.count());
                        state.addProperty("section_count", Integer.parseInt(stats[1]));
                        state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
                        state.addProperty("no_collision", collision.noCollision(source, testArea));
                        states.add(state);
                    }
                    default -> throw new IllegalArgumentException("Unknown entity section event " + eventKind);
                }
            }
            var track = new JsonObject();
            track.addProperty("label", input.get("label").getAsString());
            track.add("states", states);
            tracks.add(track);
        }
        result.add("tracks", tracks);
        return result;
    }
}
