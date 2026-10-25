import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import java.util.HashMap;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Boss bars, the locator bar and the hotbar and status HUD. */
class GuiHudOracle extends GuiCommandOracle {
    /**
     * The subtitle overlay (`SubtitleOverlay`): the sounds heard at their times (the game's clock
     * pinned to them), then the overlay drawn at a later time from a listener's place.
     */
    static JsonObject subtitles(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            // The sound manager only answers where the listener is.
            var vec = (java.util.function.Function<com.google.gson.JsonArray, net.minecraft.world.phys.Vec3>) array ->
                new net.minecraft.world.phys.Vec3(array.get(0).getAsDouble(), array.get(1).getAsDouble(),
                    array.get(2).getAsDouble());
            Access26_2.listenFrom(minecraft, new com.mojang.blaze3d.audio.ListenerTransform(
                vec.apply(row.getAsJsonArray("position")), vec.apply(row.getAsJsonArray("forward")),
                vec.apply(row.getAsJsonArray("up"))));
            minecraft.options.showSubtitles().set(true);
            minecraft.options.notificationDisplayTime().set(row.get("display_time").getAsDouble());
            var overlay = new net.minecraft.client.gui.components.SubtitleOverlay(minecraft);
            Access26_2.listenForSubtitles(overlay);
            for (var heardElement : row.getAsJsonArray("heard")) {
                var heard = heardElement.getAsJsonObject();
                long at = heard.get("at").getAsLong();
                net.minecraft.util.Util.setTimeSource(() -> at * 1_000_000L);
                var place = vec.apply(heard.getAsJsonArray("position"));
                var sound = (net.minecraft.client.resources.sounds.SoundInstance) java.lang.reflect.Proxy.newProxyInstance(
                    Oracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.resources.sounds.SoundInstance.class},
                    (proxy, method, args) -> switch (method.getName()) {
                        case "getX" -> place.x;
                        case "getY" -> place.y;
                        case "getZ" -> place.z;
                        default -> null;
                    });
                var events = new net.minecraft.client.sounds.WeighedSoundEvents(
                    net.minecraft.resources.Identifier.withDefaultNamespace("oracle"), heard.get("key").getAsString());
                float range = heard.get("infinite").getAsBoolean() ? Float.POSITIVE_INFINITY : heard.get("range").getAsFloat();
                overlay.onPlaySound(sound, events, range);
            }
            long now = row.get("now").getAsLong();
            net.minecraft.util.Util.setTimeSource(() -> now * 1_000_000L);
            overlay.extractRenderState(screen.graphics());
            net.minecraft.util.Util.setTimeSource(System::nanoTime);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The sound manager of the toasts' game: it notes the sounds asked of it. */
    static class RecordingSounds extends net.minecraft.client.sounds.SoundManager {
        final java.util.List<String> played = new java.util.ArrayList<>();

