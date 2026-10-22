import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.network.chat.Component;

/** The screens of server dialogs: the notice and confirmation screen, and the waiting screen. */
class GuiDialogOracle extends GuiSocialOracle {
    /** The screen a dialog is opened over, which a button's action leaves in place. */
    static final class PreviousScreen extends net.minecraft.client.gui.screens.Screen {
        PreviousScreen() {
            super(Component.empty());
        }
    }

    /** What the dialog tells the connection to do, written down in the order it did it. */
    static final class Access implements net.minecraft.client.gui.screens.dialog.DialogConnectionAccess {
        final java.util.List<String> events;
        final net.minecraft.client.gui.Gui gui;
        final net.minecraft.server.ServerLinks links;

        Access(RecordingGui gui, net.minecraft.server.ServerLinks links) {
            this.events = gui.events;
            this.gui = gui;
            this.links = links;
        }

        private static String name(net.minecraft.client.gui.screens.Screen screen) {
            return screen == null ? "close" : screen.getClass().getSimpleName();
        }

        @Override
        public void disconnect(Component message) {
            events.add("disconnect");
        }

        @Override
        public void runCommand(String command, net.minecraft.client.gui.screens.Screen activeScreen) {
            events.add("run_command:" + command + ":" + name(activeScreen));
            gui.setScreen(activeScreen);
        }

        @Override
        public void openDialog(net.minecraft.core.Holder<net.minecraft.server.dialog.Dialog> dialog, net.minecraft.client.gui.screens.Screen activeScreen) {
            events.add("open_dialog:" + name(activeScreen));
        }

        @Override
        public void sendCustomAction(net.minecraft.resources.Identifier id,
                java.util.Optional<net.minecraft.nbt.Tag> payload) {
            events.add("custom:" + id);
        }

        @Override
        public net.minecraft.server.ServerLinks serverLinks() {
            return links;
        }
    }

    private static net.minecraft.server.dialog.body.PlainMessage message(JsonObject json) {
        return new net.minecraft.server.dialog.body.PlainMessage(Component.literal(json.get("text").getAsString()),
            json.get("width").getAsInt());
    }

    private static net.minecraft.server.dialog.ActionButton button(JsonObject json) {
        var data = new net.minecraft.server.dialog.CommonButtonData(Component.literal(json.get("label").getAsString()),
            json.has("tooltip") ? java.util.Optional.of(Component.literal(json.get("tooltip").getAsString()))
                : java.util.Optional.empty(),
            json.get("width").getAsInt());
        java.util.Optional<net.minecraft.server.dialog.action.Action> action = java.util.Optional.empty();
        if (json.has("action")) {
            var event = json.getAsJsonObject("action");
            action = java.util.Optional.of(new net.minecraft.server.dialog.action.StaticAction(
                switch (event.get("type").getAsString()) {
                    case "run_command" -> new net.minecraft.network.chat.ClickEvent.RunCommand(
                        event.get("value").getAsString());
                    case "custom" -> new net.minecraft.network.chat.ClickEvent.Custom(
                        net.minecraft.resources.Identifier.parse(event.get("value").getAsString()),
                        java.util.Optional.empty());
                    default -> throw new IllegalArgumentException(event.toString());
                }));
        }
        return new net.minecraft.server.dialog.ActionButton(data, action);
    }

    private static net.minecraft.server.dialog.body.DialogBody body(JsonObject json) {
        if (json.has("item")) {
            return new net.minecraft.server.dialog.body.ItemBody(
                new net.minecraft.world.item.ItemStackTemplate(net.minecraft.world.item.Items.DIAMOND),
                json.has("description") ? java.util.Optional.of(message(json.getAsJsonObject("description")))
                    : java.util.Optional.empty(),
                true, false, json.get("width").getAsInt(), json.get("height").getAsInt());
        }
        return message(json);
    }

