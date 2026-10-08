import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;

/** The title screen, for a client that may play online but reaches no service. */
class GuiTitleOracle extends GuiSettingsOracle {
    /** The game's own title screen with a given splash, at a time, under a mouse, after keys. */
    static JsonObject titleScreen(JsonObject request) throws Exception {
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
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            // Online play allowed: the offline profile's properties, no ban, no name change.
            set(minecraft, "allowsMultiplayer", true);
            var offline = com.mojang.authlib.minecraft.UserApiService.class.getField("OFFLINE_PROPERTIES").get(null);
            set(minecraft, "userPropertiesFuture", java.util.concurrent.CompletableFuture.completedFuture(offline));
            set(minecraft, "profileFuture", java.util.concurrent.CompletableFuture.completedFuture(null));
            set(minecraft, "playerSocialManager",
                unsafe.allocateInstance(net.minecraft.client.gui.screens.social.PlayerSocialManager.class));
            // The panorama, held still; its cube is drawn apart from the GUI.
            var gameRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
            var panorama = new net.minecraft.client.renderer.Panorama();
            set(gameRenderer, "panorama", panorama);
            var gameState = unsafe.allocateInstance(net.minecraft.client.renderer.state.GameRenderState.class);
            set(gameState, "guiRenderState", screen.state());
            set(gameRenderer, "gameRenderState", gameState);
            set(minecraft, "gameRenderer", gameRenderer);
            var title = new net.minecraft.client.gui.screens.TitleScreen(false);
            panorama.holdSpin();
            var literal = net.minecraft.client.resources.SplashManager.class.getDeclaredMethod("literalSplash", String.class);
            literal.setAccessible(true);
            set(title, "splash", new net.minecraft.client.gui.components.SplashRenderer(
                (net.minecraft.network.chat.Component) literal.invoke(null, row.get("splash").getAsString())));
            minecraft.options.hideSplashTexts().set(row.has("hide_splash") && row.get("hide_splash").getAsBoolean());
            // The Realms notifications, for a client Realms does not answer: nothing to show.
            var realms = (com.mojang.realmsclient.gui.screens.RealmsNotificationsScreen)
                unsafe.allocateInstance(com.mojang.realmsclient.gui.screens.RealmsNotificationsScreen.class);
            set(realms, "validClient", java.util.concurrent.CompletableFuture.completedFuture(false));
            set(realms, "minecraft", minecraft);
            set(realms, "font", minecraft.font);
            set(realms, "renderables", new java.util.ArrayList<>());
            set(realms, "children", new java.util.ArrayList<>());
            set(realms, "narratables", new java.util.ArrayList<>());
            set(title, "realmsNotificationsScreen", realms);
            set(title, "width", width);
            set(title, "height", height);
            var init = net.minecraft.client.gui.screens.TitleScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(title);
            if (row.has("steps")) {
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    if (step.get("do").getAsString().equals("frame")) {
                        now[0] = step.get("ms").getAsLong();
                        var frame = new GuiRenderState();
                        var graphics = new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY);
                        title.extractRenderState(graphics, mouseX, mouseY, 0.0F);
                        graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
                    } else {
                        String name = step.get("key").getAsString();
                        int code = switch (name) {
                            case "tab", "shift+tab" -> 258;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "up" -> 265;
                            case "down" -> 264;
                            default -> throw new IllegalArgumentException(name);
                        };
                        minecraft.setLastInputType(code == 258 ? net.minecraft.client.InputType.KEYBOARD_TAB
                            : net.minecraft.client.InputType.KEYBOARD_ARROW);
                        title.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, name.startsWith("shift+") ? 1 : 0));
                    }
                }
            }
            now[0] = row.has("ms") ? row.get("ms").getAsLong() : 0;
            title.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            title.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
