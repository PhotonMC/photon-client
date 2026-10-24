import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The sign and book editing screens, over a sign block entity or a book the game itself holds. */
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
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            SilentSounds.ASKED.clear();
            var clockField = Reflect.field(net.minecraft.util.Util.class, "timeSource");
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
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
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

    /**
     * The game's own `BookEditScreen` for a book and quill of pages, after text, keys and clicks,
     * drawn at a time after it opened: its drawing, its pages and the page it is on.
     */
    static JsonObject bookEditScreen(JsonObject request) throws Exception {
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
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            SilentSounds.ASKED.clear();
            menu(screen);
            var clockField = Reflect.field(net.minecraft.util.Util.class, "timeSource");
            var clockBefore = clockField.get(null);
            long[] now = {0};
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> now[0] * 1_000_000L);
            var edit = bookEdit(row);
            edit.init(width, height);
            steps(edit, row, minecraft);
            now[0] = row.get("ms").getAsLong();
            edit.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            edit.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            var written = new JsonArray();
            ((java.util.List<?>) get(edit, "pages")).forEach(page -> written.add((String) page));
            out.add("pages", written);
            out.addProperty("page", (Integer) get(edit, "currentPage"));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A book and quill of a row's pages in the main hand of a player named Steve. */
    static net.minecraft.client.gui.screens.inventory.BookEditScreen bookEdit(JsonObject row) throws Exception {
        var owner = (net.minecraft.world.entity.player.Player)
            unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
        var profileField = Reflect.field(net.minecraft.world.entity.player.Player.class, "gameProfile");
        profileField.set(owner, new com.mojang.authlib.GameProfile(new java.util.UUID(0, 1), "Steve"));
        var pages = new java.util.ArrayList<net.minecraft.server.network.Filterable<String>>();
        row.getAsJsonArray("pages").forEach(page ->
            pages.add(net.minecraft.server.network.Filterable.passThrough(page.getAsString())));
        var book = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.WRITABLE_BOOK);
        return new net.minecraft.client.gui.screens.inventory.BookEditScreen(owner, book,
            net.minecraft.world.InteractionHand.MAIN_HAND,
            new net.minecraft.world.item.component.WritableBookContent(pages));
    }

    /** A row's steps on a screen: text typed, keys with their modifiers, clicks. */
    static void steps(net.minecraft.client.gui.screens.Screen edit, JsonObject row,
            net.minecraft.client.Minecraft minecraft) {
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
                        case "escape" -> 256;
                        case "space" -> 32;
                        case "left" -> 263;
                        case "right" -> 262;
                        case "up" -> 265;
                        case "down" -> 264;
                        case "home" -> 268;
                        case "end" -> 269;
                        case "pageup" -> 266;
                        case "pagedown" -> 267;
                        case "a" -> 65;
                        default -> throw new IllegalArgumentException(name);
                    };
                    minecraft.setLastInputType(code == 258 ? net.minecraft.client.InputType.KEYBOARD_TAB
                        : net.minecraft.client.InputType.KEYBOARD_ARROW);
                    edit.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, modifiers));
                }
                case "click" -> edit.mouseClicked(new net.minecraft.client.input.MouseButtonEvent(
                    step.get("x").getAsDouble(), step.get("y").getAsDouble(),
                    new net.minecraft.client.input.MouseButtonInfo(0, step.has("shift") ? 1 : 0)), false);
                default -> throw new IllegalArgumentException(step.toString());
            }
        }
    }

    /**
     * The game's own `BookSignScreen` of a book and quill (the one its `BookEditScreen` holds), with
     * a title kept from before and shown again or for the first time, after text, keys and clicks.
     */
    static JsonObject bookSignScreen(JsonObject request) throws Exception {
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
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            SilentSounds.ASKED.clear();
            menu(screen);
            var clockField = Reflect.field(net.minecraft.util.Util.class, "timeSource");
            var clockBefore = clockField.get(null);
            clockField.set(null, (net.minecraft.util.TimeSource.NanoTimeSource) () -> 0L);
            var sign = (net.minecraft.client.gui.screens.inventory.BookSignScreen) get(bookEdit(row), "signScreen");
            set(sign, "titleValue", row.get("title").getAsString());
            sign.init(width, height);
            if (row.get("reopened").getAsBoolean()) {
                sign.init(width, height);
            }
            steps(sign, row, minecraft);
            sign.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            sign.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            clockField.set(null, clockBefore);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            out.addProperty("title", ((net.minecraft.client.gui.components.EditBox) get(sign, "titleBox")).getValue());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The game's own `BookViewScreen` over a written book in the hand (`handleOpenBook`): pages as
     * component JSON, turned by clicks and Page Up/Down; its drawing and the page open.
     */
    static JsonObject bookView(JsonObject request) throws Exception {
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
            set(minecraft, "soundManager", unsafe.allocateInstance(SilentSounds.class));
            SilentSounds.ASKED.clear();
            menu(screen);
            var pages = new java.util.ArrayList<net.minecraft.server.network.Filterable<net.minecraft.network.chat.Component>>();
            for (var page : row.getAsJsonArray("pages")) {
                pages.add(net.minecraft.server.network.Filterable.passThrough(
                    net.minecraft.network.chat.ComponentSerialization.CODEC
                        .parse(com.mojang.serialization.JsonOps.INSTANCE, page).getOrThrow()));
            }
            var book = new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.WRITTEN_BOOK);
            book.set(net.minecraft.core.component.DataComponents.WRITTEN_BOOK_CONTENT,
                new net.minecraft.world.item.component.WrittenBookContent(
                    net.minecraft.server.network.Filterable.passThrough("Title"), "Steve", 0, pages, true));
            var view = new net.minecraft.client.gui.screens.inventory.BookViewScreen(
                net.minecraft.client.gui.screens.inventory.BookViewScreen.BookAccess.fromItem(book));
            view.init(width, height);
            steps(view, row, minecraft);
            view.extractBackground(screen.graphics(), mouseX, mouseY, 0.0F);
            view.extractRenderState(screen.graphics(), mouseX, mouseY, 0.0F);
            screen.graphics().extractDeferredElements(mouseX, mouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            var asked = new JsonArray();
            SilentSounds.ASKED.forEach(asked::add);
            out.add("sounds", asked);
            out.addProperty("page", (Integer) get(view, "currentPage"));
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
            var field = Reflect.field(net.minecraft.world.level.block.entity.SignBlockEntity.class, 
                isFront ? "frontText" : "backText");
            field.set(sign, text);
            return true;
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }
}
