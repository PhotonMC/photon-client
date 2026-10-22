import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The test instance block's editing screen, over a block entity in no level. */
class GuiTestInstanceOracle extends GuiTestBlockOracle {
    /**
     * A test instance block in no level. The client holds no test registry, so the rotation the
     * screen shows is the one the block's data holds.
     */
    static final class Instance extends net.minecraft.world.level.block.entity.TestInstanceBlockEntity {
        private final net.minecraft.world.level.block.Rotation turned;

        Instance(net.minecraft.world.level.block.Rotation turned) {
            super(net.minecraft.core.BlockPos.ZERO,
                net.minecraft.world.level.block.Blocks.TEST_INSTANCE_BLOCK.defaultBlockState());
            this.turned = turned;
        }

        @Override
        public net.minecraft.world.level.block.Rotation getRotation() {
            return turned;
        }
    }

    /** The block a row describes, as the game holds it when its screen opens. */
    static Instance testInstance(JsonObject row) {
        var turned = named(net.minecraft.world.level.block.Rotation.class, row.get("rotation").getAsString());
        var block = new Instance(turned);
        var size = row.getAsJsonArray("size");
        var test = row.get("test").isJsonNull()
            ? java.util.Optional.<net.minecraft.resources.ResourceKey<net.minecraft.gametest.framework.GameTestInstance>>empty()
            : java.util.Optional.of(net.minecraft.resources.ResourceKey.create(
                net.minecraft.core.registries.Registries.TEST_INSTANCE,
                net.minecraft.resources.Identifier.parse(row.get("test").getAsString())));
        block.set(new net.minecraft.world.level.block.entity.TestInstanceBlockEntity.Data(
            test,
            new net.minecraft.core.Vec3i(size.get(0).getAsInt(), size.get(1).getAsInt(), size.get(2).getAsInt()),
            turned, row.get("ignore_entities").getAsBoolean(),
            net.minecraft.world.level.block.entity.TestInstanceBlockEntity.Status.CLEARED,
            java.util.Optional.empty()));
        return block;
    }

    /**
     * The game's own `TestInstanceBlockEditScreen` for a block, after text, keys, clicks, the wheel
     * and new sizes, drawn once: its drawing, whether it closed and every packet it sent, the
     * first on opening.
     */
    static JsonObject testInstanceScreen(JsonObject request) throws Exception {
        net.minecraft.SharedConstants.IS_RUNNING_IN_IDE = false;
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
            SilentSounds.ASKED.clear();
            var edit = new net.minecraft.client.gui.screens.inventory.TestInstanceBlockEditScreen(testInstance(row));
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
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            var packets = new JsonArray();
            for (var packet : sent.packets) {
                var sentPacket = (net.minecraft.network.protocol.game.ServerboundTestInstanceBlockActionPacket) packet;
                var data = sentPacket.data();
                var json = new JsonObject();
                json.addProperty("action", sentPacket.action().name().toLowerCase());
                if (data.test().isPresent()) {
                    json.addProperty("test", data.test().get().identifier().toString());
                } else {
                    json.add("test", com.google.gson.JsonNull.INSTANCE);
                }
                json.add("size", triple(data.size().getX(), data.size().getY(), data.size().getZ()));
                json.addProperty("rotation", data.rotation().name().toLowerCase());
                json.addProperty("ignore_entities", data.ignoreEntities());
                packets.add(json);
            }
            out.add("packets", packets);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