    /**
     * The game's own dialog screen (or waiting screen) after some keys, clicks and wheel turns:
     * what it told the connection and the game to do, and what it draws.
     */
    static JsonObject dialogScreen(JsonObject request) throws Exception {
        // The game registers the body handlers when it starts.
        net.minecraft.client.gui.screens.dialog.body.DialogBodyHandlers.bootstrap();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            set(minecraft, "itemModelResolver", unsafe.allocateInstance(NoItems.class));
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            if (row.get("in_world").getAsBoolean()) {
                set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            }
            var gui = (RecordingGui) unsafe.allocateInstance(RecordingGui.class);
            set(gui, "events", new java.util.ArrayList<String>());
            set(gui, "hud", screen.hud());
            set(minecraft, "gui", gui);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var json = row.getAsJsonObject("dialog");
            net.minecraft.client.gui.screens.Screen shown;
            if (json.get("kind").getAsString().equals("waiting")) {
                shown = new net.minecraft.client.gui.screens.dialog.WaitingForResponseScreen(new PreviousScreen());
            } else {
                var bodies = new java.util.ArrayList<net.minecraft.server.dialog.body.DialogBody>();
                for (var item : json.getAsJsonArray("body")) {
                    bodies.add(body(item.getAsJsonObject()));
                }
                var common = new net.minecraft.server.dialog.CommonDialogData(
                    Component.literal(json.get("title").getAsString()), java.util.Optional.empty(),
                    json.get("can_close").getAsBoolean(), false,
                    net.minecraft.server.dialog.DialogAction.valueOf(json.get("after").getAsString()),
                    bodies, java.util.List.of());
                var buttons = json.getAsJsonArray("buttons");
                var links = net.minecraft.server.ServerLinks.EMPTY;
                if (json.get("kind").getAsString().equals("server_links")) {
                    var entries = new java.util.ArrayList<net.minecraft.server.ServerLinks.Entry>();
                    for (var item : json.getAsJsonArray("links")) {
                        var link = item.getAsJsonObject();
                        var uri = java.net.URI.create(link.get("link").getAsString());
                        entries.add(link.has("known")
                            ? net.minecraft.server.ServerLinks.KnownLinkType.values()[link.get("known").getAsInt()].create(uri)
                            : net.minecraft.server.ServerLinks.Entry.custom(Component.literal(link.get("text").getAsString()), uri));
                    }
                    links = new net.minecraft.server.ServerLinks(entries);
                    var linked = new net.minecraft.server.dialog.ServerLinksDialog(common,
                        json.has("exit") ? java.util.Optional.of(button(json.getAsJsonObject("exit")))
                            : java.util.Optional.empty(),
                        json.get("columns").getAsInt(), json.get("button_width").getAsInt());
                    shown = new net.minecraft.client.gui.screens.dialog.ServerLinksDialogScreen(new PreviousScreen(), linked,
                        new Access(gui, links));
                } else if (json.get("kind").getAsString().equals("multi_action")) {
                    var actions = new java.util.ArrayList<net.minecraft.server.dialog.ActionButton>();
                    for (var item : buttons) {
                        actions.add(button(item.getAsJsonObject()));
                    }
                    var list = new net.minecraft.server.dialog.MultiActionDialog(common, actions,
                        json.has("exit") ? java.util.Optional.of(button(json.getAsJsonObject("exit")))
                            : java.util.Optional.empty(),
                        json.get("columns").getAsInt());
                    shown = new net.minecraft.client.gui.screens.dialog.MultiButtonDialogScreen(new PreviousScreen(), list,
                        new Access(gui, links));
                } else {
                    net.minecraft.server.dialog.SimpleDialog dialog = buttons.size() == 2
                        ? new net.minecraft.server.dialog.ConfirmationDialog(common, button(buttons.get(0).getAsJsonObject()),
                            button(buttons.get(1).getAsJsonObject()))
                        : new net.minecraft.server.dialog.NoticeDialog(common, button(buttons.get(0).getAsJsonObject()));
                    shown = new net.minecraft.client.gui.screens.dialog.SimpleDialogScreen<>(new PreviousScreen(), dialog,
                        new Access(gui, links));
                }
            }
            set(shown, "width", width);
            set(shown, "height", height);
            var init = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(shown);
            var info = new net.minecraft.client.input.MouseButtonInfo(0, 0);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                String action = step.get("do").getAsString();
                switch (action) {
                    case "tick" -> {
                        for (int tick = 0; tick < step.get("count").getAsInt(); tick++) {
                            shown.tick();
                        }
                    }
                    case "frame" -> {
                        now[0] = step.get("ms").getAsLong();
                        var state = new net.minecraft.client.renderer.state.gui.GuiRenderState();
                        var graphics = new net.minecraft.client.gui.GuiGraphicsExtractor(minecraft, state, mouseX, mouseY);
                        shown.extractRenderState(graphics, mouseX, mouseY, 0.0F);
                        graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
                    }
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "escape" -> 256;
                            case "enter" -> 257;
                            case "space" -> 32;
                            default -> throw new IllegalArgumentException(name);
                        };
                        if (code == 258) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        }
                        shown.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                    case "click" -> {
                        minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                        shown.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                            step.get("x").getAsDouble(), step.get("y").getAsDouble(), info), false);
                    }
                    case "drag" -> shown.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info),
                        step.get("dx").getAsDouble(), step.get("dy").getAsDouble());
                    case "release" -> shown.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info));
                    case "scroll" -> shown.mouseScrolled(step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        0.0, step.get("amount").getAsDouble());
                    default -> throw new IllegalArgumentException(action);
                }
            }
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            // Without a level the panorama would draw first, which is not a GUI drawing.
            minecraft.options.menuBackgroundBlurriness().set(row.has("blurriness") ? row.get("blurriness").getAsInt() : 5);
            if (row.get("in_world").getAsBoolean()) {
                // With a level the background blurs the frame behind first.
                shown.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            } else {
                var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("extractMenuBackground", net.minecraft.client.gui.GuiGraphicsExtractor.class);
                background.setAccessible(true);
                background.invoke(shown, screen.graphics());
            }
            shown.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var blurField = net.minecraft.client.renderer.state.gui.GuiRenderState.class.getDeclaredField("firstStratumAfterBlur");
            blurField.setAccessible(true);
            out.addProperty("blur", blurField.getInt(screen.state()) != Integer.MAX_VALUE);
            var events = new JsonArray();
            gui.events.forEach(events::add);
            out.add("events", events);
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The chat restrictions screen of some restrictions, after some keys, clicks and wheel turns:
     * what its buttons opened, and what it draws.
     */
    static JsonObject restrictionsScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            int mouseX = row.get("mouse").getAsJsonArray().get(0).getAsInt();
            int mouseY = row.get("mouse").getAsJsonArray().get(1).getAsInt();
            var screen = screen(width, height, mouseX, mouseY);
            var minecraft = screen.minecraft();
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            if (row.get("in_world").getAsBoolean()) {
                set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            }
            var gui = (RecordingGui) unsafe.allocateInstance(RecordingGui.class);
            set(gui, "events", new java.util.ArrayList<String>());
            set(gui, "hud", screen.hud());
            set(minecraft, "gui", gui);
            var shown = new net.minecraft.client.gui.screens.multiplayer.RestrictionsScreen(new PreviousScreen(),
                abilities(row.getAsJsonArray("restrictions")));
            set(shown, "width", width);
            set(shown, "height", height);
            var init = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(shown);
            var info = new net.minecraft.client.input.MouseButtonInfo(0, 0);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                String action = step.get("do").getAsString();
                switch (action) {
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "escape" -> 256;
                            case "enter" -> 257;
                            case "space" -> 32;
                            default -> throw new IllegalArgumentException(name);
                        };
                        if (code == 258) {
                            minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                        }
                        shown.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                    case "click" -> {
                        minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
                        shown.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                            step.get("x").getAsDouble(), step.get("y").getAsDouble(), info), false);
                    }
                    case "drag" -> shown.mouseDragged(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info),
                        step.get("dx").getAsDouble(), step.get("dy").getAsDouble());
                    case "release" -> shown.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(
                        step.get("x").getAsDouble(), step.get("y").getAsDouble(), info));
                    case "scroll" -> shown.mouseScrolled(step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        0.0, step.get("amount").getAsDouble());
                    default -> throw new IllegalArgumentException(action);
                }
            }
            minecraft.options.menuBackgroundBlurriness().set(5);
            if (row.get("in_world").getAsBoolean()) {
                shown.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            } else {
                var background = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("extractMenuBackground", net.minecraft.client.gui.GuiGraphicsExtractor.class);
                background.setAccessible(true);
                background.invoke(shown, screen.graphics());
            }
            shown.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var events = new JsonArray();
            gui.events.forEach(events::add);
            out.add("events", events);
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
