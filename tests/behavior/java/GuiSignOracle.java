import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The sign editing screens, over a sign block entity the game itself holds. */
class GuiSignOracle extends GuiTitleOracle {
    /**
     * The game's own `SignEditScreen` or `HangingSignEditScreen` for a sign of a wood, standing,
     * on a wall or hanging, after text, keys and clicks, drawn at a time after it opened.
     */
    static JsonObject signEditScreen(JsonObject request) throws Exception {
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
            var clockField = net.minecraft.util.Util.class.getDeclaredField("timeSource");
            clockField.setAccessible(true);
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            String wood = row.get("wood").getAsString();
            String kind = row.get("kind").getAsString();
            String suffix = switch (kind) {
                case "standing" -> "_sign";
                case "wall" -> "_wall_sign";
                case "hanging" -> "_hanging_sign";
                default -> "_wall_hanging_sign";
            };
            var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(
                net.minecraft.resources.Identifier.withDefaultNamespace(wood + suffix));
            var pos = net.minecraft.core.BlockPos.ZERO;
            boolean front = row.get("front").getAsBoolean();
            // A sign in no level: its text is set where the level would be told.
            net.minecraft.world.level.block.entity.SignBlockEntity sign = kind.contains("hanging")
                ? new net.minecraft.world.level.block.entity.HangingSignBlockEntity(pos, block.defaultBlockState()) {
                    @Override
                    public boolean setText(net.minecraft.world.level.block.entity.SignText text, boolean isFront) {
                        return keep(this, text, isFront);
                    }
                }
                : new net.minecraft.world.level.block.entity.SignBlockEntity(pos, block.defaultBlockState()) {
                    @Override
                    public boolean setText(net.minecraft.world.level.block.entity.SignText text, boolean isFront) {
                        return keep(this, text, isFront);
                    }
                };
            sign.setText(signText(row.getAsJsonObject("face")), front);
            net.minecraft.client.gui.screens.inventory.AbstractSignEditScreen edit = kind.contains("hanging")
                ? new net.minecraft.client.gui.screens.inventory.HangingSignEditScreen(
                    (net.minecraft.world.level.block.entity.HangingSignBlockEntity) sign, front, false)
                : new net.minecraft.client.gui.screens.inventory.SignEditScreen(sign, front, false);
            edit.init(width, height);
            for (var stepElement : row.getAsJsonArray("steps")) {
                var step = stepElement.getAsJsonObject();
                switch (step.get("do").getAsString()) {
                    case "type" -> step.get("text").getAsString().codePoints().forEach(cp ->
                        edit.charTyped(new net.minecraft.client.input.CharacterEvent(cp)));
                    case "key" -> {
                        String name = step.get("key").getAsString();
                        int modifiers = 0;
                        if (name.startsWith("ctrl+")) {
                            modifiers = 2;
                            name = name.substring("ctrl+".length());
                        }
                        if (name.startsWith("shift+")) {
                            modifiers |= 1;
                            name = name.substring("shift+".length());
                        }
                        int code = switch (name) {
                            case "tab" -> 258;
                            case "backspace" -> 259;
                            case "delete" -> 261;
                            case "enter" -> 257;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "up" -> 265;
                            case "down" -> 264;
                            case "home" -> 268;
                            case "end" -> 269;
                            case "a" -> 65;
                            default -> throw new IllegalArgumentException(name);
                        };
                        minecraft.setLastInputType(code == 258 ? net.minecraft.client.InputType.KEYBOARD_TAB
                            : net.minecraft.client.InputType.KEYBOARD_ARROW);
                        edit.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, modifiers));
                    }
                    default -> throw new IllegalArgumentException(step.toString());
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
            var messages = new JsonArray();
            for (int i = 0; i < 4; i++) {
                messages.add(sign.getText(front).getMessage(i, false).getString());
            }
            out.add("messages", messages);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A face's text put in its field, as `setText` does before it tells the level. */
    static boolean keep(net.minecraft.world.level.block.entity.SignBlockEntity sign,
            net.minecraft.world.level.block.entity.SignText text, boolean isFront) {
        try {
            var field = net.minecraft.world.level.block.entity.SignBlockEntity.class.getDeclaredField(
                isFront ? "frontText" : "backText");
            field.setAccessible(true);
            field.set(sign, text);
            return true;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }
}
