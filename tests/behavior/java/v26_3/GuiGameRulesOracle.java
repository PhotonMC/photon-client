import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The in-world game rules screen: what it draws, and what it tells the connection and the game. */
class GuiGameRulesOracle extends GuiTutorialOracle {
    /** A connection that writes down what is sent to it, in order. */
    static final class RecordingConnection extends net.minecraft.network.Connection {
        java.util.List<String> events;

        RecordingConnection() {
            super(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
        }

        @Override
        public void send(net.minecraft.network.protocol.Packet<?> packet,
                io.netty.channel.ChannelFutureListener listener, boolean flush) {
            if (packet instanceof net.minecraft.network.protocol.game.ServerboundClientCommandPacket command) {
                events.add("send:client_command:" + command.getAction());
            } else if (packet instanceof net.minecraft.network.protocol.game.ServerboundSetGameRulePacket rules) {
                var text = new StringBuilder("send:set_game_rule");
                for (var entry : rules.entries()) {
                    text.append(':').append(entry.gameRuleKey().identifier()).append('=').append(entry.value());
                }
                events.add(text.toString());
            } else {
                events.add("send:" + packet.getClass().getSimpleName());
            }
        }
    }

    static JsonObject gameRulesScreen(JsonObject request) throws Exception {
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
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            SilentSounds.ASKED.clear();
            set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            // The loading dots follow the clock, so it stands still.
            var clockField = Reflect.field(net.minecraft.util.Util.class, "timeSource");
            var clockBefore = clockField.get(null);
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> 0L);
            var events = new java.util.ArrayList<String>();
            var recording = (RecordingGui) unsafe.allocateInstance(RecordingGui.class);
            set(recording, "hud", get(get(minecraft, "gui"), "hud"));
            set(recording, "events", events);
            set(minecraft, "gui", recording);
            var connection = (RecordingConnection) unsafe.allocateInstance(RecordingConnection.class);
            connection.events = events;
            var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "connection", connection);
            set(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.VANILLA_SET);
            var previous = new GuiDialogOracle.PreviousScreen();
            var options = new net.minecraft.client.gui.screens.options.InWorldGameRulesScreen(
                (net.minecraft.client.multiplayer.ClientPacketListener) listener,
                result -> events.add(result.isPresent() ? "exit:apply" : "exit:discard"), previous);
            set(options, "minecraft", minecraft);
            set(options, "font", minecraft.font);
            minecraft.options.menuBackgroundBlurriness().set(row.has("blurriness") ? row.get("blurriness").getAsInt() : 5);
            options.init(width, height);
            // The entries put their controls in place as they draw, so a frame comes first.
            var steps = row.getAsJsonArray("steps");
            if (row.has("values")) {
                values(options, row.getAsJsonArray("values"));
            }
            frame(options, screen, mouseX, mouseY);
            screen.state().reset();
            for (var stepElement : steps) {
                var step = stepElement.getAsJsonObject();
                double x = step.has("x") ? step.get("x").getAsDouble() : 0.0;
                double y = step.has("y") ? step.get("y").getAsDouble() : 0.0;
                var press = new net.minecraft.client.input.MouseButtonEvent(x, y,
                    SdlInput.button(0, 0));
                switch (step.get("do").getAsString()) {
                    case "click", "release" -> minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                    case "key" -> {
                        if (step.get("key").getAsInt() == 258) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        }
                    }
                    default -> { }
                }
                switch (step.get("do").getAsString()) {
                    case "click" -> options.mouseClicked(press, false);
                    case "release" -> options.mouseReleased(press);
                    case "drag" -> options.mouseDragged(press, 0.0, step.get("dy").getAsDouble());
                    case "scroll" -> options.mouseScrolled(x, y, 0.0, step.get("amount").getAsDouble());
                    case "type" -> {
                        for (var cp : (Iterable<Integer>) step.get("text").getAsString().codePoints()::iterator) {
                            options.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
                        }
                    }
                    case "resize" -> options.resize(step.get("width").getAsInt(), step.get("height").getAsInt());
                    case "key" -> options.keyPressed(SdlInput.key(
                        step.get("key").getAsInt(), 0, (step.has("shift") ? 1 : 0) | (step.has("ctrl") ? 2 : 0)));
                    case "values" -> values(options, step.getAsJsonArray("values"));
                    case "permission" -> options.onGamemasterPermissionChanged(step.get("has").getAsBoolean());
                    default -> throw new IllegalArgumentException(step.toString());
                }
                frame(options, screen, mouseX, mouseY);
                screen.state().reset();
            }
            frame(options, screen, mouseX, mouseY);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var blurField = Reflect.field(net.minecraft.client.renderer.state.gui.GuiRenderState.class, "firstStratumAfterBlur");
            out.addProperty("blur", blurField.getInt(screen.state()) != Integer.MAX_VALUE);
            out.add("ops", record(screen.state()));
            var sent = new JsonArray();
            events.forEach(sent::add);
            out.add("events", sent);
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static void frame(net.minecraft.client.gui.screens.Screen options, Screen screen, int mouseX, int mouseY) {
        options.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
        options.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
    }

    /** The server's answer: the rules in the order the row lists them. */
    private static void values(net.minecraft.client.gui.screens.options.InWorldGameRulesScreen options, JsonArray list) {
        var map = new java.util.LinkedHashMap<net.minecraft.resources.ResourceKey<net.minecraft.world.level.gamerules.GameRule<?>>, String>();
        for (var pair : list) {
            var entry = pair.getAsJsonArray();
            map.put(net.minecraft.resources.ResourceKey.create(net.minecraft.core.registries.Registries.GAME_RULE,
                net.minecraft.resources.Identifier.parse(entry.get(0).getAsString())), entry.get(1).getAsString());
        }
        options.onGameRuleValuesUpdated(map);
    }
}
