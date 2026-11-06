import com.google.gson.JsonObject;

/** Data-value events of vehicles, projectiles and displays. */
class MovementObjectEvents extends MovementMobEvents {
    static void eventBoatData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var vehicleClass = net.minecraft.world.entity.vehicle.VehicleEntity.class;
        var boatClass = net.minecraft.world.entity.vehicle.boat.AbstractBoat.class;
        if (event.has("hurt")) {
            data.add(wolfData(vehicleClass, "DATA_ID_HURT", event.get("hurt").getAsInt()));
        }
        if (event.has("hurt_dir")) {
            data.add(wolfData(vehicleClass, "DATA_ID_HURTDIR", event.get("hurt_dir").getAsInt()));
        }
        if (event.has("damage")) {
            data.add(wolfData(vehicleClass, "DATA_ID_DAMAGE", (float) event.get("damage").getAsDouble()));
        }
        if (event.has("bubble_time")) {
            data.add(wolfData(boatClass, "DATA_ID_BUBBLE_TIME", event.get("bubble_time").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventArrowData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        if (event.has("in_ground")) {
            data.add(wolfData(net.minecraft.world.entity.projectile.arrow.AbstractArrow.class, "IN_GROUND",
                event.get("in_ground").getAsBoolean()));
        }
        if (event.has("color")) {
            data.add(wolfData(net.minecraft.world.entity.projectile.arrow.Arrow.class, "ID_EFFECT_COLOR",
                event.get("color").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventMinecartData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        var vehicle = net.minecraft.world.entity.vehicle.VehicleEntity.class;
        var cart = net.minecraft.world.entity.vehicle.minecart.AbstractMinecart.class;
        if (event.has("hurt")) {
            data.add(wolfData(vehicle, "DATA_ID_HURT", event.get("hurt").getAsInt()));
        }
        if (event.has("hurt_dir")) {
            data.add(wolfData(vehicle, "DATA_ID_HURTDIR", event.get("hurt_dir").getAsInt()));
        }
        if (event.has("damage")) {
            data.add(wolfData(vehicle, "DATA_ID_DAMAGE", (float) event.get("damage").getAsDouble()));
        }
        if (event.has("display")) {
            data.add(wolfData(cart, "DATA_ID_CUSTOM_DISPLAY_BLOCK", java.util.Optional.of(
                net.minecraft.world.level.block.Block.stateById(event.get("display").getAsInt()))));
        }
        if (event.has("no_display")) {
            data.add(wolfData(cart, "DATA_ID_CUSTOM_DISPLAY_BLOCK", java.util.Optional.empty()));
        }
        if (event.has("offset")) {
            data.add(wolfData(cart, "DATA_ID_DISPLAY_OFFSET", event.get("offset").getAsInt()));
        }
        if (event.has("fuel")) {
            data.add(wolfData(net.minecraft.world.entity.vehicle.minecart.MinecartFurnace.class,
                "DATA_ID_FUEL", event.get("fuel").getAsBoolean()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }

    static void eventMinecartSteps(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        // The steps along the track the server took, with the turns as the wire packs them.
        var steps = new java.util.ArrayList<net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior.MinecartStep>();
        for (var stepElement : event.getAsJsonArray("steps")) {
            var step = stepElement.getAsJsonObject();
            steps.add(new net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior.MinecartStep(
                new net.minecraft.world.phys.Vec3(Oracle.f64(step, "x"), Oracle.f64(step, "y"), Oracle.f64(step, "z")),
                new net.minecraft.world.phys.Vec3(Oracle.f64(step, "mx"), Oracle.f64(step, "my"), Oracle.f64(step, "mz")),
                net.minecraft.util.Mth.unpackDegrees((byte) step.get("y_rot").getAsInt()),
                net.minecraft.util.Mth.unpackDegrees((byte) step.get("x_rot").getAsInt()),
                Oracle.f32(step, "weight")));
        }
        listener.handleMinecartAlongTrack(
            new net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket(entityId, steps));
    }

    static void eventDisplayData(LivingRow ctx, JsonObject event) throws Exception {
        var listener = ctx.listener();
        var entityId = ctx.entityId();
        // A display's data, each field built from the reference's own accessor.
        var display = net.minecraft.world.entity.Display.class;
        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
        for (var field : new String[][] {{"start_delta", "DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID"},
                {"duration", "DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID"},
                {"pos_rot", "DATA_POS_ROT_INTERPOLATION_DURATION_ID"}, {"brightness", "DATA_BRIGHTNESS_OVERRIDE_ID"},
                {"glow", "DATA_GLOW_COLOR_OVERRIDE_ID"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(display, field[1], event.get(field[0]).getAsInt()));
            }
        }
        for (var field : new String[][] {{"view_range", "DATA_VIEW_RANGE_ID"}, {"shadow_radius", "DATA_SHADOW_RADIUS_ID"},
                {"shadow_strength", "DATA_SHADOW_STRENGTH_ID"}, {"width", "DATA_WIDTH_ID"}, {"height", "DATA_HEIGHT_ID"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(display, field[1], Oracle.f32(event, field[0])));
            }
        }
        if (event.has("billboard")) {
            data.add(wolfData(display, "DATA_BILLBOARD_RENDER_CONSTRAINTS_ID", (byte) event.get("billboard").getAsInt()));
        }
        for (var field : new String[][] {{"translation", "DATA_TRANSLATION_ID"}, {"scale", "DATA_SCALE_ID"}}) {
            if (event.has(field[0])) {
                var parts = event.getAsJsonArray(field[0]);
                data.add(wolfData(display, field[1], new org.joml.Vector3f(
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(0).getAsString(), 16)),
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(1).getAsString(), 16)),
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(2).getAsString(), 16)))));
            }
        }
        for (var field : new String[][] {{"left_rotation", "DATA_LEFT_ROTATION_ID"}, {"right_rotation", "DATA_RIGHT_ROTATION_ID"}}) {
            if (event.has(field[0])) {
                var parts = event.getAsJsonArray(field[0]);
                data.add(wolfData(display, field[1], new org.joml.Quaternionf(
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(0).getAsString(), 16)),
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(1).getAsString(), 16)),
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(2).getAsString(), 16)),
                    Float.intBitsToFloat((int) Long.parseLong(parts.get(3).getAsString(), 16)))));
            }
        }
        if (event.has("block_state")) {
            data.add(wolfData(net.minecraft.world.entity.Display.BlockDisplay.class, "DATA_BLOCK_STATE_ID",
                net.minecraft.world.level.block.Block.stateById(event.get("block_state").getAsInt())));
        }
        if (event.has("item")) {
            var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                net.minecraft.resources.Identifier.parse(event.get("item").getAsString()));
            data.add(wolfData(net.minecraft.world.entity.Display.ItemDisplay.class, "DATA_ITEM_STACK_ID",
                new net.minecraft.world.item.ItemStack(item)));
        }
        var textDisplay = net.minecraft.world.entity.Display.TextDisplay.class;
        if (event.has("text")) {
            data.add(wolfData(textDisplay, "DATA_TEXT_ID",
                net.minecraft.network.chat.Component.literal(event.get("text").getAsString())));
        }
        for (var field : new String[][] {{"line_width", "DATA_LINE_WIDTH_ID"}, {"background", "DATA_BACKGROUND_COLOR_ID"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(textDisplay, field[1], event.get(field[0]).getAsInt()));
            }
        }
        for (var field : new String[][] {{"text_opacity", "DATA_TEXT_OPACITY_ID"}, {"style_flags", "DATA_STYLE_FLAGS_ID"}}) {
            if (event.has(field[0])) {
                data.add(wolfData(textDisplay, field[1], (byte) event.get(field[0]).getAsInt()));
            }
        }
        if (event.has("item_display")) {
            data.add(wolfData(net.minecraft.world.entity.Display.ItemDisplay.class, "DATA_ITEM_DISPLAY_ID",
                (byte) event.get("item_display").getAsInt()));
        }
        listener.handleSetEntityData(
            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
    }
}
