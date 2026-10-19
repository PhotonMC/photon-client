import java.util.ArrayList;
import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** The chat, its suggestions, titles and draft. */
class GuiChatOracle extends GuiLanguageOracle {
    /** The chat over the HUD: messages added at ticks, drawn at a later one. */
    /** The chat options a row names, set the way the options screen sets them. */
    static void chatOptions(net.minecraft.client.Options options, JsonObject values) {
        if (values.has("sensitivity")) options.sensitivity().set(values.get("sensitivity").getAsDouble());
        if (values.has("wheel")) options.mouseWheelSensitivity().set(values.get("wheel").getAsDouble());
        if (values.has("invert_x")) options.invertMouseX().set(values.get("invert_x").getAsBoolean());
        if (values.has("invert_y")) options.invertMouseY().set(values.get("invert_y").getAsBoolean());
        if (values.has("scale")) options.chatScale().set(values.get("scale").getAsDouble());
        if (values.has("opacity")) options.chatOpacity().set(values.get("opacity").getAsDouble());
        if (values.has("width")) options.chatWidth().set(values.get("width").getAsDouble());
        if (values.has("height_focused")) options.chatHeightFocused().set(values.get("height_focused").getAsDouble());
        if (values.has("height_unfocused")) options.chatHeightUnfocused().set(values.get("height_unfocused").getAsDouble());
        if (values.has("line_spacing")) options.chatLineSpacing().set(values.get("line_spacing").getAsDouble());
        if (values.has("background_opacity")) options.textBackgroundOpacity().set(values.get("background_opacity").getAsDouble());
        if (values.has("effects")) options.screenEffectScale().set(values.get("effects").getAsDouble());
        if (values.has("volumes")) {
            for (var source : net.minecraft.sounds.SoundSource.values()) {
                var volumes = values.getAsJsonObject("volumes");
                if (volumes.has(source.getName())) {
                    options.getSoundSourceOptionInstance(source).set(volumes.get(source.getName()).getAsDouble());
                }
            }
        }
        if (values.has("model_parts")) {
            int mask = values.get("model_parts").getAsInt();
            for (var part : net.minecraft.world.entity.player.PlayerModelPart.values()) {
                options.setModelPart(part, (mask & part.getMask()) != 0);
            }
        }
        if (values.has("main_hand")) {
            options.mainHand().set(net.minecraft.world.entity.HumanoidArm.valueOf(values.get("main_hand").getAsString()));
        }
        if (values.has("subtitles")) options.showSubtitles().set(values.get("subtitles").getAsBoolean());
        if (values.has("directional")) options.directionalAudio().set(values.get("directional").getAsBoolean());
        if (values.has("music_frequency")) {
            options.musicFrequency().set(net.minecraft.client.sounds.MusicManager.MusicFrequency.valueOf(
                values.get("music_frequency").getAsString()));
        }
        if (values.has("music_toast")) {
            for (var state : net.minecraft.client.MusicToastDisplayState.values()) {
                if (state.getSerializedName().equals(values.get("music_toast").getAsString())) {
                    options.musicToast().set(state);
                }
            }
        }
    }

