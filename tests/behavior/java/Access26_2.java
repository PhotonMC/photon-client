/** The private names of the 26.2 game the oracle reaches; another version replaces this class. */
final class Access26_2 {
    private Access26_2() {}

    /** The music note toast's colour waits on the wall clock: hold it where a row says. */
    static void holdMusicNote() throws Exception {
        var noteClass = net.minecraft.client.gui.components.toasts.NowPlayingToast.class;
        Reflect.setStatic(noteClass, "musicNoteColorTick", 0);
        Reflect.setStatic(noteClass, "lastMusicNoteColorChange", Long.MAX_VALUE / 2);
        Reflect.setStatic(noteClass, "musicNoteColor", -1);
    }

    /** The sound manager answers only where the listener is: one that hears from `transform`. */
    static void listenFrom(net.minecraft.client.Minecraft minecraft,
            com.mojang.blaze3d.audio.ListenerTransform transform) throws Exception {
        var unsafe = Reflect.unsafe();
        var listener = unsafe.allocateInstance(com.mojang.blaze3d.audio.Listener.class);
        Reflect.set(listener, "transform", transform);
        var engine = unsafe.allocateInstance(net.minecraft.client.sounds.SoundEngine.class);
        Reflect.set(engine, "listener", listener);
        var manager = unsafe.allocateInstance(net.minecraft.client.sounds.SoundManager.class);
        Reflect.set(manager, "soundEngine", engine);
        Reflect.set(minecraft, "soundManager", manager);
    }

    static void listenForSubtitles(net.minecraft.client.gui.components.SubtitleOverlay overlay) throws Exception {
        Reflect.set(overlay, "isListening", true);
    }

    /** A sound manager that records the names it plays into `played`. */
    static void recordSounds(net.minecraft.client.Minecraft minecraft, Object sounds,
            java.util.List<String> played) throws Exception {
        Reflect.set(sounds, "played", played);
        Reflect.set(minecraft, "soundManager", sounds);
    }

    /** An item is not drawn: the resolver adds nothing. */
    static void drawNoItems(net.minecraft.client.Minecraft minecraft, Object noItems) throws Exception {
        Reflect.set(minecraft, "itemModelResolver", noItems);
    }

    /** An option's value, set under its change callback. */
    static void optionValue(Object option, Object value) throws Exception {
        Reflect.set(option, "value", value);
    }

    /** The song the music manager plays, as the toast asks it for its key. */
    static void playSong(Object music, net.minecraft.resources.Identifier location) throws Exception {
        var unsafe = Reflect.unsafe();
        var sound = unsafe.allocateInstance(net.minecraft.client.resources.sounds.Sound.class);
        Reflect.set(sound, "location", location);
        var instance = (net.minecraft.client.resources.sounds.SoundInstance) java.lang.reflect.Proxy.newProxyInstance(
            Oracle.class.getClassLoader(),
            new Class<?>[] {net.minecraft.client.resources.sounds.SoundInstance.class},
            (proxy, method, args) -> method.getName().equals("getSound") ? sound : null);
        Reflect.set(music, "currentMusic", instance);
    }

    static void useMusicManager(net.minecraft.client.Minecraft minecraft, Object music) throws Exception {
        Reflect.set(minecraft, "musicManager", music);
    }

    static void hideHud(Object hud, boolean hidden) throws Exception {
        Reflect.set(hud, "isHidden", hidden);
    }

    static void showPauseScreen(net.minecraft.client.gui.Gui gui) throws Exception {
        Reflect.set(gui, "screen",
            Reflect.unsafe().allocateInstance(net.minecraft.client.gui.screens.PauseScreen.class));
    }

    /** The recipe toast on screen (made if there is none) shows `station` and `result`. */
    static void showRecipe(net.minecraft.client.gui.components.toasts.ToastManager manager,
            net.minecraft.world.item.ItemStack station, net.minecraft.world.item.ItemStack result) throws Exception {
        var toastClass = net.minecraft.client.gui.components.toasts.RecipeToast.class;
        var toast = manager.getToast(toastClass, net.minecraft.client.gui.components.toasts.Toast.NO_TOKEN);
        if (toast == null) {
            toast = Reflect.construct(toastClass, new Class<?>[0]);
            manager.addToast(toast);
        }
        Reflect.invoke(toast, toastClass, "addItem",
            new Class<?>[] {net.minecraft.world.item.ItemStack.class, net.minecraft.world.item.ItemStack.class},
            station, result);
    }

