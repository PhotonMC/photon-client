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
}
