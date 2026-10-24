import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The jigsaw block's editing screen, over a block entity in no level. */
class GuiJigsawOracle extends GuiStructureBlockOracle {
    static net.minecraft.world.level.block.entity.JigsawBlockEntity jigsawBlock(JsonObject row) {
        var orientation = Enum.valueOf(net.minecraft.core.FrontAndTop.class,
            row.get("orientation").getAsString().toUpperCase());
        var state = net.minecraft.world.level.block.Blocks.JIGSAW.defaultBlockState()
            .setValue(net.minecraft.world.level.block.JigsawBlock.ORIENTATION, orientation);
        var block = new net.minecraft.world.level.block.entity.JigsawBlockEntity(
            net.minecraft.core.BlockPos.ZERO, state);
        block.setName(net.minecraft.resources.Identifier.parse(row.get("name").getAsString()));
        block.setTarget(net.minecraft.resources.Identifier.parse(row.get("target").getAsString()));
        block.setPool(net.minecraft.resources.ResourceKey.create(
            net.minecraft.core.registries.Registries.TEMPLATE_POOL,
            net.minecraft.resources.Identifier.parse(row.get("pool").getAsString())));
        block.setFinalState(row.get("final_state").getAsString());
        block.setJoint(Enum.valueOf(net.minecraft.world.level.block.entity.JigsawBlockEntity.JointType.class,
            row.get("joint").getAsString().toUpperCase()));
        block.setSelectionPriority(row.get("selection_priority").getAsInt());
        block.setPlacementPriority(row.get("placement_priority").getAsInt());
        return block;
    }

    static int keyCode(String name) {
        return switch (name) {
            case "tab" -> 258;
            case "backspace" -> 259;
            case "delete" -> 261;
            case "enter" -> 257;
            case "escape" -> 256;
            case "space" -> 32;
            case "left" -> 263;
            case "right" -> 262;
            case "up" -> 265;
            case "down" -> 264;
            case "home" -> 268;
            case "end" -> 269;
            case "a" -> 65;
            default -> throw new IllegalArgumentException(name);
        };
    }

    /** One step: what the keyboard and mouse handlers do before the screen gets the event. */
    static void jigsawStep(net.minecraft.client.gui.screens.inventory.JigsawBlockEditScreen edit,
            JsonObject step, net.minecraft.client.Minecraft minecraft) throws Exception {
        double x = step.has("x") ? step.get("x").getAsDouble() : 0.0;
        double y = step.has("y") ? step.get("y").getAsDouble() : 0.0;
        var button = new net.minecraft.client.input.MouseButtonInfo(0, step.has("shift") ? 1 : 0);
        switch (step.get("do").getAsString()) {
            case "type" -> step.get("text").getAsString().codePoints().forEach(cp ->
                edit.charTyped(new net.minecraft.client.input.CharacterEvent(cp)));
            case "key" -> {
                String name = step.get("key").getAsString();
                int modifiers = 0;
                if (name.startsWith("ctrl+")) {
                    modifiers = 2;
                    name = name.substring("ctrl+".length());
                }
                if (name.startsWith("shift+")) {
                    modifiers |= 1;
                    name = name.substring("shift+".length());
                }
                int code = keyCode(name);
                if (code == 258) {
                    minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                } else if (code >= 262 && code <= 265) {
                    minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_ARROW);
                }
                edit.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, modifiers));
            }
            case "click" -> {
                minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                edit.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(x, y, button), false);
            }
            case "drag" -> edit.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(x, y, button), 0.0, 0.0);
            case "release" -> edit.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(x, y, button));
            case "wheel" -> edit.mouseScrolled(x, y, 0.0, step.get("amount").getAsDouble());
            case "resize" -> {
                edit.resize(step.get("width").getAsInt(), step.get("height").getAsInt());
                withoutAlt(edit);
            }
            default -> throw new IllegalArgumentException(step.toString());
        }
    }

    /**
     * The game's own `JigsawBlockEditScreen` for a block, after text, keys, clicks, the wheel and
     * new sizes, drawn twice (a tooltip waits for a second frame) with the second frame recorded:
     * its drawing, whether it closed and the packets it sent, in order.
     */
    static JsonObject jigsawScreen(JsonObject request) throws Exception {
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
            var clockField = Reflect.field(net.minecraft.util.Util.class, "timeSource");
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var edit = new net.minecraft.client.gui.screens.inventory.JigsawBlockEditScreen(jigsawBlock(row));
            edit.init(width, height);
            withoutAlt(edit);
            SilentSounds.ASKED.clear();
            for (var step : row.getAsJsonArray("steps")) {
                jigsawStep(edit, step.getAsJsonObject(), minecraft);
            }
            now[0] = row.get("ms").getAsLong();
            edit.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            edit.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            screen.state().reset();
            now[0] += 50;
            edit.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            edit.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            out.addProperty("closed", gui.closed);
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            var packets = new JsonArray();
            for (var packet : sent.packets) {
                var json = new JsonObject();
                if (packet instanceof net.minecraft.network.protocol.game.ServerboundSetJigsawBlockPacket) {
                    json.addProperty("type", "set");
                    json.addProperty("name", get(packet, "name").toString());
                    json.addProperty("target", get(packet, "target").toString());
                    json.addProperty("pool", get(packet, "pool").toString());
                    json.addProperty("final_state", (String) get(packet, "finalState"));
                    json.addProperty("joint", ((Enum<?>) get(packet, "joint")).name().toLowerCase());
                    json.addProperty("selection_priority", (Integer) get(packet, "selectionPriority"));
                    json.addProperty("placement_priority", (Integer) get(packet, "placementPriority"));
                } else {
                    json.addProperty("type", "generate");
                    json.addProperty("levels", (Integer) get(packet, "levels"));
                    json.addProperty("keep_jigsaws", (Boolean) get(packet, "keepJigsaws"));
                }
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
