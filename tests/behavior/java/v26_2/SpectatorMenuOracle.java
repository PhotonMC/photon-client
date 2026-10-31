import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * The spectator menu's paging and selection, on players made without a connection. The root
 * category and the menu are allocated without their constructors (those read the running
 * client), and given the player category the menu would build.
 */
final class SpectatorMenuOracle {
    private SpectatorMenuOracle() {
    }

    static JsonObject run(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "stepped");
        try {
            entry.add("steps", steps(row));
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
        return entry;
    }

    private static JsonArray steps(JsonObject row) throws ReflectiveOperationException {
        var unsafe = Reflect.unsafe();
        var players = new java.util.ArrayList<net.minecraft.client.multiplayer.PlayerInfo>();
        for (var element : row.getAsJsonArray("players")) {
            var spec = element.getAsJsonObject();
            var id = new java.util.UUID(
                Long.parseUnsignedLong(spec.get("hi").getAsString(), 16),
                Long.parseUnsignedLong(spec.get("lo").getAsString(), 16));
            var info = new net.minecraft.client.multiplayer.PlayerInfo(
                new com.mojang.authlib.GameProfile(id, spec.get("name").getAsString()), false);
            var setMode = Reflect.method(info.getClass(), "setGameMode", net.minecraft.world.level.GameType.class);
            setMode.invoke(info, net.minecraft.world.level.GameType.byId(spec.get("mode").getAsInt()));
            players.add(info);
        }
        var rootClass = net.minecraft.client.gui.spectator.RootSpectatorMenuCategory.class;
        var root = unsafe.allocateInstance(rootClass);
        var items = new java.util.ArrayList<net.minecraft.client.gui.spectator.SpectatorMenuItem>();
        items.add(new net.minecraft.client.gui.spectator.categories.TeleportToPlayerMenuCategory(players));
        // The team category reads the scoreboard when built; with no teams it lists nothing.
        var teamClass = net.minecraft.client.gui.spectator.categories.TeleportToTeamMenuCategory.class;
        var teams = unsafe.allocateInstance(teamClass);
        unsafe.putObject(teams, unsafe.objectFieldOffset(Reflect.field(teamClass, "items")), java.util.List.of());
        items.add((net.minecraft.client.gui.spectator.SpectatorMenuItem) teams);
        unsafe.putObject(root, unsafe.objectFieldOffset(Reflect.field(rootClass, "items")), items);
        var menuClass = net.minecraft.client.gui.spectator.SpectatorMenu.class;
        var menu = (net.minecraft.client.gui.spectator.SpectatorMenu) unsafe.allocateInstance(menuClass);
        unsafe.putObject(menu, unsafe.objectFieldOffset(Reflect.field(menuClass, "category")), root);
        unsafe.putInt(menu, unsafe.objectFieldOffset(Reflect.field(menuClass, "selectedSlot")), -1);
        var guiClass = net.minecraft.client.gui.components.spectator.SpectatorGui.class;
        var gui = unsafe.allocateInstance(guiClass);
        unsafe.putObject(gui, unsafe.objectFieldOffset(Reflect.field(guiClass, "menu")), menu);
        var scrolled = Reflect.method(guiClass, "onMouseScrolled", int.class);
        var result = new JsonArray();
        for (var element : row.getAsJsonArray("steps")) {
            var step = element.getAsString().split(":");
            var step_result = new JsonObject();
            var number = Integer.parseInt(step[1]);
            if (step[0].equals("select")) {
                var item = menu.getItem(number);
                var fires = item instanceof net.minecraft.client.gui.spectator.PlayerMenuItem
                    && menu.getSelectedSlot() == number && item.isEnabled();
                if (fires) {
                    step_result.addProperty("teleport", item.getName().getString());
                } else {
                    menu.selectSlot(number);
                }
            } else {
                scrolled.invoke(gui, number);
            }
            step_result.addProperty("selected", menu.getSelectedSlot());
            step_result.addProperty("prompt", key(menu.getSelectedCategory().getPrompt()));
            var slots = new JsonArray();
            for (var item : menu.getItems()) {
                var slot = new JsonObject();
                slot.addProperty("name", key(item.getName()));
                slot.addProperty("enabled", item.isEnabled());
                slots.add(slot);
            }
            step_result.add("items", slots);
            result.add(step_result);
        }
        return result;
    }

    /** A translation key, or a plain text. */
    private static String key(net.minecraft.network.chat.Component component) {
        var contents = component.getContents();
        if (contents instanceof net.minecraft.network.chat.contents.TranslatableContents translatable) {
            return translatable.getKey();
        }
        return component.getString();
    }
}
