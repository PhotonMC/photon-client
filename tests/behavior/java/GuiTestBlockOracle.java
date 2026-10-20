import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The test block's editing screen, over a block entity in no level. */
class GuiTestBlockOracle extends GuiJigsawOracle {
    /** The block a row describes, as the game holds it when its screen opens: the mode is the state's. */
    static net.minecraft.world.level.block.entity.TestBlockEntity testBlock(JsonObject row) {
        var mode = Enum.valueOf(net.minecraft.world.level.block.state.properties.TestBlockMode.class,
            row.get("mode").getAsString().toUpperCase());
        var state = net.minecraft.world.level.block.Blocks.TEST_BLOCK.defaultBlockState()
            .setValue(net.minecraft.world.level.block.TestBlock.MODE, mode);
        var block = new net.minecraft.world.level.block.entity.TestBlockEntity(
            net.minecraft.core.BlockPos.ZERO, state);
        block.setMessage(row.get("message").getAsString());
        return block;
    }

    /**
     * The game's own `TestBlockEditScreen` for a block, after text, keys, clicks, the wheel and new
     * sizes, drawn once: its drawing, whether it closed and the packets it sent.
     */
    static JsonObject testBlockScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            set(minecraft, "fontFilterFishy", get(minecraft, "font"));
            var sent = (Sent) unsafe.allocateInstance(Sent.class);
            set(sent, "packets", new java.util.ArrayList<Object>());
            var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "connection", sent);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            var gui = (Closing) unsafe.allocateInstance(Closing.class);
            set(gui, "hud", screen.hud());
            set(minecraft, "gui", gui);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var edit = new net.minecraft.client.gui.screens.inventory.TestBlockEditScreen(testBlock(row));
            edit.init(width, height);
            withoutAlt(edit);
            var steps = new JsonArray();
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                switch (step.get("do").getAsString()) {
                    case "resize" -> {
                        edit.resize(step.get("width").getAsInt(), step.get("height").getAsInt());
                        withoutAlt(edit);
                    }
                    case "wheel" -> edit.mouseScrolled(step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        0.0, step.get("amount").getAsDouble());
                    default -> {
                        var one = new JsonObject();
                        steps.add(step);
                        one.add("steps", steps);
                        steps(edit, one, minecraft);
                        steps = new JsonArray();
                    }
                }
            }
            now[0] = row.get("ms").getAsLong();
            edit.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            edit.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            out.addProperty("closed", gui.closed);
            var packets = new JsonArray();
            for (var packet : sent.packets) {
                var json = new JsonObject();
                json.addProperty("mode", ((Enum<?>) get(packet, "mode")).name().toLowerCase());
                json.addProperty("message", (String) get(packet, "message"));
                packets.add(json);
            }
            out.add("packets", packets);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
