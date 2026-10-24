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
}
