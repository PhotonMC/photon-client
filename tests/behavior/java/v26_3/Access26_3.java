/** The private names of the 26.3 game the oracle reaches; another version replaces this class. */
final class Access26_3 {
    private Access26_3() {}

    /** A stack's component patch as entries: a value, or empty for a removed component. */
    static java.util.List<java.util.Map.Entry<net.minecraft.core.component.DataComponentType<?>, java.util.Optional<?>>>
        patchEntries(net.minecraft.core.component.DataComponentPatch patch) {
        var out = new java.util.ArrayList<java.util.Map.Entry<net.minecraft.core.component.DataComponentType<?>, java.util.Optional<?>>>();
        var split = patch.split();
        for (var typed : split.added()) {
            out.add(java.util.Map.entry(typed.type(), java.util.Optional.of(typed.value())));
        }
        for (var type : split.removed()) {
            out.add(java.util.Map.entry(type, java.util.Optional.empty()));
        }
        return out;
    }

    /** The ticks the current swing has run (the swing state keeps them private). */
    static int swingTicks(net.minecraft.world.entity.LivingEntity living) throws Exception {
        var state = Reflect.get(living, "swingState");
        return (Integer) Reflect.get(state, "ticks");
    }

    /**
     * What the pre-26.3 rows said about a swing (how far it is, which arm, which animation),
     * as the render state now carries it: a current swing on a hand, and its progress.
     */
    static void swing(
        net.minecraft.client.renderer.entity.state.ArmedEntityRenderState state,
        float progress,
        net.minecraft.world.entity.HumanoidArm attackArm,
        net.minecraft.world.item.SwingAnimationType type
    ) {
        var hand = attackArm == state.mainArm
            ? net.minecraft.world.InteractionHand.MAIN_HAND
            : net.minecraft.world.InteractionHand.OFF_HAND;
        state.currentSwing = new net.minecraft.world.entity.LivingEntity.SwingDescription(
            hand, new net.minecraft.world.item.component.SwingAnimation(type, 6), 6);
        state.swingAnimation = progress;
    }

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
    static void installDevice(com.mojang.renderpearl.backend.api.GpuDeviceBackend backend) throws Exception {
        // `GpuDevice` is an interface: a proxy that answers the device info from the backend stub.
        var device = java.lang.reflect.Proxy.newProxyInstance(Access26_3.class.getClassLoader(),
            new Class<?>[] {com.mojang.renderpearl.api.device.GpuDevice.class},
            (proxy, method, args) -> method.getName().equals("getDeviceInfo") ? backend.getDeviceInfo() : null);
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

    /** The nodes that are tasks: the tree now derives them from its nodes. */
    static Iterable<net.minecraft.advancements.AdvancementNode> tasks(Object tree) {
        return ((net.minecraft.advancements.AdvancementTree) tree).tasks();
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
        return Reflect.get(null, net.minecraft.client.gui.components.debug.DebugEntrySystemSpecs.class, "cpuInfo");
    }

    static void setCpuInfo(Object value) throws Exception {
        Reflect.setStatic(net.minecraft.client.gui.components.debug.DebugEntrySystemSpecs.class, "cpuInfo", value);
    }

    /** A private method of the `Hud`, by name. */
    static java.lang.reflect.Method hudMethod(String name, Class<?>... types) throws Exception {
        var method = Reflect.method(net.minecraft.client.gui.Hud.class, name, types);
        return method;
    }

    /** `DebugScreenOverlay.extractLines`, drawn for one side. */
    static void extractDebugLines(Object overlay, Object graphics, java.util.List<?> lines, boolean left)
            throws Exception {
        Reflect.invoke(overlay, net.minecraft.client.gui.components.DebugScreenOverlay.class, "extractLines",
            new Class<?>[] {net.minecraft.client.gui.GuiGraphicsExtractor.class, java.util.List.class, boolean.class,
                int.class},
            graphics, lines, left, ((net.minecraft.client.gui.GuiGraphicsExtractor) graphics).guiWidth());
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

/**
 * The input the 26.3 client takes is SDL's: keys are scancodes (with the key's SDL keycode beside
 * them), mouse buttons count from 1 and the modifier bits are SDL's. The cases name keys, buttons
 * and modifiers the way Photon holds them (GLFW's numbers), so every event the oracle makes goes
 * through here.
 */
final class SdlInput {
    private SdlInput() {}

    private static final int SDL_SHIFT = 1;
    private static final int SDL_CONTROL = 64;
    private static final int SDL_ALT = 256;
    private static final int SDL_SUPER = 1024;

    /** A key event for the key GLFW numbers `glfwKey`, with GLFW's modifier bits. */
    static net.minecraft.client.input.KeyEvent key(int glfwKey, int unused, int glfwModifiers) {
        return new net.minecraft.client.input.KeyEvent(scancode(glfwKey), keycode(glfwKey), modifiers(glfwModifiers));
    }

    /** A mouse button as SDL numbers it, with SDL's modifier bits. */
    static net.minecraft.client.input.MouseButtonInfo button(int glfwButton, int glfwModifiers) {
        return new net.minecraft.client.input.MouseButtonInfo(button(glfwButton), modifiers(glfwModifiers));
    }

    static int button(int glfwButton) {
        return switch (glfwButton) {
            case 0 -> 1;
            case 1 -> 3;
            case 2 -> 2;
            default -> glfwButton + 1;
        };
    }

    static int modifiers(int glfw) {
        int sdl = 0;
        if ((glfw & 1) != 0) {
            sdl |= SDL_SHIFT;
        }
        if ((glfw & 2) != 0) {
            sdl |= SDL_CONTROL;
        }
        if ((glfw & 4) != 0) {
            sdl |= SDL_ALT;
        }
        if ((glfw & 8) != 0) {
            sdl |= SDL_SUPER;
        }
        return sdl;
    }

    /** The scancode of the key GLFW numbers `key`; 0 for a key SDL has no scancode for. */
    static int scancode(int key) {
        if (key >= 65 && key <= 90) {
            return 4 + key - 65;
        }
        if (key >= 49 && key <= 57) {
            return 30 + key - 49;
        }
        if (key >= 290 && key <= 301) {
            return 58 + key - 290;
        }
        if (key >= 302 && key <= 313) {
            return 104 + key - 302;
        }
        if (key >= 321 && key <= 329) {
            return 89 + key - 321;
        }
        return switch (key) {
            case 48 -> 39;
            case 32 -> 44;
            case 39 -> 52;
            case 44 -> 54;
            case 45 -> 45;
            case 46 -> 55;
            case 47 -> 56;
            case 59 -> 51;
            case 61 -> 46;
            case 91 -> 47;
            case 92 -> 49;
            case 93 -> 48;
            case 96 -> 53;
            case 256 -> 41;
            case 257 -> 40;
            case 258 -> 43;
            case 259 -> 42;
            case 260 -> 73;
            case 261 -> 76;
            case 262 -> 79;
            case 263 -> 80;
            case 264 -> 81;
            case 265 -> 82;
            case 266 -> 75;
            case 267 -> 78;
            case 268 -> 74;
            case 269 -> 77;
            case 280 -> 57;
            case 281 -> 71;
            case 282 -> 83;
            case 283 -> 70;
            case 284 -> 72;
            case 320 -> 98;
            case 330 -> 99;
            case 331 -> 84;
            case 332 -> 85;
            case 333 -> 86;
            case 334 -> 87;
            case 335 -> 88;
            case 336 -> 103;
            case 340 -> 225;
            case 341 -> 224;
            case 342 -> 226;
            case 343 -> 227;
            case 344 -> 229;
            case 345 -> 228;
            case 346 -> 230;
            case 347 -> 231;
            case 348 -> 101;
            default -> 0;
        };
    }

    /** The SDL keycode of the key GLFW numbers `key` (letters are lower case). */
    static int keycode(int key) {
        if (key >= 65 && key <= 90) {
            return key + 32;
        }
        if ((key >= 48 && key <= 57) || key == 32 || key == 39 || key == 44 || key == 45 || key == 46
                || key == 47 || key == 59 || key == 61 || key == 91 || key == 92 || key == 93 || key == 96) {
            return key;
        }
        int scancode = scancode(key);
        return switch (key) {
            case 256 -> 27;
            case 257 -> 13;
            case 258 -> 9;
            case 259 -> 8;
            case 261 -> 127;
            case 335 -> 1073741912;
            case 348 -> 1073741925;
            default -> scancode == 0 ? 0 : 0x40000000 | scancode;
        };
    }
}
