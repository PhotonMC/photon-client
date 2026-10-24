import com.google.gson.JsonObject;

/** What a row reads back from animals after a tick. */
class MovementAnimalExtras extends MovementObjectEvents {
    static void extraSwing(LivingRow ctx, JsonObject described) throws Exception {
        var living = ctx.living();
        described.addProperty("swinging", living.swinging);
        described.addProperty("swing_time", living.swingTime);
        described.addProperty("off_hand",
            living.swingingArm == net.minecraft.world.InteractionHand.OFF_HAND);
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("attack_anim_" + partial, Oracle.bits(living.getAttackAnim(partial)));
        }
    }

    static void extraSheep(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var sheep = (net.minecraft.world.entity.animal.sheep.Sheep) entity;
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("eat_position_" + partial,
                Oracle.bits(sheep.getHeadEatPositionScale(partial)));
            described.addProperty("eat_angle_" + partial,
                Oracle.bits(sheep.getHeadEatAngleScale(partial)));
        }
    }

    static void extraChicken(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var chicken = (net.minecraft.world.entity.animal.chicken.Chicken) entity;
        described.addProperty("flap", Oracle.bits(chicken.flap));
        described.addProperty("flap_speed", Oracle.bits(chicken.flapSpeed));
        described.addProperty("old_flap", Oracle.bits(chicken.oFlap));
        described.addProperty("old_flap_speed", Oracle.bits(chicken.oFlapSpeed));
    }

    static void extraCamel(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var camel = (net.minecraft.world.entity.animal.camel.Camel) entity;
        var states = new net.minecraft.world.entity.AnimationState[] {
            camel.sitAnimationState, camel.sitPoseAnimationState, camel.sitUpAnimationState,
            camel.idleAnimationState, camel.dashAnimationState};
        String[] names = {"sit", "sit_pose", "sit_up", "idle", "dash"};
        for (int i = 0; i < names.length; i++) {
            described.addProperty(names[i] + "_started", states[i].isStarted());
            described.addProperty(names[i] + "_millis",
                states[i].isStarted() ? states[i].getTimeInMillis(camel.tickCount) : 0L);
        }
        described.addProperty("dashing", camel.isDashing());
    }

    static void extraHorse(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var horse = (net.minecraft.world.entity.animal.equine.AbstractHorse) entity;
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("eat_" + partial, Oracle.bits(horse.getEatAnim(partial)));
            described.addProperty("stand_" + partial, Oracle.bits(horse.getStandAnim(partial)));
            described.addProperty("mouth_" + partial, Oracle.bits(horse.getMouthAnim(partial)));
        }
        described.addProperty("tail_counter", horse.tailCounter);
    }

    static void extraBee(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var bee = (net.minecraft.world.entity.animal.bee.Bee) entity;
        described.addProperty("roll_0.0", Oracle.bits(bee.getRollAmount(0.0F)));
        described.addProperty("roll_0.5", Oracle.bits(bee.getRollAmount(0.5F)));
        described.addProperty("stung", bee.hasStung());
        described.addProperty("nectar", bee.hasNectar());
        described.addProperty("angry", bee.isAngry());
    }

    static void extraWolf(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var wolf = (net.minecraft.world.entity.animal.wolf.Wolf) entity;
        described.addProperty("angry", wolf.isAngry());
        described.addProperty("sitting", wolf.isInSittingPose());
        described.addProperty("tame", wolf.isTame());
        described.addProperty("tail_angle", Oracle.bits(wolf.getTailAngle()));
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("head_roll_" + partial, Oracle.bits(wolf.getHeadRollAngle(partial)));
            described.addProperty("shake_" + partial, Oracle.bits(wolf.getShakeAnim(partial)));
        }
    }

    static void extraPanda(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var panda = (net.minecraft.world.entity.animal.panda.Panda) entity;
        described.addProperty("variant", panda.getVariant().getId());
        described.addProperty("unhappy_counter", panda.getUnhappyCounter());
        described.addProperty("sneeze_counter", panda.getSneezeCounter());
        described.addProperty("sneezing", panda.isSneezing());
        described.addProperty("eating", panda.isEating());
        described.addProperty("sitting", panda.isSitting());
        described.addProperty("rolling", panda.isRolling());
        described.addProperty("on_back", panda.isOnBack());
        described.addProperty("roll_counter", panda.rollCounter);
        described.addProperty("scared", panda.isScared());
        described.addProperty("x_rot", Oracle.bits(panda.getXRot()));
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("sit_" + partial, Oracle.bits(panda.getSitAmount(partial)));
            described.addProperty("back_" + partial, Oracle.bits(panda.getLieOnBackAmount(partial)));
            described.addProperty("roll_" + partial, Oracle.bits(panda.getRollAmount(partial)));
        }
    }

    static void extraFox(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var fox = (net.minecraft.world.entity.animal.fox.Fox) entity;
        described.addProperty("variant", fox.getVariant().getId());
        described.addProperty("sitting", fox.isSitting());
        described.addProperty("crouching", fox.isCrouching());
        described.addProperty("sleeping", fox.isSleeping());
        described.addProperty("faceplanted", fox.isFaceplanted());
        described.addProperty("pouncing", fox.isPouncing());
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("head_roll_" + partial, Oracle.bits(fox.getHeadRollAngle(partial)));
            described.addProperty("crouch_" + partial, Oracle.bits(fox.getCrouchAmount(partial)));
        }
    }

    static void extraStrider(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("suffocating",
            ((net.minecraft.world.entity.monster.Strider) entity).isSuffocating());
    }

    static void extraRavager(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var ravager = (net.minecraft.world.entity.monster.Ravager) entity;
        described.addProperty("attack_tick", ravager.getAttackTick());
        described.addProperty("stunned_tick", ravager.getStunnedTick());
        described.addProperty("roar_tick", ravager.getRoarTick());
    }

    static void extraHoglin(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("attack_ticks",
            ((net.minecraft.world.entity.monster.hoglin.HoglinBase) entity).getAttackAnimationRemainingTicks());
    }

    static void extraPolarBear(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var bear = (net.minecraft.world.entity.animal.polarbear.PolarBear) entity;
        described.addProperty("standing", bear.isStanding());
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("stand_" + partial, Oracle.bits(bear.getStandingAnimationScale(partial)));
        }
    }

    static void extraCat(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var cat = (net.minecraft.world.entity.animal.feline.Cat) entity;
        described.addProperty("sitting", cat.isInSittingPose());
        described.addProperty("tame", cat.isTame());
        described.addProperty("crouching", cat.isCrouching());
        described.addProperty("on_player", cat.isLyingOnTopOfSleepingPlayer());
        for (float partial : new float[] {0.0F, 0.5F}) {
            described.addProperty("lie_" + partial, Oracle.bits(cat.getLieDownAmount(partial)));
            described.addProperty("lie_tail_" + partial, Oracle.bits(cat.getLieDownAmountTail(partial)));
            described.addProperty("relax_" + partial, Oracle.bits(cat.getRelaxStateOneAmount(partial)));
        }
    }

    static void extraRabbit(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var rabbit = (net.minecraft.world.entity.animal.rabbit.Rabbit) entity;
        described.addProperty("hop_started", rabbit.hopAnimationState.isStarted());
        described.addProperty("idle_started", rabbit.idleHeadTiltAnimationState.isStarted());
        described.addProperty("hop_millis", rabbit.hopAnimationState.getTimeInMillis(rabbit.tickCount));
        described.addProperty("idle_millis",
            rabbit.idleHeadTiltAnimationState.getTimeInMillis(rabbit.tickCount));
        described.addProperty("jump_completion", Oracle.bits(rabbit.getJumpCompletion(0.5F)));
    }

    static void extraTurtle(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var turtle = (net.minecraft.world.entity.animal.turtle.Turtle) entity;
        described.addProperty("in_water", turtle.isInWater());
        described.addProperty("on_ground", turtle.onGround());
        described.addProperty("has_egg", turtle.hasEgg());
        described.addProperty("laying_egg", turtle.isLayingEgg());
        described.addProperty("is_on_land", !turtle.isInWater() && turtle.onGround());
        var box = turtle.getBoundingBox();
        described.addProperty("max_y", Oracle.bits(box.maxY));
        described.addProperty("max_x", Oracle.bits(box.maxX));
    }

    static void extraAxolotl(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var axolotl = (net.minecraft.world.entity.animal.axolotl.Axolotl) entity;
        described.addProperty("in_water", axolotl.isInWater());
        described.addProperty("variant", axolotl.getVariant().getId());
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("playing_dead_" + partial, Oracle.bits(axolotl.playingDeadAnimator.getFactor(partial)));
            described.addProperty("in_water_" + partial, Oracle.bits(axolotl.inWaterAnimator.getFactor(partial)));
            described.addProperty("on_ground_" + partial, Oracle.bits(axolotl.onGroundAnimator.getFactor(partial)));
            described.addProperty("moving_" + partial, Oracle.bits(axolotl.movingAnimator.getFactor(partial)));
        }
        var states = new Object[][] {
            {"swim", axolotl.swimAnimationState}, {"walk", axolotl.walkAnimationState},
            {"walk_under_water", axolotl.walkUnderWaterAnimationState},
            {"idle_under_water", axolotl.idleUnderWaterAnimationState},
            {"idle_under_water_on_ground", axolotl.idleUnderWaterOnGroundAnimationState},
            {"idle_on_ground", axolotl.idleOnGroundAnimationState},
            {"play_dead", axolotl.playDeadAnimationState},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) axolotl.tickCount) : 0L);
        }
    }

    static void extraBat(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var bat = (net.minecraft.world.entity.ambient.Bat) entity;
        described.addProperty("resting", bat.isResting());
        described.addProperty("y", Oracle.bits(bat.getY()));
        described.addProperty("motion_y", Oracle.bits(bat.getDeltaMovement().y));
        var states = new Object[][] {{"fly", bat.flyAnimationState}, {"rest", bat.restAnimationState}};
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) bat.tickCount) : 0L);
        }
    }

    static void extraBreeze(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var breeze = (net.minecraft.world.entity.monster.breeze.Breeze) entity;
        var states = new Object[][] {
            {"idle", breeze.idle}, {"shoot", breeze.shoot}, {"slide", breeze.slide},
            {"slide_back", breeze.slideBack}, {"inhale", breeze.inhale},
            {"long_jump", breeze.longJump},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) breeze.tickCount) : 0L);
        }
    }

    static void extraFrog(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("in_water", entity.isInWater());
        if (entity instanceof net.minecraft.world.entity.animal.frog.Frog frog) {
            var states = new Object[][] {
                {"jump", frog.jumpAnimationState}, {"croak", frog.croakAnimationState},
                {"tongue", frog.tongueAnimationState},
                {"swim_idle", frog.swimIdleAnimationState},
            };
            for (var described2 : states) {
                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                described.addProperty(described2[0] + "_started", animation.isStarted());
                described.addProperty(described2[0] + "_millis",
                    animation.isStarted() ? animation.getTimeInMillis((float) frog.tickCount) : 0L);
            }
        }
    }

    static void extraDolphin(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("max_x", Oracle.bits(entity.getBoundingBox().maxX));
        var motion = entity.getDeltaMovement();
        described.addProperty("is_moving", motion.horizontalDistanceSqr() > 1.0E-7);
        described.addProperty("motion_x", Oracle.bits(motion.x));
        described.addProperty("motion_z", Oracle.bits(motion.z));
    }

    static void extraFish(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("in_water", entity.isInWater());
        var box = entity.getBoundingBox();
        described.addProperty("min_x", Oracle.bits(box.minX));
        described.addProperty("min_y", Oracle.bits(box.minY));
        described.addProperty("min_z", Oracle.bits(box.minZ));
        described.addProperty("max_x", Oracle.bits(box.maxX));
        described.addProperty("max_y", Oracle.bits(box.maxY));
        described.addProperty("max_z", Oracle.bits(box.maxZ));
    }

    static void extraSquid(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var squid = (net.minecraft.world.entity.animal.squid.Squid) entity;
        described.addProperty("in_water", squid.isInWater());
        described.addProperty("y_rot", Oracle.bits(squid.getYRot()));
        described.addProperty("y_body_rot", Oracle.bits(squid.yBodyRot));
        described.addProperty("tentacle_angle", Oracle.bits(squid.tentacleAngle));
        described.addProperty("x_body_rot", Oracle.bits(squid.xBodyRot));
        described.addProperty("z_body_rot", Oracle.bits(squid.zBodyRot));
        described.addProperty("tentacle_angle_half",
            Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.oldTentacleAngle, squid.tentacleAngle)));
        described.addProperty("x_body_rot_half",
            Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.xBodyRotO, squid.xBodyRot)));
        described.addProperty("z_body_rot_half",
            Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.zBodyRotO, squid.zBodyRot)));
        if (squid instanceof net.minecraft.world.entity.animal.squid.GlowSquid glow) {
            described.addProperty("dark_ticks", glow.getDarkTicksRemaining());
        }
        var motion = squid.getDeltaMovement();
        described.addProperty("motion_x", Oracle.bits(motion.x));
        described.addProperty("motion_y", Oracle.bits(motion.y));
        described.addProperty("motion_z", Oracle.bits(motion.z));
    }

    static void extraGoat(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var goat = (net.minecraft.world.entity.animal.goat.Goat) entity;
        described.addProperty("ramming", Oracle.bits(goat.getRammingXHeadRot()));
    }

    static void extraParrot(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var parrot = (net.minecraft.world.entity.animal.parrot.Parrot) entity;
        described.addProperty("variant", parrot.getVariant().getId());
        described.addProperty("motion_y", Oracle.bits(parrot.getDeltaMovement().y));
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            float flap = net.minecraft.util.Mth.lerp(partial, parrot.oFlap, parrot.flap);
            float speed = net.minecraft.util.Mth.lerp(partial, parrot.oFlapSpeed, parrot.flapSpeed);
            described.addProperty("flap_angle_" + partial,
                Oracle.bits((net.minecraft.util.Mth.sin(flap) + 1.0F) * speed));
        }
    }

    static void extraVex(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        described.addProperty("charging",
            ((net.minecraft.world.entity.monster.Vex) entity).isCharging());
    }

    static void extraAllay(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var allay = (net.minecraft.world.entity.animal.allay.Allay) entity;
        described.addProperty("dancing", allay.isDancing());
        described.addProperty("spinning", allay.isSpinning());
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("holding_" + partial, Oracle.bits(allay.getHoldingItemAnimationProgress(partial)));
            described.addProperty("spin_" + partial, Oracle.bits(allay.getSpinningProgress(partial)));
        }
    }

    static void extraSniffer(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var sniffer = (net.minecraft.world.entity.animal.sniffer.Sniffer) entity;
        var states = new Object[][] {
            {"digging", sniffer.diggingAnimationState}, {"sniffing", sniffer.sniffingAnimationState},
            {"rising", sniffer.risingAnimationState}, {"happy", sniffer.feelingHappyAnimationState},
            {"scenting", sniffer.scentingAnimationState},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) sniffer.tickCount) : 0L);
        }
        described.addProperty("searching", sniffer.isSearching());
    }

    static void extraArmadillo(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var armadillo = (net.minecraft.world.entity.animal.armadillo.Armadillo) entity;
        var states = new Object[][] {
            {"roll_out", armadillo.rollOutAnimationState}, {"roll_up", armadillo.rollUpAnimationState},
            {"peek", armadillo.peekAnimationState},
        };
        for (var described2 : states) {
            var animation = (net.minecraft.world.entity.AnimationState) described2[1];
            described.addProperty(described2[0] + "_started", animation.isStarted());
            described.addProperty(described2[0] + "_millis",
                animation.isStarted() ? animation.getTimeInMillis((float) armadillo.tickCount) : 0L);
        }
        described.addProperty("hiding", armadillo.shouldHideInShell());
        described.addProperty("scared", armadillo.isScared());
        described.addProperty("head_yaw", Oracle.bits(armadillo.getYHeadRot()));
    }
}