    static JsonObject chat(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int chatMouseX = row.has("mouse") ? row.get("mouse").getAsJsonArray().get(0).getAsInt() : 0;
            int chatMouseY = row.has("mouse") ? row.get("mouse").getAsJsonArray().get(1).getAsInt() : 0;
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), chatMouseX, chatMouseY);
            var chat = new net.minecraft.client.gui.components.ChatComponent(screen.minecraft());
            if (row.has("options")) {
                chatOptions(screen.minecraft().options, row.getAsJsonObject("options"));
            }
            // The messages and the deletions, each at its tick, the chat's own
            // tick after each.
            var events = new ArrayList<JsonObject>();
            for (var messageElement : row.getAsJsonArray("messages")) {
                events.add(messageElement.getAsJsonObject());
            }
            if (row.has("deletions")) {
                for (var deletion : row.getAsJsonArray("deletions")) {
                    events.add(deletion.getAsJsonObject());
                }
            }
            events.sort(java.util.Comparator.comparingInt(event -> event.get("tick").getAsInt()));
            int now = 0;
            for (var event : events) {
                int at = event.get("tick").getAsInt();
                for (; now < at; now++) {
                    set(screen.hud(), "tickCount", now);
                    chat.tick();
                }
                set(screen.hud(), "tickCount", at);
                if (event.has("delete")) {
                    chat.deleteMessage(signature(event.get("delete").getAsInt()));
                    continue;
                }
                var tag = event.has("tag") ? tag(event.get("tag").getAsString()) : null;
                var sig = event.has("signature") ? signature(event.get("signature").getAsInt()) : null;
                var text = event.has("key") ? Component.translatable(event.get("key").getAsString())
                    : event.has("json") ? net.minecraft.network.chat.ComponentSerialization.CODEC
                        .parse(com.mojang.serialization.JsonOps.INSTANCE,
                            com.google.gson.JsonParser.parseString(event.get("json").getAsString()))
                        .getOrThrow()
                    : Component.literal(event.get("text").getAsString());
                if (tag == null && sig == null) {
                    chat.addServerSystemMessage(text);
                } else {
                    chat.addPlayerMessage(text, sig, tag);
                }
            }
            for (; now < row.get("tick").getAsInt(); now++) {
                set(screen.hud(), "tickCount", now);
                chat.tick();
            }
            set(screen.hud(), "tickCount", row.get("tick").getAsInt());
            var languageBefore = net.minecraft.locale.Language.getInstance();
            if (row.has("lang_after")) {
                net.minecraft.locale.Language.inject(clientLanguage(row.get("lang_after").getAsString()));
                chat.rescaleChat();
            }
            if (row.has("rescale")) {
                chatOptions(screen.minecraft().options, row.getAsJsonObject("rescale"));
                chat.rescaleChat();
            }
            boolean focused = row.has("focused") && row.get("focused").getAsBoolean();
            if (focused) {
                // The chat is focused while the chat screen is the open one.
                set(get(screen.minecraft(), "gui"), "screen",
                    unsafe.allocateInstance(net.minecraft.client.gui.screens.ChatScreen.class));
            }
            if (row.has("scroll")) {
                chat.scrollChat(row.get("scroll").getAsInt());
            }
            if (row.has("queued")) {
                // Messages held back by the chat delay: only their count is drawn.
                var listener = get(get(screen.minecraft(), "gui"), "chatListener");
                @SuppressWarnings("unchecked")
                var queue = (java.util.Queue<Object>) get(listener, "delayedMessageQueue");
                var messageClass = Class.forName("net.minecraft.client.multiplayer.chat.ChatListener$Message");
                var constructor = messageClass.getDeclaredConstructors()[0];
                constructor.setAccessible(true);
                for (int queued = 0; queued < row.get("queued").getAsInt(); queued++) {
                    queue.add(constructor.newInstance(null, (java.util.function.BooleanSupplier) () -> true));
                }
            }
            chat.extractRenderState(screen.graphics(), font, row.get("tick").getAsInt(), chatMouseX, chatMouseY,
                focused ? net.minecraft.client.gui.components.ChatComponent.DisplayMode.FOREGROUND
                    : net.minecraft.client.gui.components.ChatComponent.DisplayMode.BACKGROUND, false);
            screen.graphics().extractDeferredElements(chatMouseX, chatMouseY, 0.0F);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            net.minecraft.locale.Language.inject(languageBefore);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The chat box's suggestion list: the matches the server sent for a
     * typed text, then keys pressed in turn. After each, the box's text and
     * what the box and the list draw.
     */
    static JsonObject suggestions(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var screen = screen(width, height, 0, 0);
            var chatScreen = (net.minecraft.client.gui.screens.ChatScreen)
                unsafe.allocateInstance(net.minecraft.client.gui.screens.ChatScreen.class);
            set(chatScreen, "width", width);
            set(chatScreen, "height", height);
            var input = new net.minecraft.client.gui.components.EditBox(font, 4, height - 12, width - 4, 12,
                Component.empty());
            input.setBordered(false);
            input.setMaxLength(256);
            input.setValue(row.get("text").getAsString());
            input.setFocused(true);
            set(chatScreen, "focused", input);
            var commandSuggestions = new net.minecraft.client.gui.components.CommandSuggestions(
                screen.minecraft(), chatScreen, input, font, false, false, 1, 10, true, -805306368);
            commandSuggestions.setAllowSuggestions(true);
            int start = row.get("start").getAsInt();
            var range = com.mojang.brigadier.context.StringRange.between(start, start + row.get("length").getAsInt());
            var list = new ArrayList<com.mojang.brigadier.suggestion.Suggestion>();
            for (var match : row.getAsJsonArray("matches")) {
                list.add(new com.mojang.brigadier.suggestion.Suggestion(range, match.getAsString()));
            }
            set(commandSuggestions, "pendingSuggestions", java.util.concurrent.CompletableFuture.completedFuture(
                new com.mojang.brigadier.suggestion.Suggestions(range, list)));
            commandSuggestions.showSuggestions(false);
            var steps = new JsonArray();
            var keys = new ArrayList<String>();
            keys.add("");
            row.getAsJsonArray("keys").forEach(key -> keys.add(key.getAsString()));
            for (String key : keys) {
                var step = new JsonObject();
                step.addProperty("key", key);
                if (!key.isEmpty()) {
                    int code = switch (key) {
                        case "tab" -> 258;
                        case "up" -> 265;
                        case "down" -> 264;
                        case "escape" -> 256;
                        case "shift+tab" -> 258;
                        default -> throw new IllegalArgumentException(key);
                    };
                    int modifiers = key.startsWith("shift+") ? 1 : 0;
                    commandSuggestions.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, modifiers));
                }
                var drawn = screen(width, height, 0, 0);
                // `ChatScreen.extractRenderState`'s band under the box.
                drawn.graphics().fill(2, height - 14, width - 2, height - 2,
                    drawn.minecraft().options.getBackgroundColor(Integer.MIN_VALUE));
                // The cursor blinks from when the box took the focus.
                set(input, "focusedTime", net.minecraft.util.Util.getMillis());
                input.extractRenderState(drawn.graphics(), 0, 0, 0.0F);
                commandSuggestions.extractRenderState(drawn.graphics(), 0, 0);
                step.addProperty("value", input.getValue());
                step.addProperty("cursor", input.getCursorPosition());
                step.add("ops", record(drawn.state()));
                steps.add(step);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The title, the subtitle and the action bar, each at a point of its time. */
    static JsonObject titles(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var hud = screen.hud();
            int fadeIn = row.get("fade_in").getAsInt();
            int stay = row.get("stay").getAsInt();
            int fadeOut = row.get("fade_out").getAsInt();
            set(hud, "titleFadeInTime", fadeIn);
            set(hud, "titleStayTime", stay);
            set(hud, "titleFadeOutTime", fadeOut);
            if (row.has("title")) {
                set(hud, "title", Component.literal(row.get("title").getAsString()));
                set(hud, "titleTime", fadeIn + stay + fadeOut - row.get("elapsed").getAsInt());
            }
            if (row.has("subtitle")) {
                set(hud, "subtitle", Component.literal(row.get("subtitle").getAsString()));
            }
            if (row.has("overlay")) {
                set(hud, "overlayMessageString", Component.literal(row.get("overlay").getAsString()));
                set(hud, "overlayMessageTime", 60 - row.get("overlay_elapsed").getAsInt());
            }
            float partial = row.get("partial").getAsFloat();
            var tracker = new net.minecraft.client.DeltaTracker() {
                @Override
                public float getGameTimeDeltaTicks() {
                    return partial;
                }

                @Override
                public float getGameTimeDeltaPartialTick(boolean ignoreFrozenGame) {
                    return partial;
                }

                @Override
                public float getRealtimeDeltaTicks() {
                    return partial;
                }
            };
            for (String name : new String[] {"extractOverlayMessage", "extractTitle"}) {
                var method = net.minecraft.client.gui.Hud.class.getDeclaredMethod(name,
                    GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class);
                method.setAccessible(true);
                method.invoke(hud, screen.graphics(), tracker);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * The chat box's draft: boxes opened, typed into, keyed and closed in turn by the game's own
     * ChatScreen and ChatComponent. After each event, what the box holds and whether it is a
     * draft, or what the draft kept is.
     */
    static JsonObject chatDraft(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var screen = screen(width, height, 0, 0);
            var minecraft = screen.minecraft();
            minecraft.setLastInputType(net.minecraft.client.InputType.NONE);
            minecraft.options.saveChatDrafts().set(row.get("save").getAsBoolean());
            var chat = new net.minecraft.client.gui.components.ChatComponent(minecraft);
            set(screen.hud(), "chat", chat);
            set(minecraft, "fontFilterFishy", font);
            var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "commands", new com.mojang.brigadier.CommandDispatcher<
                net.minecraft.client.multiplayer.ClientSuggestionProvider>());
            set(listener, "suggestionsProvider", new OracleSource(java.util.List.of(), java.util.List.of(), true));
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(player, "chatAbilities", net.minecraft.client.multiplayer.chat.ChatAbilities.NO_RESTRICTIONS);
            set(minecraft, "player", player);
            net.minecraft.client.gui.screens.ChatScreen current = null;
            var events = new JsonArray();
            for (var eventElement : row.getAsJsonArray("events")) {
                var event = eventElement.getAsJsonObject();
                var out = new JsonObject();
                String action = event.get("do").getAsString();
                out.addProperty("do", action);
                switch (action) {
                    case "open" -> {
                        var method = event.get("method").getAsString().equals("command")
                            ? net.minecraft.client.gui.components.ChatComponent.ChatMethod.COMMAND
                            : net.minecraft.client.gui.components.ChatComponent.ChatMethod.MESSAGE;
                        current = event.has("bed")
                            ? chat.createScreen(method, net.minecraft.client.gui.screens.InBedChatScreen::new)
                            : chat.createScreen(method, net.minecraft.client.gui.screens.ChatScreen::new);
                        current.init(width, height);
                    }
                    case "type" -> {
                        for (int cp : event.get("text").getAsString().codePoints().toArray()) {
                            current.charTyped(new net.minecraft.client.input.CharacterEvent(cp));
                        }
                    }
                    case "key" -> {
                        int code = switch (event.get("key").getAsString()) {
                            case "backspace" -> 259;
                            case "left" -> 263;
                            case "right" -> 262;
                            case "home" -> 268;
                            case "end" -> 269;
                            case "delete" -> 261;
                            case "up" -> 265;
                            case "down" -> 264;
                            default -> throw new IllegalArgumentException(event.get("key").getAsString());
                        };
                        current.keyPressed(new net.minecraft.client.input.KeyEvent(code, 0, 0));
                    }
                    case "close" -> {
                        var reason = net.minecraft.client.gui.screens.ChatScreen.class.getDeclaredClasses();
                        Object value = null;
                        for (var type : reason) {
                            if (type.getSimpleName().equals("ExitReason")) {
                                for (var constant : type.getEnumConstants()) {
                                    if (constant.toString().equalsIgnoreCase(event.get("reason").getAsString())) {
                                        value = constant;
                                    }
                                }
                            }
                        }
                        set(current, "exitReason", value);
                        current.removed();
                        current = null;
                    }
                    case "draw" -> {
                        var drawn = screen(width, height, 0, 0);
                        var input = (net.minecraft.client.gui.components.EditBox) get(current, "input");
                        var suggestions = (net.minecraft.client.gui.components.CommandSuggestions)
                            get(current, "commandSuggestions");
                        drawn.graphics().fill(2, height - 14, width - 2, height - 2,
                            drawn.minecraft().options.getBackgroundColor(Integer.MIN_VALUE));
                        set(input, "focusedTime", net.minecraft.util.Util.getMillis());
                        input.extractRenderState(drawn.graphics(), 0, 0, 0.0F);
                        if (current instanceof net.minecraft.client.gui.screens.InBedChatScreen) {
                            ((net.minecraft.client.gui.components.Button) get(current, "leaveBedButton"))
                                .extractRenderState(drawn.graphics(), 0, 0, 0.0F);
                        }
                        suggestions.extractRenderState(drawn.graphics(), 0, 0);
                        out.add("ops", record(drawn.state()));
                    }
                    default -> throw new IllegalArgumentException(action);
                }
                if (current != null) {
                    var input = (net.minecraft.client.gui.components.EditBox) get(current, "input");
                    out.addProperty("value", input.getValue());
                    out.addProperty("draft", (boolean) get(current, "isDraft"));
                }
                var latest = get(chat, "latestDraft");
                if (latest == null) {
                    out.add("kept", com.google.gson.JsonNull.INSTANCE);
                } else {
                    var kept = new JsonObject();
                    var record = latest.getClass();
                    var text = record.getDeclaredMethod("text");
                    text.setAccessible(true);
                    var method = record.getDeclaredMethod("chatMethod");
                    method.setAccessible(true);
                    kept.addProperty("text", (String) text.invoke(latest));
                    kept.addProperty("method", method.invoke(latest).toString());
                    out.add("kept", kept);
                }
                events.add(out);
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("events", events);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
