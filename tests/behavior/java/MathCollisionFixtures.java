import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import java.util.UUID;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.EntityDimensions;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.entity.monster.Shulker;
import net.minecraft.world.entity.vehicle.boat.Boat;
import net.minecraft.world.entity.vehicle.minecart.Minecart;
import net.minecraft.world.level.BlockGetter;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.CollisionGetter;
import net.minecraft.world.level.EntityGetter;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.SlabBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.SlabType;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.entity.LevelEntityGetter;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.VoxelShape;
import sun.misc.Unsafe;

class MathCollisionFixtures extends MathWorldLevelOracle {
    static final class CollisionFixtureEntity extends Entity {
        boolean spectator;
        boolean collidable;
        boolean sameVehicle;
        boolean pushable;

        CollisionFixtureEntity(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle
        ) {
            super(net.minecraft.world.entity.EntityTypes.ARMOR_STAND, null);
            this.spectator = spectator;
            this.collidable = collidable;
            this.sameVehicle = sameVehicle;
            this.setBoundingBox(boundingBox);
        }

        static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle
        ) {
            return create(boundingBox, spectator, collidable, sameVehicle, null, false);
        }

        static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle,
            Vec3 position
        ) {
            return create(boundingBox, spectator, collidable, sameVehicle, position, false);
        }

        static CollisionFixtureEntity create(
            AABB boundingBox, boolean spectator, boolean collidable, boolean sameVehicle,
            Vec3 position, boolean pushable
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                CollisionFixtureEntity entity =
                    (CollisionFixtureEntity)unsafe.allocateInstance(CollisionFixtureEntity.class);
                entity.spectator = spectator;
                entity.collidable = collidable;
                entity.sameVehicle = sameVehicle;
                entity.pushable = pushable;
                entity.setBoundingBox(boundingBox);
                if (position != null) {
                    var field = Entity.class.getDeclaredField("position");
                    unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                }
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate collision fixture entity", error);
            }
        }

        static Entity createIndexed(
            int entityId,
            int typeId,
            Vec3 position,
            boolean spectator,
            boolean alive,
            boolean onClimbable,
            boolean flying,
            boolean collidable,
            boolean sameVehicle,
            JsonObject climbableContext
        ) {
            var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
            if (entityType == null) {
                throw new IllegalArgumentException("Unknown entity type id " + typeId);
            }
            AABB boundingBox = entityType.getDimensions().makeBoundingBox(position);
            Entity entity = switch (typeId) {
                case 0 -> BoatCollisionFixtureEntity.create(boundingBox, position);
                case 25, 29, 56, 65, 85, 122, 134
                    -> MinecartCollisionFixtureEntity.create(boundingBox, position);
                case 112 -> ShulkerCollisionFixtureEntity.create(boundingBox, sameVehicle, position);
                case 156 -> PlayerCollisionFixtureEntity.create(
                    boundingBox, position, spectator, alive, onClimbable, flying,
                    climbableContext);
                default -> create(boundingBox, spectator, collidable, sameVehicle, position, false);
            };
            indexEntity(entity, entityId, position);
            return entity;
        }

        static Entity createHappyGhastIndexed(JsonObject input) {
            int entityId = input.get("entity_id").getAsInt();
            int typeId = input.get("type_id").getAsInt();
            Vec3 position = vec3(input.getAsJsonObject("position"));
            boolean baby = input.has("baby") && input.get("baby").getAsBoolean();
            boolean alive = !input.has("alive") || input.get("alive").getAsBoolean();
            boolean stillTimeout = input.has("still_timeout")
                && input.get("still_timeout").getAsBoolean();
            boolean vehicle = input.has("is_vehicle")
                && input.get("is_vehicle").getAsBoolean();
            boolean spectator = input.has("spectator")
                && input.get("spectator").getAsBoolean();
            boolean collidable = input.has("can_be_collided_with")
                && input.get("can_be_collided_with").getAsBoolean();
            boolean sameVehicle = input.has("same_vehicle")
                && input.get("same_vehicle").getAsBoolean();
            var entityType = BuiltInRegistries.ENTITY_TYPE.byId(typeId);
            if (entityType == null) {
                throw new IllegalArgumentException("Unknown entity type id " + typeId);
            }
            Entity entity;
            if (typeId == 58) {
                EntityDimensions dimensions = entityType.getDimensions();
                if (baby) {
                    dimensions = dimensions.scale(0.2375F).withEyeHeight(0.46875F);
                }
                entity = HappyGhastCollisionFixtureEntity.create(
                    dimensions.makeBoundingBox(position), position, baby, alive,
                    stillTimeout, vehicle);
            } else if (typeId == 156) {
                entity = PlayerCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), position);
            } else if (typeId == 0) {
                entity = BoatCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), position);
            } else if (typeId == 112) {
                entity = ShulkerCollisionFixtureEntity.create(
                    entityType.getDimensions().makeBoundingBox(position), sameVehicle, position);
            } else {
                entity = create(
                    entityType.getDimensions().makeBoundingBox(position), spectator,
                    collidable, sameVehicle, position, false);
            }
            indexEntity(entity, entityId, position);
            return entity;
        }

        static void indexEntity(Entity entity, int entityId, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                var idField = Entity.class.getDeclaredField("id");
                unsafe.putInt(entity, unsafe.objectFieldOffset(idField), entityId);
                var uuidField = Entity.class.getDeclaredField("uuid");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(uuidField),
                    new UUID(0L, Integer.toUnsignedLong(entityId)));
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                BlockPos blockPosition = BlockPos.containing(position.x, position.y, position.z);
                var blockPositionField = Entity.class.getDeclaredField("blockPosition");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(blockPositionField), blockPosition);
                var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
                unsafe.putObject(
                    entity, unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(blockPosition));
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to index collision fixture entity", error);
            }
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
        }

        @Override
        public boolean isSpectator() {
            return this.spectator;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public boolean canBeCollidedWith(Entity other) {
            return this.collidable;
        }

        @Override
        public boolean isPushable() {
            return this.pushable;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return other instanceof CollisionFixtureEntity fixture && fixture.sameVehicle;
        }

        @Override
        public boolean hurtServer(
            net.minecraft.server.level.ServerLevel level,
            net.minecraft.world.damagesource.DamageSource source,
            float damage
        ) {
            return false;
        }

        @Override
        protected void readAdditionalSaveData(net.minecraft.world.level.storage.ValueInput input) {
        }

        @Override
        protected void addAdditionalSaveData(net.minecraft.world.level.storage.ValueOutput output) {
        }
    }

    static final class BoatCollisionFixtureEntity extends Boat {
        BoatCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.OAK_BOAT, null, null);
        }

        static BoatCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                BoatCollisionFixtureEntity entity =
                    (BoatCollisionFixtureEntity)unsafe.allocateInstance(BoatCollisionFixtureEntity.class);
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate boat collision fixture entity", error);
            }
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    static final class MinecartCollisionFixtureEntity extends Minecart {
        static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();

        MinecartCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.MINECART, null);
        }

        static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(HappyGhastClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate minecart client level fixture", error);
            }
        }

        static MinecartCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            try {
                Unsafe unsafe = unsafeInstance();
                MinecartCollisionFixtureEntity entity =
                    (MinecartCollisionFixtureEntity)unsafe.allocateInstance(MinecartCollisionFixtureEntity.class);
                var levelField = Entity.class.getDeclaredField("level");
                unsafe.putObject(entity, unsafe.objectFieldOffset(levelField), CLIENT_LEVEL);
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate minecart collision fixture entity", error);
            }
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    static final class ShulkerCollisionFixtureEntity extends Shulker {
        boolean sameVehicle;

        ShulkerCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.SHULKER, null);
        }

        static ShulkerCollisionFixtureEntity create(
            AABB boundingBox, boolean sameVehicle, Vec3 position
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                ShulkerCollisionFixtureEntity entity =
                    (ShulkerCollisionFixtureEntity)unsafe.allocateInstance(ShulkerCollisionFixtureEntity.class);
                entity.sameVehicle = sameVehicle;
                entity.setBoundingBox(boundingBox);
                var field = Entity.class.getDeclaredField("position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate shulker collision fixture entity", error);
            }
        }

        @Override
        public boolean isAlive() {
            return true;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return other instanceof ShulkerCollisionFixtureEntity fixture && fixture.sameVehicle;
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public boolean hurtServer(
            net.minecraft.server.level.ServerLevel level,
            net.minecraft.world.damagesource.DamageSource source,
            float damage
        ) {
            return false;
        }

        @Override
        protected void readAdditionalSaveData(net.minecraft.world.level.storage.ValueInput input) {
        }

        @Override
        protected void addAdditionalSaveData(net.minecraft.world.level.storage.ValueOutput output) {
        }
    }

    static final class EntityCollisionFixture implements EntityGetter {
        final java.util.List<Entity> entities;

        EntityCollisionFixture(java.util.List<Entity> entities) {
            this.entities = entities;
        }

        @Override
        public java.util.List<Entity> getEntities(
            Entity except, AABB box, java.util.function.Predicate<? super Entity> selector
        ) {
            var result = new java.util.ArrayList<Entity>();
            for (Entity entity : this.entities) {
                if (entity != except && entity.getBoundingBox().intersects(box) && selector.test(entity)) {
                    result.add(entity);
                }
            }
            return result;
        }

        @Override
        public <T extends Entity> java.util.List<T> getEntities(
            net.minecraft.world.level.entity.EntityTypeTest<Entity, T> type,
            AABB box,
            java.util.function.Predicate<? super T> selector
        ) {
            return java.util.List.of();
        }

        @Override
        public java.util.List<? extends net.minecraft.world.entity.player.Player> players() {
            return java.util.List.of();
        }
    }

    static final class IndexedEntityCollisionFixture implements EntityGetter {
        final LevelEntityGetter<Entity> entityGetter;

        IndexedEntityCollisionFixture(LevelEntityGetter<Entity> entityGetter) {
            this.entityGetter = entityGetter;
        }

        @Override
        public java.util.List<Entity> getEntities(
            Entity except, AABB box, java.util.function.Predicate<? super Entity> selector
        ) {
            var result = new java.util.ArrayList<Entity>();
            this.entityGetter.get(box, entity -> {
                if (entity != except && selector.test(entity)) {
                    result.add(entity);
                }
            });
            return result;
        }

        @Override
        public <T extends Entity> java.util.List<T> getEntities(
            net.minecraft.world.level.entity.EntityTypeTest<Entity, T> type,
            AABB box,
            java.util.function.Predicate<? super T> selector
        ) {
            return java.util.List.of();
        }

        @Override
        public java.util.List<? extends net.minecraft.world.entity.player.Player> players() {
            return java.util.List.of();
        }
    }

    static CollisionGetter collisionFixture(JsonArray placements) {
        return collisionFixture(placements, java.util.List.of(), null);
    }

    static CollisionGetter collisionFixture(
        JsonArray placements, java.util.List<Entity> entities, WorldBorder worldBorder
    ) {
        return collisionFixture(
            placements, new EntityCollisionFixture(entities), worldBorder);
    }

    static CollisionGetter collisionFixture(
        JsonArray placements, EntityGetter entityGetter, WorldBorder worldBorder
    ) {
        var states = new java.util.HashMap<String, BlockState>();
        for (var entry : placements) {
            JsonObject placement = entry.getAsJsonObject();
            int x = placement.get("x").getAsInt();
            int y = placement.get("y").getAsInt();
            int z = placement.get("z").getAsInt();
            BlockState state = switch (placement.get("kind").getAsString()) {
                case "air" -> Blocks.AIR.defaultBlockState();
                case "stone" -> Blocks.STONE.defaultBlockState();
                case "slab_bottom" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.BOTTOM);
                case "slab_top" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.TOP);
                case "slab_double" -> Blocks.OAK_SLAB.defaultBlockState()
                    .setValue(SlabBlock.TYPE, SlabType.DOUBLE);
                default -> throw new IllegalArgumentException("Unknown collision fixture block");
            };
            states.put(x + "," + y + "," + z, state);
        }
        return new CollisionGetter() {
            private String key(BlockPos pos) {
                return pos.getX() + "," + pos.getY() + "," + pos.getZ();
            }

            @Override
            public WorldBorder getWorldBorder() {
                return worldBorder;
            }

            @Override
            public BlockGetter getChunkForCollisions(int chunkX, int chunkZ) {
                return this;
            }

            @Override
            public java.util.List<VoxelShape> getEntityCollisions(Entity source, AABB testArea) {
                return entityGetter.getEntityCollisions(source, testArea);
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
}
