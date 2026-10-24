import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import io.netty.buffer.Unpooled;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.WorldVersion;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Options;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeMap;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;
import org.joml.Quaternionf;
import org.joml.Vector3f;
import sun.misc.Unsafe;

class MathSupport {
    // Shared with the other bootstrapping families; see Oracle.
    static final WorldVersion TEST_VERSION = Oracle.TEST_VERSION;

    static Holder<Attribute> attributeHolder(int attributeId) {
        Holder<Attribute> holder = BuiltInRegistries.ATTRIBUTE.asHolderIdMap().byId(attributeId);
        if (holder == null) {
            throw new IllegalArgumentException("Unknown attribute registry ID: " + attributeId);
        }
        return holder;
    }

    static void applyPlayerAttributeSnapshot(AttributeMap attributes, JsonObject snapshot) {
        Holder<Attribute> holder = attributeHolder(snapshot.get("attribute_id").getAsInt());
        AttributeInstance instance = attributes.getInstance(holder);
        if (instance == null) {
            throw new IllegalArgumentException("Player has no requested attribute instance");
        }
        instance.setBaseValue(f64(snapshot, "base"));
        instance.removeModifiers();
        for (JsonElement modifierEntry : snapshot.getAsJsonArray("modifiers")) {
            JsonObject modifier = modifierEntry.getAsJsonObject();
            AttributeModifier.Operation operation = AttributeModifier.Operation.BY_ID.apply(
                modifier.get("operation").getAsInt());
            instance.addTransientModifier(new AttributeModifier(
                net.minecraft.resources.Identifier.parse(modifier.get("id").getAsString()),
                f64(modifier, "amount"),
                operation));
        }
    }

    static JsonObject tickEntityState(Entity entity) {
        var state = new JsonObject();
        state.addProperty("entity_id", entity.getId());
        state.addProperty("tick_count", entity.tickCount);
        state.add("position", snapshotClip(entity.position()));
        state.add("previous_position", snapshotClip(entity.oldPosition()));
        state.add("motion", snapshotClip(entity.getDeltaMovement()));
        state.addProperty("rotation_y", Oracle.bits(entity.getYRot()));
        state.addProperty("rotation_x", Oracle.bits(entity.getXRot()));
        state.addProperty("previous_rotation_y", Oracle.bits(entity.yRotO));
        state.addProperty("previous_rotation_x", Oracle.bits(entity.xRotO));
        state.addProperty("no_gravity", entity.isNoGravity());
        return state;
    }

    static double f64(JsonObject object, String name) {
        return f64(object.get(name).getAsString());
    }

