import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.Locale;
import net.minecraft.SharedConstants;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.StairBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.StairsShape;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.entity.TransientEntitySectionManager;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.VoxelShape;
import sun.misc.Unsafe;

class MathShapeWorldOracle extends MathShapeIndexOracle {
    static StairsShape stairShape(
        BlockState state, BlockGetter level, BlockPos pos
    ) {
        try {
            var method = Reflect.method(StairBlock.class, 
                "getStairsShape", BlockState.class, BlockGetter.class, BlockPos.class);
            return (StairsShape) method.invoke(null, state, level, pos);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("StairBlock.getStairsShape reflection failed", error);
        }
    }

    static JsonObject shapeclientWorldCollisionCase(JsonObject request) {
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
            boolean sourcePresent = input.has("source_present")
                && input.get("source_present").getAsBoolean();
            CollisionFixtureEntity source = null;
            if (sourcePresent) {
                AABB sourceBox = box(input.getAsJsonObject("source_box"));
                JsonObject sourcePosition = input.getAsJsonObject("source_position");
                source = CollisionFixtureEntity.create(
                    sourceBox,
                    false,
                    true,
                    false,
                    new Vec3(
                    f64(sourcePosition, "x"),
                    f64(sourcePosition, "y"),
                    f64(sourcePosition, "z")
                    )
                );
            }
            var entities = new java.util.ArrayList<Entity>();
            for (var candidateEntry : input.getAsJsonArray("candidates")) {
                JsonObject candidate = candidateEntry.getAsJsonObject();
                boolean isSource = candidate.has("is_source")
                    && candidate.get("is_source").getAsBoolean();
                entities.add(sourcePresent && isSource
                    ? source
                    : CollisionFixtureEntity.create(
                        box(candidate.getAsJsonObject("box")),
                        candidate.has("spectator")
                            && candidate.get("spectator").getAsBoolean(),
                        candidate.has("can_be_collided_with")
                            && candidate.get("can_be_collided_with").getAsBoolean(),
                        candidate.has("same_vehicle")
                            && candidate.get("same_vehicle").getAsBoolean()));
            }
            JsonObject borderInput = input.getAsJsonObject("border");
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
            border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
            border.setSize(f64(borderInput, "size"));
            CollisionGetter collision = collisionFixture(
                input.getAsJsonArray("placements"), entities, border);
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("no_block_collision", collision.noBlockCollision(source, testArea));
            state.addProperty("no_entity_collision", collision.noEntityCollision(source, testArea));
            state.addProperty("no_border_collision", collision.noBorderCollision(source, testArea));
            state.addProperty("no_collision", collision.noCollision(source, testArea));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeclientWorldHappyGhastCollisionCase(JsonObject request) {
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
            var entityStorage = new TransientEntitySectionManager<Entity>(
                Entity.class, new NoopEntityCallbacks());
            for (var entityEntry : input.getAsJsonArray("entities")) {
                Entity entity = CollisionFixtureEntity.createHappyGhastIndexed(
                    entityEntry.getAsJsonObject());
                entityStorage.addEntity(entity);
            }
            Integer sourceId = input.has("source_entity_id")
                && !input.get("source_entity_id").isJsonNull()
                ? input.get("source_entity_id").getAsInt()
                : null;
            Entity source = null;
            if (sourceId != null) {
                source = entityStorage.getEntityGetter().get(sourceId);
                if (source == null) {
                    throw new IllegalArgumentException("Unknown collision source " + sourceId);
                }
            }
            JsonObject borderInput = input.getAsJsonObject("border");
            WorldBorder border = new WorldBorder();
            border.setCenter(f64(borderInput, "center_x"), f64(borderInput, "center_z"));
            border.setAbsoluteMaxSize(borderInput.get("absolute_max_size").getAsInt());
            border.setSize(f64(borderInput, "size"));
            CollisionGetter collision = collisionFixture(
                input.getAsJsonArray("placements"),
                new IndexedEntityCollisionFixture(entityStorage.getEntityGetter()),
                border);
            AABB testArea = box(input.getAsJsonObject("test_area"));
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

    static JsonObject shapeclientWorldRegistryCollisionCase(JsonObject request) {
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
        var resolvedStates = new JsonArray();
        for (var stateEntry : request.getAsJsonArray("state_ids")) {
            int requestedStateId = stateEntry.getAsInt();
            BlockState state = Block.stateById(requestedStateId);
            var stateSnapshot = new JsonObject();
            stateSnapshot.addProperty("requested_state_id", requestedStateId);
            stateSnapshot.addProperty("resolved_state_id", Block.getId(state));
            stateSnapshot.add("collision_shape", snapshot(
                state.getCollisionShape(
                    net.minecraft.world.level.EmptyBlockGetter.INSTANCE,
                    BlockPos.ZERO)));
            resolvedStates.add(stateSnapshot);
        }
        AABB testArea = box(request.getAsJsonObject("test_area"));
        CollisionGetter collision = registryCollisionFixture(
            request.getAsJsonArray("placements"));
        var collisions = new JsonArray();
        for (VoxelShape shape : collision.getBlockCollisions(null, testArea)) {
            collisions.add(snapshot(shape));
        }
        result.add("states", resolvedStates);
        result.add("collisions", collisions);
        result.addProperty("no_block_collision", collision.noBlockCollision(null, testArea));
        return result;
    }

    static JsonObject shapeclientWorldStairNeighborCollisionCase(JsonObject request) {
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
        for (var inputElement : request.getAsJsonArray("inputs")) {
            JsonObject input = inputElement.getAsJsonObject();
            BlockPos targetPos = blockPos(input.getAsJsonObject("target"));
            var stateByPosition = registryStates(input.getAsJsonArray("placements"));
            String targetKey = registryKey(targetPos);
            BlockState before = stateByPosition.getOrDefault(
                targetKey, Blocks.AIR.defaultBlockState());
            if (!(before.getBlock() instanceof StairBlock)) {
                throw new IllegalArgumentException("stair target must resolve to a stair block");
            }
            StairsShape updatedShape = stairShape(
                before,
                registryCollisionFixture(stateByPosition),
                targetPos);
            BlockState updated = before.setValue(StairBlock.SHAPE, updatedShape);
            stateByPosition.put(targetKey, updated);
            CollisionGetter collision = registryCollisionFixture(stateByPosition);
            AABB testArea = box(input.getAsJsonObject("test_area"));
            var collisions = new JsonArray();
            for (VoxelShape shape : collision.getBlockCollisions(null, testArea)) {
                collisions.add(snapshot(shape));
            }
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("before_state_id", Block.getId(before));
            state.addProperty("updated_state_id", Block.getId(updated));
            state.addProperty("updated_shape", updatedShape.name().toLowerCase(Locale.ROOT));
            state.add("updated_collision_shape", snapshot(
                updated.getCollisionShape(net.minecraft.world.level.EmptyBlockGetter.INSTANCE, targetPos)));
            state.add("collisions", collisions);
            state.addProperty("no_block_collision", collision.noBlockCollision(null, testArea));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapelocalPlayerMinorHorizontalCollisionCase(JsonObject request) {
        var result = new JsonObject();
        try {
            Unsafe unsafe = unsafeInstance();
            var method = Reflect.method(LocalPlayer.class, 
                "isHorizontalCollisionMinor", Vec3.class);
            var yRotField = Reflect.field(Entity.class, "yRot");
            var xxaField = Reflect.field(LivingEntity.class, "xxa");
            var zzaField = Reflect.field(LivingEntity.class, "zza");
            var states = new JsonArray();
            for (var inputEntry : request.getAsJsonArray("inputs")) {
                JsonObject input = inputEntry.getAsJsonObject();
                LocalPlayer player = (LocalPlayer)unsafe.allocateInstance(LocalPlayer.class);
                unsafe.putFloat(player, unsafe.objectFieldOffset(yRotField),
                    Oracle.f32(input, "y_rot"));
                unsafe.putFloat(player, unsafe.objectFieldOffset(xxaField),
                    Oracle.f32(input, "xxa"));
                unsafe.putFloat(player, unsafe.objectFieldOffset(zzaField),
                    Oracle.f32(input, "zza"));
                Vec3 movement = vec3(input.getAsJsonObject("movement"));
                var state = new JsonObject();
                state.addProperty("label", input.get("label").getAsString());
                state.addProperty("minor_horizontal_collision",
                    (Boolean)method.invoke(player, movement));
                states.add(state);
            }
            result.add("states", states);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "LocalPlayer.isHorizontalCollisionMinor reflection failed", error);
        }
        return result;
    }
}
