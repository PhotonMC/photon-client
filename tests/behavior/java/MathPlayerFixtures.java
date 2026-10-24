import com.google.gson.JsonObject;
import net.minecraft.client.player.ClientInput;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.component.DataComponentMap;
import net.minecraft.core.component.DataComponents;
import net.minecraft.core.component.PatchedDataComponentMap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.food.FoodData;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.component.UseEffects;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;

class MathPlayerFixtures extends MathPassengerRotationOracle {
    static final class LocalPlayerInputFixture extends LocalPlayerStateFixture {
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
            final boolean controlledCamera
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
        }

        void configureSprintGate(
            final float moveY,
            final boolean sprinting,
            final boolean mobilityRestricted,
            final boolean enoughFood,
            final boolean mayfly,
            final boolean shallowWater,
            final boolean flying,
            final boolean usingItem,
            final boolean fallFlying,
            final boolean underwater,
            final boolean movingSlowly
        ) throws ReflectiveOperationException {
            this.input = new FixtureInput(new Vec2(0.0F, moveY), Input.EMPTY);
            this.sprinting = sprinting;
            this.mobilityRestricted = mobilityRestricted;
            this.shallowWater = shallowWater;
            this.fallFlying = fallFlying;
            this.underwater = underwater;
            this.usingItem = usingItem;
            this.movingSlowly = movingSlowly;
            this.foodData = new FoodData();
            this.foodData.setFoodLevel(enoughFood ? 20 : 6);
            this.abilities = new Abilities();
            this.abilities.mayfly = mayfly;
            this.abilities.flying = flying;
            if (usingItem) {
                this.setItemUseSpeed(1.0F);
            }
        }

        void configureSprintTrigger(
            final Vec2 initialMoveVector,
            final Input initialInput,
            final JsonObject conditions
        ) throws ReflectiveOperationException {
            this.input = new FixtureInput(initialMoveVector, initialInput);
            this.passenger = false;
            this.sprinting = false;
            this.sprintTriggerTime = 0;
            this.mobilityRestricted = conditions.get("mobility_restricted").getAsBoolean();
            this.shallowWater = conditions.get("shallow_water").getAsBoolean();
            this.fallFlying = conditions.get("fall_flying").getAsBoolean();
            this.underwater = conditions.get("underwater").getAsBoolean();
            this.usingItem = conditions.get("slow_due_to_using_item").getAsBoolean();
            this.movingSlowly = conditions.get("moving_slowly").getAsBoolean();
            this.foodData = new FoodData();
            this.foodData.setFoodLevel(
                conditions.get("enough_food").getAsBoolean() ? 20 : 6);
            this.abilities = new Abilities();
            this.abilities.mayfly = conditions.get("mayfly").getAsBoolean();
            this.abilities.flying = conditions.get("flying").getAsBoolean();
            if (this.usingItem) {
                this.setItemUseSpeed(1.0F);
            }
        }

        void setSprintTriggerInput(final Input input, final Vec2 moveVector) {
            ((FixtureInput)this.input).setNext(moveVector, input);
        }

        void tickSprintTrigger(
            final java.lang.reflect.Method canStartSprinting,
            final int sprintWindow
        ) throws ReflectiveOperationException {
            if (this.sprintTriggerTime > 0) {
                this.sprintTriggerTime--;
            }

            boolean wasShiftKeyDown = this.input.keyPresses.shift();
            boolean hasForwardImpulse = this.input.hasForwardImpulse();
            this.input.tick();
            if (wasShiftKeyDown
                || this.usingItem && !this.isPassenger()
                || this.input.keyPresses.backward()) {
                this.sprintTriggerTime = 0;
            }

            if ((boolean)canStartSprinting.invoke(this)) {
                if (!hasForwardImpulse) {
                    if (this.sprintTriggerTime > 0) {
                        this.setSprinting(true);
                    } else {
                        this.sprintTriggerTime = sprintWindow;
                    }
                }

                if (this.input.keyPresses.sprint()) {
                    this.setSprinting(true);
                }
            }
        }

        void configureTravelFlying(
            final boolean inWater,
            final boolean inLava,
            final float yRot,
            final Vec3 deltaMovement
        ) {
            this.inWaterState = inWater;
            this.inLavaState = inLava;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
        }

