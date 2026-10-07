import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.player.RemotePlayer;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathTickFixtures extends MathShapeRotationOracle {
    static final class TickEffect {
        final int effectId;
        final MobEffectInstance value;
        final int flags;

        TickEffect(int effectId, MobEffectInstance value, int flags) {
            this.effectId = effectId;
            this.value = value;
            this.flags = flags;
        }
    }

    static final class TickLocalPlayerFixture extends LocalPlayer {
        List<TickEffect> effects;

        TickLocalPlayerFixture() {
            super(null, null, null, null, null, Input.EMPTY, false, null);
        }

        static TickLocalPlayerFixture create(
            Vec3 position, float yRot, float xRot, float yHeadRot
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                TickLocalPlayerFixture player =
                    (TickLocalPlayerFixture)unsafe.allocateInstance(TickLocalPlayerFixture.class);
                var typeField = Entity.class.getDeclaredField("type");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(typeField), net.minecraft.world.entity.EntityTypes.PLAYER);
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(player, unsafe.objectFieldOffset(positionField), position);
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                player.setYRot(yRot);
                player.setXRot(xRot);
                player.yHeadRot = yHeadRot;
                player.yHeadRotO = yHeadRot;
                player.yBodyRot = yRot;
                player.yBodyRotO = yRot;
                player.effects = new ArrayList<>();
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct local player tick fixture", error);
            }
        }

        void setFixtureState(Vec3 position, float yRot, float xRot) {
            try {
                Unsafe unsafe = unsafeInstance();
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(this, unsafe.objectFieldOffset(positionField), position);
                this.setYRot(yRot);
                this.setXRot(xRot);
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to update local player tick fixture", error);
            }
        }

        void replaceEffect(JsonObject input) {
            int effectId = input.get("effect_id").getAsInt();
            MobEffectInstance value = new MobEffectInstance(
                MobEffects.SPEED,
                input.get("duration_ticks").getAsInt(),
                input.get("amplifier").getAsInt());
            int flags = input.get("flags").getAsInt();
            TickEffect replacement = new TickEffect(effectId, value, flags);
            for (int index = 0; index < this.effects.size(); index++) {
                if (this.effects.get(index).effectId == effectId) {
                    this.effects.set(index, replacement);
                    return;
                }
            }
            this.effects.add(replacement);
        }

        void tickThroughClientLevel() {
            this.setOldPosAndRot();
            this.tickCount++;
            this.tick();
        }

        @Override
        public void tick() {
            for (TickEffect effect : this.effects) {
                effect.value.tickClient();
            }
            this.yHeadRotO = this.yHeadRot;
            if (this.lerpHeadSteps > 0) {
                this.lerpHeadRotationStep(this.lerpHeadSteps, this.lerpYHeadRot);
                this.lerpHeadSteps--;
            }
        }
    }

    static final class TickRemotePlayerFixture extends RemotePlayer {
        TickRemotePlayerFixture() {
            super(null, null);
        }

        static TickRemotePlayerFixture create(
            Vec3 position, float yRot, float xRot, float yHeadRot
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                TickRemotePlayerFixture player =
                    (TickRemotePlayerFixture)unsafe.allocateInstance(TickRemotePlayerFixture.class);
                var typeField = Entity.class.getDeclaredField("type");
                unsafe.putObject(
                    player, unsafe.objectFieldOffset(typeField), net.minecraft.world.entity.EntityTypes.PLAYER);
                var positionField = Entity.class.getDeclaredField("position");
                unsafe.putObject(player, unsafe.objectFieldOffset(positionField), position);
                var passengersField = Entity.class.getDeclaredField("passengers");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(passengersField),
                    com.google.common.collect.ImmutableList.of());
                player.setYRot(yRot);
                player.setXRot(xRot);
                player.yHeadRot = yHeadRot;
                player.yHeadRotO = yHeadRot;
                player.yBodyRot = yRot;
                player.yBodyRotO = yRot;
                player.lerpYHeadRot = yHeadRot;
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct remote player head fixture", error);
            }
        }

        float headTarget() {
            return (float)this.lerpYHeadRot;
        }

        int headSteps() {
            return this.lerpHeadSteps;
        }

        void tickThroughClientLevel() {
            this.setOldPosAndRot();
            this.tickCount++;
            this.yHeadRotO = this.yHeadRot;
            if (this.lerpHeadSteps > 0) {
                this.lerpHeadRotationStep(this.lerpHeadSteps, this.lerpYHeadRot);
                this.lerpHeadSteps--;
            }
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

    static final class TickBoatFixture {
        Vec3 fixturePosition;
        Vec3 previousFixturePosition;
        Vec3 motion;
        int fixtureEntityId;
        int tickCount;
        float yRot;
        float xRot;
        float yRotO;
        float xRotO;
        float deltaRotation;
        boolean noGravity;
        boolean localAuthority;
        boolean inputLeft;
        boolean inputRight;
        boolean inputForward;
        boolean inputBackward;
        boolean paddleLeft;
        boolean paddleRight;

        static TickBoatFixture create(
            int entityId, Vec3 position, Vec3 movement,
            float yRot, float xRot, boolean noGravity,
            boolean localAuthority,
            boolean inputLeft, boolean inputRight,
            boolean inputForward, boolean inputBackward
        ) {
            TickBoatFixture boat = new TickBoatFixture();
            boat.fixtureEntityId = entityId;
            boat.fixturePosition = position;
            boat.previousFixturePosition = position;
            boat.motion = movement;
            boat.yRot = yRot;
            boat.xRot = xRot;
            boat.yRotO = yRot;
            boat.xRotO = xRot;
            boat.noGravity = noGravity;
            boat.localAuthority = localAuthority;
            boat.inputLeft = inputLeft;
            boat.inputRight = inputRight;
            boat.inputForward = inputForward;
            boat.inputBackward = inputBackward;
            return boat;
        }

        int entityId() {
            return this.fixtureEntityId;
        }

        Vec3 position() {
            return this.fixturePosition;
        }

        Vec3 previousPosition() {
            return this.previousFixturePosition;
        }

        void setNoGravity(boolean noGravity) {
            this.noGravity = noGravity;
        }

        void tickThroughClientLevel() {
            this.previousFixturePosition = this.fixturePosition;
            this.yRotO = this.yRot;
            this.xRotO = this.xRot;
            this.tickCount++;

            if (!this.localAuthority) {
                this.motion = Vec3.ZERO;
                return;
            }

            float invFriction = 0.9F;
            double verticalSpeed = this.noGravity ? 0.0 : -0.04;
            this.motion = new Vec3(
                this.motion.x * invFriction,
                this.motion.y + verticalSpeed,
                this.motion.z * invFriction);
            this.deltaRotation *= invFriction;

            float acceleration = 0.0F;
            if (this.inputLeft) {
                this.deltaRotation--;
            }
            if (this.inputRight) {
                this.deltaRotation++;
            }
            if (this.inputRight != this.inputLeft
                && !this.inputForward && !this.inputBackward) {
                acceleration += 0.005F;
            }

            this.yRot += this.deltaRotation;
            if (this.inputForward) {
                acceleration += 0.04F;
            }
            if (this.inputBackward) {
                acceleration -= 0.005F;
            }

            this.motion = this.motion.add(
                Mth.sin(-this.yRot * Mth.DEG_TO_RAD) * acceleration,
                0.0,
                Mth.cos(this.yRot * Mth.DEG_TO_RAD) * acceleration);
            this.paddleLeft = this.inputRight && !this.inputLeft || this.inputForward;
            this.paddleRight = this.inputLeft && !this.inputRight || this.inputForward;
            this.fixturePosition = this.fixturePosition.add(this.motion);
        }
    }

    static final class TickEligibilityFixture {
        int localPlayerId;
        int remoteEntityId;
        int localPlayerTickCount;
        int remoteTickCount;
        long gameTime;
        boolean remoteTicking;

        static TickEligibilityFixture create(int localPlayerId, int remoteEntityId) {
            TickEligibilityFixture fixture = new TickEligibilityFixture();
            fixture.localPlayerId = localPlayerId;
            fixture.remoteEntityId = remoteEntityId;
            fixture.remoteTicking = true;
            return fixture;
        }

        void tick(int ticks) {
            for (int tick = 0; tick < ticks; tick++) {
                this.gameTime++;
                this.localPlayerTickCount++;
                if (this.remoteTicking) {
                    this.remoteTickCount++;
                }
            }
        }

        void setRemoteTicking(boolean ticking) {
            this.remoteTicking = ticking;
        }
    }
}