    static double f64(String bits) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(bits, 16));
    }

    static byte[] hexBytes(String value) {
        if (!value.isEmpty() && (value.length() & 1) != 0) {
            throw new IllegalArgumentException("hex must contain complete byte pairs");
        }
        byte[] bytes = new byte[value.length() / 2];
        for (int i = 0; i < bytes.length; i++) {
            int high = Character.digit(value.charAt(i * 2), 16);
            int low = Character.digit(value.charAt(i * 2 + 1), 16);
            if (high < 0 || low < 0) {
                throw new IllegalArgumentException("invalid hex byte");
            }
            bytes[i] = (byte)((high << 4) | low);
        }
        return bytes;
    }

    static List<SynchedEntityData.DataValue<?>> decodeEntityMetadata(String value) {
        return decodeEntityMetadata(value, RegistryAccess.EMPTY);
    }

    static List<SynchedEntityData.DataValue<?>> decodeEntityMetadata(
        String value, RegistryAccess registryAccess
    ) {
        RegistryFriendlyByteBuf input = new RegistryFriendlyByteBuf(
            Unpooled.wrappedBuffer(hexBytes(value)), registryAccess);
        try {
            var values = new ArrayList<SynchedEntityData.DataValue<?>>();
            while (input.isReadable()) {
                int id = input.readUnsignedByte();
                if (id == 255) {
                    if (input.isReadable()) {
                        throw new IllegalArgumentException("metadata has trailing bytes");
                    }
                    return values;
                }
                values.add(SynchedEntityData.DataValue.read(input, id));
            }
            throw new IllegalArgumentException("metadata has no terminator");
        } finally {
            input.release();
        }
    }

    static void setPassengerPositionFixturePassengers(
        Entity vehicle, List<? extends Entity> passengers
    ) {
        try {
            Unsafe unsafe = unsafeInstance();
            var passengersField = Reflect.field(Entity.class, "passengers");
            unsafe.putObject(
                vehicle,
                unsafe.objectFieldOffset(passengersField),
                com.google.common.collect.ImmutableList.copyOf(passengers));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(
                "Unable to update passenger position fixture relation", error);
        }
    }

    static CollisionGetter registryCollisionFixture(JsonArray placements) {
        return registryCollisionFixture(registryStates(placements));
    }

    static java.util.Map<String, BlockState> registryStates(JsonArray placements) {
        var states = new java.util.HashMap<String, BlockState>();
        for (var entry : placements) {
            JsonObject placement = entry.getAsJsonObject();
            int x = placement.get("x").getAsInt();
            int y = placement.get("y").getAsInt();
            int z = placement.get("z").getAsInt();
            states.put(
                x + "," + y + "," + z,
                Block.stateById(placement.get("state_id").getAsInt()));
        }
        return states;
    }

    static String registryKey(BlockPos pos) {
        return pos.getX() + "," + pos.getY() + "," + pos.getZ();
    }

    static CollisionGetter registryCollisionFixture(
        java.util.Map<String, BlockState> states
    ) {
        return new CollisionGetter() {
            private String key(BlockPos pos) {
                return registryKey(pos);
            }

            @Override
            public WorldBorder getWorldBorder() {
                return null;
            }

            @Override
            public BlockGetter getChunkForCollisions(int chunkX, int chunkZ) {
                return this;
            }

            @Override
            public java.util.List<VoxelShape> getEntityCollisions(Entity source, AABB testArea) {
                return java.util.List.of();
            }

            @Override
            public BlockEntity getBlockEntity(BlockPos pos) {
                return null;
            }

            @Override
            public BlockState getBlockState(BlockPos pos) {
                return states.getOrDefault(key(pos), Blocks.AIR.defaultBlockState());
            }

            @Override
            public FluidState getFluidState(BlockPos pos) {
                return getBlockState(pos).getFluidState();
            }

            @Override
            public int getHeight() {
                return 384;
            }

            @Override
            public int getMinY() {
                return -64;
            }
        };
    }

    static AABB box(JsonObject object) {
        return new AABB(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    static VoxelShape voxelShape(JsonObject object) {
        return Shapes.box(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    static BlockPos blockPos(JsonObject object) {
        return new BlockPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("y").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    static Vec3 vec3(JsonObject object) {
        return new Vec3(f64(object, "x"), f64(object, "y"), f64(object, "z"));
    }

    static ChunkPos chunkPos(JsonObject object) {
        return new ChunkPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    static JsonObject snapshot(AABB box) {
        var state = new JsonObject();
        state.addProperty("min_x", Oracle.bits(box.minX));
        state.addProperty("min_y", Oracle.bits(box.minY));
        state.addProperty("min_z", Oracle.bits(box.minZ));
        state.addProperty("max_x", Oracle.bits(box.maxX));
        state.addProperty("max_y", Oracle.bits(box.maxY));
        state.addProperty("max_z", Oracle.bits(box.maxZ));
        return state;
    }

    static JsonObject snapshot(EntityDimensions dimensions) {
        var state = new JsonObject();
        state.addProperty("width", Oracle.bits(dimensions.width()));
        state.addProperty("height", Oracle.bits(dimensions.height()));
        state.addProperty("eye_height", Oracle.bits(dimensions.eyeHeight()));
        state.addProperty("fixed", dimensions.fixed());
        return state;
    }

    static JsonObject snapshot(VoxelShape shape) {
        var state = new JsonObject();
        state.addProperty("empty", shape.isEmpty());
        var boxes = new JsonArray();
        for (AABB box : shape.toAabbs()) {
            boxes.add(snapshot(box));
        }
        state.add("boxes", boxes);
        if (!shape.isEmpty()) {
            state.add("bounds", snapshot(shape.bounds()));
        }
        return state;
    }

    static JsonObject snapshot(BlockPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.getX()));
        state.addProperty("y", Integer.toString(pos.getY()));
        state.addProperty("z", Integer.toString(pos.getZ()));
        return state;
    }

    static JsonObject snapshot(ChunkPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("z", Integer.toString(pos.z()));
        return state;
    }

    static JsonObject snapshot(SectionPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("y", Integer.toString(pos.y()));
        state.addProperty("z", Integer.toString(pos.z()));
        state.addProperty("packed", Long.toUnsignedString(pos.asLong(), 16));
        return state;
    }

    static JsonObject snapshot(Direction facing) {
        var state = new JsonObject();
        state.addProperty("name", facing.getName());
        state.addProperty("data3d", facing.get3DDataValue());
        state.addProperty("data2d", facing.get2DDataValue());
        state.addProperty("opposite", facing.getOpposite().getName());
        state.addProperty("axis", facing.getAxis().getName());
        state.addProperty("axis_direction", facing.getAxisDirection().getName());
        state.addProperty("step_x", facing.getStepX());
        state.addProperty("step_y", facing.getStepY());
        state.addProperty("step_z", facing.getStepZ());
        return state;
    }

    static JsonObject snapshot(BlockHitResult hit) {
        var state = new JsonObject();
        if (hit == null) {
            state.addProperty("hit", false);
            return state;
        }
        state.addProperty("hit", true);
        state.addProperty("type", hit.getType().name());
        state.add("location", snapshotClip(hit.getLocation()));
        state.addProperty("direction", hit.getDirection().getName());
        state.add("block_pos", snapshot(hit.getBlockPos()));
        state.addProperty("inside", hit.isInside());
        state.addProperty("world_border_hit", hit.isWorldBorderHit());
        return state;
    }

    static JsonObject snapshot(Vec3 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSqr()));
        return state;
    }

    static JsonObject snapshot(Vector3f vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        return state;
    }

    static JsonObject snapshot(Quaternionf quaternion) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(quaternion.x));
        state.addProperty("y", Oracle.bits(quaternion.y));
        state.addProperty("z", Oracle.bits(quaternion.z));
        state.addProperty("w", Oracle.bits(quaternion.w));
        return state;
    }

    static JsonObject snapshotClip(Vec3 point) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(point.x));
        state.addProperty("y", Oracle.bits(point.y));
        state.addProperty("z", Oracle.bits(point.z));
        return state;
    }

    static JsonObject snapshot(Vec2 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSquared()));
        return state;
    }

    static JsonObject snapshot(Input input) {
        var state = new JsonObject();
        state.addProperty("forward", input.forward());
        state.addProperty("backward", input.backward());
        state.addProperty("left", input.left());
        state.addProperty("right", input.right());
        state.addProperty("jump", input.jump());
        state.addProperty("shift", input.shift());
        state.addProperty("sprint", input.sprint());
        return state;
    }

    static Options optionsFor(Input keys) {
        try {
            Unsafe unsafe = unsafeInstance();
            Options options = (Options)unsafe.allocateInstance(Options.class);
            putKey(unsafe, options, "keyUp", "behavior.keyboard.up", 100, keys.forward());
            putKey(unsafe, options, "keyDown", "behavior.keyboard.down", 101, keys.backward());
            putKey(unsafe, options, "keyLeft", "behavior.keyboard.left", 102, keys.left());
            putKey(unsafe, options, "keyRight", "behavior.keyboard.right", 103, keys.right());
            putKey(unsafe, options, "keyJump", "behavior.keyboard.jump", 104, keys.jump());
            putKey(unsafe, options, "keyShift", "behavior.keyboard.shift", 105, keys.shift());
            putKey(unsafe, options, "keySprint", "behavior.keyboard.sprint", 106, keys.sprint());
            return options;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Unable to construct controlled keyboard options", error);
        }
    }

    static final class InterpolationClientLevelFixture
        extends net.minecraft.client.multiplayer.ClientLevel {
        InterpolationClientLevelFixture() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        @Override
        public boolean isClientSide() {
            return true;
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlagSet.of();
        }

        @Override
        public boolean noCollision(Entity source, AABB box) {
            return true;
        }

        @Override
        public boolean isTickingEntity(Entity entity) {
            return true;
        }
    }

    static Unsafe unsafeInstance() {
        return Reflect.unsafe();
    }

    static void putKey(Unsafe unsafe, Options options, String fieldName, String name,
                               int keyCode, boolean down) throws ReflectiveOperationException {
        KeyMapping key = new KeyMapping(name, keyCode, KeyMapping.Category.MOVEMENT);
        key.setDown(down);
        var field = Options.class.getField(fieldName);
        unsafe.putObject(options, unsafe.objectFieldOffset(field), key);
    }
}
