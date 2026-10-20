import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The command block's editing screen, over a block entity in no level. */
class GuiCommandBlockOracle extends GuiSignOracle {
    /** The packets the screen sent. */
    static final class Sent extends net.minecraft.network.Connection {
        final java.util.List<Object> packets = new java.util.ArrayList<>();

        Sent() {
            super(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
        }

        @Override
        public void send(net.minecraft.network.protocol.Packet<?> packet) {
            packets.add(packet);
        }
    }

    /** The game's GUI, told when a screen is closed. */
    static final class Closing extends net.minecraft.client.gui.Gui {
        boolean closed;

        Closing() {
            super(null, null, null);
        }

        @Override
        public void setScreen(net.minecraft.client.gui.screens.Screen screen) {
            closed = screen == null;
        }
    }

    /**
     * The game's own `CommandBlockEditScreen` for a block of a mode, condition, command and last
     * output, after the block's data arrives, text, keys and clicks, drawn at a time after it
     * opened: its drawing, its screen being closed and the packet it sent.
     */
    static JsonObject commandBlockScreen(JsonObject request) throws Exception {
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
            set(listener, "commands", new com.mojang.brigadier.CommandDispatcher<
                net.minecraft.client.multiplayer.ClientSuggestionProvider>());
            set(listener, "suggestionsProvider", new OracleSource(java.util.List.of(), java.util.List.of(), true));
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            // Closing the screen is the only thing asked of the game's GUI.
            var gui = (Closing) unsafe.allocateInstance(Closing.class);
            set(gui, "hud", screen.hud());
            set(minecraft, "gui", gui);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var block = switch (row.get("mode").getAsString()) {
                case "sequence" -> net.minecraft.world.level.block.Blocks.CHAIN_COMMAND_BLOCK;
                case "auto" -> net.minecraft.world.level.block.Blocks.REPEATING_COMMAND_BLOCK;
                default -> net.minecraft.world.level.block.Blocks.COMMAND_BLOCK;
            };
            var state = block.defaultBlockState().setValue(
                net.minecraft.world.level.block.CommandBlock.CONDITIONAL, row.get("conditional").getAsBoolean());
            // A block in no level: its condition is read from the state it was given.
            boolean conditional = row.get("conditional").getAsBoolean();
            var entity = new net.minecraft.world.level.block.entity.CommandBlockEntity(
                net.minecraft.core.BlockPos.ZERO, state) {
                @Override
                public boolean isConditional() {
                    return conditional;
                }
            };
            var base = entity.getCommandBlock();
            base.setCommand(row.get("command").getAsString());
            base.setTrackOutput(row.get("track_output").getAsBoolean());
            if (row.has("last_output") && !row.get("last_output").isJsonNull()) {
                base.setLastOutput(net.minecraft.network.chat.Component.literal(row.get("last_output").getAsString()));
            }
            entity.setAutomatic(row.get("automatic").getAsBoolean());
            var edit = new net.minecraft.client.gui.screens.inventory.CommandBlockEditScreen(entity);
            edit.init(width, height);
            var steps = new JsonArray();
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                switch (step.get("do").getAsString()) {
                    case "update" -> edit.updateGui();
                    case "resize" -> edit.resize(step.get("width").getAsInt(), step.get("height").getAsInt());
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
                json.addProperty("command", (String) get(packet, "command"));
                json.addProperty("mode", ((Enum<?>) get(packet, "mode")).name().toLowerCase());
                json.addProperty("track_output", (Boolean) get(packet, "trackOutput"));
                json.addProperty("conditional", (Boolean) get(packet, "conditional"));
                json.addProperty("automatic", (Boolean) get(packet, "automatic"));
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
