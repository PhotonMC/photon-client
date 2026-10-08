import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/**
 * An item's tooltip lines: the game's own ItemStack.getTooltipLines for a
 * stack written as its item, count and wire component patch, with no player,
 * in the normal or advanced (F3+H) style and optionally as the creative
 * inventory's. A row may give the map data the client holds for the stack's
 * map id. Each line is its runs in the language: text and the style it ends
 * up in, neighbours of one style joined.
 */
final class GuiTooltipOracle {
    private GuiTooltipOracle() {}

    static JsonObject tooltipLines(JsonObject request) throws Exception {
        GuiFonts.setup();
        var registries = Oracle.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var stack = MovementSupport.stack(row);
            net.minecraft.world.level.saveddata.maps.MapItemSavedData map = null;
            if (row.has("map")) {
                var described = row.getAsJsonObject("map");
                map = net.minecraft.world.level.saveddata.maps.MapItemSavedData.createForClient(
                    described.get("scale").getAsByte(), described.get("locked").getAsBoolean(),
                    net.minecraft.world.level.Level.OVERWORLD);
            }
            var held = map;
            var context = new net.minecraft.world.item.Item.TooltipContext() {
                @Override
                public net.minecraft.core.HolderLookup.Provider registries() {
                    return registries;
                }

                @Override
                public float tickRate() {
                    return 20.0F;
                }

                @Override
                public net.minecraft.world.level.saveddata.maps.MapItemSavedData mapData(
                        net.minecraft.world.level.saveddata.maps.MapId id) {
                    return held;
                }

                @Override
                public boolean isPeaceful() {
                    return false;
                }
            };
            var flag = row.get("advanced").getAsBoolean()
                ? net.minecraft.world.item.TooltipFlag.Default.ADVANCED
                : net.minecraft.world.item.TooltipFlag.Default.NORMAL;
            var lines = new JsonArray();
            for (var line : stack.getTooltipLines(context, null,
                    row.get("creative").getAsBoolean() ? flag.asCreative() : flag)) {
                lines.add(runs(line));
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("lines", lines);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A stack's tooltip as a container screen draws it: its lines and its image
     * (a bundle's ClientBundleTooltip), by the mouse on a GUI of the given size,
     * with a bundle item selected if the row says so. Items draw nothing; their
     * decorations are recorded.
     */
    static JsonObject bundleTooltip(JsonObject request) throws Exception {
        GuiFonts.setup();
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int mouseX = row.get("mouse_x").getAsInt();
            int mouseY = row.get("mouse_y").getAsInt();
            var screen = GuiRecorder.screen(row.get("width").getAsInt(), row.get("height").getAsInt(), mouseX, mouseY);
            GuiFonts.set(screen.minecraft(), "itemModelResolver",
                GuiFonts.unsafe.allocateInstance(GuiSettingsOracle.NoItems.class));
            var stack = MovementSupport.stack(row);
            if (row.has("selected")) {
                net.minecraft.world.item.BundleItem.toggleSelectedItem(stack, row.get("selected").getAsInt());
            }
            var lines = stack.getTooltipLines(net.minecraft.world.item.Item.TooltipContext.EMPTY, null,
                net.minecraft.world.item.TooltipFlag.Default.NORMAL);
            var graphics = screen.graphics();
            graphics.setTooltipForNextFrame(GuiFonts.font, lines, stack.getTooltipImage(), mouseX, mouseY);
            graphics.extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", GuiRecorder.record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A line's runs: [text, style] pairs, the style a colour (or "-") and the set flags. */
    private static JsonArray runs(net.minecraft.network.chat.Component line) {
        var out = new JsonArray();
        var last = new String[] {null};
        var text = new StringBuilder();
        line.visit((style, piece) -> {
            if (piece.isEmpty()) {
                return java.util.Optional.empty();
            }
            var colour = style.getColor();
            var key = (colour == null ? "-" : String.format("%06x", colour.getValue()))
                + (style.isBold() ? "b" : "") + (style.isItalic() ? "i" : "")
                + (style.isUnderlined() ? "u" : "") + (style.isStrikethrough() ? "s" : "")
                + (style.isObfuscated() ? "o" : "");
            if (!key.equals(last[0]) && last[0] != null) {
                out.add(pair(text.toString(), last[0]));
                text.setLength(0);
            }
            last[0] = key;
            text.append(piece);
            return java.util.Optional.empty();
        }, net.minecraft.network.chat.Style.EMPTY);
        if (last[0] != null) {
            out.add(pair(text.toString(), last[0]));
        }
        return out;
    }

    private static JsonArray pair(String text, String style) {
        var pair = new JsonArray();
        pair.add(text);
        pair.add(style);
        return pair;
    }
}
