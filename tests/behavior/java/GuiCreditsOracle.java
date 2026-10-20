import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The credits screen (`WinScreen`): the end poem and the credits roll. */
class GuiCreditsOracle extends GuiJigsawOracle {
    /** The game's own credits screen, scrolled, with keys pressed and released, drawn once. */
    static JsonObject creditsScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var screen = screen(width, height, 0, 0);
            var minecraft = screen.minecraft();
            var manager = (net.minecraft.server.packs.resources.ReloadableResourceManager)
                unsafe.allocateInstance(net.minecraft.server.packs.resources.ReloadableResourceManager.class);
            set(manager, "resources", resources);
            set(minecraft, "resourceManager", manager);
            set(minecraft, "user", new net.minecraft.client.User(row.get("player").getAsString(),
                java.util.UUID.randomUUID(), "", java.util.Optional.empty(), java.util.Optional.empty()));
            // In a game, as the credits come: no panorama behind the menu's background.
            set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            boolean[] finished = {false};
            var win = new net.minecraft.client.gui.screens.WinScreen(row.get("poem").getAsBoolean(),
                () -> finished[0] = true);
            plainLogo(win);
            set(win, "minecraft", minecraft);
            set(win, "font", minecraft.font);
            set(win, "width", width);
            set(win, "height", height);
            var init = net.minecraft.client.gui.screens.WinScreen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(win);
            set(win, "scroll", (float) row.get("scroll").getAsDouble());
            for (var step : row.getAsJsonArray("keys")) {
                int code = step.getAsJsonObject().get("code").getAsInt();
                var event = new net.minecraft.client.input.KeyEvent(code, 0, 0);
                if (step.getAsJsonObject().get("down").getAsBoolean()) {
                    win.keyPressed(event);
                } else {
                    win.keyReleased(event);
                }
            }
            // `tick` also runs the music and sound managers: only its end check is the screen's.
            var maxScroll = ((Number) get(win, "totalScrollLength")).floatValue() + height + height + 24;
            if (((Number) get(win, "scroll")).floatValue() > maxScroll) {
                win.onClose();
            }
            win.extractBackground(screen.graphics(), 0, 0, 0.0F);
            win.extractRenderState(screen.graphics(), 0, 0, (float) row.get("partial").getAsDouble());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.addProperty("finished", finished[0]);
            out.addProperty("scroll", ((Number) get(win, "scroll")).floatValue());
            out.addProperty("lines", ((java.util.List<?>) get(win, "lines")).size());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