    /** The index of the item the recipe toast shows. */
    static int recipeIndex(Object toast) throws Exception {
        return (Integer) Reflect.get(toast, "displayedRecipeIndex");
    }

    /** The render system's device, with `backend` as its backend. */
    static void installDevice(com.mojang.blaze3d.systems.GpuDeviceBackend backend) throws Exception {
        var device = Reflect.unsafe().allocateInstance(com.mojang.blaze3d.systems.GpuDevice.class);
        Reflect.set(device, "backend", backend);
        Reflect.setStatic(com.mojang.blaze3d.systems.RenderSystem.class, "DEVICE", device);
    }

    /** A camera turned and placed as given, with its field of view. */
    static net.minecraft.client.Camera camera(float yaw, float pitch,
            net.minecraft.world.phys.Vec3 position, float fov) throws Exception {
        var cameraClass = net.minecraft.client.Camera.class;
        var camera = Reflect.construct(cameraClass, new Class<?>[0]);
        Reflect.invoke(camera, cameraClass, "setRotation", new Class<?>[] {float.class, float.class}, yaw, pitch);
        Reflect.invoke(camera, cameraClass, "setPosition",
            new Class<?>[] {net.minecraft.world.phys.Vec3.class}, position);
        Reflect.set(camera, "fov", fov);
        return camera;
    }

    static net.minecraft.client.renderer.Projection projection(Object camera) throws Exception {
        return (net.minecraft.client.renderer.Projection) Reflect.get(camera, "projection");
    }

    /** The game renders through `camera`. */
    static void renderThrough(net.minecraft.client.Minecraft minecraft, Object camera) throws Exception {
        var renderer = Reflect.unsafe().allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        Reflect.set(renderer, "mainCamera", camera);
        Reflect.set(minecraft, "gameRenderer", renderer);
    }

    /** A local player at `position` in a level, whose connection tracks `waypoints`; the camera follows it. */
    static void localPlayer(net.minecraft.client.Minecraft minecraft, Object camera,
            net.minecraft.world.phys.Vec3 position, java.util.UUID uuid,
            net.minecraft.client.waypoints.ClientWaypointManager waypoints) throws Exception {
        var unsafe = Reflect.unsafe();
        var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
        Reflect.set(level, "tickRateManager", new net.minecraft.world.TickRateManager());
        var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
        Reflect.set(player, "level", level);
        Reflect.set(player, "position", position);
        Reflect.set(player, "blockPosition", net.minecraft.core.BlockPos.containing(position));
        Reflect.set(player, "uuid", uuid);
        var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
        Reflect.set(listener, "waypointManager", waypoints);
        Reflect.set(player, "connection", listener);
        Reflect.set(minecraft, "player", player);
        Reflect.set(camera, "entity", player);
    }

    /** The clock the game reads (`Util.timeSource`). */
    static Object clock() throws Exception {
        return Reflect.get(null, net.minecraft.util.Util.class, "timeSource");
    }

    static void setClock(Object source) throws Exception {
        Reflect.setStatic(net.minecraft.util.Util.class, "timeSource", source);
    }

    /** The toasts waiting to show. */
    @SuppressWarnings("unchecked")
    static java.util.Collection<Object> queuedToasts(Object manager) throws Exception {
        return (java.util.Collection<Object>) Reflect.get(manager, "queued");
    }

    static net.minecraft.advancements.AdvancementHolder toastAdvancement(Object toast) throws Exception {
        return (net.minecraft.advancements.AdvancementHolder) Reflect.get(toast, "advancement");
    }

