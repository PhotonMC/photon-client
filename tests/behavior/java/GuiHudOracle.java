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
            var listener = unsafe.allocateInstance(com.mojang.blaze3d.audio.Listener.class);
            var vec = (java.util.function.Function<com.google.gson.JsonArray, net.minecraft.world.phys.Vec3>) array ->
                new net.minecraft.world.phys.Vec3(array.get(0).getAsDouble(), array.get(1).getAsDouble(),
                    array.get(2).getAsDouble());
            set(listener, "transform", new com.mojang.blaze3d.audio.ListenerTransform(
                vec.apply(row.getAsJsonArray("position")), vec.apply(row.getAsJsonArray("forward")),
                vec.apply(row.getAsJsonArray("up"))));
            var engine = unsafe.allocateInstance(net.minecraft.client.sounds.SoundEngine.class);
            set(engine, "listener", listener);
            var manager = unsafe.allocateInstance(net.minecraft.client.sounds.SoundManager.class);
            set(manager, "soundEngine", engine);
            set(minecraft, "soundManager", manager);
            minecraft.options.showSubtitles().set(true);
            minecraft.options.notificationDisplayTime().set(row.get("display_time").getAsDouble());
            var overlay = new net.minecraft.client.gui.components.SubtitleOverlay(minecraft);
            set(overlay, "isListening", true);
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
            set(sounds, "played", new java.util.ArrayList<String>());
            set(minecraft, "soundManager", sounds);
            // The options' own change callbacks reach for the game's toast manager: set the values
            // under them.
            var musicState = net.minecraft.client.MusicToastDisplayState.valueOf(
                row.get("music_toast").getAsString());
            set(minecraft.options.musicToast(), "value", musicState);
            set(minecraft.options.notificationDisplayTime(), "value", row.get("display_time").getAsDouble());
            // The song that plays, as the toast asks the music manager for its key.
            var music = unsafe.allocateInstance(net.minecraft.client.sounds.MusicManager.class);
            if (row.has("song")) {
                var sound = unsafe.allocateInstance(net.minecraft.client.resources.sounds.Sound.class);
                set(sound, "location", net.minecraft.resources.Identifier.parse(row.get("song").getAsString()));
                var instance = (net.minecraft.client.resources.sounds.SoundInstance) java.lang.reflect.Proxy.newProxyInstance(
                    Oracle.class.getClassLoader(),
                    new Class<?>[] {net.minecraft.client.resources.sounds.SoundInstance.class},
                    (proxy, method, args) -> method.getName().equals("getSound") ? sound : null);
                set(music, "currentMusic", instance);
            }
            set(minecraft, "musicManager", music);
            set(screen.hud(), "isHidden", row.get("hud_hidden").getAsBoolean());
            if (row.get("pause_screen").getAsBoolean()) {
                set(screen.minecraft().gui, "screen", unsafe.allocateInstance(net.minecraft.client.gui.screens.PauseScreen.class));
            }
            // The music notes' colour waits on the wall clock: held where the row says.
            var noteClass = net.minecraft.client.gui.components.toasts.NowPlayingToast.class;
            var tick = noteClass.getDeclaredField("musicNoteColorTick");
            tick.setAccessible(true);
            tick.setInt(null, 0);
            var last = noteClass.getDeclaredField("lastMusicNoteColorChange");
            last.setAccessible(true);
            last.setLong(null, Long.MAX_VALUE / 2);
            var colour = noteClass.getDeclaredField("musicNoteColor");
            colour.setAccessible(true);
            colour.setInt(null, -1);
            var manager = new net.minecraft.client.gui.components.toasts.ToastManager(minecraft, minecraft.options);
            var events = new JsonArray();
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
        var device = unsafe.allocateInstance(com.mojang.blaze3d.systems.GpuDevice.class);
        set(device, "backend", backend);
        var deviceField = com.mojang.blaze3d.systems.RenderSystem.class.getDeclaredField("DEVICE");
        deviceField.setAccessible(true);
        deviceField.set(null, device);
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
            var fresh = net.minecraft.client.Camera.class.getDeclaredConstructor();
            fresh.setAccessible(true);
            var cameraObject = fresh.newInstance();
            var setRotation = net.minecraft.client.Camera.class.getDeclaredMethod("setRotation", float.class,
                float.class);
            setRotation.setAccessible(true);
            setRotation.invoke(cameraObject, (float) camera.get("yaw").getAsDouble(),
                (float) camera.get("pitch").getAsDouble());
            var setPosition = net.minecraft.client.Camera.class.getDeclaredMethod("setPosition",
                net.minecraft.world.phys.Vec3.class);
            setPosition.setAccessible(true);
            setPosition.invoke(cameraObject, new net.minecraft.world.phys.Vec3(at.get(0).getAsDouble(),
                at.get(1).getAsDouble(), at.get(2).getAsDouble()));
            float fov = (float) camera.get("fov").getAsDouble();
            set(cameraObject, "fov", fov);
            var projection = (net.minecraft.client.renderer.Projection) get(cameraObject, "projection");
            projection.setupPerspective(0.05F, (float) camera.get("far").getAsDouble(), fov,
                (float) camera.get("aspect").getAsDouble(), 1.0F);
            var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
            set(renderer, "mainCamera", cameraObject);
            set(minecraft, "gameRenderer", renderer);

            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "tickRateManager", new net.minecraft.world.TickRateManager());
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            var position = new net.minecraft.world.phys.Vec3(feet.get(0).getAsDouble(), feet.get(1).getAsDouble(),
                feet.get(2).getAsDouble());
            set(player, "level", level);
            set(player, "position", position);
            set(player, "blockPosition", net.minecraft.core.BlockPos.containing(position));
            set(player, "uuid", new java.util.UUID(uuid.get(0).getAsLong(), uuid.get(1).getAsLong()));
            var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            var manager = new net.minecraft.client.waypoints.ClientWaypointManager();
            set(listener, "waypointManager", manager);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            set(cameraObject, "entity", player);

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
            var styleManager = unsafe.allocateInstance(net.minecraft.client.resources.WaypointStyleManager.class);
            set(styleManager, "waypointStyles", styles);
            set(screen.hud(), "waypointStyles", styleManager);

            for (var packetElement : row.getAsJsonArray("packets")) {
                var hex = packetElement.getAsString();
                var bytes = new byte[hex.length() / 2];
                for (int i = 0; i < bytes.length; i++) {
                    bytes[i] = (byte) Integer.parseInt(hex.substring(2 * i, 2 * i + 2), 16);
                }
                var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
                    io.netty.buffer.Unpooled.wrappedBuffer(bytes), (net.minecraft.core.RegistryAccess) Oracle.vanillaRegistries());
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
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, mode);
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
        var method = net.minecraft.client.gui.Hud.class.getDeclaredMethod(name, types);
        method.setAccessible(true);
        return method;
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
            var method = net.minecraft.client.gui.components.DebugScreenOverlay.class.getDeclaredMethod(
                "extractLines", GuiGraphicsExtractor.class, java.util.List.class, boolean.class);
            method.setAccessible(true);
            for (var side : new String[] {"left", "right"}) {
                var lines = new java.util.ArrayList<String>();
                for (var line : row.getAsJsonArray(side)) {
                    lines.add(line.getAsString());
                }
                method.invoke(overlay, scene.screen().graphics(), lines, side.equals("left"));
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
            statuses.keySet().removeIf(id -> !java.util.Set.of("game_version", "player_position",
                "player_section_position", "3d_crosshair").contains(id.getPath()));
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
    static JsonObject hudStatus(JsonObject request) throws Exception {
        var rows = new JsonArray();
        var clock = net.minecraft.util.Util.class.getDeclaredField("timeSource");
        clock.setAccessible(true);
        var clockBefore = clock.get(null);
        long[] now = {0};
        clock.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
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
                var level = net.minecraft.client.gui.contextualbar.ContextualBar.class.getDeclaredMethod(
                    "extractExperienceLevel", GuiGraphicsExtractor.class, net.minecraft.client.gui.Font.class, int.class);
                level.setAccessible(true);
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
                    var bar = new net.minecraft.client.gui.contextualbar.ExperienceBar(minecraft);
                    bar.extractBackground(graphics, tracker);
                    if (local.experienceLevel > 0) {
                        level.invoke(null, graphics, minecraft.font, local.experienceLevel);
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
            clock.set(null, clockBefore);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
