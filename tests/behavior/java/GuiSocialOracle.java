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

    /**
     * The game's own debug options screen over a real entry list, after the row's statuses were set and
     * a search typed, drawn once (the debug overlay behind it is the game's and not drawn here).
     */
    static JsonObject debugOptionsScreen(JsonObject request) throws Exception {
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
            set(minecraft, "level", unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class));
            minecraft.options.reducedDebugInfo().set(row.get("reduced").getAsBoolean());
            var directory = java.nio.file.Files.createTempDirectory("debug-options").toFile();
            var list = new net.minecraft.client.gui.components.debug.DebugScreenEntryList(directory,
                net.minecraft.util.datafix.DataFixers.getDataFixer());
            if (row.has("profile")) {
                list.loadProfile(net.minecraft.client.gui.components.debug.DebugScreenProfile
                    .valueOf(row.get("profile").getAsString()));
            }
            for (var setting : row.getAsJsonArray("set")) {
                var pair = setting.getAsJsonArray();
                list.setStatus(net.minecraft.resources.Identifier.withDefaultNamespace(pair.get(0).getAsString()),
                    net.minecraft.client.gui.components.debug.DebugScreenEntryStatus.valueOf(pair.get(1).getAsString()));
            }
            // What the setup saved is not what the screen saves.
            new java.io.File(directory, "debug-profile.json").delete();
            set(minecraft, "debugEntries", list);
            // The game that hears of the screens opened, as for the social screen.
            var hud = get(get(minecraft, "gui"), "hud");
            var recording = (RecordingGui) unsafe.allocateInstance(RecordingGui.class);
            set(recording, "hud", hud);
            set(recording, "events", new java.util.ArrayList<String>());
            set(minecraft, "gui", recording);
            var overlay = unsafe.allocateInstance(net.minecraft.client.gui.components.DebugScreenOverlay.class);
            set(overlay, "minecraft", minecraft);
            set(hud, "debugOverlay", overlay);
            var options = new net.minecraft.client.gui.screens.debug.DebugOptionsScreen();
            set(options, "minecraft", minecraft);
            set(options, "font", minecraft.font);
            options.init(width, height);
            var box = (net.minecraft.client.gui.components.EditBox) get(options, "searchBox");
            for (var cp : (Iterable<Integer>) row.get("search").getAsString().codePoints()::iterator) {
                box.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
            }
            if (row.has("scroll")) {
                options.mouseScrolled(width / 2.0, height / 2.0, 0.0, row.get("scroll").getAsDouble());
            }
            // The steps, each followed by a frame, as the game draws between its events; the entries
            // put their buttons in place as they draw, so a frame comes first.
            if (row.has("steps")) {
                options.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
                options.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
                screen.state().reset();
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    double x = step.has("x") ? step.get("x").getAsDouble() : 0.0;
                    double y = step.has("y") ? step.get("y").getAsDouble() : 0.0;
                    var press = new net.minecraft.client.input.MouseButtonEvent(x, y,
                        new net.minecraft.client.input.MouseButtonInfo(0, 0));
                    switch (step.get("do").getAsString()) {
                        case "click" -> options.mouseClicked(press, false);
                        case "release" -> options.mouseReleased(press);
                        case "drag" -> options.mouseDragged(press, 0.0, step.get("dy").getAsDouble());
                        case "type" -> {
                            for (var cp : (Iterable<Integer>) step.get("text").getAsString().codePoints()::iterator) {
                                options.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
                            }
                        }
                        case "key" -> options.keyPressed(new net.minecraft.client.input.KeyEvent(
                            step.get("key").getAsInt(), 0, step.has("shift") ? 1 : 0));
                        default -> throw new IllegalArgumentException(step.toString());
                    }
                    options.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
                    options.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
                    screen.state().reset();
                }
            }
            options.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            options.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var events = new JsonArray();
            for (var event : recording.events) {
                events.add(event);
            }
            out.add("events", events);
            // What the entry list saved: the file, none when nothing was saved.
            var saved = new java.io.File(directory, "debug-profile.json");
            if (saved.exists()) {
                out.add("file", com.google.gson.JsonParser.parseString(java.nio.file.Files.readString(saved.toPath())));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A game whose music situation and volume the scenario sets. */
    static final class MusicGame extends net.minecraft.client.Minecraft {
        net.minecraft.sounds.Music music;
        float volume = 1.0F;
        net.minecraft.client.sounds.SoundManager sounds;

        MusicGame() {
            super(null);
        }

        @Override
        public net.minecraft.sounds.Music getSituationalMusic() {
            return music;
        }

        @Override
        public float getMusicVolume() {
            return volume;
        }

        @Override
        public net.minecraft.client.sounds.SoundManager getSoundManager() {
            return sounds;
        }
    }

    /** A sound manager that writes down what the music manager asks of it. */
    static final class MusicSounds extends net.minecraft.client.sounds.SoundManager {
        final java.util.List<String> calls = new java.util.ArrayList<>();
        boolean active;

        MusicSounds() {
            super(null);
        }

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
                net.minecraft.client.resources.sounds.SoundInstance instance) {
            calls.add("play " + instance.getIdentifier());
            active = true;
            return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
        }

        @Override
        public void stop(net.minecraft.client.resources.sounds.SoundInstance instance) {
            calls.add("stop " + instance.getIdentifier());
            active = false;
        }

        @Override
        public boolean isActive(net.minecraft.client.resources.sounds.SoundInstance instance) {
            return active;
        }

        @Override
        public void updateCategoryVolume(net.minecraft.sounds.SoundSource source, float gain) {
            calls.add("volume " + Oracle.bits(gain));
        }
    }

    private static net.minecraft.sounds.Music musicOf(JsonObject object) {
        return new net.minecraft.sounds.Music(
            net.minecraft.core.Holder.direct(net.minecraft.sounds.SoundEvent.createVariableRangeEvent(
                net.minecraft.resources.Identifier.parse(object.get("sound").getAsString()))),
            object.get("min").getAsInt(), object.get("max").getAsInt(), object.get("replace").getAsBoolean());
    }

    /**
     * The reference's own `MusicManager`, ticked with what the scenario says the game wants: its
     * delay, the song it has and what it asked the sound manager for after each tick.
     */
    static JsonObject musicManager(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var game = (MusicGame) unsafe.allocateInstance(MusicGame.class);
            var sounds = (MusicSounds) unsafe.allocateInstance(MusicSounds.class);
            set(sounds, "calls", new java.util.ArrayList<String>());
            game.sounds = sounds;
            set(game, "gui", unsafe.allocateInstance(net.minecraft.client.gui.Gui.class));
            var manager = (net.minecraft.client.sounds.MusicManager) unsafe.allocateInstance(
                net.minecraft.client.sounds.MusicManager.class);
            var random = net.minecraft.util.RandomSource.create(row.get("seed").getAsLong());
            set(manager, "random", random);
            set(manager, "minecraft", game);
            set(manager, "gameMusicFrequency",
                net.minecraft.client.sounds.MusicManager.MusicFrequency.valueOf(row.get("frequency").getAsString()));
            set(manager, "currentGain", 1.0F);
            set(manager, "nextSongDelay", 100);
            var ticks = new JsonArray();
            for (var tickElement : row.getAsJsonArray("ticks")) {
                var tick = tickElement.getAsJsonObject();
                game.music = tick.get("music").isJsonNull() ? null : musicOf(tick.get("music").getAsJsonObject());
                game.volume = Oracle.f32(tick, "volume");
                if (tick.has("active")) {
                    sounds.active = tick.get("active").getAsBoolean();
                }
                sounds.calls.clear();
                manager.tick();
                var out = new JsonObject();
                out.addProperty("delay", (int) get(manager, "nextSongDelay"));
                var current = get(manager, "currentMusic");
                out.addProperty("current", current == null ? ""
                    : ((net.minecraft.client.resources.sounds.SoundInstance) current).getIdentifier().toString());
                out.addProperty("gain", Oracle.bits((float) get(manager, "currentGain")));
                var calls = new JsonArray();
                sounds.calls.forEach(calls::add);
                out.add("calls", calls);
                ticks.add(out);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ticks", ticks);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

}