    @SuppressWarnings("unchecked")
    static java.util.Map<String, net.minecraft.advancements.CriterionProgress> criteria(Object progress)
            throws Exception {
        return (java.util.Map<String, net.minecraft.advancements.CriterionProgress>) Reflect.get(progress, "criteria");
    }

    static void obtained(Object criterion, java.time.Instant at) throws Exception {
        Reflect.set(criterion, "obtained", at);
    }

    @SuppressWarnings("unchecked")
    static java.util.Collection<net.minecraft.advancements.AdvancementNode> tasks(Object tree) throws Exception {
        return (java.util.Collection<net.minecraft.advancements.AdvancementNode>) Reflect.get(tree, "tasks");
    }

    @SuppressWarnings("unchecked")
    static java.util.Map<net.minecraft.advancements.AdvancementHolder, net.minecraft.advancements.AdvancementProgress>
            heldProgress(Object advancements) throws Exception {
        return (java.util.Map<net.minecraft.advancements.AdvancementHolder,
            net.minecraft.advancements.AdvancementProgress>) Reflect.get(advancements, "progress");
    }

    static net.minecraft.advancements.AdvancementHolder selectedTab(Object advancements) throws Exception {
        return (net.minecraft.advancements.AdvancementHolder) Reflect.get(advancements, "selectedTab");
    }

    /** The frame rate the debug screen shows. */
    static void fps(int value) throws Exception {
        Reflect.setStatic(net.minecraft.client.Minecraft.class, "fps", value);
    }

    static Object cpuInfo() throws Exception {
        return Reflect.get(null, com.mojang.blaze3d.platform.GLX.class, "cpuInfo");
    }

    static void setCpuInfo(Object value) throws Exception {
        Reflect.setStatic(com.mojang.blaze3d.platform.GLX.class, "cpuInfo", value);
    }

    /** A private method of the `Hud`, by name. */
    static java.lang.reflect.Method hudMethod(String name, Class<?>... types) throws Exception {
        var method = net.minecraft.client.gui.Hud.class.getDeclaredMethod(name, types);
        method.setAccessible(true);
        return method;
    }

    /** `DebugScreenOverlay.extractLines`, drawn for one side. */
    static void extractDebugLines(Object overlay, Object graphics, java.util.List<?> lines, boolean left)
            throws Exception {
        Reflect.invoke(overlay, net.minecraft.client.gui.components.DebugScreenOverlay.class, "extractLines",
            new Class<?>[] {net.minecraft.client.gui.GuiGraphicsExtractor.class, java.util.List.class, boolean.class},
            graphics, lines, left);
    }

    /** `ContextualBar.extractExperienceLevel` (static). */
    static void extractExperienceLevel(Object graphics, Object font, int level) throws Exception {
        Reflect.invoke(null, net.minecraft.client.gui.contextualbar.ContextualBar.class, "extractExperienceLevel",
            new Class<?>[] {net.minecraft.client.gui.GuiGraphicsExtractor.class, net.minecraft.client.gui.Font.class,
                int.class}, graphics, font, level);
    }

    static void setGameMode(Object playerInfo, net.minecraft.world.level.GameType mode) throws Exception {
        Reflect.invoke(playerInfo, net.minecraft.client.multiplayer.PlayerInfo.class, "setGameMode",
            new Class<?>[] {net.minecraft.world.level.GameType.class}, mode);
    }

    /** The hud draws waypoints in `styles`. */
    static void waypointStyles(Object hud, Object styleManager, Object styles) throws Exception {
        Reflect.set(styleManager, "waypointStyles", styles);
        Reflect.set(hud, "waypointStyles", styleManager);
    }

    /** The sound engine plays through `library`; the manager runs on that engine. */
    static void soundEngine(net.minecraft.client.Minecraft minecraft, Object library) throws Exception {
        var unsafe = Reflect.unsafe();
        var engine = unsafe.allocateInstance(net.minecraft.client.sounds.SoundEngine.class);
        Reflect.set(engine, "library", library);
        var manager = unsafe.allocateInstance(net.minecraft.client.sounds.SoundManager.class);
        Reflect.set(manager, "soundEngine", engine);
        Reflect.set(minecraft, "soundManager", manager);
    }
}