        void configureTravelInWater(
            final boolean sprinting,
            final float waterSlowDown,
            final float waterEfficiency,
            final boolean onGround,
            final float speed,
            final boolean dolphinsGrace,
            final boolean horizontalCollision,
            final boolean onClimbable,
            final boolean freeAfterJump,
            final float yRot,
            final double currentY,
            final Vec3 deltaMovement
        ) throws ReflectiveOperationException {
            this.sprinting = sprinting;
            this.waterSlowDown = waterSlowDown;
            this.sneakingMovementFactor = waterEfficiency;
            this.onGroundState = onGround;
            this.configuredSpeed = speed;
            this.dolphinsGrace = dolphinsGrace;
            this.horizontalCollision = horizontalCollision;
            this.onClimbableState = onClimbable;
            this.freeAfterFluidJump = freeAfterJump;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
            var position = Reflect.field(net.minecraft.world.entity.Entity.class, "position");
            position.set(this, new Vec3(0.0, currentY, 0.0));
        }

        void configureTravelInLava(
            final boolean sprinting,
            final boolean shallowLava,
            final boolean horizontalCollision,
            final boolean freeAfterJump,
            final float yRot,
            final double currentY,
            final Vec3 deltaMovement
        ) throws ReflectiveOperationException {
            this.sprinting = sprinting;
            this.shallowLava = shallowLava;
            this.horizontalCollision = horizontalCollision;
            this.freeAfterFluidJump = freeAfterJump;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
            var position = Reflect.field(net.minecraft.world.entity.Entity.class, "position");
            position.set(this, new Vec3(0.0, currentY, 0.0));
        }

        void configureHandleRelativeFriction(
            final boolean onGround,
            final float speed,
            final float flyingSpeed,
            final boolean flying,
            final boolean sprinting,
            final boolean passenger,
            final boolean onClimbable,
            final boolean scaffolding,
            final boolean suppressSlidingDownLadder,
            final boolean horizontalCollision,
            final boolean jumping,
            final float yRot,
            final Vec3 deltaMovement
        ) throws ReflectiveOperationException {
            this.onGroundState = onGround;
            this.configuredSpeed = speed;
            this.abilities = new Abilities();
            this.abilities.flying = flying;
            this.abilities.setFlyingSpeed(flyingSpeed);
            var abilitiesField = Reflect.field(net.minecraft.world.entity.player.Player.class, "abilities");
            abilitiesField.set(this, this.abilities);
            this.sprinting = sprinting;
            this.passenger = passenger;
            this.onClimbableState = onClimbable;
            this.scaffolding = scaffolding;
            this.suppressSlidingDownLadder = suppressSlidingDownLadder;
            this.horizontalCollision = horizontalCollision;
            this.jumping = jumping;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
        }

        void configureTravelInAir(
            final boolean onGround,
            final String belowBlock,
            final float frictionModifier,
            final float speed,
            final float flyingSpeed,
            final boolean flying,
            final boolean sprinting,
            final boolean passenger,
            final boolean onClimbable,
            final boolean scaffolding,
            final boolean suppressSlidingDownLadder,
            final boolean horizontalCollision,
            final boolean jumping,
            final int levitationAmplifier,
            final double gravity,
            final boolean discardFriction,
            final float airDragModifier,
            final boolean omnidirectionalAirMover,
            final boolean clientSide,
            final boolean hasChunkAt,
            final int minY,
            final float yRot,
            final double currentY,
            final Vec3 deltaMovement
        ) throws ReflectiveOperationException {
            this.onGroundState = onGround;
            this.configuredSpeed = speed;
            this.abilities = new Abilities();
            this.abilities.flying = flying;
            this.abilities.setFlyingSpeed(flyingSpeed);
            var abilitiesField = Reflect.field(net.minecraft.world.entity.player.Player.class, "abilities");
            abilitiesField.set(this, this.abilities);
            this.sprinting = sprinting;
            this.passenger = passenger;
            this.onClimbableState = onClimbable;
            this.scaffolding = scaffolding;
            this.suppressSlidingDownLadder = suppressSlidingDownLadder;
            this.horizontalCollision = horizontalCollision;
            this.jumping = jumping;
            this.frictionModifier = frictionModifier;
            this.airDragModifier = airDragModifier;
            this.gravity = gravity;
            this.levitationAmplifier = levitationAmplifier;
            this.discardFrictionState = discardFriction;
            this.omnidirectionalAirMoverState = omnidirectionalAirMover;
            this.yRotation = yRot;
            this.travelDeltaMovement = deltaMovement;
            ClimbableClientLevelFixture.setTravelState(clientSide, hasChunkAt, minY);
            ClimbableClientLevelFixture.setBelowState(switch (belowBlock) {
                case "air" -> Blocks.AIR.defaultBlockState();
                case "ice" -> Blocks.ICE.defaultBlockState();
                case "slime" -> Blocks.SLIME_BLOCK.defaultBlockState();
                default -> throw new IllegalArgumentException("Unknown travel air block");
            });
            var position = Reflect.field(net.minecraft.world.entity.Entity.class, "position");
            position.set(this, new Vec3(0.0, currentY, 0.0));
        }

