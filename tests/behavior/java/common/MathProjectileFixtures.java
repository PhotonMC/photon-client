import com.google.gson.JsonObject;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.ExperienceOrb;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.item.FallingBlockEntity;
import net.minecraft.world.entity.item.PrimedTnt;
import net.minecraft.world.entity.projectile.FireworkRocketEntity;
import net.minecraft.world.entity.projectile.arrow.Arrow;
import net.minecraft.world.entity.projectile.throwableitemprojectile.Snowball;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathProjectileFixtures extends MathPlayerStateFixtures {
    static JsonObject tickSnowballEntityState(TickSnowballFixture snowball) {
        var state = new JsonObject();
        state.addProperty("entity_id", snowball.entityId());
        state.addProperty("tick_count", snowball.tickCount);
        state.add("position", snapshotClip(snowball.position()));
        state.add("previous_position", snapshotClip(snowball.previousPosition()));
        state.add("motion", snapshotClip(snowball.getDeltaMovement()));
        state.addProperty("rotation_y", Oracle.bits(snowball.getYRot()));
        state.addProperty("rotation_x", Oracle.bits(snowball.getXRot()));
        state.addProperty("previous_rotation_y", Oracle.bits(snowball.yRotO));
        state.addProperty("previous_rotation_x", Oracle.bits(snowball.xRotO));
        state.addProperty("no_gravity", snowball.noGravity);
        return state;
    }

    static final class TickSnowballFixture extends Snowball {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickSnowballFixture() {
            super(null, null);
        }

        static TickSnowballFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickSnowballFixture snowball;
            try {
                Unsafe unsafe = unsafeInstance();
                snowball = (TickSnowballFixture)unsafe.allocateInstance(TickSnowballFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct snowball tick fixture", error);
            }
            snowball.fixtureEntityId = entityId;
            snowball.fixturePosition = position;
            snowball.previousFixturePosition = position;
            snowball.noGravity = noGravity;
            snowball.setYRot(yRot);
            snowball.setXRot(xRot);
            snowball.yRotO = yRot;
            snowball.xRotO = xRot;
            snowball.setDeltaMovement(movement);
            try {
                Unsafe unsafe = unsafeInstance();
                var idField = Reflect.field(Entity.class, "id");
                unsafe.putInt(snowball, unsafe.objectFieldOffset(idField), entityId);
                var typeField = Reflect.field(Entity.class, "type");
                unsafe.putObject(
                    snowball, unsafe.objectFieldOffset(typeField),
                    net.minecraft.world.entity.EntityTypes.SNOWBALL);
                var levelField = Reflect.field(Entity.class, "level");
                unsafe.putObject(
                    snowball, unsafe.objectFieldOffset(levelField),
                    PassengerRelationFixtureEntity.CLIENT_LEVEL);
                var positionField = Reflect.field(Entity.class, "position");
                unsafe.putObject(snowball, unsafe.objectFieldOffset(positionField), position);
                var passengersField = Reflect.field(Entity.class, "passengers");
                unsafe.putObject(
                    snowball,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to initialize snowball relation fixture", error);
            }
            return snowball;
        }

        @Override
        public net.minecraft.world.level.Level level() {
            return PassengerRelationFixtureEntity.CLIENT_LEVEL;
        }

        @Override
        public void setPos(double x, double y, double z) {
            this.fixturePosition = new Vec3(x, y, z);
        }

        @Override
        public Vec3 getVehicleAttachmentPoint(Entity vehicle) {
            return Vec3.ZERO;
        }

        @Override
        public void setPose(Pose pose) {
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            this.applyGravity();
            Vec3 movement = this.getDeltaMovement();
            this.setDeltaMovement(movement.scale(this.getAirDrag()));
            this.fixturePosition = this.fixturePosition.add(this.getDeltaMovement());
            this.updateRotation();
            this.firstTick = false;
        }

        void rideTickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            this.setDeltaMovement(Vec3.ZERO);
            this.applyGravity();
            Vec3 movement = this.getDeltaMovement();
            this.setDeltaMovement(movement.scale(this.getAirDrag()));
            this.fixturePosition = this.fixturePosition.add(this.getDeltaMovement());
            this.updateRotation();
            this.firstTick = false;
        }
    }

    static final class TickArrowFixture extends Arrow {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickArrowFixture() {
            super(null, null);
        }

        static TickArrowFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickArrowFixture arrow;
            try {
                Unsafe unsafe = unsafeInstance();
                arrow = (TickArrowFixture)unsafe.allocateInstance(TickArrowFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct arrow tick fixture", error);
            }
            arrow.fixtureEntityId = entityId;
            arrow.fixturePosition = position;
            arrow.previousFixturePosition = position;
            arrow.noGravity = noGravity;
            arrow.setYRot(yRot);
            arrow.setXRot(xRot);
            arrow.yRotO = yRot;
            arrow.xRotO = xRot;
            arrow.setDeltaMovement(movement);
            return arrow;
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            Vec3 movement = this.getDeltaMovement();
            this.fixturePosition = this.fixturePosition.add(movement);
            float yRot = (float)(Mth.atan2(movement.x, movement.z) * 57.2957763671875);
            float xRot = (float)(Mth.atan2(movement.y, movement.horizontalDistance())
                * 57.2957763671875);
            this.setXRot(lerpRotation(this.getXRot(), xRot));
            this.setYRot(lerpRotation(this.getYRot(), yRot));
            this.setDeltaMovement(this.getDeltaMovement().scale(this.getAirDrag()));
            this.applyGravity();
            this.firstTick = false;
        }
    }

    static final class TickExperienceOrbFixture extends ExperienceOrb {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickExperienceOrbFixture() {
            super(null, null);
        }

        static TickExperienceOrbFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickExperienceOrbFixture orb;
            try {
                Unsafe unsafe = unsafeInstance();
                orb = (TickExperienceOrbFixture)unsafe.allocateInstance(
                    TickExperienceOrbFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct experience orb tick fixture", error);
            }
            orb.fixtureEntityId = entityId;
            orb.fixturePosition = position;
            orb.previousFixturePosition = position;
            orb.noGravity = noGravity;
            orb.setYRot(yRot);
            orb.setXRot(xRot);
            orb.yRotO = yRot;
            orb.xRotO = xRot;
            orb.setDeltaMovement(movement);
            orb.firstTick = true;
            return orb;
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        @Override
        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            if (this.firstTick) {
                this.firstTick = false;
                return;
            }
            this.applyGravity();
            this.fixturePosition = this.fixturePosition.add(this.getDeltaMovement());
            this.setDeltaMovement(this.getDeltaMovement().scale(this.getAirDrag()));
        }
    }

    static final class TickPrimedTntFixture extends PrimedTnt {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickPrimedTntFixture() {
            super(null, null);
        }

        static TickPrimedTntFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickPrimedTntFixture tnt;
            try {
                Unsafe unsafe = unsafeInstance();
                tnt = (TickPrimedTntFixture)unsafe.allocateInstance(
                    TickPrimedTntFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct primed TNT tick fixture", error);
            }
            tnt.fixtureEntityId = entityId;
            tnt.fixturePosition = position;
            tnt.previousFixturePosition = position;
            tnt.noGravity = noGravity;
            tnt.setYRot(yRot);
            tnt.setXRot(xRot);
            tnt.yRotO = yRot;
            tnt.xRotO = xRot;
            tnt.setDeltaMovement(movement);
            return tnt;
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        @Override
        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            this.applyGravity();
            this.fixturePosition = this.fixturePosition.add(this.getDeltaMovement());
            this.setDeltaMovement(this.getDeltaMovement().scale(this.getAirDrag()));
        }
    }

    static final class TickFallingBlockFixture extends FallingBlockEntity {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickFallingBlockFixture() {
            super(null, null);
        }

        static TickFallingBlockFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickFallingBlockFixture fallingBlock;
            try {
                Unsafe unsafe = unsafeInstance();
                fallingBlock = (TickFallingBlockFixture)unsafe.allocateInstance(
                    TickFallingBlockFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct falling block tick fixture", error);
            }
            fallingBlock.fixtureEntityId = entityId;
            fallingBlock.fixturePosition = position;
            fallingBlock.previousFixturePosition = position;
            fallingBlock.noGravity = noGravity;
            fallingBlock.setYRot(yRot);
            fallingBlock.setXRot(xRot);
            fallingBlock.yRotO = yRot;
            fallingBlock.xRotO = xRot;
            fallingBlock.setDeltaMovement(movement);
            return fallingBlock;
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        @Override
        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            this.time++;
            this.applyGravity();
            this.fixturePosition = this.fixturePosition.add(this.getDeltaMovement());
            this.setDeltaMovement(this.getDeltaMovement().scale(this.getAirDrag()));
        }
    }

    static final class TickFireworkRocketFixture extends FireworkRocketEntity {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        int fixtureEntityId;
        boolean noGravity;

        TickFireworkRocketFixture() {
            super(null, null);
        }

        static TickFireworkRocketFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity
        ) {
            TickFireworkRocketFixture firework;
            try {
                Unsafe unsafe = unsafeInstance();
                firework = (TickFireworkRocketFixture)unsafe.allocateInstance(
                    TickFireworkRocketFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException(
                    "Unable to construct firework tick fixture", error);
            }
            firework.fixtureEntityId = entityId;
            firework.fixturePosition = position;
            firework.previousFixturePosition = position;
            firework.noGravity = noGravity;
            firework.setYRot(yRot);
            firework.setXRot(xRot);
            firework.yRotO = yRot;
            firework.xRotO = xRot;
            firework.setDeltaMovement(movement);
            return firework;
        }

        @Override
        public boolean isNoGravity() {
            return this.noGravity;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        @Override
        public Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.setOldRot();
            this.tickCount++;
            this.setDeltaMovement(
                this.getDeltaMovement()
                    .multiply(1.15, 1.0, 1.15)
                    .add(0.0, 0.04, 0.0));
            Vec3 movement = this.getDeltaMovement();
            this.fixturePosition = this.fixturePosition.add(movement);
            this.setDeltaMovement(movement);
            this.updateRotation();
        }
    }
}
