import java.util.List;
import net.minecraft.client.player.RemotePlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.animal.cow.Cow;
import net.minecraft.world.entity.monster.breeze.Breeze;
import net.minecraft.world.entity.vehicle.boat.Boat;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.entity.EntityInLevelCallback;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathPassengerFixtures extends MathOptionOracle {
    static final class PassengerRelationFixtureEntity extends Entity {
        static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();

        PassengerRelationFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.ARMOR_STAND, CLIENT_LEVEL);
        }

        static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(InterpolationClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to allocate passenger relation client level fixture", error);
            }
        }

        static PassengerRelationFixtureEntity create(int entityId) {
            return create(entityId, Vec3.ZERO);
        }

        static PassengerRelationFixtureEntity create(int entityId, Vec3 position) {
            try {
                PassengerRelationFixtureEntity entity = new PassengerRelationFixtureEntity();
                var idField = Entity.class.getDeclaredField("id");
                unsafeInstance().putInt(entity, unsafeInstance().objectFieldOffset(idField), entityId);
                entity.setPos(position);
                entity.setOldPosAndRot();
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger relation fixture entity", error);
            }
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public void tick() {
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

    static void initializePassengerPositionVehicle(
        Entity vehicle,
        int entityId,
        net.minecraft.world.entity.EntityType<?> type,
        Vec3 position,
        float yRot
    ) throws ReflectiveOperationException {
        Unsafe unsafe = unsafeInstance();
        var idField = Entity.class.getDeclaredField("id");
        unsafe.putInt(vehicle, unsafe.objectFieldOffset(idField), entityId);
        var typeField = Entity.class.getDeclaredField("type");
        unsafe.putObject(vehicle, unsafe.objectFieldOffset(typeField), type);
        var levelField = Entity.class.getDeclaredField("level");
        unsafe.putObject(
            vehicle,
            unsafe.objectFieldOffset(levelField),
            PassengerRelationFixtureEntity.CLIENT_LEVEL);
        var dimensionsField = Entity.class.getDeclaredField("dimensions");
        unsafe.putObject(
            vehicle,
            unsafe.objectFieldOffset(dimensionsField),
            type.getDimensions());
        var positionField = Entity.class.getDeclaredField("position");
        unsafe.putObject(vehicle, unsafe.objectFieldOffset(positionField), position);
        var blockPositionField = Entity.class.getDeclaredField("blockPosition");
        unsafe.putObject(
            vehicle,
            unsafe.objectFieldOffset(blockPositionField),
            BlockPos.containing(position.x, position.y, position.z));
        var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
        unsafe.putObject(
            vehicle,
            unsafe.objectFieldOffset(chunkPositionField),
            ChunkPos.containing(BlockPos.containing(position.x, position.y, position.z)));
        var passengersField = Entity.class.getDeclaredField("passengers");
        unsafe.putObject(
            vehicle,
            unsafe.objectFieldOffset(passengersField),
            com.google.common.collect.ImmutableList.of());
        vehicle.setLevelCallback(EntityInLevelCallback.NULL);
        vehicle.setDeltaMovement(Vec3.ZERO);
        vehicle.setYRot(yRot);
        vehicle.setXRot(0.0F);
        vehicle.yRotO = yRot;
        vehicle.xRotO = 0.0F;
        vehicle.setOldPosAndRot();
    }

    static final class PassengerPositionBoatFixture extends Boat {
        boolean noGravity;

        PassengerPositionBoatFixture() {
            super(null, null, null);
        }

        static PassengerPositionBoatFixture create(
            int entityId, Vec3 position, float yRot, boolean noGravity
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionBoatFixture boat =
                    (PassengerPositionBoatFixture)unsafe.allocateInstance(
                        PassengerPositionBoatFixture.class);
                initializePassengerPositionVehicle(
                    boat,
                    entityId,
                    net.minecraft.world.entity.EntityTypes.OAK_BOAT,
                    position,
                    yRot);
                boat.noGravity = noGravity;
                return boat;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger position boat fixture", error);
            }
        }

        void setFixturePassengers(List<? extends Entity> passengers) {
            setPassengerPositionFixturePassengers(this, passengers);
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }
    }

    static final class PassengerPositionChestBoatFixture
        extends net.minecraft.world.entity.vehicle.boat.ChestBoat {
        boolean noGravity;

        PassengerPositionChestBoatFixture() {
            super(null, null, null);
        }

        static PassengerPositionChestBoatFixture create(
            int entityId, Vec3 position, float yRot, boolean noGravity
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionChestBoatFixture boat =
                    (PassengerPositionChestBoatFixture)unsafe.allocateInstance(
                        PassengerPositionChestBoatFixture.class);
                initializePassengerPositionVehicle(
                    boat,
                    entityId,
                    net.minecraft.world.entity.EntityTypes.OAK_CHEST_BOAT,
                    position,
                    yRot);
                boat.noGravity = noGravity;
                return boat;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger position chest boat fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }
    }

    static final class PassengerPositionRaftFixture
        extends net.minecraft.world.entity.vehicle.boat.Raft {
        boolean noGravity;

        PassengerPositionRaftFixture() {
            super(null, null, null);
        }

        static PassengerPositionRaftFixture create(
            int entityId, Vec3 position, float yRot, boolean noGravity
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionRaftFixture raft =
                    (PassengerPositionRaftFixture)unsafe.allocateInstance(
                        PassengerPositionRaftFixture.class);
                initializePassengerPositionVehicle(
                    raft,
                    entityId,
                    net.minecraft.world.entity.EntityTypes.BAMBOO_RAFT,
                    position,
                    yRot);
                raft.noGravity = noGravity;
                return raft;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger position raft fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }
    }

    static final class PassengerPositionChestRaftFixture
        extends net.minecraft.world.entity.vehicle.boat.ChestRaft {
        boolean noGravity;

        PassengerPositionChestRaftFixture() {
            super(null, null, null);
        }

        static PassengerPositionChestRaftFixture create(
            int entityId, Vec3 position, float yRot, boolean noGravity
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionChestRaftFixture raft =
                    (PassengerPositionChestRaftFixture)unsafe.allocateInstance(
                        PassengerPositionChestRaftFixture.class);
                initializePassengerPositionVehicle(
                    raft,
                    entityId,
                    net.minecraft.world.entity.EntityTypes.BAMBOO_CHEST_RAFT,
                    position,
                    yRot);
                raft.noGravity = noGravity;
                return raft;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger position chest raft fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }
    }

    static final class PassengerPositionPlayerFixture extends RemotePlayer {
        boolean noGravity;

        PassengerPositionPlayerFixture() {
            super(null, null);
        }

        static PassengerPositionPlayerFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity, Entity vehicle
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionPlayerFixture player =
                    (PassengerPositionPlayerFixture)unsafe.allocateInstance(
                        PassengerPositionPlayerFixture.class);
                var idField = Entity.class.getDeclaredField("id");
                unsafe.putInt(player, unsafe.objectFieldOffset(idField), entityId);
                var typeField = Entity.class.getDeclaredField("type");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(typeField),
                    net.minecraft.world.entity.EntityTypes.PLAYER);
                var levelField = Entity.class.getDeclaredField("level");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(levelField),
                    PassengerRelationFixtureEntity.CLIENT_LEVEL);
                var dimensionsField = Entity.class.getDeclaredField("dimensions");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(dimensionsField),
                    net.minecraft.world.entity.EntityTypes.PLAYER.getDimensions());
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(player, unsafe.objectFieldOffset(positionField), position);
                var deltaMovementField = Entity.class.getDeclaredField("deltaMovement");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(deltaMovementField), movement);
                var blockPositionField = Entity.class.getDeclaredField("blockPosition");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(blockPositionField),
                    BlockPos.containing(position.x, position.y, position.z));
                var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(BlockPos.containing(position.x, position.y, position.z)));
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                var vehicleField = Entity.class.getDeclaredField("vehicle");
                unsafe.putObject(player, unsafe.objectFieldOffset(vehicleField), vehicle);
                player.setLevelCallback(EntityInLevelCallback.NULL);
                player.noGravity = noGravity;
                player.setYRot(yRot);
                player.setXRot(xRot);
                player.yRotO = yRot;
                player.xRotO = xRot;
                player.yHeadRot = yRot;
                player.yHeadRotO = yRot;
                player.yBodyRot = yRot;
                player.yBodyRotO = yRot;
                player.setOldPosAndRot();
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger player position fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }

        @Override
        public void rideTick() {
            this.setDeltaMovement(Vec3.ZERO);
            this.tick();
            if (this.isPassenger()) {
                this.getVehicle().positionRider(this);
            }
        }
    }

    static final class PassengerPositionBreezeFixture extends Breeze {
        boolean noGravity;

        PassengerPositionBreezeFixture() {
            super(null, null);
        }

        static PassengerPositionBreezeFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity, Entity vehicle
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionBreezeFixture breeze =
                    (PassengerPositionBreezeFixture)unsafe.allocateInstance(
                        PassengerPositionBreezeFixture.class);
                var idField = Entity.class.getDeclaredField("id");
                unsafe.putInt(breeze, unsafe.objectFieldOffset(idField), entityId);
                var typeField = Entity.class.getDeclaredField("type");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(typeField),
                    net.minecraft.world.entity.EntityTypes.BREEZE);
                var levelField = Entity.class.getDeclaredField("level");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(levelField),
                    PassengerRelationFixtureEntity.CLIENT_LEVEL);
                var dimensionsField = Entity.class.getDeclaredField("dimensions");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(dimensionsField),
                    net.minecraft.world.entity.EntityTypes.BREEZE.getDimensions());
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(breeze, unsafe.objectFieldOffset(positionField), position);
                var deltaMovementField = Entity.class.getDeclaredField("deltaMovement");
                unsafe.putObject(
                    breeze, unsafe.objectFieldOffset(deltaMovementField), movement);
                var blockPositionField = Entity.class.getDeclaredField("blockPosition");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(blockPositionField),
                    BlockPos.containing(position.x, position.y, position.z));
                var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(BlockPos.containing(position.x, position.y, position.z)));
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    breeze,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                var vehicleField = Entity.class.getDeclaredField("vehicle");
                unsafe.putObject(breeze, unsafe.objectFieldOffset(vehicleField), vehicle);
                breeze.setLevelCallback(EntityInLevelCallback.NULL);
                breeze.noGravity = noGravity;
                breeze.setYRot(yRot);
                breeze.setXRot(xRot);
                breeze.yRotO = yRot;
                breeze.xRotO = xRot;
                breeze.yHeadRot = yRot;
                breeze.yHeadRotO = yRot;
                breeze.yBodyRot = yRot;
                breeze.yBodyRotO = yRot;
                breeze.setOldPosAndRot();
                return breeze;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger Breeze turn exemption fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }

    }

    static final class PassengerPositionCowFixture extends Cow {
        boolean noGravity;

        PassengerPositionCowFixture() {
            super(null, null);
        }

        static PassengerPositionCowFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity, Entity vehicle
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PassengerPositionCowFixture cow =
                    (PassengerPositionCowFixture)unsafe.allocateInstance(
                        PassengerPositionCowFixture.class);
                var idField = Entity.class.getDeclaredField("id");
                unsafe.putInt(cow, unsafe.objectFieldOffset(idField), entityId);
                var typeField = Entity.class.getDeclaredField("type");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(typeField),
                    net.minecraft.world.entity.EntityTypes.COW);
                var levelField = Entity.class.getDeclaredField("level");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(levelField),
                    PassengerRelationFixtureEntity.CLIENT_LEVEL);
                var dimensionsField = Entity.class.getDeclaredField("dimensions");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(dimensionsField),
                    net.minecraft.world.entity.EntityTypes.COW.getDimensions());
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(cow, unsafe.objectFieldOffset(positionField), position);
                var deltaMovementField = Entity.class.getDeclaredField("deltaMovement");
                unsafe.putObject(cow, unsafe.objectFieldOffset(deltaMovementField), movement);
                var blockPositionField = Entity.class.getDeclaredField("blockPosition");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(blockPositionField),
                    BlockPos.containing(position.x, position.y, position.z));
                var chunkPositionField = Entity.class.getDeclaredField("chunkPosition");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(BlockPos.containing(position.x, position.y, position.z)));
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    cow,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                var vehicleField = Entity.class.getDeclaredField("vehicle");
                unsafe.putObject(cow, unsafe.objectFieldOffset(vehicleField), vehicle);
                cow.setLevelCallback(EntityInLevelCallback.NULL);
                cow.noGravity = noGravity;
                cow.setYRot(yRot);
                cow.setXRot(xRot);
                cow.yRotO = yRot;
                cow.xRotO = xRot;
                cow.yHeadRot = yRot;
                cow.yHeadRotO = yRot;
                cow.yBodyRot = yRot;
                cow.yBodyRotO = yRot;
                cow.setOldPosAndRot();
                return cow;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct passenger Cow rotation fixture", error);
            }
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        @Override
        public void tick() {
        }
    }
}
