import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** What a row reads back from vehicles, projectiles and displays after a tick. */
class MovementObjectExtras extends MovementMobExtras {
    static void extraBoat(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var boat = (net.minecraft.world.entity.vehicle.boat.AbstractBoat) entity;
        described.addProperty("under_water", boat.isUnderWater());
        described.addProperty("hurt_time", boat.getHurtTime());
        described.addProperty("hurt_dir", boat.getHurtDir());
        described.addProperty("damage", Oracle.bits(boat.getDamage()));
        for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
            described.addProperty("bubble_angle_" + partial, Oracle.bits(boat.getBubbleAngle(partial)));
            for (int side = 0; side < 2; side++) {
                described.addProperty("rowing_" + side + "_" + partial,
                    Oracle.bits(boat.getRowingTime(side, partial)));
            }
        }
    }

    static void extraArrow(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var arrow = (net.minecraft.world.entity.projectile.arrow.AbstractArrow) entity;
        described.addProperty("shake_time", arrow.shakeTime);
        var inGround = net.minecraft.world.entity.projectile.arrow.AbstractArrow.class.getDeclaredMethod("isInGround");
        inGround.setAccessible(true);
        described.addProperty("in_ground", (boolean) inGround.invoke(arrow));
        described.addProperty("tipped", arrow instanceof net.minecraft.world.entity.projectile.arrow.Arrow plain
            && plain.getColor() > 0);
    }

    static void extraMinecart(LivingRow ctx, JsonObject described, int tick) throws Exception {
        var entity = ctx.entity();
        var row = ctx.row();
        var cart = (net.minecraft.world.entity.vehicle.minecart.AbstractMinecart) entity;
        described.addProperty("hurt_time", cart.getHurtTime());
        described.addProperty("hurt_dir", cart.getHurtDir());
        described.addProperty("damage", Oracle.bits(cart.getDamage()));
        described.addProperty("display_block", net.minecraft.world.level.block.Block.getId(
            cart.getDisplayBlockState()));
        described.addProperty("display_offset", cart.getDisplayOffset());
        described.addProperty("fuse", cart instanceof net.minecraft.world.entity.vehicle.minecart.MinecartTNT tntCart
            ? tntCart.getFuse() : -1);
        described.addProperty("x", Oracle.bits(cart.getX()));
        described.addProperty("y", Oracle.bits(cart.getY()));
        described.addProperty("z", Oracle.bits(cart.getZ()));
        described.addProperty("y_rot", Oracle.bits(cart.getYRot()));
        described.addProperty("x_rot", Oracle.bits(cart.getXRot()));
        if (cart.getBehavior() instanceof net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior track) {
            // The new behaviour: its motion, and where its playback of the steps puts it between two ticks.
            described.addProperty("motion_x", Oracle.bits(cart.getDeltaMovement().x));
            described.addProperty("motion_y", Oracle.bits(cart.getDeltaMovement().y));
            described.addProperty("motion_z", Oracle.bits(cart.getDeltaMovement().z));
            described.addProperty("playing", track.cartHasPosRotLerp());
            var samples = new JsonArray();
            // Unseen ticks are not asked (a place asked for is kept by the reference for the next ask).
            if (track.cartHasPosRotLerp() && tick >= (row.has("sample_from") ? row.get("sample_from").getAsInt() : 0)) {
                for (float partial : new float[] {0.0F, 0.5F}) {
                    var at = track.getCartLerpPosition(partial);
                    samples.add(Oracle.bits(at.x) + Oracle.bits(at.y) + Oracle.bits(at.z)
                        + Oracle.bits(track.getCartLerpXRot(partial)) + Oracle.bits(track.getCartLerpYRot(partial)));
                }
            }
            described.add("lerp", samples);
        }
        // Where the rail puts it, and a little ahead and behind it, at some places
        // (asked on the last tick only: it is the world that answers).
        var probes = new JsonArray();
        if (tick + 1 == row.get("tick_count").getAsInt() && row.has("rail_probes")
            && cart.getBehavior() instanceof net.minecraft.world.entity.vehicle.minecart.OldMinecartBehavior behavior) {
            for (var probe : row.getAsJsonArray("rail_probes")) {
                var at = probe.getAsJsonArray();
                double px = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(0).getAsString(), 16));
                double py = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(1).getAsString(), 16));
                double pz = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(2).getAsString(), 16));
                var found = new JsonArray();
                for (var place : new net.minecraft.world.phys.Vec3[] {
                        behavior.getPos(px, py, pz), behavior.getPosOffs(px, py, pz, 0.3F),
                        behavior.getPosOffs(px, py, pz, -0.3F)}) {
                    found.add(place == null ? "none" : Oracle.bits(place.x) + Oracle.bits(place.y) + Oracle.bits(place.z));
                }
                probes.add(found);
            }
        }
        described.add("rail", probes);
    }

    static void extraDisplay(LivingRow ctx, JsonObject described) throws Exception {
        var entity = ctx.entity();
        var shown = (net.minecraft.world.entity.Display) entity;
        described.addProperty("x", Oracle.bits(shown.getX()));
        described.addProperty("y", Oracle.bits(shown.getY()));
        described.addProperty("z", Oracle.bits(shown.getZ()));
        described.addProperty("y_rot", Oracle.bits(shown.getYRot()));
        described.addProperty("x_rot", Oracle.bits(shown.getXRot()));
        if (shown instanceof net.minecraft.world.entity.Display.BlockDisplay block
                && block.blockRenderState() != null) {
            described.addProperty("block_state", net.minecraft.world.level.block.Block.getId(
                block.blockRenderState().blockState()));
        }
        if (shown instanceof net.minecraft.world.entity.Display.ItemDisplay itemDisplay
                && itemDisplay.itemRenderState() != null) {
            described.addProperty("item_context", itemDisplay.itemRenderState().itemTransform().getId());
            described.addProperty("item_empty", itemDisplay.itemRenderState().itemStack().isEmpty());
        }
        // What the render state gives between two ticks (asked as the renderer does, each asks remembering its progress).
        var samples = new JsonArray();
        var renderState = shown.renderState();
        described.addProperty("has_state", renderState != null);
        if (renderState != null) {
            for (float partial : new float[] {0.0F, 0.5F}) {
                float progress = shown.calculateInterpolationProgress(partial);
                var matrix = new float[16];
                renderState.transformation().get(progress).getMatrix().get(matrix);
                var sample = new StringBuilder(Oracle.bits(progress));
                for (float value : matrix) {
                    sample.append(Oracle.bits(value));
                }
                sample.append(Oracle.bits(renderState.shadowRadius().get(progress)));
                sample.append(Oracle.bits(renderState.shadowStrength().get(progress)));
                samples.add(sample.toString());
            }
            described.addProperty("billboard", renderState.billboardConstraints().ordinal());
            described.addProperty("brightness", renderState.brightnessOverride());
            described.addProperty("glow", renderState.glowColorOverride());
        }
        described.add("samples", samples);
        // A text display's own render state, as it is read at the same part ticks.
        if (shown instanceof net.minecraft.world.entity.Display.TextDisplay textShown
                && textShown.textRenderState() != null) {
            var text = textShown.textRenderState();
            described.addProperty("text", text.text().getString());
            described.addProperty("line_width", text.lineWidth());
            described.addProperty("style_flags", (int) text.flags());
            var textSamples = new JsonArray();
            for (float partial : new float[] {0.0F, 0.5F}) {
                float progress = shown.calculateInterpolationProgress(partial);
                textSamples.add(text.textOpacity().get(progress) + "," + text.backgroundColor().get(progress));
            }
            described.add("text_samples", textSamples);
        }
        // Whether it is near enough to be drawn, at some squared distances.
        var near = new JsonArray();
        for (double distance : new double[] {0.0, 4095.99, 4096.0, 16383.99, 16384.0, 1.0E9}) {
            near.add(shown.shouldRenderAtSqrDistance(distance));
        }
        described.add("render_at", near);
    }
}