        RecordingSounds() {
            super(null);
        }

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
                final net.minecraft.client.resources.sounds.SoundInstance instance) {
            this.played.add(instance.getIdentifier().toString());
            return net.minecraft.client.sounds.SoundEngine.PlayResult.STARTED;
        }
    }

    /**
     * The toasts (`ToastManager`): a script of events (toasts added or updated, the manager's
     * update at a time of the game's clock, the music toast shown, the manager drawn), each
     * answered with the sounds it made and the ops it drew.
     */
    static JsonObject toasts(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            var sounds = (RecordingSounds) unsafe.allocateInstance(RecordingSounds.class);
            Access26_2.recordSounds(minecraft, sounds, new java.util.ArrayList<String>());
            // An item is not drawn here: its resolver adds nothing.
            Access26_2.drawNoItems(minecraft, unsafe.allocateInstance(GuiSettingsOracle.NoItems.class));
            // The options' own change callbacks reach for the game's toast manager: set the values
            // under them.
            var musicState = net.minecraft.client.MusicToastDisplayState.valueOf(
                row.get("music_toast").getAsString());
            Access26_2.optionValue(minecraft.options.musicToast(), musicState);
            Access26_2.optionValue(minecraft.options.notificationDisplayTime(), row.get("display_time").getAsDouble());
            // The song that plays, as the toast asks the music manager for its key.
            var music = unsafe.allocateInstance(net.minecraft.client.sounds.MusicManager.class);
            if (row.has("song")) {
                Access26_2.playSong(music, net.minecraft.resources.Identifier.parse(row.get("song").getAsString()));
            }
            Access26_2.useMusicManager(minecraft, music);
            Access26_2.hideHud(screen.hud(), row.get("hud_hidden").getAsBoolean());
            if (row.get("pause_screen").getAsBoolean()) {
                Access26_2.showPauseScreen(screen.minecraft().gui);
            }
            Access26_2.holdMusicNote();
            var manager = new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options);
            var events = new JsonArray();
            net.minecraft.client.gui.components.toasts.TutorialToast[] tutorial = {null};
            for (var eventElement : row.getAsJsonArray("events")) {
                var event = eventElement.getAsJsonObject();
                var out = new JsonObject();
                out.addProperty("do", event.get("do").getAsString());
                var systemId = (java.util.function.Function<String, net.minecraft.client.gui.components.toasts.SystemToast.SystemToastId>) name -> {
                    try {
                        return (net.minecraft.client.gui.components.toasts.SystemToast.SystemToastId)
                            net.minecraft.client.gui.components.toasts.SystemToast.SystemToastId.class
                                .getField(name).get(null);
                    } catch (ReflectiveOperationException error) {
                        throw new IllegalStateException(error);
                    }
                };
                var text = (java.util.function.Function<String, Component>) literal -> literal == null ? null : Component.literal(literal);
                switch (event.get("do").getAsString()) {
                    case "system" -> net.minecraft.client.gui.components.toasts.SystemToast.add(manager,
                        systemId.apply(event.get("id").getAsString()),
                        text.apply(event.get("title").getAsString()),
                        event.get("message").isJsonNull() ? null : text.apply(event.get("message").getAsString()));
                    case "system_update" -> net.minecraft.client.gui.components.toasts.SystemToast.addOrUpdate(manager,
                        systemId.apply(event.get("id").getAsString()),
                        text.apply(event.get("title").getAsString()),
                        event.get("message").isJsonNull() ? null : text.apply(event.get("message").getAsString()));
                    case "advancement" -> manager.addToast(new net.minecraft.client.gui.components.toasts.AdvancementToast(
                        advancementHolder(event.getAsJsonObject("advancement"))));
                    case "recipe" -> {
                        var stack = (java.util.function.Function<com.google.gson.JsonElement, net.minecraft.world.item.ItemStack>) name ->
                            name.isJsonNull() ? net.minecraft.world.item.ItemStack.EMPTY
                                : new net.minecraft.world.item.ItemStack(net.minecraft.core.registries.BuiltInRegistries.ITEM
                                    .getValue(net.minecraft.resources.Identifier.parse(name.getAsString())));
                        Access26_2.showRecipe(manager, stack.apply(event.get("station")), stack.apply(event.get("result")));
                    }
                    case "tutorial" -> {
                        var icons = net.minecraft.client.gui.components.toasts.TutorialToast.Icons.values();
                        tutorial[0] = new net.minecraft.client.gui.components.toasts.TutorialToast(minecraft.font,
                            icons[event.get("icon").getAsInt()], Component.literal(event.get("title").getAsString()),
                            event.get("message").isJsonNull() ? null : Component.literal(event.get("message").getAsString()),
                            event.get("progressable").getAsBoolean(), event.get("time_ms").getAsInt());
                        manager.addToast(tutorial[0]);
                    }
                    case "tutorial_progress" -> tutorial[0].updateProgress(event.get("value").getAsFloat());
                    case "tutorial_hide" -> tutorial[0].hide();
                    case "force_hide" -> net.minecraft.client.gui.components.toasts.SystemToast.forceHide(manager,
                        systemId.apply(event.get("id").getAsString()));
                    case "note_colours" -> {
                        var colours = new JsonArray();
                        for (var tickElement : event.getAsJsonArray("ticks")) {
                            colours.add(net.minecraft.client.color.ColorLerper.getLerpedColor(
                                net.minecraft.client.color.ColorLerper.Type.MUSIC_NOTE, tickElement.getAsFloat()));
                        }
                        out.add("colours", colours);
                    }
                    case "show_now_playing" -> manager.showNowPlayingToast();
                    case "hide_now_playing" -> manager.hideNowPlayingToast();
                    case "update" -> {
                        long at = event.get("at").getAsLong();
                        net.minecraft.util.Util.setTimeSource(() -> at * 1_000_000L);
                        manager.update();
                    }
                    case "draw" -> {
                        screen.state().reset();
                        manager.extractRenderState(screen.graphics());
                        out.add("ops", record(screen.state()));
                    }
                    default -> throw new IllegalArgumentException(event.get("do").getAsString());
                }
                var heard = new JsonArray();
                for (var name : sounds.played) {
                    heard.add(name);
                }
                sounds.played.clear();
                out.add("sounds", heard);
                var recipeClass = net.minecraft.client.gui.components.toasts.RecipeToast.class;
                var recipeToast = manager.getToast(recipeClass, net.minecraft.client.gui.components.toasts.Toast.NO_TOKEN);
                if (recipeToast == null) {
                    out.add("recipe_index", com.google.gson.JsonNull.INSTANCE);
                } else {
                    out.addProperty("recipe_index", Access26_2.recipeIndex(recipeToast));
                }
                events.add(out);
            }
            net.minecraft.util.Util.setTimeSource(System::nanoTime);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("events", events);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The boss bars: each event as the packet adds it, drawn by the overlay. */
    static JsonObject bossBar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var overlay = new net.minecraft.client.gui.components.BossHealthOverlay(screen.minecraft());
            int index = 0;
            for (var barElement : row.getAsJsonArray("bars")) {
                var bar = barElement.getAsJsonObject();
                var colors = net.minecraft.world.BossEvent.BossBarColor.values();
                var overlays = net.minecraft.world.BossEvent.BossBarOverlay.values();
                var id = new java.util.UUID(0L, ++index);
                ((java.util.Map<java.util.UUID, net.minecraft.client.gui.components.LerpingBossEvent>)
                    get(overlay, "events")).put(id, new net.minecraft.client.gui.components.LerpingBossEvent(
                        id, Component.literal(bar.get("name").getAsString()), bar.get("progress").getAsFloat(),
                        colors[bar.get("color").getAsInt()], overlays[bar.get("overlay").getAsInt()],
                        false, false, false));
            }
            overlay.extractRenderState(screen.graphics());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The locator bar over the waypoints packets track, seen from a camera and a player. */
    static JsonObject locatorBar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        var deviceInfo = new com.mojang.blaze3d.systems.DeviceInfo(null, null, null, true, null, 0.0F, null, null,
            null, null, null);
        var backend = (com.mojang.blaze3d.systems.GpuDeviceBackend) java.lang.reflect.Proxy.newProxyInstance(
            GuiOracle.class.getClassLoader(), new Class<?>[] {com.mojang.blaze3d.systems.GpuDeviceBackend.class},
            (proxy, method, args) -> method.getName().equals("getDeviceInfo") ? deviceInfo : null);
        Access26_2.installDevice(backend);
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var screen = screen(width, height, 0, 0);
            var minecraft = screen.minecraft();
            var camera = row.getAsJsonObject("camera");
            var at = camera.getAsJsonArray("pos");
            var feet = row.getAsJsonObject("player").getAsJsonArray("pos");
            var uuid = row.getAsJsonObject("player").getAsJsonArray("uuid");
            float fov = (float) camera.get("fov").getAsDouble();
            var cameraObject = Access26_2.camera((float) camera.get("yaw").getAsDouble(),
                (float) camera.get("pitch").getAsDouble(), new net.minecraft.world.phys.Vec3(at.get(0).getAsDouble(),
                    at.get(1).getAsDouble(), at.get(2).getAsDouble()), fov);
            var projection = Access26_2.projection(cameraObject);
                        projection.setupPerspective(0.05F, (float) camera.get("far").getAsDouble(), fov,
                (float) camera.get("aspect").getAsDouble(), 1.0F);
            Access26_2.renderThrough(minecraft, cameraObject);
            var manager = new net.minecraft.client.waypoints.ClientWaypointManager();
            Access26_2.localPlayer(minecraft, cameraObject, new net.minecraft.world.phys.Vec3(feet.get(0).getAsDouble(),
                feet.get(1).getAsDouble(), feet.get(2).getAsDouble()),
                new java.util.UUID(uuid.get(0).getAsLong(), uuid.get(1).getAsLong()), manager);

            var styles = new HashMap<net.minecraft.resources.ResourceKey<
                net.minecraft.world.waypoints.WaypointStyleAsset>, net.minecraft.client.resources.WaypointStyle>();
            for (var name : new String[] {"default", "bowtie"}) {
                try (var stream = GuiOracle.class.getClassLoader().getResourceAsStream(
                        "assets/minecraft/waypoint_style/" + name + ".json")) {
                    var json = com.google.gson.JsonParser.parseReader(new java.io.InputStreamReader(stream));
                    styles.put(net.minecraft.world.waypoints.WaypointStyleAssets.createId(name),
                        net.minecraft.client.resources.WaypointStyle.CODEC
                            .parse(com.mojang.serialization.JsonOps.INSTANCE, json).getOrThrow());
                }
            }
            Access26_2.waypointStyles(screen.hud(),
                unsafe.allocateInstance(net.minecraft.client.resources.WaypointStyleManager.class), styles);

            for (var packetElement : row.getAsJsonArray("packets")) {
                var hex = packetElement.getAsString();
                var bytes = new byte[hex.length() / 2];
                for (int i = 0; i < bytes.length; i++) {
                    bytes[i] = (byte) Integer.parseInt(hex.substring(2 * i, 2 * i + 2), 16);
                }
                var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                    io.netty.buffer.Unpooled.wrappedBuffer(bytes), (net.minecraft.core.RegistryAccess) Game26_2.vanillaRegistries());
                net.minecraft.network.protocol.game.ClientboundTrackedWaypointPacket.STREAM_CODEC.decode(buffer)
                    .apply(manager);
            }
            var bar = new net.minecraft.client.gui.contextualbar.LocatorBar(minecraft);
            bar.extractBackground(screen.graphics(), net.minecraft.client.DeltaTracker.ZERO);
            bar.extractRenderState(screen.graphics(), net.minecraft.client.DeltaTracker.ZERO);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** What the HUD's methods are called against: a real level, a real local player and the stub game. */
    record HudScene(net.minecraft.client.Minecraft minecraft, net.minecraft.client.multiplayer.ClientLevel level,
                    net.minecraft.client.player.LocalPlayer local, Screen screen) {}

    /**
     * The scene the HUD is drawn in: a window of this size, a real local player in a game mode, the camera
     * on it, and the settings the HUD asks of the game.
     */
    static HudScene hudScene(int width, int height, boolean spectator, boolean hardcore) throws Exception {
        MovementOracle.bootstrap();
        var empty = new JsonObject();
        empty.add("blocks", new JsonArray());
        empty.addProperty("chunk_radius", 0);
        var scene = MovementOracle.buildScene(empty);
        var level = scene.level();
        var minecraft = scene.minecraft();
        var screen = prepare(minecraft, width, height, 0, 0);
        if (hardcore) {
            MovementOracle.setField(level.getLevelData(), "hardcore", true);
        }
        var local = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, scene.listener(), null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        local.setId(1);
        var mode = spectator ? net.minecraft.world.level.GameType.SPECTATOR : net.minecraft.world.level.GameType.SURVIVAL;
        var info = new net.minecraft.client.multiplayer.PlayerInfo(scene.listener().getLocalGameProfile(), false);
        Access26_2.setGameMode(info, mode);
        MovementOracle.setField(local, "playerInfo", info);
        MovementOracle.setField(minecraft, "player", local);
        var gameMode = unsafe.allocateInstance(net.minecraft.client.multiplayer.MultiPlayerGameMode.class);
        MovementOracle.setField(gameMode, "localPlayerMode", mode);
        MovementOracle.setField(minecraft, "gameMode", gameMode);
        var entries = unsafe.allocateInstance(net.minecraft.client.gui.components.debug.DebugScreenEntryList.class);
        MovementOracle.setField(entries, "currentlyEnabled", new java.util.ArrayList<>());
        MovementOracle.setField(minecraft, "debugEntries", entries);
        // The camera is the player's; the game renderer holds it.
        var camera = new net.minecraft.client.Camera();
        camera.setEntity(local);
        var gameRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        MovementOracle.setField(gameRenderer, "mainCamera", camera);
        MovementOracle.setField(minecraft, "gameRenderer", gameRenderer);
        return new HudScene(minecraft, level, local, screen);
    }

    /**
     * The block atlas sprite the portal overlay is drawn with: a model manager whose every block
     * has one particle sprite, in the atlas the recorder names `textures/atlas/blocks.png`.
     */
    static void stubPortalSprite(net.minecraft.client.Minecraft minecraft) throws Exception {
        var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) GuiRecorder.unsafe
            .allocateInstance(net.minecraft.client.renderer.texture.TextureAtlasSprite.class);
        MovementOracle.setField(sprite, "atlasLocation",
            net.minecraft.resources.Identifier.withDefaultNamespace("textures/atlas/blocks.png"));
        MovementOracle.setField(sprite, "u0", 0.25F);
        MovementOracle.setField(sprite, "u1", 0.5F);
        MovementOracle.setField(sprite, "v0", 0.0F);
        MovementOracle.setField(sprite, "v1", 0.25F);
        var baked = new net.minecraft.client.resources.model.sprite.Material.Baked(sprite, false);
        var model = (net.minecraft.client.renderer.block.dispatch.BlockStateModel) java.lang.reflect.Proxy
            .newProxyInstance(GuiHudOracle.class.getClassLoader(),
                new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModel.class},
                (proxy, method, arguments) -> baked);
        var models = new net.minecraft.client.renderer.block.BlockStateModelSet(java.util.Map.of(), model);
        var manager = GuiRecorder.unsafe.allocateInstance(net.minecraft.client.resources.model.ModelManager.class);
        MovementOracle.setField(manager, "blockStateModelSet", models);
        MovementOracle.setField(minecraft, "modelManager", manager);
    }

    /** A private method of the `Hud`, by name. */
    static java.lang.reflect.Method hudMethod(String name, Class<?>... types) throws Exception {
        return Access26_2.hudMethod(name, types);
    }

    /**
     * The game's own sleep veil (`Hud.extractSleepOverlay`) for a real local player whose sleep
     * counter is set.
     */
    static JsonObject hudSleep(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), false, false);
            MovementOracle.setField(scene.local(), "sleepCounter", row.get("timer").getAsInt());
            hudMethod("extractSleepOverlay", GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class)
                .invoke(scene.screen().hud(), scene.screen().graphics(), net.minecraft.client.DeltaTracker.ZERO);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(scene.screen().state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The game's own effect icons (`Hud.extractEffects`) for a real local player's effects. */
    static JsonObject hudEffects(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), false, false);
            for (var effectElement : row.getAsJsonArray("effects")) {
                var effect = effectElement.getAsJsonObject();
                var holder = net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT.get(
                    net.minecraft.resources.Identifier.parse(effect.get("effect").getAsString())).orElseThrow();
                scene.local().forceAddEffect(new net.minecraft.world.effect.MobEffectInstance(
                    holder, effect.get("duration").getAsInt(), effect.get("amplifier").getAsInt(),
                    effect.get("ambient").getAsBoolean(), true, effect.get("show_icon").getAsBoolean()), null);
            }
            hudMethod("extractEffects", GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class)
                .invoke(scene.screen().hud(), scene.screen().graphics(), net.minecraft.client.DeltaTracker.ZERO);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(scene.screen().state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own `StatsScreen` over a real `StatsCounter`: in the loading state or with the
     * statistics in, on a tab, after some scrolls and clicks, drawn in a world.
     */
    @SuppressWarnings("unchecked")
    static JsonObject statsScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var mouse = row.getAsJsonArray("mouse");
            int mouseX = mouse.get(0).getAsInt();
            int mouseY = mouse.get(1).getAsInt();
            var scene = hudScene(width, height, false, false);
            var minecraft = scene.minecraft();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            MovementOracle.setField(minecraft, "level", scene.level());
            MovementOracle.setField(minecraft, "soundManager", unsafe.allocateInstance(GuiScreenOracle.SilentSounds.class));
            GuiScreenOracle.SilentSounds.ASKED.clear();
            // Items are not drawn: the model resolver gives nothing to draw.
            MovementOracle.setField(minecraft, "itemModelResolver", unsafe.allocateInstance(GuiSettingsOracle.NoItems.class));
            var clockBefore = Access26_2.clock();
            long[] now = {row.get("now").getAsLong()};
            Access26_2.setClock((net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var languageBefore = net.minecraft.locale.Language.getInstance();
            net.minecraft.locale.Language.inject(clientLanguage("en_us"));
            try {
                var connection = new net.minecraft.network.Connection(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
                MovementOracle.setField(scene.local().connection, "connection", connection);
                var counter = new net.minecraft.stats.StatsCounter();
                for (var stat : row.getAsJsonArray("stats")) {
                    var entry = stat.getAsJsonArray();
                    var type = net.minecraft.core.registries.BuiltInRegistries.STAT_TYPE.getValue(
                        net.minecraft.resources.Identifier.parse(entry.get(0).getAsString()));
                    var value = ((net.minecraft.core.Registry<Object>) type.getRegistry()).getValue(
                        net.minecraft.resources.Identifier.parse(entry.get(1).getAsString()));
                    counter.setValue(null, ((net.minecraft.stats.StatType<Object>) type).get(value), entry.get(2).getAsInt());
                }
                var stats = new net.minecraft.client.gui.screens.achievement.StatsScreen(null, counter);
                stats.init(width, height);
                if (row.get("loaded").getAsBoolean()) {
                    stats.onStatsUpdated();
                }
                for (var step : row.getAsJsonArray("steps")) {
                    var action = step.getAsJsonObject();
                    double x = action.has("x") ? action.get("x").getAsDouble() : 0.0;
                    double y = action.has("y") ? action.get("y").getAsDouble() : 0.0;
                    switch (action.get("do").getAsString()) {
                        case "click" -> {
                            var event = new net.minecraft.client.input.MouseButtonEvent(x, y,
                                new net.minecraft.client.input.MouseButtonInfo(0, 0));
                            stats.mouseClicked(event, false);
                            stats.mouseReleased(event);
                        }
                        case "scroll" -> stats.mouseScrolled(x, y, 0.0, action.get("delta").getAsDouble());
                        case "key" -> {
                            // `KeyboardHandler.keyPress` notes what kind of key it was.
                            int code = action.get("key").getAsInt();
                            if (code == 258) {
                                minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_TAB);
                            } else if (code >= 262 && code <= 265) {
                                minecraft.setLastInputType(net.minecraft.client.InputType.KEYBOARD_ARROW);
                            }
                            stats.keyPressed(new net.minecraft.client.input.KeyEvent(
                                code, 0, action.get("modifiers").getAsInt()));
                        }
                        case "resize" -> {
                            // The window is the new size too: the scissors clamp to it.
                            MovementOracle.setField(minecraft.getWindow(), "guiScaledWidth", action.get("width").getAsInt());
                            MovementOracle.setField(minecraft.getWindow(), "guiScaledHeight", action.get("height").getAsInt());
                            stats.init(action.get("width").getAsInt(), action.get("height").getAsInt());
                        }
                        case "wait" -> now[0] += action.get("ms").getAsLong();
                        case "frame" -> {
                            // A frame drawn and thrown away: the header's buttons take their places in one.
                            var thrown = new GuiGraphicsExtractor(minecraft, new GuiRenderState(), mouseX, mouseY);
                            stats.extractRenderStateWithTooltipAndSubtitles(thrown, mouseX, mouseY, 0.0F);
                        }
                        default -> throw new IllegalArgumentException(action.get("do").getAsString());
                    }
                }
                var frame = new GuiRenderState();
                var graphics = new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY);
                stats.extractRenderStateWithTooltipAndSubtitles(graphics, mouseX, mouseY, 0.0F);
                var out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.add("ops", record(frame));
                var asked = new JsonArray();
                GuiScreenOracle.SilentSounds.ASKED.forEach(asked::add);
                out.add("sounds", asked);
                rows.add(out);
            } finally {
                net.minecraft.locale.Language.inject(languageBefore);
                Access26_2.setClock(clockBefore);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own `ClientAdvancements` over a script of updates built from JSON: after each step the roots and
     * the other advancements in order, each node's parent and children, each progress (the percent as bits, the
     * text, done, started), the tab selected and the toasts the step queued.
     */
    @SuppressWarnings("unchecked")
    static JsonObject clientAdvancements(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(427, 240, false, false);
            var minecraft = scene.minecraft();
            var toasts = new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options);
            MovementOracle.setField(minecraft.gui, "toastManager", toasts);
            var advancements = new net.minecraft.client.multiplayer.ClientAdvancements(minecraft, null);
            var languageBefore = net.minecraft.locale.Language.getInstance();
            net.minecraft.locale.Language.inject(clientLanguage("en_us"));
            var states = new JsonArray();
            try {
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    Access26_2.queuedToasts(toasts).clear();
                    if (step.has("tab")) {
                        var tab = step.get("tab");
                        advancements.setSelectedTab(tab.isJsonNull() ? null : advancements.get(
                            net.minecraft.resources.Identifier.parse(tab.getAsString())), false);
                    } else {
                        advancements.update(updatePacket(step));
                    }
                    var state = new JsonObject();
                    var tree = advancements.getTree();
                    var roots = new JsonArray();
                    tree.roots().forEach(node -> roots.add(node.holder().id().toString()));
                    state.add("roots", roots);
                    var nodes = new java.util.ArrayList<>(tree.nodes());
                    nodes.sort(java.util.Comparator.comparing(node -> node.holder().id().toString()));
                    var nodeList = new JsonArray();
                    var progressList = new JsonArray();
                    for (var node : nodes) {
                        var one = new JsonObject();
                        one.addProperty("id", node.holder().id().toString());
                        one.add("parent", node.parent() == null ? com.google.gson.JsonNull.INSTANCE
                            : new com.google.gson.JsonPrimitive(node.parent().holder().id().toString()));
                        var children = new java.util.ArrayList<String>();
                        node.children().forEach(child -> children.add(child.holder().id().toString()));
                        java.util.Collections.sort(children);
                        var childList = new JsonArray();
                        children.forEach(childList::add);
                        one.add("children", childList);
                        nodeList.add(one);
                    }
                    state.add("nodes", nodeList);
                    var tasks = new JsonArray();
                    for (var task : Access26_2.tasks(tree)) {
                        tasks.add(task.holder().id().toString());
                    }
                    state.add("tasks", tasks);
                    var held = Access26_2.heldProgress(advancements);
                    var heldIds = new java.util.ArrayList<net.minecraft.advancements.AdvancementHolder>(held.keySet());
                    heldIds.sort(java.util.Comparator.comparing(holder -> holder.id().toString()));
                    for (var holder : heldIds) {
                        var progressOne = held.get(holder);
                        var one = new JsonObject();
                        one.addProperty("id", holder.id().toString());
                        one.addProperty("percent", Oracle.bits(progressOne.getPercent()));
                        var text = progressOne.getProgressText();
                        one.add("text", text == null ? com.google.gson.JsonNull.INSTANCE : new com.google.gson.JsonPrimitive(text.getString()));
                        one.addProperty("done", progressOne.isDone());
                        one.addProperty("started", progressOne.hasProgress());
                        var remaining = new java.util.ArrayList<String>();
                        progressOne.getRemainingCriteria().forEach(remaining::add);
                        java.util.Collections.sort(remaining);
                        var completed = new java.util.ArrayList<String>();
                        progressOne.getCompletedCriteria().forEach(completed::add);
                        java.util.Collections.sort(completed);
                        var remainingList = new JsonArray();
                        remaining.forEach(remainingList::add);
                        var completedList = new JsonArray();
                        completed.forEach(completedList::add);
                        one.add("remaining", remainingList);
                        one.add("completed", completedList);
                        progressList.add(one);
                    }
                    state.add("progress", progressList);
                    var selected = advancementsSelected(advancements);
                    state.add("selected", selected == null ? com.google.gson.JsonNull.INSTANCE : new com.google.gson.JsonPrimitive(selected));
                    var queuedIds = new java.util.ArrayList<String>();
                    for (var toast : Access26_2.queuedToasts(toasts)) {
                        queuedIds.add(Access26_2.toastAdvancement(toast).id().toString());
                    }
                    java.util.Collections.sort(queuedIds);
                    var queuedList = new JsonArray();
                    queuedIds.forEach(queuedList::add);
                    state.add("toasts", queuedList);
                    states.add(state);
                }
            } finally {
                net.minecraft.locale.Language.inject(languageBefore);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("states", states);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own `AdvancementsScreen` over a real `ClientAdvancements`: updates arriving, frames, clicks,
     * drags, scrolls, ticks with the pointer somewhere, resizes; drawn in a world.
     */
    static JsonObject advancementsScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var mouse = row.getAsJsonArray("mouse");
            int mouseX = mouse.get(0).getAsInt();
            int mouseY = mouse.get(1).getAsInt();
            var scene = hudScene(width, height, false, false);
            var minecraft = scene.minecraft();
            minecraft.setLastInputType(net.minecraft.client.InputType.MOUSE);
            MovementOracle.setField(minecraft, "soundManager", unsafe.allocateInstance(GuiScreenOracle.SilentSounds.class));
            GuiScreenOracle.SilentSounds.ASKED.clear();
            var toasts = new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options);
            MovementOracle.setField(minecraft.gui, "toastManager", toasts);
            MovementOracle.setField(minecraft, "itemModelResolver", unsafe.allocateInstance(GuiSettingsOracle.NoItems.class));
            var connection = new net.minecraft.network.Connection(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
            MovementOracle.setField(scene.local().connection, "connection", connection);
            var window = minecraft.getWindow();
            MovementOracle.setField(window, "width", width);
            MovementOracle.setField(window, "height", height);
            var mouseHandler = unsafe.allocateInstance(net.minecraft.client.MouseHandler.class);
            MovementOracle.setField(minecraft, "mouseHandler", mouseHandler);
            var languageBefore = net.minecraft.locale.Language.getInstance();
            net.minecraft.locale.Language.inject(clientLanguage("en_us"));
            try {
                var advancements = new net.minecraft.client.multiplayer.ClientAdvancements(minecraft, null);
                for (var update : row.getAsJsonArray("before")) {
                    advancements.update(updatePacket(update.getAsJsonObject()));
                }
                var screen = new net.minecraft.client.gui.screens.advancements.AdvancementsScreen(advancements);
                MovementOracle.setField(screen, "minecraft", minecraft);
                screen.init(width, height);
                for (var stepElement : row.getAsJsonArray("steps")) {
                    var step = stepElement.getAsJsonObject();
                    double x = step.has("x") ? step.get("x").getAsDouble() : 0.0;
                    double y = step.has("y") ? step.get("y").getAsDouble() : 0.0;
                    switch (step.get("do").getAsString()) {
                        case "update" -> advancements.update(updatePacket(step));
                        case "tab" -> advancements.setSelectedTab(advancements.get(
                            net.minecraft.resources.Identifier.parse(step.get("id").getAsString())), false);
                        case "click" -> {
                            var event = new net.minecraft.client.input.MouseButtonEvent(x, y,
                                new net.minecraft.client.input.MouseButtonInfo(0, 0));
                            try {
                                screen.mouseClicked(event, false);
                            } catch (NullPointerException ignored) {
                                // Done sounds, then closes the screen, which needs a live game.
                            }
                            screen.mouseReleased(event);
                        }
                        case "drag" -> {
                            var event = new net.minecraft.client.input.MouseButtonEvent(x, y,
                                new net.minecraft.client.input.MouseButtonInfo(0, 0));
                            screen.mouseDragged(event, step.get("dx").getAsDouble(), step.get("dy").getAsDouble());
                        }
                        case "release" -> screen.mouseReleased(new net.minecraft.client.input.MouseButtonEvent(x, y,
                            new net.minecraft.client.input.MouseButtonInfo(0, 0)));
                        case "scroll" -> screen.mouseScrolled(x, y, step.get("sx").getAsDouble(), step.get("sy").getAsDouble());
                        case "tick" -> {
                            MovementOracle.setField(mouseHandler, "xpos", x);
                            MovementOracle.setField(mouseHandler, "ypos", y);
                            screen.tick();
                        }
                        case "resize" -> {
                            MovementOracle.setField(window, "guiScaledWidth", step.get("width").getAsInt());
                            MovementOracle.setField(window, "guiScaledHeight", step.get("height").getAsInt());
                            MovementOracle.setField(window, "width", step.get("width").getAsInt());
                            MovementOracle.setField(window, "height", step.get("height").getAsInt());
                            screen.init(step.get("width").getAsInt(), step.get("height").getAsInt());
                        }
                        case "frame" -> {
                            MovementOracle.setField(minecraft, "level", scene.level());
                            screen.extractRenderStateWithTooltipAndSubtitles(
                                new GuiGraphicsExtractor(minecraft, new GuiRenderState(), mouseX, mouseY), mouseX, mouseY, 0.0F);
                            MovementOracle.setField(minecraft, "level", null);
                        }
                        default -> throw new IllegalArgumentException(step.get("do").getAsString());
                    }
                }
                MovementOracle.setField(minecraft, "level", scene.level());
                var frame = new GuiRenderState();
                var graphics = new GuiGraphicsExtractor(minecraft, frame, mouseX, mouseY);
                screen.extractRenderStateWithTooltipAndSubtitles(graphics, mouseX, mouseY, 0.0F);
                var out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.add("ops", record(frame));
                var asked = new JsonArray();
                GuiScreenOracle.SilentSounds.ASKED.forEach(asked::add);
                out.add("sounds", asked);
                rows.add(out);
            } finally {
                net.minecraft.locale.Language.inject(languageBefore);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** An update packet from its JSON: the flags, the advancements, the ids removed and the progress. */
    @SuppressWarnings("unchecked")
    static net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket updatePacket(JsonObject step) throws Exception {
        var holders = new java.util.ArrayList<net.minecraft.advancements.AdvancementHolder>();
        for (var added : step.getAsJsonArray("added")) {
            holders.add(advancementHolder(added.getAsJsonObject()));
        }
        var removed = new java.util.LinkedHashSet<net.minecraft.resources.Identifier>();
        for (var id : step.getAsJsonArray("removed")) {
            removed.add(net.minecraft.resources.Identifier.parse(id.getAsString()));
        }
        var progress = new java.util.HashMap<net.minecraft.resources.Identifier, net.minecraft.advancements.AdvancementProgress>();
        for (var progressElement : step.getAsJsonArray("progress")) {
            var entry = progressElement.getAsJsonObject();
            var one = new net.minecraft.advancements.AdvancementProgress();
            var criteria = Access26_2.criteria(one);
            for (var criterion : entry.getAsJsonArray("criteria")) {
                var pair = criterion.getAsJsonArray();
                var obtained = new net.minecraft.advancements.CriterionProgress();
                if (!pair.get(1).isJsonNull()) {
                    Access26_2.obtained(obtained, java.time.Instant.ofEpochMilli(pair.get(1).getAsLong()));
                }
                criteria.put(pair.get(0).getAsString(), obtained);
            }
            progress.put(net.minecraft.resources.Identifier.parse(entry.get("id").getAsString()), one);
        }
        return new net.minecraft.network.protocol.game.ClientboundUpdateAdvancementsPacket(
            step.get("reset").getAsBoolean(), holders, removed, progress, step.get("show").getAsBoolean());
    }

    private static String advancementsSelected(net.minecraft.client.multiplayer.ClientAdvancements advancements) throws Exception {
        var selected = Access26_2.selectedTab(advancements);
        return selected == null ? null : selected.id().toString();
    }

    /** An advancement from its JSON: the id, the parent, the requirements and the display. */
    static net.minecraft.advancements.AdvancementHolder advancementHolder(JsonObject json) {
        var requirements = new java.util.ArrayList<java.util.List<String>>();
        for (var group : json.getAsJsonArray("requirements")) {
            var names = new java.util.ArrayList<String>();
            group.getAsJsonArray().forEach(name -> names.add(name.getAsString()));
            requirements.add(names);
        }
        java.util.Optional<net.minecraft.advancements.DisplayInfo> display = java.util.Optional.empty();
        if (json.has("display") && !json.get("display").isJsonNull()) {
            var shown = json.getAsJsonObject("display");
            var info = new net.minecraft.advancements.DisplayInfo(
                new net.minecraft.world.item.ItemStackTemplate(net.minecraft.world.item.Items.STONE),
                net.minecraft.network.chat.Component.literal(shown.get("title").getAsString()),
                net.minecraft.network.chat.Component.literal(shown.get("description").getAsString()),
                shown.has("background") && !shown.get("background").isJsonNull()
                    ? java.util.Optional.of(new net.minecraft.core.ClientAsset.ResourceTexture(
                        net.minecraft.resources.Identifier.parse(shown.get("background").getAsString())))
                    : java.util.Optional.empty(),
                net.minecraft.advancements.AdvancementType.values()[shown.get("frame").getAsInt()],
                shown.get("show_toast").getAsBoolean(), false, shown.get("hidden").getAsBoolean());
            info.setLocation(Oracle.f32(shown, "x"), Oracle.f32(shown, "y"));
            display = java.util.Optional.of(info);
        }
        java.util.Optional<net.minecraft.resources.Identifier> parent = json.has("parent") && !json.get("parent").isJsonNull()
            ? java.util.Optional.of(net.minecraft.resources.Identifier.parse(json.get("parent").getAsString()))
            : java.util.Optional.empty();
        var advancement = new net.minecraft.advancements.Advancement(parent, display,
            net.minecraft.advancements.AdvancementRewards.EMPTY, java.util.Map.of(),
            new net.minecraft.advancements.AdvancementRequirements(requirements), false);
        return new net.minecraft.advancements.AdvancementHolder(
            net.minecraft.resources.Identifier.parse(json.get("id").getAsString()), advancement);
    }

    /** `StatFormatter` for numbers: each formatter of the game on each value. */
    static JsonObject statFormat(JsonObject request) {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            net.minecraft.stats.StatFormatter formatter = switch (row.get("formatter").getAsString()) {
                case "default" -> net.minecraft.stats.StatFormatter.DEFAULT;
                case "divide_by_ten" -> net.minecraft.stats.StatFormatter.DIVIDE_BY_TEN;
                case "distance" -> net.minecraft.stats.StatFormatter.DISTANCE;
                case "time" -> net.minecraft.stats.StatFormatter.TIME;
                default -> throw new IllegalArgumentException(row.get("formatter").getAsString());
            };
            var texts = new JsonArray();
            for (var value : row.getAsJsonArray("values")) {
                texts.add(formatter.format(value.getAsInt()));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("texts", texts);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The debug screen's entries as the game keeps them (`DebugScreenEntries`, `DebugScreenEntryList`): what is
     * registered with its group and whether reduced debug info allows it, the profiles, and a real entry list
     * put through a row's operations, with its statuses, its enabled ids and the file it saved after each.
     */
    static JsonObject debugEntries(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(427, 240, false, false);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var all = new java.util.TreeMap<>(net.minecraft.client.gui.components.debug.DebugScreenEntries.allEntries());
            if (row.has("registry")) {
                var registry = new JsonArray();
                for (var entry : all.entrySet()) {
                    var described = new JsonObject();
                    described.addProperty("id", entry.getKey().getPath());
                    described.addProperty("sort_key", Oracle.bits(entry.getValue().category().sortKey()));
                    described.addProperty("category", ((net.minecraft.network.chat.contents.TranslatableContents)
                        entry.getValue().category().label().getContents()).getKey());
                    described.addProperty("allowed", entry.getValue().isAllowed(false));
                    described.addProperty("allowed_reduced", entry.getValue().isAllowed(true));
                    registry.add(described);
                }
                out.add("registry", registry);
                var profiles = new JsonObject();
                for (var profile : net.minecraft.client.gui.components.debug.DebugScreenProfile.values()) {
                    var statuses = new JsonObject();
                    new java.util.TreeMap<>(net.minecraft.client.gui.components.debug.DebugScreenEntries.PROFILES.get(profile))
                        .forEach((id, status) -> statuses.addProperty(id.getPath(), status.getSerializedName()));
                    profiles.add(profile.getSerializedName(), statuses);
                }
                out.add("profiles", profiles);
            }
            if (row.has("ops")) {
                var directory = java.nio.file.Files.createTempDirectory("debug-profile").toFile();
                var file = new java.io.File(directory, "debug-profile.json");
                var list = new net.minecraft.client.gui.components.debug.DebugScreenEntryList(directory,
                    net.minecraft.util.datafix.DataFixers.getDataFixer());
                var states = new JsonArray();
                states.add(debugEntriesState(list, all.keySet(), file));
                for (var opElement : row.getAsJsonArray("ops")) {
                    var op = opElement.getAsJsonObject();
                    var id = op.has("id") ? net.minecraft.resources.Identifier.withDefaultNamespace(op.get("id").getAsString()) : null;
                    switch (op.get("op").getAsString()) {
                        case "set" -> list.setStatus(id, net.minecraft.client.gui.components.debug.DebugScreenEntryStatus
                            .valueOf(op.get("status").getAsString()));
                        case "toggle" -> op.addProperty("result", list.toggleStatus(id));
                        case "profile" -> {
                            list.loadProfile(net.minecraft.client.gui.components.debug.DebugScreenProfile
                                .valueOf(op.get("profile").getAsString()));
                            list.save();
                        }
                        case "overlay" -> list.setOverlayVisible(op.get("visible").getAsBoolean());
                        case "reduced" -> {
                            MovementOracle.setField(scene.local(), "reducedDebugInfo", op.get("on").getAsBoolean());
                            list.rebuildCurrentList();
                        }
                        case "write" -> {
                            java.nio.file.Files.writeString(file.toPath(), op.get("text").getAsString());
                            list.load();
                        }
                        default -> throw new IllegalStateException(op.toString());
                    }
                    var state = debugEntriesState(list, all.keySet(), file);
                    if (op.has("result")) {
                        state.addProperty("result", op.get("result").getAsBoolean());
                    }
                    states.add(state);
                }
                out.add("states", states);
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    static JsonObject debugEntriesState(net.minecraft.client.gui.components.debug.DebugScreenEntryList list,
            java.util.Set<net.minecraft.resources.Identifier> ids, java.io.File file) throws Exception {
        var state = new JsonObject();
        var statuses = new JsonObject();
        for (var id : ids) {
            statuses.addProperty(id.getPath(), list.getStatus(id).getSerializedName());
        }
        state.add("statuses", statuses);
        var enabled = new JsonArray();
        list.getCurrentlyEnabled().forEach(id -> enabled.add(id.getPath()));
        state.add("enabled", enabled);
        for (var profile : net.minecraft.client.gui.components.debug.DebugScreenProfile.values()) {
            state.addProperty("using_" + profile.getSerializedName(), list.isUsingProfile(profile));
        }
        if (file.isFile()) {
            state.add("file", com.google.gson.JsonParser.parseString(java.nio.file.Files.readString(file.toPath())));
        }
        return state;
    }

    /**
     * The game's own `DebugScreenOverlay.extractLines` for lines given: the left column then the right,
     * as the overlay draws them, so the order of the backgrounds and the text and the right alignment show.
     */
    static JsonObject hudDebugLines(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), false, false);
            var overlay = new net.minecraft.client.gui.components.DebugScreenOverlay(scene.minecraft());
            // The game's own language reorders each string into pieces, as it does in the game.
            var languageBefore = net.minecraft.locale.Language.getInstance();
            net.minecraft.locale.Language.inject(clientLanguage("en_us"));
            try {
                for (var side : new String[] {"left", "right"}) {
                    var lines = new java.util.ArrayList<String>();
                    for (var line : row.getAsJsonArray(side)) {
                        lines.add(line.getAsString());
                    }
                    Access26_2.extractDebugLines(overlay, scene.screen().graphics(), lines, side.equals("left"));
                }
            } finally {
                net.minecraft.locale.Language.inject(languageBefore);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(scene.screen().state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own debug overlay (`DebugScreenOverlay.extractRenderState`) over a real local player: the
     * default profile's entry list as the game rebuilds it (the overlay up or not, reduced debug info or
     * not), the camera entity at a position and a rotation, and the interface hidden or shown.
     */
    static JsonObject hudDebugOverlay(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), false, false);
            var minecraft = scene.minecraft();
            var local = scene.local();
            MovementOracle.setField(minecraft, "gameLoadFinished", true);
            MovementOracle.setField(minecraft, "level", scene.level());
            MovementOracle.setField(minecraft, "launchedVersion", row.get("launched").getAsString());
            // The frame rate line: the count of the last second, the limit the tracker gives (a real tracker
            // over the options, with input just received) and the surface's configuration.
            Access26_2.fps(row.get("fps").getAsInt());
            minecraft.options.framerateLimit().set(row.get("limit").getAsInt());
            MovementOracle.setField(minecraft.getWindow(), "iconified", row.get("iconified").getAsBoolean());
            var tracker = new com.mojang.blaze3d.platform.FramerateLimitTracker(minecraft.options, minecraft);
            tracker.onInputReceived();
            MovementOracle.setField(minecraft, "framerateLimitTracker", tracker);
            var surface = (com.mojang.blaze3d.systems.GpuSurface) unsafe.allocateInstance(
                com.mojang.blaze3d.systems.GpuSurface.class);
            var present = row.get("present").getAsString();
            MovementOracle.setField(surface, "currentConfiguration", present.equals("none")
                ? java.util.Optional.empty()
                : java.util.Optional.of(new com.mojang.blaze3d.systems.GpuSurface.Configuration(854, 480,
                    com.mojang.blaze3d.systems.GpuSurface.PresentMode.valueOf(present))));
            MovementOracle.setField(minecraft, "windowSurface", surface);
            // The server line: the brand the server told, what the connection counted and the tick rate.
            var connection = new net.minecraft.network.Connection(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
            MovementOracle.setField(connection, "averageSentPackets", Oracle.f32(row, "sent"));
            MovementOracle.setField(connection, "averageReceivedPackets", Oracle.f32(row, "received"));
            MovementOracle.setField(local.connection, "connection", connection);
            MovementOracle.setField(local.connection, "serverBrand",
                row.get("brand").isJsonNull() ? null : row.get("brand").getAsString());
            scene.level().tickRateManager().setFrozen(row.get("frozen").getAsBoolean());
            scene.level().tickRateManager().setFrozenTicksToRun(row.get("stepping").getAsBoolean() ? 3 : 0);
            MovementOracle.setField(local, "reducedDebugInfo", row.get("reduced").getAsBoolean());
            local.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
                Oracle.f32(row, "yaw"), Oracle.f32(row, "pitch"));
            var entries = (net.minecraft.client.gui.components.debug.DebugScreenEntryList) unsafe.allocateInstance(
                net.minecraft.client.gui.components.debug.DebugScreenEntryList.class);
            // Only the entries that take nothing from a running game: the others (the frame rate, the
            // memory) read state the headless scene has none of.
            var statuses = new java.util.HashMap<>(
                net.minecraft.client.gui.components.debug.DebugScreenEntries.PROFILES.get(
                    net.minecraft.client.gui.components.debug.DebugScreenProfile.DEFAULT));
            var kept = new java.util.HashSet<>(java.util.Set.of("fps", "tps", "game_version", "player_position",
                "player_section_position", "3d_crosshair"));
            // The device the entries ask for: its limits for the anisotropy, its description for the specs.
            int anisotropyLimit = 0;
            if (row.has("impactors")) {
                // The options that entry reads.
                kept.add("simple_performance_impactors");
                var impactors = row.getAsJsonObject("impactors");
                var options = minecraft.options;
                options.improvedTransparency().set(impactors.get("transparency").getAsBoolean());
                options.cloudStatus().set(net.minecraft.client.CloudStatus.values()[impactors.get("clouds").getAsInt()]);
                options.biomeBlendRadius().set(impactors.get("blend").getAsInt());
                options.textureFiltering().set(
                    net.minecraft.client.TextureFilteringMethod.values()[impactors.get("filtering").getAsInt()]);
                options.maxAnisotropyBit().set(impactors.get("bit").getAsInt());
                anisotropyLimit = impactors.get("limit").getAsInt();
            }
            // What the system entry reads from the process, put back after the row.
            var cpuBefore = Access26_2.cpuInfo();
            var javaVersionBefore = System.getProperty("java.version");
            String deviceName = null, vendor = null, driver = null, backendName = null;
            var deviceType = com.mojang.blaze3d.systems.DeviceType.OTHER;
            if (row.has("system")) {
                // The machine that entry names: the runtime, the processor, the window and the device.
                kept.add("system_specs");
                var system = row.getAsJsonObject("system");
                System.setProperty("java.version", system.get("runtime").getAsString());
                Access26_2.setCpuInfo(system.get("cpu").getAsString());
                MovementOracle.setField(minecraft.getWindow(), "framebufferWidth", system.get("width").getAsInt());
                MovementOracle.setField(minecraft.getWindow(), "framebufferHeight", system.get("height").getAsInt());
                deviceName = system.get("name").getAsString();
                vendor = system.get("vendor").getAsString();
                driver = system.get("driver").getAsString();
                backendName = system.get("backend").getAsString();
                deviceType = com.mojang.blaze3d.systems.DeviceType.values()[system.get("type").getAsInt()];
            }
            if (row.has("impactors") || row.has("system")) {
                var limits = new com.mojang.blaze3d.systems.DeviceLimits(anisotropyLimit, 0, 0, 0L, 0, 0);
                var deviceInfo = new com.mojang.blaze3d.systems.DeviceInfo(deviceName, vendor, driver, true,
                    backendName, 0.0F, limits, null, null, null, deviceType);
                var backend = (com.mojang.blaze3d.systems.GpuDeviceBackend) java.lang.reflect.Proxy.newProxyInstance(
                    GuiOracle.class.getClassLoader(), new Class<?>[] {com.mojang.blaze3d.systems.GpuDeviceBackend.class},
                    (proxy, method, args) -> method.getName().equals("getDeviceInfo") ? deviceInfo : null);
                Access26_2.installDevice(backend);
            }
            // The entries a row turns on besides the defaults.
            if (row.has("extra")) {
                for (var extra : row.getAsJsonArray("extra")) {
                    var id = net.minecraft.resources.Identifier.withDefaultNamespace(extra.getAsString());
                    kept.add(id.getPath());
                    statuses.put(id, net.minecraft.client.gui.components.debug.DebugScreenEntryStatus.IN_OVERLAY);
                }
            }
            if (row.has("gpu")) {
                // The share of the frame the GPU was busy, as the frame loop last measured it.
                kept.add("gpu_utilization");
                MovementOracle.setField(minecraft, "gpuUtilization", Oracle.f64(row, "gpu"));
            }
            if (row.has("mood")) {
                // The sound library as it is with no device (empty pools), and the mood the player's
                // biome sounds handler has built up.
                var library = new com.mojang.blaze3d.audio.Library();
                if (row.has("channels")) {
                    // The pools of a device and the channels in use in them.
                    var channels = row.getAsJsonArray("channels");
                    library.setPools(channels.get(0).getAsInt(), channels.get(1).getAsInt());
                    for (int i = 0; i < channels.get(2).getAsInt(); i++) {
                        library.acquireChannel(com.mojang.blaze3d.audio.Library.Pool.STATIC);
                    }
                    for (int i = 0; i < channels.get(3).getAsInt(); i++) {
                        library.acquireChannel(com.mojang.blaze3d.audio.Library.Pool.STREAMING);
                    }
                }
                Access26_2.soundEngine(minecraft, library);
                for (var handler : (java.util.List<?>) MovementOracle.getField(local, "ambientSoundHandlers")) {
                    if (handler instanceof net.minecraft.client.resources.sounds.BiomeAmbientSoundsHandler) {
                        MovementOracle.setField(handler, "moodiness", Oracle.f32(row, "mood"));
                    }
                }
            }
            // The blocks the view meets, put in the level: the one target and any others.
            var placed = new java.util.ArrayList<JsonObject>();
            if (row.has("target")) {
                placed.add(row.getAsJsonObject("target"));
            }
            if (row.has("blocks")) {
                for (var each : row.getAsJsonArray("blocks")) {
                    placed.add(each.getAsJsonObject());
                }
            }
            for (var target : placed) {
                var parsed = net.minecraft.commands.arguments.blocks.BlockStateParser.parseForBlock(
                    scene.level().registryAccess().lookupOrThrow(net.minecraft.core.registries.Registries.BLOCK),
                    target.get("state").getAsString(), false);
                scene.level().setBlock(new net.minecraft.core.BlockPos(target.get("x").getAsInt(),
                    target.get("y").getAsInt(), target.get("z").getAsInt()), parsed.blockState(), 3);
            }
            if (row.has("entity")) {
                // The entity the crosshair is on.
                var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                    net.minecraft.resources.Identifier.parse(row.get("entity").getAsString()));
                MovementOracle.setField(minecraft, "crosshairPickEntity",
                    type.create(scene.level(), net.minecraft.world.entity.EntitySpawnReason.COMMAND));
            }
            if (row.has("sky")) {
                // The light at the feet: the section holds one value in each layer.
                var engine = scene.level().getChunkSource().getLightEngine();
                var section = net.minecraft.core.SectionPos.of(local.blockPosition());
                engine.updateSectionStatus(section, false);
                engine.queueSectionData(net.minecraft.world.level.LightLayer.SKY, section,
                    new net.minecraft.world.level.chunk.DataLayer(row.get("sky").getAsInt()));
                engine.queueSectionData(net.minecraft.world.level.LightLayer.BLOCK, section,
                    new net.minecraft.world.level.chunk.DataLayer(row.get("block").getAsInt()));
                engine.setLightEnabled(section.chunk(), true);
                engine.runLightUpdates();
            }
            if (row.has("day")) {
                // The overworld clock's total ticks, which the day count entry divides by the day's period.
                var clock = scene.level().registryAccess()
                    .get(net.minecraft.world.clock.WorldClocks.OVERWORLD).orElseThrow();
                scene.level().clockManager().handleUpdates(0L, java.util.Map.of(clock,
                    new net.minecraft.world.clock.ClockNetworkState(row.get("day").getAsLong(), 0.0F, 1.0F)));
            }
            statuses.keySet().removeIf(id -> !kept.contains(id.getPath()));
            MovementOracle.setField(entries, "allStatuses", statuses);
            MovementOracle.setField(entries, "currentlyEnabled", new java.util.ArrayList<>());
            MovementOracle.setField(entries, "isOverlayVisible", row.get("overlay").getAsBoolean());
            MovementOracle.setField(minecraft, "debugEntries", entries);
            entries.rebuildCurrentList();
            MovementOracle.setField(scene.screen().hud(), "isHidden", row.get("hidden").getAsBoolean());
            // The footer names the keys, which the game's own language gives.
            var languageBefore = net.minecraft.locale.Language.getInstance();
            net.minecraft.locale.Language.inject(clientLanguage("en_us", java.util.Map.of(
                "key.keyboard.1", "1", "key.keyboard.2", "2", "key.keyboard.3", "3", "key.keyboard.4", "4")));
            try {
                new net.minecraft.client.gui.components.DebugScreenOverlay(minecraft)
                    .extractRenderState(scene.screen().graphics());
            } finally {
                net.minecraft.locale.Language.inject(languageBefore);
                Access26_2.setCpuInfo(cpuBefore);
                System.setProperty("java.version", javaVersionBefore);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(scene.screen().state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own crosshair and hotbar (`Hud.extractCrosshair`, then `extractItemHotbar` for
     * a player who is not spectating) for a real local player: its attack strength, the attack
     * indicator option, the game mode, the camera and what the crosshair is on.
     */
    static JsonObject hudHotbar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            boolean spectator = row.get("spectator").getAsBoolean();
            var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), spectator, false);
            var minecraft = scene.minecraft();
            var level = scene.level();
            var local = scene.local();
            var options = minecraft.options;
            options.attackIndicator().set(net.minecraft.client.AttackIndicatorStatus.values()[
                row.get("attack_indicator").getAsInt()]);
            if (row.get("third_person").getAsBoolean()) {
                options.setCameraType(net.minecraft.client.CameraType.THIRD_PERSON_BACK);
            }
            local.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.ATTACK_SPEED)
                .setBaseValue(row.get("attack_speed").getAsDouble());
            MovementOracle.setField(local, "attackStrengthTicker", row.get("attack_ticks").getAsInt());
            local.getInventory().setSelectedSlot(row.get("selected_slot").getAsInt());

            if (row.has("debug")) {
                // The overlay up or not and reduced debug info or not, for the entries the game lists
                // as shown now: the 3D crosshair takes the 2D one's place.
                var debug = row.getAsJsonObject("debug");
                MovementOracle.setField(local, "reducedDebugInfo", debug.get("reduced").getAsBoolean());
                var entries = (net.minecraft.client.gui.components.debug.DebugScreenEntryList) unsafe.allocateInstance(
                    net.minecraft.client.gui.components.debug.DebugScreenEntryList.class);
                MovementOracle.setField(entries, "allStatuses", new java.util.HashMap<>(
                    net.minecraft.client.gui.components.debug.DebugScreenEntries.PROFILES.get(
                        net.minecraft.client.gui.components.debug.DebugScreenProfile.DEFAULT)));
                MovementOracle.setField(entries, "currentlyEnabled", new java.util.ArrayList<>());
                MovementOracle.setField(entries, "isOverlayVisible", debug.get("overlay").getAsBoolean());
                MovementOracle.setField(minecraft, "debugEntries", entries);
                entries.rebuildCurrentList();
            }

            // What the crosshair is on.
            net.minecraft.world.entity.Entity target = switch (row.get("target").getAsString()) {
                case "none" -> null;
                case "living" -> new net.minecraft.world.entity.animal.pig.Pig(
                    net.minecraft.world.entity.EntityTypes.PIG, level);
                case "dead" -> {
                    var pig = new net.minecraft.world.entity.animal.pig.Pig(
                        net.minecraft.world.entity.EntityTypes.PIG, level);
                    pig.setHealth(0.0F);
                    yield pig;
                }
                case "item" -> new net.minecraft.world.entity.item.ItemEntity(level, 0.0, 0.0, 0.0,
                    new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.STONE));
                default -> throw new IllegalArgumentException(row.get("target").getAsString());
            };
            MovementOracle.setField(minecraft, "crosshairPickEntity", target);

            var graphics = scene.screen().graphics();
            var hud = scene.screen().hud();
            var tracker = net.minecraft.client.DeltaTracker.ZERO;
            boolean vignette = row.has("vignette");
            if (row.has("frozen_ticks") || row.has("portal") || row.has("nausea") || row.has("equipment") || row.has("scope") || vignette) {
                // The camera overlays; the vignette only where the row asks for it.
                if (row.has("frozen_ticks")) {
                    local.setTicksFrozen(row.get("frozen_ticks").getAsInt());
                }
                if (row.has("scope")) {
                    var scope = row.getAsJsonObject("scope");
                    MovementOracle.setField(hud, "scopeScale", Oracle.f32(scope, "start"));
                    final float delta = scope.get("millis").getAsInt() / 50.0F;
                    tracker = (net.minecraft.client.DeltaTracker) java.lang.reflect.Proxy.newProxyInstance(
                        GuiHudOracle.class.getClassLoader(), new Class<?>[] {net.minecraft.client.DeltaTracker.class},
                        (proxy, method, arguments) -> method.getName().equals("getGameTimeDeltaTicks") ? (Object) delta : (Object) 0.0F);
                    if (scope.get("scoping").getAsBoolean()) {
                        MovementOracle.setField(local, "useItem",
                            new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SPYGLASS));
                        MovementOracle.setField(local, "startedUsingItem", true);
                    }
                }
                if (row.has("equipment")) {
                    for (var slot : row.getAsJsonObject("equipment").entrySet()) {
                        var item = net.minecraft.core.registries.BuiltInRegistries.ITEM
                            .get(net.minecraft.resources.Identifier.parse(slot.getValue().getAsString())).orElseThrow().value();
                        local.setItemSlot(net.minecraft.world.entity.EquipmentSlot.byName(slot.getKey()),
                            new net.minecraft.world.item.ItemStack(item));
                    }
                }
                if (row.has("nausea")) {
                    var nausea = row.getAsJsonObject("nausea");
                    options.screenEffectScale().set(nausea.get("scale").getAsDouble());
                    var holder = net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT
                        .get(net.minecraft.resources.Identifier.parse("minecraft:nausea")).orElseThrow();
                    var instance = new net.minecraft.world.effect.MobEffectInstance(holder, nausea.get("duration").getAsInt());
                    for (int tick = 0; tick < nausea.get("ticks").getAsInt(); tick++) {
                        instance.tickClient();
                    }
                    var active = (java.util.Map) MovementOracle.getField(local, "activeEffects");
                    active.put(holder, instance);
                }
                if (row.has("portal")) {
                    stubPortalSprite(minecraft);
                    local.portalEffectIntensity = (float) row.get("portal").getAsDouble();
                    local.oPortalEffectIntensity = local.portalEffectIntensity;
                }
                options.vignette().set(vignette);
                if (vignette) {
                    var setting = row.getAsJsonObject("vignette");
                    MovementOracle.setField(minecraft, "level", level);
                    ((net.minecraft.client.gui.Hud) hud).vignetteBrightness = Oracle.f32(setting, "brightness");
                    local.setPos(setting.get("x").getAsDouble(), 64.0, setting.get("z").getAsDouble());
                    var border = level.getWorldBorder();
                    border.setSize(setting.get("size").getAsDouble());
                    border.setWarningBlocks(setting.get("warning_blocks").getAsInt());
                    border.setWarningTime(setting.get("warning_time").getAsInt());
                    if (!setting.get("to").isJsonNull()) {
                        border.lerpSizeBetween(setting.get("size").getAsDouble(), setting.get("to").getAsDouble(),
                            setting.get("ticks").getAsLong(), 0L);
                    }
                }
                hudMethod("extractCameraOverlays", GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class)
                    .invoke(hud, graphics, tracker);
            }
            hudMethod("extractCrosshair", GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class)
                .invoke(hud, graphics, tracker);
            if (!spectator) {
                hudMethod("extractItemHotbar", GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class)
                    .invoke(hud, graphics, tracker);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(scene.screen().state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own status bars (`Hud.extractPlayerHealth`: armor, hearts, food and air bubbles, then the
     * experience bar and its level) for a real local player, drawn through frames: each sets the HUD's tick
     * count, the clock and the player's health and damage immunity, as the hearts blink by them.
     */
    /**
     * The local player rides a saddled horse with the health the row names, and its jump is charged
     * as far as the row says (`LocalPlayer.getJumpRidingScale`).
     */
    static void seatOnHorse(net.minecraft.client.multiplayer.ClientLevel level,
            net.minecraft.client.player.LocalPlayer local, JsonObject vehicle) throws Exception {
        var horse = new net.minecraft.world.entity.animal.equine.Horse(
            net.minecraft.world.entity.EntityTypes.HORSE, level);
        horse.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.MAX_HEALTH)
            .setBaseValue(vehicle.get("max_health").getAsDouble());
        horse.setHealth(Oracle.f32(vehicle, "health"));
        horse.setItemSlot(net.minecraft.world.entity.EquipmentSlot.SADDLE,
            new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SADDLE));
        MovementOracle.setField(local, "vehicle", horse);
        MovementOracle.setField(horse, "passengers", com.google.common.collect.ImmutableList.<net.minecraft.world.entity.Entity>of(local));
        MovementOracle.setField(local, "jumpRidingScale", Oracle.f32(vehicle, "jump"));
    }

    static JsonObject hudStatus(JsonObject request) throws Exception {
        var rows = new JsonArray();
        var clockBefore = Access26_2.clock();
        long[] now = {0};
        Access26_2.setClock((net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
        try {
            for (var element : request.getAsJsonArray("rows")) {
                var row = element.getAsJsonObject();
                var scene = hudScene(row.get("width").getAsInt(), row.get("height").getAsInt(), false,
                    row.get("hardcore").getAsBoolean());
                var minecraft = scene.minecraft();
                var local = scene.local();
                local.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.MAX_HEALTH)
                    .setBaseValue(row.get("max_health").getAsDouble());
                local.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.ARMOR)
                    .setBaseValue(row.get("armor").getAsDouble());
                // Absorption is held to the attribute's most.
                local.getAttribute(net.minecraft.world.entity.ai.attributes.Attributes.MAX_ABSORPTION)
                    .setBaseValue(1000.0);
                local.setAbsorptionAmount(Oracle.f32(row, "absorption"));
                local.getFoodData().setFoodLevel(row.get("food").getAsInt());
                local.getFoodData().setSaturation(Oracle.f32(row, "saturation"));
                local.setAirSupply(row.get("air").getAsInt());
                local.setTicksFrozen(row.get("frozen_ticks").getAsInt());
                local.experienceLevel = row.get("xp_level").getAsInt();
                local.experienceProgress = Oracle.f32(row, "xp_progress");
                for (var effect : row.getAsJsonArray("effects")) {
                    var holder = switch (effect.getAsString()) {
                        case "regeneration" -> net.minecraft.world.effect.MobEffects.REGENERATION;
                        case "poison" -> net.minecraft.world.effect.MobEffects.POISON;
                        case "wither" -> net.minecraft.world.effect.MobEffects.WITHER;
                        case "hunger" -> net.minecraft.world.effect.MobEffects.HUNGER;
                        default -> throw new IllegalArgumentException(effect.getAsString());
                    };
                    local.addEffect(new net.minecraft.world.effect.MobEffectInstance(holder, 1000));
                }
                if (row.get("eye_in_water").getAsBoolean()) {
                    var interaction = get(local, "fluidInteraction");
                    var trackers = (java.util.Map<?, ?>) get(interaction, "trackerByFluid");
                    MovementOracle.setField(trackers.get(net.minecraft.tags.FluidTags.WATER), "eyesInside", true);
                }
                var hud = scene.screen().hud();
                MovementOracle.setField(hud, "random", net.minecraft.util.RandomSource.create());
                var tracker = net.minecraft.client.DeltaTracker.ZERO;
                var health = hudMethod("extractPlayerHealth", GuiGraphicsExtractor.class);
                if (row.has("vehicle")) {
                    seatOnHorse(scene.level(), local, row.getAsJsonObject("vehicle"));
                }
                var vehicleHealth = hudMethod("extractVehicleHealth", GuiGraphicsExtractor.class);
                var nextBar = hudMethod("nextContextualInfoState");
                var infoType = Class.forName("net.minecraft.client.gui.Hud$ContextualInfo");
                var barMap = new java.util.HashMap<Object, java.util.function.Supplier<?>>();
                for (var constant : infoType.getEnumConstants()) {
                    barMap.put(constant, switch (constant.toString()) {
                        case "EXPERIENCE" -> () -> new net.minecraft.client.gui.contextualbar.ExperienceBar(minecraft);
                        case "LOCATOR" -> () -> new net.minecraft.client.gui.contextualbar.LocatorBar(minecraft);
                        case "JUMPABLE_VEHICLE" ->
                            () -> new net.minecraft.client.gui.contextualbar.JumpableVehicleBar(minecraft);
                        default -> () -> net.minecraft.client.gui.contextualbar.ContextualBar.EMPTY;
                    });
                }
                MovementOracle.setField(hud, "contextualInfoBars", barMap);
                var connection = get(local, "connection");
                if (connection != null && get(connection, "waypointManager") == null) {
                    MovementOracle.setField(connection, "waypointManager",
                        new net.minecraft.client.waypoints.ClientWaypointManager());
                }
                var frames = new JsonArray();
                for (var frameElement : row.getAsJsonArray("frames")) {
                    var frame = frameElement.getAsJsonObject();
                    MovementOracle.setField(hud, "tickCount", frame.get("tick").getAsInt());
                    now[0] = frame.get("ms").getAsLong();
                    local.setHealth(Oracle.f32(frame, "health"));
                    MovementOracle.setField(local, "invulnerableTime", frame.get("invulnerable").getAsInt());
                    var state = new GuiRenderState();
                    var graphics = new GuiGraphicsExtractor(minecraft, state, 0, 0);
                    health.invoke(hud, graphics);
                    vehicleHealth.invoke(hud, graphics);
                    var bars = (java.util.Map<?, ?>) get(hud, "contextualInfoBars");
                    var bar = (net.minecraft.client.gui.contextualbar.ContextualBar)
                        ((java.util.function.Supplier<?>) bars.get(nextBar.invoke(hud))).get();
                    bar.extractBackground(graphics, tracker);
                    if (local.experienceLevel > 0) {
                        Access26_2.extractExperienceLevel(graphics, minecraft.font, local.experienceLevel);
                    }
                    bar.extractRenderState(graphics, tracker);
                    frames.add(record(state));
                }
                var out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.add("frames", frames);
                rows.add(out);
            }
        } finally {
            Access26_2.setClock(clockBefore);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
