import com.google.gson.JsonObject;
import java.util.List;
import net.minecraft.client.player.RemotePlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.entity.EntityInLevelCallback;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathEntityFixtures extends MathCollisionFixtures {
    static final class SharedFlagsFixtureEntity extends Entity {
        static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();

        SharedFlagsFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.ARMOR_STAND, CLIENT_LEVEL);
        }

        static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(HappyGhastClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate metadata client level fixture", error);
            }
        }

        int sharedFlags() {
            return Byte.toUnsignedInt(this.getEntityData().get(DATA_SHARED_FLAGS_ID));
        }

        @Override
        protected void defineSynchedData(net.minecraft.network.syncher.SynchedEntityData.Builder builder) {
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

    static final class LivingFlagsFixtureEntity
        extends net.minecraft.world.entity.decoration.ArmorStand {
        static final net.minecraft.network.syncher.EntityDataAccessor<
            List<net.minecraft.core.particles.ParticleOptions>> EFFECT_PARTICLES_DATA =
            new net.minecraft.network.syncher.EntityDataAccessor<>(
                10, net.minecraft.network.syncher.EntityDataSerializers.PARTICLES);
        static final net.minecraft.network.syncher.EntityDataAccessor<Boolean>
            EFFECT_AMBIENCE_DATA = new net.minecraft.network.syncher.EntityDataAccessor<>(
                11, net.minecraft.network.syncher.EntityDataSerializers.BOOLEAN);

        LivingFlagsFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.ARMOR_STAND,
                SharedFlagsFixtureEntity.CLIENT_LEVEL);
        }

        int livingFlags() {
            return Byte.toUnsignedInt(this.getEntityData().get(DATA_LIVING_ENTITY_FLAGS));
        }

        boolean effectAmbience() {
            return this.getEntityData().get(EFFECT_AMBIENCE_DATA);
        }

        List<Integer> effectParticleTypeIds() {
            return this.getEntityData().get(EFFECT_PARTICLES_DATA).stream()
                .map(particle -> BuiltInRegistries.PARTICLE_TYPE.getId(particle.getType()))
                .toList();
        }
    }

    static final class MobFlagsFixtureEntity
        extends net.minecraft.world.entity.animal.golem.IronGolem {
        MobFlagsFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.IRON_GOLEM,
                SharedFlagsFixtureEntity.CLIENT_LEVEL);
        }

        int mobFlags() {
            return (this.isNoAi() ? 1 : 0)
                | (this.isLeftHanded() ? 2 : 0)
                | (this.isAggressive() ? 4 : 0);
        }
    }

    static final class RemotePlayerInterpolationFixture extends RemotePlayer {
        static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();

        RemotePlayerInterpolationFixture() {
            super(null, null);
        }

        static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(InterpolationClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to allocate interpolation client level fixture", error);
            }
        }

        static RemotePlayerInterpolationFixture create(
            Vec3 position, float yRot, float xRot, float yHeadRot
        ) {
            return create(position, yRot, xRot, yHeadRot, Vec3.ZERO);
        }

        static RemotePlayerInterpolationFixture create(
            Vec3 position, float yRot, float xRot, float yHeadRot, Vec3 initialMotion
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                RemotePlayerInterpolationFixture player =
                    (RemotePlayerInterpolationFixture)unsafe.allocateInstance(
                        RemotePlayerInterpolationFixture.class);
                var typeField = Reflect.field(Entity.class, "type");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(typeField),
                    net.minecraft.world.entity.EntityTypes.PLAYER);
                var positionField = Reflect.field(Entity.class, "position");
                unsafe.putObject(player, unsafe.objectFieldOffset(positionField), position);
                var deltaMovementField = Reflect.field(Entity.class, "deltaMovement");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(deltaMovementField), initialMotion);
                var blockPositionField = Reflect.field(Entity.class, "blockPosition");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(blockPositionField),
                    BlockPos.containing(position.x, position.y, position.z));
                var chunkPositionField = Reflect.field(Entity.class, "chunkPosition");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(chunkPositionField),
                    ChunkPos.containing(BlockPos.containing(position.x, position.y, position.z)));
                var passengersField = Reflect.field(Entity.class, "passengers");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                player.setLevelCallback(EntityInLevelCallback.NULL);
                var interpolationField = Reflect.field(LivingEntity.class, "interpolation");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(interpolationField),
                    new net.minecraft.world.entity.InterpolationHandler(player));
                player.setYRot(yRot);
                player.setXRot(xRot);
                player.yHeadRot = yHeadRot;
                player.yHeadRotO = yHeadRot;
                player.yBodyRot = yRot;
                player.yBodyRotO = yRot;
                player.setOldPosAndRot();
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct remote player interpolation fixture", error);
            }
        }

        int interpolationSteps() {
            try {
                var dataField = Reflect.field(net.minecraft.world.entity.InterpolationHandler.class, "interpolationData");
                Object data = dataField.get(this.getInterpolation());
                var stepsField = Reflect.field(data.getClass(), "steps");
                return stepsField.getInt(data);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to read interpolation step fixture", error);
            }
        }

        Vec3 motionTarget() {
            try {
                var targetField = Reflect.field(RemotePlayer.class, "lerpDeltaMovement");
                return (Vec3)targetField.get(this);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to read remote player motion target fixture", error);
            }
        }

        int motionSteps() {
            try {
                var stepsField = Reflect.field(RemotePlayer.class, "lerpDeltaMovementSteps");
                return stepsField.getInt(this);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to read remote player motion steps fixture", error);
            }
        }

        void setMotionSteps(int steps) {
            try {
                var stepsField = Reflect.field(RemotePlayer.class, "lerpDeltaMovementSteps");
                stepsField.setInt(this, steps);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to write remote player motion steps fixture", error);
            }
        }

        void tickRemotePlayerMotion() {
            int steps = this.motionSteps();
            if (steps > 0) {
                Vec3 target = this.motionTarget();
                Vec3 current = this.getDeltaMovement();
                this.addDeltaMovement(new Vec3(
                    (target.x - current.x) / steps,
                    (target.y - current.y) / steps,
                    (target.z - current.z) / steps));
                this.setMotionSteps(steps - 1);
            }
        }

        void applyTeleport(
            Vec3 changePosition, Vec3 changeMotion, float yRot, float xRot,
            int relativeFlags, boolean interpolate
        ) {
            var relatives = net.minecraft.world.entity.Relative.unpack(relativeFlags);
            var change = new net.minecraft.world.entity.PositionMoveRotation(
                changePosition, changeMotion, yRot, xRot);
            var currentValues = net.minecraft.world.entity.PositionMoveRotation.of(this);
            var newValues = net.minecraft.world.entity.PositionMoveRotation.calculateAbsolute(
                currentValues, change, relatives);
            boolean tooBigToInterpolate = currentValues.position().distanceToSqr(
                newValues.position()) > 4096.0;
            if (interpolate && !tooBigToInterpolate) {
                this.moveOrInterpolateTo(
                    newValues.position(), newValues.yRot(), newValues.xRot());
                this.setDeltaMovement(newValues.deltaMovement());
            } else {
                this.setPos(newValues.position());
                this.setDeltaMovement(newValues.deltaMovement());
                this.setYRot(newValues.yRot());
                this.setXRot(newValues.xRot());
                var currentInterpolationValues =
                    new net.minecraft.world.entity.PositionMoveRotation(
                        this.oldPosition(), Vec3.ZERO, this.yRotO, this.xRotO);
                var interpolationValues =
                    net.minecraft.world.entity.PositionMoveRotation.calculateAbsolute(
                        currentInterpolationValues, change, relatives);
                this.setOldPosAndRot(
                    interpolationValues.position(),
                    interpolationValues.yRot(),
                    interpolationValues.xRot());
            }
        }

        void applyPositionSync(Vec3 position, float yRot, float xRot) {
            boolean withinInterpolationRange = this.position().distanceToSqr(position) <= 4096.0;
            if (((InterpolationClientLevelFixture)CLIENT_LEVEL).isTickingEntity(this)
                && withinInterpolationRange) {
                this.moveOrInterpolateTo(position, yRot, xRot);
            } else {
                this.snapTo(position, yRot, xRot);
            }
        }

        void tickThroughClientLevel() {
            this.setOldPosAndRot();
            this.tickCount++;
            if (this.isInterpolating()) {
                this.getInterpolation().interpolate();
            }
        }

        void tickThroughRemotePlayer() {
            this.setOldPosAndRot();
            this.tickCount++;
            this.tickRemotePlayerMotion();
        }

        @Override
        protected AABB makeBoundingBox(Vec3 position) {
            return AABB.ofSize(position, 0.6, 1.8, 0.6);
        }

        @Override
        public net.minecraft.world.level.Level level() {
            return CLIENT_LEVEL;
        }

        @Override
        protected void updateSwingTime() {
        }

        @Override
        protected void updateBob() {
        }

        @Override
        protected void pushEntities() {
        }
    }

}