        void configureFallFlyingMovement(
            final Vec3 lookAngle,
            final float xRot,
            final double gravity
        ) {
            this.fallFlyingLookAngle = lookAngle;
            this.xRotation = xRot;
            this.gravity = gravity;
        }

        void configureTravelDispatch(JsonObject input) throws ReflectiveOperationException {
            configureTravelInAir(
                input.get("on_ground").getAsBoolean(),
                input.get("below_block").getAsString(),
                Oracle.f32(input, "friction_modifier"),
                Oracle.f32(input, "speed"),
                Oracle.f32(input, "flying_speed"),
                input.get("flying").getAsBoolean(),
                input.get("sprinting").getAsBoolean(),
                input.get("passenger").getAsBoolean(),
                input.get("on_climbable").getAsBoolean(),
                input.get("scaffolding").getAsBoolean(),
                input.get("suppress_sliding_down_ladder").getAsBoolean(),
                input.get("horizontal_collision").getAsBoolean(),
                input.get("jumping").getAsBoolean(),
                input.get("levitation_amplifier").getAsInt(),
                f64(input, "gravity"),
                input.get("should_discard_friction").getAsBoolean(),
                Oracle.f32(input, "air_drag_modifier"),
                input.get("omnidirectional_air_mover").getAsBoolean(),
                input.get("client_side").getAsBoolean(),
                input.get("has_chunk_at").getAsBoolean(),
                input.get("min_y").getAsInt(),
                Oracle.f32(input, "y_rot"),
                f64(input, "current_y"),
                vec3(input.getAsJsonObject("delta")));
            this.inWaterState = input.get("in_water").getAsBoolean();
            this.inLavaState = input.get("in_lava").getAsBoolean();
            this.affectedByFluidsState = input.get("affected_by_fluids").getAsBoolean();
            this.canStandOnFluidState = input.get("can_stand_on_fluid").getAsBoolean();
            this.sneakingMovementFactor = Oracle.f32(input, "water_efficiency");
            this.waterSlowDown = Oracle.f32(input, "water_slow_down");
            this.shallowLava = input.get("shallow_lava").getAsBoolean();
            this.fallFlying = input.get("fall_flying").getAsBoolean();
            this.fallFlyingLookAngle = vec3(input.getAsJsonObject("look"));
            this.xRotation = Oracle.f32(input, "x_rot");
            this.travelPath = null;
            var blockPosition = Reflect.field(net.minecraft.world.entity.Entity.class, "blockPosition");
            blockPosition.set(this, BlockPos.ZERO);
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

    static final class FixtureInput extends ClientInput {
        Input nextKeyPresses;
        Vec2 nextMoveVector;

        FixtureInput(final Vec2 moveVector, final Input keyPresses) {
            this.moveVector = moveVector;
            this.keyPresses = keyPresses;
            this.nextMoveVector = moveVector;
            this.nextKeyPresses = keyPresses;
        }

        void setNext(final Vec2 moveVector, final Input keyPresses) {
            this.nextMoveVector = moveVector;
            this.nextKeyPresses = keyPresses;
        }

        @Override
        public void tick() {
            this.moveVector = this.nextMoveVector;
            this.keyPresses = this.nextKeyPresses;
        }
    }
}
