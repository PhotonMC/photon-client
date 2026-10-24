import net.minecraft.client.player.KeyboardInput;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponentMap;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.component.PatchedDataComponentMap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeMap;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.effect.MobEffect;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.component.UseEffects;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathAttributeFixtures extends MathShapeWorldOracle {
    static final class AttributeSpeedFixture extends LocalPlayer {
        boolean onGroundState;
        boolean sprinting;
        float waterSlowDown;
        float waterEfficiency;
        boolean dolphinsGrace;
        boolean onClimbableState;
        boolean freeAfterFluidJump;
        float yRotation;
        Vec3 travelDeltaMovement;

        AttributeSpeedFixture() {
            super(null, null, null, null, null, Input.EMPTY, false, null);
        }

        @Override
        public boolean onGround() {
            return this.onGroundState;
        }

        @Override
        public boolean isSprinting() {
            return this.sprinting;
        }

        @Override
        protected float getWaterSlowDown() {
            return this.waterSlowDown;
        }

        @Override
        public double getAttributeValue(final Holder<Attribute> attribute) {
            if (attribute == Attributes.WATER_MOVEMENT_EFFICIENCY) {
                return this.waterEfficiency;
            }
            return super.getAttributeValue(attribute);
        }

        @Override
        public boolean hasEffect(final Holder<MobEffect> effect) {
            return this.dolphinsGrace && effect == MobEffects.DOLPHINS_GRACE;
        }

        @Override
        public boolean onClimbable() {
            return this.onClimbableState;
        }

        @Override
        public float getYRot() {
            return this.yRotation;
        }

        @Override
        public Vec3 getDeltaMovement() {
            return this.travelDeltaMovement;
        }

        @Override
        public void setDeltaMovement(final Vec3 deltaMovement) {
            if (deltaMovement.isFinite()) {
                this.travelDeltaMovement = deltaMovement;
            }
        }

        @Override
        public void move(final MoverType moverType, final Vec3 delta) {
        }

        @Override
        public boolean isFree(final double xa, final double ya, final double za) {
            return this.freeAfterFluidJump;
        }

        void configureTravelInWater(
            final boolean sprinting,
            final float waterSlowDown,
            final float waterEfficiency,
            final boolean onGround,
            final boolean dolphinsGrace,
            final boolean horizontalCollision,
            final boolean onClimbable,
            final boolean freeAfterJump,
            final float yRot,
            final Vec3 deltaMovement
        ) {
            this.sprinting = sprinting;
            this.waterSlowDown = waterSlowDown;
            this.waterEfficiency = waterEfficiency;
            this.onGroundState = onGround;
            this.dolphinsGrace = dolphinsGrace;
            this.horizontalCollision = horizontalCollision;
            this.onClimbableState = onClimbable;
            this.freeAfterFluidJump = freeAfterJump;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
        }

        static AttributeSpeedFixture create(AttributeMap attributes) {
            try {
                Unsafe unsafe = unsafeInstance();
                AttributeSpeedFixture player =
                    (AttributeSpeedFixture)unsafe.allocateInstance(AttributeSpeedFixture.class);
                var attributesField = LivingEntity.class.getDeclaredField("attributes");
                unsafe.putObject(player, unsafe.objectFieldOffset(attributesField), attributes);
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct player attribute speed fixture", error);
            }
        }
    }

    static final class SprintAttributeFixture extends LocalPlayer {
        SprintAttributeFixture() {
            super(null, null, null, null, null, Input.EMPTY, false, null);
        }

        static SprintAttributeFixture create(AttributeMap attributes) {
            try {
                Unsafe unsafe = unsafeInstance();
                SprintAttributeFixture player =
                    (SprintAttributeFixture)unsafe.allocateInstance(SprintAttributeFixture.class);
                var attributesField = LivingEntity.class.getDeclaredField("attributes");
                unsafe.putObject(player, unsafe.objectFieldOffset(attributesField), attributes);
                SharedFlagsFixtureEntity entityDataFixture = new SharedFlagsFixtureEntity();
                var entityDataField = Entity.class.getDeclaredField("entityData");
                unsafe.putObject(
                    player,
                    unsafe.objectFieldOffset(entityDataField),
                    entityDataFixture.getEntityData());
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct player sprint attribute fixture", error);
            }
        }
    }

    static final class SneakingSpeedInputFixture extends LocalPlayer {
        boolean usingItem;
        boolean passenger;
        boolean movingSlowly;
        boolean controlledCamera;
        float xRotation;
        float yRotation;

        SneakingSpeedInputFixture() {
            super(null, null, null, null, null, Input.EMPTY, false, null);
        }

        static SneakingSpeedInputFixture create(AttributeMap attributes) {
            try {
                Unsafe unsafe = unsafeInstance();
                SneakingSpeedInputFixture player =
                    (SneakingSpeedInputFixture)unsafe.allocateInstance(SneakingSpeedInputFixture.class);
                var attributesField = LivingEntity.class.getDeclaredField("attributes");
                unsafe.putObject(player, unsafe.objectFieldOffset(attributesField), attributes);
                return player;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(
                    "Unable to construct player sneaking speed fixture", error);
            }
        }

        @Override
        public boolean isUsingItem() {
            return this.usingItem;
        }

        @Override
        public boolean isPassenger() {
            return this.passenger;
        }

        @Override
        public boolean isMovingSlowly() {
            return this.movingSlowly;
        }

        @Override
        protected boolean isControlledCamera() {
            return this.controlledCamera;
        }

        @Override
        public float getXRot() {
            return this.xRotation;
        }

        @Override
        public float getYRot() {
            return this.yRotation;
        }

        void configureApplyInput(
            final float moveX,
            final float moveY,
            final boolean jump,
            final float xxa,
            final float zza,
            final boolean jumping,
            final float xBob,
            final float yBob,
            final float xBobO,
            final float yBobO,
            final float xRot,
            final float yRot,
            final boolean controlledCamera,
            final boolean usingItem,
            final boolean passenger,
            final boolean movingSlowly
        ) {
            this.input = new FixtureInput(new Vec2(moveX, moveY), new Input(
                false, false, false, false, jump, false, false));
            this.xxa = xxa;
            this.zza = zza;
            this.jumping = jumping;
            this.xBob = xBob;
            this.yBob = yBob;
            this.xBobO = xBobO;
            this.yBobO = yBobO;
            this.xRotation = xRot;
            this.yRotation = yRot;
            this.controlledCamera = controlledCamera;
            this.usingItem = usingItem;
            this.passenger = passenger;
            this.movingSlowly = movingSlowly;
        }

        void configureKeyboardInput(
            final KeyboardInput keyboard,
            final float xxa,
            final float zza,
            final boolean jumping,
            final float xBob,
            final float yBob,
            final float xBobO,
            final float yBobO,
            final float xRot,
            final float yRot,
            final boolean controlledCamera,
            final boolean usingItem,
            final boolean passenger,
            final boolean movingSlowly
        ) {
            this.input = keyboard;
            this.xxa = xxa;
            this.zza = zza;
            this.jumping = jumping;
            this.xBob = xBob;
            this.yBob = yBob;
            this.xBobO = xBobO;
            this.yBobO = yBobO;
            this.xRotation = xRot;
            this.yRotation = yRot;
            this.controlledCamera = controlledCamera;
            this.usingItem = usingItem;
            this.passenger = passenger;
            this.movingSlowly = movingSlowly;
        }

        void setItemUseSpeed(final float speed) throws ReflectiveOperationException {
            var constructor = Reflect.constructor(ItemStack.class, 
                Holder.class, int.class, PatchedDataComponentMap.class);
            this.useItem = constructor.newInstance(
                Items.APPLE.builtInRegistryHolder(),
                1,
                new PatchedDataComponentMap(DataComponentMap.EMPTY));
            this.useItem.set(
                DataComponents.USE_EFFECTS,
                new UseEffects(false, true, speed));
        }
    }
}
