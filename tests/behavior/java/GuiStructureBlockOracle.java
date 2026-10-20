import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The structure block's editing screen, over a block entity in no level. */
class GuiStructureBlockOracle extends GuiCommandBlockOracle {
    /** A structure block in no level: setting its mode changes its own field and no block state. */
    static final class Block extends net.minecraft.world.level.block.entity.StructureBlockEntity {
        Block() {
            super(net.minecraft.core.BlockPos.ZERO,
                net.minecraft.world.level.block.Blocks.STRUCTURE_BLOCK.defaultBlockState());
        }

        @Override
        public void setMode(net.minecraft.world.level.block.state.properties.StructureMode mode) {
            try {
                set(this, "mode", mode);
            } catch (Exception e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * Cycle buttons ask the window whether Alt is held, which there is none to answer: each takes
     * its default list, the one used with Alt up.
     */
    @SuppressWarnings({"unchecked", "rawtypes"})
    static void withoutAlt(net.minecraft.client.gui.screens.Screen screen) throws Exception {
        for (var child : screen.children()) {
            if (child instanceof net.minecraft.client.gui.components.CycleButton button) {
                var values = (net.minecraft.client.gui.components.CycleButton.ValueListSupplier) get(button, "values");
                set(button, "values", new net.minecraft.client.gui.components.CycleButton.ValueListSupplier() {
                    @Override
                    public java.util.List getSelectedList() {
                        return values.getDefaultList();
                    }

                    @Override
                    public java.util.List getDefaultList() {
                        return values.getDefaultList();
                    }
                });
            }
        }
    }

    static <E extends Enum<E>> E named(Class<E> type, String name) {
        return Enum.valueOf(type, name.toUpperCase());
    }

    static JsonArray triple(int x, int y, int z) {
        var out = new JsonArray();
        out.add(x);
        out.add(y);
        out.add(z);
        return out;
    }

    /** The block a row describes, as the game holds it when its screen opens. */
    static Block block(JsonObject row) {
        var block = new Block();
        block.setStructureName(row.get("name").getAsString());
        block.setMetaData(row.get("metadata").getAsString());
        var pos = row.getAsJsonArray("pos");
        block.setStructurePos(new net.minecraft.core.BlockPos(
            pos.get(0).getAsInt(), pos.get(1).getAsInt(), pos.get(2).getAsInt()));
        var size = row.getAsJsonArray("size");
        block.setStructureSize(new net.minecraft.core.Vec3i(
            size.get(0).getAsInt(), size.get(1).getAsInt(), size.get(2).getAsInt()));
        block.setMirror(named(net.minecraft.world.level.block.Mirror.class, row.get("mirror").getAsString()));
        block.setRotation(named(net.minecraft.world.level.block.Rotation.class, row.get("rotation").getAsString()));
        block.setMode(named(net.minecraft.world.level.block.state.properties.StructureMode.class,
            row.get("mode").getAsString()));
        block.setIgnoreEntities(row.get("ignore_entities").getAsBoolean());
        block.setStrict(row.get("strict").getAsBoolean());
        block.setShowAir(row.get("show_air").getAsBoolean());
        block.setShowBoundingBox(row.get("show_bounding_box").getAsBoolean());
        block.setIntegrity(row.get("integrity").getAsFloat());
        block.setSeed(row.get("seed").getAsLong());
        return block;
    }

    /**
     * The game's own `StructureBlockEditScreen` for a block, after text, keys, clicks, the wheel and
     * new sizes, drawn at a time after it opened: its drawing, whether it closed, the packets it
     * sent and the values the block holds in the end.
     */
    static JsonObject structureBlockScreen(JsonObject request) throws Exception {
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
            set(minecraft, "fontFilterFishy", get(minecraft, "font"));
            var sent = (Sent) unsafe.allocateInstance(Sent.class);
            set(sent, "packets", new java.util.ArrayList<Object>());
            var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "connection", sent);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            var gui = (Closing) unsafe.allocateInstance(Closing.class);
            set(gui, "hud", screen.hud());
            set(minecraft, "gui", gui);
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var structure = block(row);
            var edit = new net.minecraft.client.gui.screens.inventory.StructureBlockEditScreen(structure);
            edit.init(width, height);
            withoutAlt(edit);
            var steps = new JsonArray();
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                switch (step.get("do").getAsString()) {
                    case "resize" -> {
                        edit.resize(step.get("width").getAsInt(), step.get("height").getAsInt());
                        withoutAlt(edit);
                    }
                    case "wheel" -> edit.mouseScrolled(step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                        0.0, step.get("amount").getAsDouble());
                    default -> {
                        var one = new JsonObject();
                        steps.add(step);
                        one.add("steps", steps);
                        steps(edit, one, minecraft);
                        steps = new JsonArray();
                    }
                }
            }
            now[0] = row.get("ms").getAsLong();
            edit.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            edit.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            out.addProperty("closed", gui.closed);
            var packets = new JsonArray();
            for (var packet : sent.packets) {
                var json = new JsonObject();
                json.addProperty("update", ((Enum<?>) get(packet, "updateType")).name().toLowerCase());
                json.addProperty("mode", ((Enum<?>) get(packet, "mode")).name().toLowerCase());
                json.addProperty("name", (String) get(packet, "name"));
                var offset = (net.minecraft.core.BlockPos) get(packet, "offset");
                json.add("offset", triple(offset.getX(), offset.getY(), offset.getZ()));
                var size = (net.minecraft.core.Vec3i) get(packet, "size");
                json.add("size", triple(size.getX(), size.getY(), size.getZ()));
                json.addProperty("mirror", ((Enum<?>) get(packet, "mirror")).name().toLowerCase());
                json.addProperty("rotation", ((Enum<?>) get(packet, "rotation")).name().toLowerCase());
                json.addProperty("data", (String) get(packet, "data"));
                json.addProperty("ignore_entities", (Boolean) get(packet, "ignoreEntities"));
                json.addProperty("strict", (Boolean) get(packet, "strict"));
                json.addProperty("show_air", (Boolean) get(packet, "showAir"));
                json.addProperty("show_bounding_box", (Boolean) get(packet, "showBoundingBox"));
                json.addProperty("integrity_bits", Float.floatToRawIntBits((Float) get(packet, "integrity")));
                json.addProperty("seed", (Long) get(packet, "seed"));
                packets.add(json);
            }
            out.add("packets", packets);
            var held = new JsonObject();
            held.addProperty("mode", structure.getMode().name().toLowerCase());
            held.addProperty("mirror", structure.getMirror().name().toLowerCase());
            held.addProperty("rotation", structure.getRotation().name().toLowerCase());
            held.addProperty("ignore_entities", structure.isIgnoreEntities());
            held.addProperty("strict", structure.isStrict());
            held.addProperty("show_air", structure.getShowAir());
            held.addProperty("show_bounding_box", structure.getShowBoundingBox());
            out.add("held", held);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
