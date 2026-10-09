import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The social interactions screen: who is online, hidden or blocked, and the search over them. */
class GuiSocialOracle extends GuiCreditsOracle {
    /** A game that remembers the screens it was told to show instead of switching to them. */
    static final class RecordingGui extends net.minecraft.client.gui.Gui {
        final java.util.List<String> events = new java.util.ArrayList<>();

        RecordingGui() {
            super(null, null, null);
        }

        @Override
        public void setScreen(net.minecraft.client.gui.screens.Screen screen) {
            events.add(screen == null ? "close"
                : screen instanceof net.minecraft.client.gui.screens.ConfirmLinkScreen ? "link"
                : screen.getClass().getSimpleName());
        }
    }

    /** A listed player whose face is the default skin its id picks, as an offline profile's is. */
    private static net.minecraft.client.multiplayer.PlayerInfo info(JsonObject player) throws Exception {
        var profile = new com.mojang.authlib.GameProfile(java.util.UUID.fromString(player.get("id").getAsString()),
            player.get("name").getAsString());
        var info = new net.minecraft.client.multiplayer.PlayerInfo(profile, false);
        var skin = net.minecraft.client.resources.DefaultPlayerSkin.get(profile);
        set(info, "skinLookup", (java.util.function.Supplier<net.minecraft.world.entity.player.PlayerSkin>) () -> skin);
        return info;
    }

    /**
     * The game's own screen over a server of the row's players, some of them hidden or gone, on
     * one of its pages, after a search was typed, drawn once.
     */
    static JsonObject socialInteractionsScreen(JsonObject request) throws Exception {
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
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            var online = new java.util.HashMap<java.util.UUID, net.minecraft.client.multiplayer.PlayerInfo>();
            for (var player : row.getAsJsonArray("players")) {
                var described = info(player.getAsJsonObject());
                online.put(described.getProfile().id(), described);
            }
            var seen = new java.util.HashMap<java.util.UUID, net.minecraft.client.multiplayer.PlayerInfo>();
            for (var player : row.getAsJsonArray("gone")) {
                var described = info(player.getAsJsonObject());
                seen.put(described.getProfile().id(), described);
            }
            set(listener, "playerInfoMap", online);
            set(listener, "seenPlayers", seen);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            var localId = java.util.UUID.fromString(row.get("local").getAsString());
            set(player, "uuid", localId);
            set(minecraft, "user", new net.minecraft.client.User("local", localId, "",
                java.util.Optional.empty(), java.util.Optional.empty()));
            set(player, "chatAbilities", net.minecraft.client.multiplayer.chat.ChatAbilities.NO_RESTRICTIONS);
            set(minecraft, "player", player);
            // The buttons click when pressed, heard by no one.
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            // The game that hears of the screens opened and the chat it is told.
            var recording = (RecordingGui) unsafe.allocateInstance(RecordingGui.class);
            var hud = get(get(minecraft, "gui"), "hud");
            var chat = new net.minecraft.client.gui.components.ChatComponent(minecraft);
            set(hud, "chat", chat);
            set(recording, "hud", hud);
            set(recording, "events", new java.util.ArrayList<String>());
            set(minecraft, "gui", recording);
            // Opened in a game: the dimmed world behind, no panorama.
            set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            var server = new net.minecraft.client.multiplayer.ServerData(row.get("server").getAsString(),
                "127.0.0.1:1", net.minecraft.client.multiplayer.ServerData.Type.OTHER);
            set(listener, "serverData", server);
            set(minecraft, "isLocalServer", false);
            var social = unsafe.allocateInstance(net.minecraft.client.gui.screens.social.PlayerSocialManager.class);
            var hidden = new java.util.HashSet<java.util.UUID>();
            for (var id : row.getAsJsonArray("hidden")) {
                hidden.add(java.util.UUID.fromString(id.getAsString()));
            }
            set(social, "hiddenPlayers", hidden);
            set(minecraft, "playerSocialManager", social);
            var sender = java.lang.reflect.Proxy.newProxyInstance(GuiSocialOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.multiplayer.chat.report.AbuseReportSender.class},
                (proxy, method, args) -> row.get("reporting").getAsBoolean());
            set(minecraft, "reportingContext", new net.minecraft.client.multiplayer.chat.report.ReportingContext(
                (net.minecraft.client.multiplayer.chat.report.AbuseReportSender) sender,
                net.minecraft.client.multiplayer.chat.report.ReportEnvironment.local(),
                new net.minecraft.client.multiplayer.chat.ChatLog(16)));
            var page = net.minecraft.client.gui.screens.social.SocialInteractionsScreen.Page.valueOf(
                row.get("page").getAsString().toUpperCase());
            var social_screen = new net.minecraft.client.gui.screens.social.SocialInteractionsScreen();
            set(social_screen, "page", page);
            set(social_screen, "minecraft", minecraft);
            set(social_screen, "font", minecraft.font);
            social_screen.init(width, height);
            var box = (net.minecraft.client.gui.components.EditBox) get(social_screen, "searchBox");
            for (var cp : (Iterable<Integer>) row.get("search").getAsString().codePoints()::iterator) {
                box.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
            }
            // The frames before the one recorded: the blocking hint shows only in the first.
            for (int frame = 1; frame < row.get("frames").getAsInt(); frame++) {
                social_screen.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
                social_screen.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
                screen.state().reset();
            }
            // The steps, each followed by a frame, as the game draws between its events.
            if (row.has("steps")) {
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    double x = step.has("x") ? step.get("x").getAsDouble() : 0.0;
                    double y = step.has("y") ? step.get("y").getAsDouble() : 0.0;
                    var press = new net.minecraft.client.input.MouseButtonEvent(x, y,
                        new net.minecraft.client.input.MouseButtonInfo(0, 0));
                    switch (step.get("do").getAsString()) {
                        case "click" -> social_screen.mouseClicked(press, false);
                        case "release" -> social_screen.mouseReleased(press);
                        case "drag" -> social_screen.mouseDragged(press, 0.0, step.get("dy").getAsDouble());
                        case "scroll" -> social_screen.mouseScrolled(x, y, 0.0, step.get("dy").getAsDouble());
                        case "type" -> {
                            for (var cp : (Iterable<Integer>) step.get("text").getAsString().codePoints()::iterator) {
                                social_screen.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
                            }
                        }
                        case "key" -> social_screen.keyPressed(
                            new net.minecraft.client.input.KeyEvent(step.get("key").getAsInt(), 0, 0));
                        default -> throw new IllegalArgumentException(step.toString());
                    }
                    social_screen.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
                    social_screen.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
                    screen.state().reset();
                }
            }
            social_screen.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            social_screen.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            // What the screen asked of the game, and the chat it was told.
            var events = new JsonArray();
            for (var event : recording.events) {
                events.add(event);
            }
            @SuppressWarnings("unchecked")
            var told = (java.util.List<net.minecraft.client.multiplayer.chat.GuiMessage>) get(chat, "allMessages");
            for (int i = told.size() - 1; i >= 0; i--) {
                events.add("chat:" + told.get(i).content().getString());
            }
            out.add("events", events);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
