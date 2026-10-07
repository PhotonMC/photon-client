import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.effect.MobEffectInstance;
import net.minecraft.world.effect.MobEffect;
import net.minecraft.world.effect.MobEffects;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.Fluid;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.phys.Vec3;

class MathPlayerStateFixtures extends MathEntityFixtures {
    static class LocalPlayerStateFixture extends LocalPlayer {
        static final net.minecraft.server.level.ServerLevel TRAVEL_LEVEL =
            ClimbableClientLevelFixture.createClientLevel();
        boolean usingItem;
        boolean passenger;
        boolean movingSlowly;
        float sneakingMovementFactor;
        boolean controlledCamera;
        float xRotation;
        float yRotation;
        boolean sprinting;
        boolean mobilityRestricted;
        boolean shallowWater;
        boolean fallFlying;
        boolean underwater;
        Abilities abilities;
        boolean onClimbableState;
        boolean scaffolding;
        boolean suppressSlidingDownLadder;
        boolean onGroundState;
        float configuredSpeed;
        boolean inWaterState;
        boolean inLavaState;
        Vec3 travelDeltaMovement;
        float waterSlowDown;
        boolean shallowLava;
        boolean dolphinsGrace;
        boolean freeAfterFluidJump;
        double frictionModifier;
        double airDragModifier;
        double gravity;
        int levitationAmplifier;
        boolean discardFrictionState;
        boolean omnidirectionalAirMoverState;
        Vec3 fallFlyingLookAngle;
        boolean affectedByFluidsState;
        boolean canStandOnFluidState;
        String travelPath;

        LocalPlayerStateFixture() {
            super(null, null, null, null, null, Input.EMPTY, false, null);
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
        public double getAttributeValue(final Holder<Attribute> attribute) {
            if (attribute == Attributes.FRICTION_MODIFIER) {
                return this.frictionModifier;
            }
            if (attribute == Attributes.AIR_DRAG_MODIFIER) {
                return this.airDragModifier;
            }
            if (attribute == Attributes.GRAVITY) {
                return this.gravity;
            }
            return this.sneakingMovementFactor;
        }

        @Override
        public net.minecraft.server.level.ServerLevel level() {
            return TRAVEL_LEVEL;
        }

        @Override
        public BlockPos getBlockPosBelowThatAffectsMyMovement() {
            return BlockPos.ZERO;
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
        public Vec3 getLookAngle() {
            return this.fallFlyingLookAngle;
        }

        @Override
        public float getYRot() {
            return this.yRotation;
        }

        @Override
        public boolean isSprinting() {
            return this.sprinting;
        }

        @Override
        public void setSprinting(final boolean sprinting) {
            this.sprinting = sprinting;
        }

        int sprintTriggerTime() {
            return this.sprintTriggerTime;
        }

        @Override
        public Abilities getAbilities() {
            return this.abilities;
        }

        @Override
        public boolean isMobilityRestricted() {
            return this.mobilityRestricted;
        }

        @Override
        public boolean isInShallowWater() {
            return this.shallowWater;
        }

        @Override
        public boolean isFallFlying() {
            return this.fallFlying;
        }

        @Override
        public boolean isUnderWater() {
            return this.underwater;
        }

        @Override
        public boolean onClimbable() {
            return this.onClimbableState;
        }

        @Override
        public BlockState getInBlockState() {
            return this.scaffolding
                ? Blocks.SCAFFOLDING.defaultBlockState()
                : Blocks.AIR.defaultBlockState();
        }

        @Override
        public boolean isSuppressingSlidingDownLadder() {
            return this.suppressSlidingDownLadder;
        }

        @Override
        public boolean onGround() {
            return this.onGroundState;
        }

        @Override
        public float getSpeed() {
            return this.configuredSpeed;
        }

        @Override
        protected float getWaterSlowDown() {
            return this.waterSlowDown;
        }

        @Override
        public boolean isInWater() {
            return this.inWaterState;
        }

        @Override
        public boolean isInLava() {
            return this.inLavaState;
        }

        @Override
        public boolean isAffectedByFluids() {
            return this.affectedByFluidsState;
        }

        @Override
        public boolean isSwimming() {
            return false;
        }

        @Override
        public boolean canStandOnFluid(final FluidState fluid) {
            return this.canStandOnFluidState;
        }

        @Override
        public boolean isVehicle() {
            return false;
        }

        @Override
        public boolean is(final TagKey<net.minecraft.world.entity.EntityType<?>> tag) {
            return false;
        }

        @Override
        protected boolean isInShallowFluid(final TagKey<Fluid> fluidTag) {
            return this.shallowLava;
        }

        @Override
        public boolean hasEffect(final Holder<MobEffect> effect) {
            return this.dolphinsGrace && effect == MobEffects.DOLPHINS_GRACE;
        }

        @Override
        public MobEffectInstance getEffect(final Holder<MobEffect> effect) {
            return effect == MobEffects.LEVITATION && this.levitationAmplifier >= 0
                ? new MobEffectInstance(MobEffects.LEVITATION, 20, this.levitationAmplifier)
                : null;
        }

        @Override
        public boolean shouldDiscardFriction() {
            return this.discardFrictionState;
        }

        @Override
        protected boolean omnidirectionalAirMover() {
            return this.omnidirectionalAirMoverState;
        }

        @Override
        protected double getEffectiveGravity() {
            return this.gravity;
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
            if (this.travelPath == null) {
                this.travelPath = this.fallFlying && !this.onClimbableState
                    ? "fall_flying"
                    : "air";
            }
        }

        @Override
        protected void travelInFluid(final Vec3 input) {
            this.travelPath = this.isInWater() ? "water" : "lava";
            super.travelInFluid(input);
        }

        @Override
        public void stopFallFlying() {
            this.travelPath = "fall_flying_climbable";
        }

        @Override
        public boolean isFree(final double xa, final double ya, final double za) {
            return this.freeAfterFluidJump;
        }
    }
}
