import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.mojang.blaze3d.font.GlyphInfo;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GlyphSource;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.font.TextRenderable;
import net.minecraft.client.gui.font.glyphs.BakedGlyph;
import net.minecraft.client.gui.font.providers.GlyphProviderDefinition;
import net.minecraft.client.renderer.state.gui.BlitRenderState;
import net.minecraft.client.renderer.state.gui.ColoredRectangleRenderState;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import net.minecraft.client.renderer.state.gui.GuiTextRenderState;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.FontDescription;
import net.minecraft.network.chat.Style;
import net.minecraft.resources.Identifier;

/**
 * The game's own GUI drawing, recorded: the rectangles and text each screen
 * element asks for, in order, with the pose applied. The glyphs are not
 * drawn (no GPU); their advances come from the game's own font providers, so
 * the layout is the reference's.
 */
public final class GuiOracle {
    private GuiOracle() {}

    private static sun.misc.Unsafe unsafe;
    private static Font font;
    /** Each font's providers, and the advances asked of them. */
    private static final Map<String, List<com.mojang.blaze3d.font.GlyphProvider>> FONTS = new HashMap<>();
    private static final Map<String, Map<Integer, Float>> ADVANCES = new HashMap<>();
    private static final Map<String, GlyphSource> SOURCES = new HashMap<>();

    static void set(Object target, String name, Object value) throws Exception {
        for (Class<?> type = target.getClass(); type != null; type = type.getSuperclass()) {
            try {
                Field field = type.getDeclaredField(name);
                field.setAccessible(true);
                field.set(target, value);
                return;
            } catch (NoSuchFieldException ignored) {
                // keep looking up the class chain
            }
        }
        throw new NoSuchFieldException(name);
    }

    static Object get(Object target, String name) throws Exception {
        Field field = target.getClass().getDeclaredField(name);
        field.setAccessible(true);
        return field.get(target);
    }

    /** Loads `font/<id>.json` the way the font manager does, bitmap and space providers only. */
    private static void loadFont(net.minecraft.server.packs.resources.ResourceManager manager, Identifier id,
            List<com.mojang.blaze3d.font.GlyphProvider> providers) throws Exception {
        var location = id.withPrefix("font/").withSuffix(".json");
        try (var reader = manager.openAsReader(location)) {
            var json = com.google.gson.JsonParser.parseReader(reader).getAsJsonObject();
            for (var element : json.getAsJsonArray("providers")) {
                var conditional = GlyphProviderDefinition.Conditional.CODEC
                    .parse(com.mojang.serialization.JsonOps.INSTANCE, element);
                var parsed = conditional.result();
                if (parsed.isEmpty()) {
                    continue; // unihex, true type and others need natives or hex files
                }
                if (!parsed.get().filter().apply(Set.of())) {
                    continue;
                }
                var unpacked = parsed.get().definition().unpack();
                if (unpacked.left().isPresent()) {
                    try {
                        providers.add(unpacked.left().get().load(manager));
                    } catch (Exception skipped) {
                        // the provider needs a resource this run does not have
                    }
                } else {
                    loadFont(manager, unpacked.right().get().id(), providers);
                }
            }
        } catch (com.google.gson.JsonParseException | java.io.IOException skipped) {
            // a provider the headless run cannot read
        }
    }

    private static float advance(String font, int codepoint) {
        return ADVANCES.computeIfAbsent(font, name -> new HashMap<>()).computeIfAbsent(codepoint, cp -> {
            for (var provider : FONTS.get(font)) {
                var glyph = provider.getGlyph(cp);
                if (glyph != null) {
                    return glyph.info().getAdvance();
                }
            }
            return 7.0F;
        });
    }

    /** The glyphs of one font: each as wide as the font's providers say. */
    private static GlyphSource glyphSource(String fontId) {
        return new GlyphSource() {
            @Override
            public BakedGlyph getGlyph(int codepoint) {
                float advance = advance(fontId, codepoint);
                return new BakedGlyph() {
                    @Override
                    public GlyphInfo info() {
                        return GlyphInfo.simple(advance);
                    }

                    @Override
                    public TextRenderable.Styled createGlyph(float x, float y, int color, int shadowColor,
                            Style style, float boldOffset, float shadowOffset) {
                        return new Box(x, y, x + advance, y + 8, style);
                    }
                };
            }

            @Override
            public BakedGlyph getRandomGlyph(net.minecraft.util.RandomSource random, int width) {
                return getGlyph('?');
            }
        };
    }

    private static synchronized void setup() throws Exception {
        if (font != null) {
            return;
        }
        var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        field.setAccessible(true);
        unsafe = (sun.misc.Unsafe) field.get(null);
        // The game's registries, which the client classes read as they load.
        Oracle.vanillaRegistries();
        // The game's own assets, read from its jar.
        var jar = java.nio.file.Path.of(net.minecraft.client.Minecraft.class
            .getProtectionDomain().getCodeSource().getLocation().toURI());
        var jarSystem = java.nio.file.FileSystems.newFileSystem(jar);
        var pack = new net.minecraft.server.packs.VanillaPackResourcesBuilder()
            .pushAssetPath(net.minecraft.server.packs.PackType.CLIENT_RESOURCES, jarSystem.getPath("/assets"))
            .exposeNamespace("minecraft")
            .build(new net.minecraft.server.packs.PackLocationInfo("vanilla", Component.literal("vanilla"),
                net.minecraft.server.packs.repository.PackSource.BUILT_IN, java.util.Optional.empty()));
        var manager = new net.minecraft.server.packs.resources.MultiPackResourceManager(
            net.minecraft.server.packs.PackType.CLIENT_RESOURCES, List.of(pack));
        for (String name : new String[] {"default", "alt"}) {
            var providers = new ArrayList<com.mojang.blaze3d.font.GlyphProvider>();
            loadFont(manager, Identifier.withDefaultNamespace(name), providers);
            FONTS.put("minecraft:" + name, providers);
        }
        font = new Font(new Font.Provider() {
            @Override
            public GlyphSource glyphs(FontDescription description) {
                String id = description instanceof FontDescription.Resource resource
                    ? resource.id().toString() : "minecraft:default";
                return SOURCES.computeIfAbsent(FONTS.containsKey(id) ? id : "minecraft:default",
                    GuiOracle::glyphSource);
            }

            @Override
            public net.minecraft.client.gui.font.glyphs.EffectGlyph effect() {
                return null;
            }
        });
    }

    /** The sprites asked of the GUI atlas, each with a texture coordinate of its own to tell it by. */
    private static final Map<Float, String> SPRITE_NAMES = new HashMap<>();
    private static final Map<String, Float> SPRITE_COORDINATES = new HashMap<>();

    /** An atlas with no texture: it hands out a stand-in sprite for each name. */
    static final class StubAtlas extends net.minecraft.client.renderer.texture.TextureAtlas {
        StubAtlas() {
            super(null);
        }

        @Override
        public net.minecraft.client.renderer.texture.TextureAtlasSprite getSprite(Identifier location) {
            try {
                float coordinate = SPRITE_COORDINATES.computeIfAbsent(location.getPath(), name -> {
                    float next = SPRITE_COORDINATES.size() + 1;
                    SPRITE_NAMES.put(next, name);
                    return next;
                });
                var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) unsafe.allocateInstance(
                    net.minecraft.client.renderer.texture.TextureAtlasSprite.class);
                var contents = unsafe.allocateInstance(net.minecraft.client.renderer.texture.SpriteContents.class);
                set(contents, "additionalMetadata", List.of());
                set(sprite, "contents", contents);
                set(sprite, "atlasLocation", Identifier.withDefaultNamespace("textures/atlas/gui.png"));
                set(sprite, "u0", coordinate);
                set(sprite, "u1", coordinate);
                set(sprite, "v0", 0.0F);
                set(sprite, "v1", 0.0F);
                return sprite;
            } catch (Exception error) {
                throw new IllegalStateException(error);
            }
        }
    }

    /** A texture view that is only its name. */
    static final class NamedView extends com.mojang.blaze3d.textures.GpuTextureView {
        String name;

        NamedView() {
            super(null, 0, 1);
        }

        @Override
        public void close() {}

        @Override
        public boolean isClosed() {
            return false;
        }
    }

    /** A texture with nothing behind it. */
    static final class BlankTexture extends net.minecraft.client.renderer.texture.AbstractTexture {
        NamedView view;

        @Override
        public com.mojang.blaze3d.textures.GpuTextureView getTextureView() {
            return view;
        }

        @Override
        public com.mojang.blaze3d.textures.GpuSampler getSampler() {
            return null;
        }
    }

    /** A narrator that says nothing, for the code that narrates what it selects. */
    static final class SilentNarrator extends net.minecraft.client.GameNarrator {
        SilentNarrator() {
            super(null);
        }

        @Override
        public void saySystemNow(Component message) {}

        @Override
        public void saySystemNow(String message) {}
    }

    /** A glyph's box: all the bounds the game asks of a drawn glyph. */
    private record Box(float left, float top, float right, float bottom, Style style)
            implements TextRenderable.Styled {
        @Override
        public void render(org.joml.Matrix4fc pose, com.mojang.blaze3d.vertex.VertexConsumer buffer,
                int light, boolean flat) {}

        @Override
        public net.minecraft.client.renderer.rendertype.RenderType renderType(Font.DisplayMode mode) {
            return null;
        }

        @Override
        public com.mojang.blaze3d.textures.GpuTextureView textureView() {
            return null;
        }

        @Override
        public com.mojang.blaze3d.pipeline.RenderPipeline guiPipeline() {
            return null;
        }
    }

    /** The game around the screen: a window of the given GUI size, options, a HUD with a tick. */
    record Screen(net.minecraft.client.Minecraft minecraft, Object hud, GuiRenderState state,
                  GuiGraphicsExtractor graphics) {}

    static Screen screen(int width, int height, int mouseX, int mouseY) throws Exception {
        setup();
        var minecraft = (net.minecraft.client.Minecraft)
            unsafe.allocateInstance(net.minecraft.client.Minecraft.class);
        var instance = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        instance.setAccessible(true);
        instance.set(null, minecraft);
        return prepare(minecraft, width, height, mouseX, mouseY);
    }

    /** The GUI around a game: a window, the atlas, textures, fonts and the HUD. */
    static Screen prepare(net.minecraft.client.Minecraft minecraft, int width, int height, int mouseX,
            int mouseY) throws Exception {
        setup();
        var window = unsafe.allocateInstance(com.mojang.blaze3d.platform.Window.class);
        set(window, "guiScaledWidth", width);
        set(window, "guiScaledHeight", height);
        set(minecraft, "window", window);
        var atlasManager = unsafe.allocateInstance(net.minecraft.client.resources.model.sprite.AtlasManager.class);
        var atlas = unsafe.allocateInstance(StubAtlas.class);
        var textures = unsafe.allocateInstance(net.minecraft.client.renderer.texture.TextureManager.class);
        // Every texture asked of is a blank one whose view carries its name.
        set(textures, "byPath", new HashMap<Identifier, net.minecraft.client.renderer.texture.AbstractTexture>() {
            @Override
            public net.minecraft.client.renderer.texture.AbstractTexture get(Object key) {
                try {
                    var texture = (BlankTexture) unsafe.allocateInstance(BlankTexture.class);
                    texture.view = (NamedView) unsafe.allocateInstance(NamedView.class);
                    texture.view.name = ((Identifier) key).getPath();
                    return texture;
                } catch (Exception error) {
                    throw new IllegalStateException(error);
                }
            }
        });
        set(minecraft, "textureManager", textures);
        Class<?> entryClass = Class.forName(
            "net.minecraft.client.resources.model.sprite.AtlasManager$AtlasEntry");
        var constructor = entryClass.getDeclaredConstructors()[0];
        constructor.setAccessible(true);
        set(atlasManager, "atlasById",
            Map.of(net.minecraft.data.AtlasIds.GUI, constructor.newInstance(atlas, null)));
        set(minecraft, "atlasManager", atlasManager);
        var directory = java.nio.file.Files.createTempDirectory("oracle-gui").toFile();
        set(minecraft, "gameDirectory", directory);
        set(minecraft, "options", new net.minecraft.client.Options(minecraft, directory));
        var gui = unsafe.allocateInstance(net.minecraft.client.gui.Gui.class);
        var listener = unsafe.allocateInstance(net.minecraft.client.multiplayer.chat.ChatListener.class);
        set(listener, "delayedMessageQueue", new java.util.ArrayDeque<>());
        set(gui, "chatListener", listener);
        var hud = unsafe.allocateInstance(net.minecraft.client.gui.Hud.class);
        set(hud, "tickCount", 0);
        set(hud, "minecraft", minecraft);
        set(gui, "hud", hud);
        set(minecraft, "gui", gui);
        set(minecraft, "font", font);
        set(minecraft, "narrator", unsafe.allocateInstance(SilentNarrator.class));
        set(minecraft, "textInputManager", unsafe.allocateInstance(com.mojang.blaze3d.platform.TextInputManager.class));
        var state = new GuiRenderState();
        return new Screen(minecraft, hud, state, new GuiGraphicsExtractor(minecraft, state, mouseX, mouseY));
    }

    private static net.minecraft.network.chat.MessageSignature signature(int id) {
        var bytes = new byte[256];
        java.util.Arrays.fill(bytes, (byte) id);
        return new net.minecraft.network.chat.MessageSignature(bytes);
    }

    private static net.minecraft.client.multiplayer.chat.GuiMessageTag tag(String name) {
        return switch (name) {
            case "system" -> net.minecraft.client.multiplayer.chat.GuiMessageTag.system();
            case "system_single_player" ->
                net.minecraft.client.multiplayer.chat.GuiMessageTag.systemSinglePlayer();
            case "not_secure" -> net.minecraft.client.multiplayer.chat.GuiMessageTag.chatNotSecure();
            case "error" -> net.minecraft.client.multiplayer.chat.GuiMessageTag.chatError();
            default -> net.minecraft.client.multiplayer.chat.GuiMessageTag.chatModified(
                name.substring("modified:".length()));
        };
    }

    /**
     * What was drawn: rectangles and text, each with its pose applied, each
     * kind in a fixed order. The game layers what overlaps above what it
     * overlaps, which is an order for the GPU and not for what is drawn.
     */
    static JsonArray record(GuiRenderState state) {
        var ops = new JsonArray();
        state.forEachElement(element -> {
            if (element instanceof BlitRenderState blit) {
                var op = new JsonObject();
                var view = blit.textureSetup().texure0() instanceof NamedView named ? named.name : "";
                if (view.equals("textures/atlas/gui.png")) {
                    op.addProperty("op", "sprite");
                    op.addProperty("name", SPRITE_NAMES.get(blit.u0()));
                } else {
                    // A whole texture, a part of it: where in it, as fractions.
                    op.addProperty("op", "texture");
                    op.addProperty("name", view.replace("textures/gui/", "").replace(".png", ""));
                    op.addProperty("u0", (double) blit.u0());
                    op.addProperty("u1", (double) blit.u1());
                    op.addProperty("v0", (double) blit.v0());
                    op.addProperty("v1", (double) blit.v1());
                }
                var pose = blit.pose();
                op.addProperty("x0", pose.m00() * blit.x0() + pose.m10() * blit.y0() + pose.m20());
                op.addProperty("y0", pose.m01() * blit.x0() + pose.m11() * blit.y0() + pose.m21());
                op.addProperty("x1", pose.m00() * blit.x1() + pose.m10() * blit.y1() + pose.m20());
                op.addProperty("y1", pose.m01() * blit.x1() + pose.m11() * blit.y1() + pose.m21());
                op.addProperty("color", blit.color());
                ops.add(op);
            }
            if (element instanceof ColoredRectangleRenderState rectangle) {
                var op = new JsonObject();
                op.addProperty("op", "fill");
                var pose = rectangle.pose();
                float[] a = {rectangle.x0(), rectangle.y0()};
                float[] b = {rectangle.x1(), rectangle.y1()};
                float[] p = {pose.m00() * a[0] + pose.m10() * a[1] + pose.m20(),
                    pose.m01() * a[0] + pose.m11() * a[1] + pose.m21()};
                float[] q = {pose.m00() * b[0] + pose.m10() * b[1] + pose.m20(),
                    pose.m01() * b[0] + pose.m11() * b[1] + pose.m21()};
                op.addProperty("x0", Math.min(p[0], q[0]));
                op.addProperty("y0", Math.min(p[1], q[1]));
                op.addProperty("x1", Math.max(p[0], q[0]));
                op.addProperty("y1", Math.max(p[1], q[1]));
                op.addProperty("color", rectangle.col1());
                ops.add(op);
            }
        }, GuiRenderState.TraverseRange.ALL);
        state.forEachText(text -> {
            try {
                var op = new JsonObject();
                op.addProperty("op", "text");
                var pose = text.pose;
                float x = ((Number) get(text, "x")).floatValue();
                float y = ((Number) get(text, "y")).floatValue();
                op.addProperty("x", pose.m00() * x + pose.m10() * y + pose.m20());
                op.addProperty("y", pose.m01() * x + pose.m11() * y + pose.m21());
                op.addProperty("scale", pose.m00());
                op.addProperty("color", (Integer) get(text, "color"));
                op.addProperty("shadow", (Boolean) get(text, "dropShadow"));
                var runs = new JsonArray();
                var sequence = (net.minecraft.util.FormattedCharSequence) get(text, "text");
                var current = new StringBuilder();
                Style[] last = {null};
                Runnable flush = () -> {
                    if (current.length() > 0) {
                        var run = new JsonObject();
                        run.addProperty("text", current.toString());
                        var style = last[0];
                        if (style.getColor() != null) {
                            run.addProperty("rgb", style.getColor().getValue());
                        }
                        run.addProperty("bold", style.isBold());
                        run.addProperty("italic", style.isItalic());
                        runs.add(run);
                        current.setLength(0);
                    }
                };
                sequence.accept((index, style, codepoint) -> {
                    if (last[0] != null && !last[0].equals(style)) {
                        flush.run();
                    }
                    last[0] = style;
                    current.appendCodePoint(codepoint);
                    return true;
                });
                flush.run();
                op.add("runs", runs);
                ops.add(op);
            } catch (Exception error) {
                throw new IllegalStateException(error);
            }
        });
        var sorted = new ArrayList<JsonObject>();
        ops.forEach(op -> sorted.add(op.getAsJsonObject()));
        // Fills before text; then top to bottom, left to right, and by what
        // is left to tell two apart.
        sorted.sort(java.util.Comparator
            .comparing((JsonObject op) -> switch (op.get("op").getAsString()) {
                case "fill" -> 0;
                case "sprite", "texture" -> 1;
                default -> 2;
            })
            .thenComparing(op -> op.has("y0") ? op.get("y0").getAsDouble() : op.get("y").getAsDouble())
            .thenComparing(op -> op.has("x0") ? op.get("x0").getAsDouble() : op.get("x").getAsDouble())
            .thenComparing(op -> op.has("x1") ? op.get("x1").getAsDouble() : 0.0)
            .thenComparing(op -> op.has("y1") ? op.get("y1").getAsDouble() : 0.0)
            .thenComparing(op -> op.has("runs") ? flatten(op.getAsJsonArray("runs"))
                : op.has("name") ? op.get("name").getAsString() : "")
            .thenComparing(op -> op.get("color").getAsLong()));
        var ordered = new JsonArray();
        sorted.forEach(ordered::add);
        return ordered;
    }

    private static String flatten(JsonArray runs) {
        var text = new StringBuilder();
        runs.forEach(run -> text.append(run.getAsJsonObject().get("text").getAsString()));
        return text.toString();
    }

    public static JsonObject run(JsonObject request) {
        try {
            return switch (request.get("operation").getAsString()) {
                case "chat" -> chat(request);
                case "suggestions" -> suggestions(request);
                case "titles" -> titles(request);
                case "boss_bar" -> bossBar(request);
                case "sidebar" -> sidebar(request);
                case "command_parse" -> commandParse(request);
                case "command_suggest" -> commandSuggest(request);
                case "tab_list" -> tabList(request);
                case "container_screen" -> containerScreen(request);
                default -> throw new IllegalArgumentException("Unknown gui operation");
            };
        } catch (Exception error) {
            throw new IllegalStateException(error);
        }
    }

    /** The chat over the HUD: messages added at ticks, drawn at a later one. */
    private static JsonObject chat(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var chat = new net.minecraft.client.gui.components.ChatComponent(screen.minecraft());
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
                var text = Component.literal(event.get("text").getAsString());
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
            boolean focused = row.has("focused") && row.get("focused").getAsBoolean();
            if (focused) {
                // The chat is focused while the chat screen is the open one.
                set(get(screen.minecraft(), "gui"), "screen",
                    unsafe.allocateInstance(net.minecraft.client.gui.screens.ChatScreen.class));
            }
            if (row.has("scroll")) {
                chat.scrollChat(row.get("scroll").getAsInt());
            }
            chat.extractRenderState(screen.graphics(), font, row.get("tick").getAsInt(), 0, 0,
                focused ? net.minecraft.client.gui.components.ChatComponent.DisplayMode.FOREGROUND
                    : net.minecraft.client.gui.components.ChatComponent.DisplayMode.BACKGROUND, false);
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
     * The chat box's suggestion list: the matches the server sent for a
     * typed text, then keys pressed in turn. After each, the box's text and
     * what the box and the list draw.
     */
    private static JsonObject suggestions(JsonObject request) throws Exception {
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
    private static JsonObject titles(JsonObject request) throws Exception {
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

    /** The boss bars: each event as the packet adds it, drawn by the overlay. */
    private static JsonObject bossBar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var overlay = new net.minecraft.client.gui.components.BossHealthOverlay(screen.minecraft());
            int index = 0;
            for (var barElement : row.getAsJsonArray("bars")) {
                var bar = barElement.getAsJsonObject();
                var colors = net.minecraft.world.BossEvent.BossBarColor.values();
                var overlays = net.minecraft.world.BossEvent.BossBarOverlay.values();
                var id = new java.util.UUID(0L, ++index);
                ((java.util.Map<java.util.UUID, net.minecraft.client.gui.components.LerpingBossEvent>)
                    get(overlay, "events")).put(id, new net.minecraft.client.gui.components.LerpingBossEvent(
                        id, Component.literal(bar.get("name").getAsString()), bar.get("progress").getAsFloat(),
                        colors[bar.get("color").getAsInt()], overlays[bar.get("overlay").getAsInt()],
                        false, false, false));
            }
            overlay.extractRenderState(screen.graphics());
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A formatted sequence as plain text. */
    private static String plain(net.minecraft.util.FormattedCharSequence sequence) {
        var text = new StringBuilder();
        sequence.accept((index, style, codepoint) -> {
            text.appendCodePoint(codepoint);
            return true;
        });
        return text.toString();
    }

    /** The dispatcher a `ClientboundCommandsPacket` payload makes, as the client builds it. */
    @SuppressWarnings("unchecked")
    private static com.mojang.brigadier.CommandDispatcher<net.minecraft.client.multiplayer.ClientSuggestionProvider>
            commandDispatcher(String hex) throws Exception {
        var registries = Oracle.vanillaRegistries();
        var bytes = java.util.HexFormat.of().parseHex(hex);
        var buffer = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.wrappedBuffer(bytes));
        var packet = net.minecraft.network.protocol.game.ClientboundCommandsPacket.STREAM_CODEC.decode(buffer);
        var builderField = net.minecraft.client.multiplayer.ClientPacketListener.class.getDeclaredField("COMMAND_NODE_BUILDER");
        builderField.setAccessible(true);
        var root = packet.getRoot(net.minecraft.commands.CommandBuildContext.simple(registries,
                net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS),
            (net.minecraft.network.protocol.game.ClientboundCommandsPacket.NodeBuilder<
                net.minecraft.client.multiplayer.ClientSuggestionProvider>) builderField.get(null));
        return new com.mojang.brigadier.CommandDispatcher<>(root);
    }

    /** A suggestion source with no game behind it: it says whether the server was asked. */
    static final class OracleSource extends net.minecraft.client.multiplayer.ClientSuggestionProvider {
        boolean asked;
        final java.util.List<String> players;
        final java.util.List<String> teams;

        OracleSource(java.util.List<String> players, java.util.List<String> teams) {
            super(null, null, permission -> true);
            this.players = players;
            this.teams = teams;
        }

        @Override
        public java.util.Collection<String> getOnlinePlayerNames() {
            return players;
        }

        @Override
        public java.util.Collection<String> getCustomTabSuggestions() {
            return players;
        }

        @Override
        public java.util.Collection<String> getSelectedEntities() {
            return java.util.List.of();
        }

        @Override
        public java.util.Collection<String> getAllTeams() {
            return teams;
        }

        @Override
        public java.util.stream.Stream<Identifier> getAvailableSounds() {
            return java.util.stream.Stream.empty();
        }

        @Override
        public java.util.Collection<TextCoordinates> getRelevantCoordinates() {
            return java.util.Collections.singleton(TextCoordinates.DEFAULT_GLOBAL);
        }

        @Override
        public java.util.Collection<TextCoordinates> getAbsoluteCoordinates() {
            return java.util.Collections.singleton(TextCoordinates.DEFAULT_GLOBAL);
        }

        @Override
        public java.util.Set<net.minecraft.resources.ResourceKey<net.minecraft.world.level.Level>> levels() {
            return java.util.Set.of();
        }

        @Override
        public net.minecraft.core.RegistryAccess registryAccess() {
            return Oracle.vanillaRegistries();
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS;
        }

        @Override
        public java.util.concurrent.CompletableFuture<com.mojang.brigadier.suggestion.Suggestions> customSuggestion(
                com.mojang.brigadier.context.CommandContext<?> context) {
            asked = true;
            return com.mojang.brigadier.suggestion.Suggestions.empty();
        }
    }

    /**
     * What the tree and the types suggest for text at a cursor, by the game's
     * own `getCompletionSuggestions`, and whether the server would be asked.
     */
    private static JsonObject commandSuggest(JsonObject request) throws Exception {
        var dispatcher = commandDispatcher(request.get("tree").getAsString());
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            String text = row.get("text").getAsString();
            int cursor = row.has("cursor") ? row.get("cursor").getAsInt() : text.length();
            var players = new java.util.ArrayList<String>();
            for (var player : row.getAsJsonArray("players")) {
                players.add(player.getAsString());
            }
            var teams = new java.util.ArrayList<String>();
            for (var team : row.getAsJsonArray("teams")) {
                teams.add(team.getAsString());
            }
            var source = new OracleSource(players, teams);
            if (row.has("registry_backed")) {
                // The game lists these from its registries; this port asks the server.
                var out = new JsonObject();
                out.addProperty("label", row.get("label").getAsString());
                out.add("matches", new JsonArray());
                out.addProperty("start", -1);
                out.addProperty("asked", true);
                rows.add(out);
                continue;
            }
            var reader = new com.mojang.brigadier.StringReader(text);
            if (reader.canRead() && reader.peek() == '/') {
                reader.skip();
            }
            var parse = dispatcher.parse(reader, source);
            var suggestions = dispatcher.getCompletionSuggestions(parse, cursor).get();
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var matches = new JsonArray();
            for (var suggestion : suggestions.getList()) {
                matches.add(suggestion.getText());
            }
            out.add("matches", matches);
            out.addProperty("start", suggestions.isEmpty() ? -1 : suggestions.getRange().getStart());
            out.addProperty("asked", source.asked);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * A command tree the way the client reads one off the wire, and what the
     * chat box makes of texts typed against it: where each node read, what
     * could not be read, the colours of the text and the lines under it.
     */
    @SuppressWarnings("unchecked")
    private static JsonObject commandParse(JsonObject request) throws Exception {
        var dispatcher = commandDispatcher(request.get("tree").getAsString());
        net.minecraft.client.multiplayer.ClientSuggestionProvider source = new net.minecraft.client.multiplayer.ClientSuggestionProvider(
            null, null, permission -> true);
        var formatText = net.minecraft.client.gui.components.CommandSuggestions.class.getDeclaredMethod("formatText",
            com.mojang.brigadier.ParseResults.class, String.class, int.class);
        formatText.setAccessible(true);
        var exceptionMessage = net.minecraft.client.gui.components.CommandSuggestions.class
            .getDeclaredMethod("getExceptionMessage", com.mojang.brigadier.exceptions.CommandSyntaxException.class);
        exceptionMessage.setAccessible(true);
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            String text = row.get("text").getAsString();
            var reader = new com.mojang.brigadier.StringReader(text);
            if (reader.canRead() && reader.peek() == '/') {
                reader.skip();
            }
            var parse = dispatcher.parse(reader, source);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var nodes = new JsonArray();
            for (var context = parse.getContext(); context != null; context = context.getChild()) {
                for (var node : context.getNodes()) {
                    var described = new JsonArray();
                    described.add(node.getNode().getUsageText());
                    described.add(node.getRange().getStart());
                    described.add(node.getRange().getEnd());
                    nodes.add(described);
                }
            }
            out.add("nodes", nodes);
            var arguments = new JsonArray();
            for (var entry : parse.getContext().getLastChild().getArguments().entrySet()) {
                var described = new JsonArray();
                described.add(entry.getKey());
                described.add(entry.getValue().getRange().getStart());
                described.add(entry.getValue().getRange().getEnd());
                arguments.add(described);
            }
            out.add("arguments", arguments);
            out.addProperty("cursor", parse.getReader().getCursor());
            out.addProperty("executable", parse.getContext().getLastChild().getCommand() != null);
            var exceptions = new JsonArray();
            for (var entry : parse.getExceptions().entrySet()) {
                var exception = entry.getValue();
                var described = new JsonArray();
                described.add(net.minecraft.network.chat.ComponentUtils.fromMessage(exception.getRawMessage()).getString());
                described.add(exception.getCursor());
                exceptions.add(described);
            }
            out.add("exceptions", exceptions);
            var runs = new JsonArray();
            var sequence = (net.minecraft.util.FormattedCharSequence) formatText.invoke(null, parse, text, 0);
            var current = new StringBuilder();
            int[] colour = {-2};
            sequence.accept((index, style, codepoint) -> {
                int rgb = style.getColor() == null ? -1 : style.getColor().getValue();
                if (rgb != colour[0] && current.length() > 0) {
                    var described = new JsonArray();
                    described.add(current.toString());
                    described.add(colour[0]);
                    runs.add(described);
                    current.setLength(0);
                }
                colour[0] = rgb;
                current.appendCodePoint(codepoint);
                return true;
            });
            if (current.length() > 0) {
                var described = new JsonArray();
                described.add(current.toString());
                described.add(colour[0]);
                runs.add(described);
            }
            out.add("runs", runs);
            // `updateUsageInfo` with the cursor at the end of the text.
            var usage = new JsonArray();
            boolean trailing = false;
            if (row.get("suggestions_empty").getAsBoolean() && !parse.getExceptions().isEmpty()) {
                int literals = 0;
                for (var entry : parse.getExceptions().entrySet()) {
                    var exception = entry.getValue();
                    if (exception.getType() == com.mojang.brigadier.exceptions.CommandSyntaxException.BUILT_IN_EXCEPTIONS.literalIncorrect()) {
                        literals++;
                    } else {
                        usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null, exception)));
                    }
                }
                if (literals > 0) {
                    usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null,
                        com.mojang.brigadier.exceptions.CommandSyntaxException.BUILT_IN_EXCEPTIONS
                            .dispatcherUnknownArgument().createWithContext(parse.getReader()))));
                }
            } else if (parse.getReader().canRead()) {
                trailing = true;
            }
            var suggestionContext = parse.getContext().findSuggestionContext(text.length());
            if (usage.isEmpty()) {
                var entries = new JsonArray();
                for (var entry : dispatcher.getSmartUsage(suggestionContext.parent, source).entrySet()) {
                    if (!(entry.getKey() instanceof com.mojang.brigadier.tree.LiteralCommandNode)) {
                        entries.add(entry.getValue());
                    }
                }
                if (entries.isEmpty() && trailing) {
                    usage.add(plain((net.minecraft.util.FormattedCharSequence) exceptionMessage.invoke(null,
                        net.minecraft.commands.Commands.getParseException(parse))));
                }
                usage.addAll(entries);
            }
            out.add("usage", usage);
            out.addProperty("usage_start", suggestionContext.startPos);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** A number format named in a request: `blank`, `style:<colour>` or `fixed:<text>`. */
    private static net.minecraft.network.chat.numbers.NumberFormat numberFormat(String spec) {
        if (spec.equals("blank")) {
            return net.minecraft.network.chat.numbers.BlankFormat.INSTANCE;
        }
        if (spec.startsWith("style:")) {
            return new net.minecraft.network.chat.numbers.StyledFormat(net.minecraft.network.chat.Style.EMPTY
                .withColor(net.minecraft.ChatFormatting.valueOf(spec.substring("style:".length()).toUpperCase())));
        }
        return new net.minecraft.network.chat.numbers.FixedFormat(Component.literal(spec.substring("fixed:".length())));
    }

    /**
     * A scoreboard a request describes: objectives, scores, the slots they
     * show in and teams, built through the game's own `Scoreboard`.
     */
    private static net.minecraft.world.scores.Scoreboard scoreboard(JsonObject row) {
        var scoreboard = new net.minecraft.world.scores.Scoreboard();
        var objectives = new HashMap<String, net.minecraft.world.scores.Objective>();
        for (var element : row.has("objectives") ? row.getAsJsonArray("objectives") : new JsonArray()) {
            var described = element.getAsJsonObject();
            var name = described.get("name").getAsString();
            objectives.put(name, scoreboard.addObjective(name, net.minecraft.world.scores.criteria.ObjectiveCriteria.DUMMY,
                Component.literal(described.get("title").getAsString()),
                described.has("hearts") && described.get("hearts").getAsBoolean()
                    ? net.minecraft.world.scores.criteria.ObjectiveCriteria.RenderType.HEARTS
                    : net.minecraft.world.scores.criteria.ObjectiveCriteria.RenderType.INTEGER,
                false, described.has("format") ? numberFormat(described.get("format").getAsString()) : null));
        }
        for (var element : row.has("scores") ? row.getAsJsonArray("scores") : new JsonArray()) {
            var described = element.getAsJsonObject();
            var access = scoreboard.getOrCreatePlayerScore(
                net.minecraft.world.scores.ScoreHolder.forNameOnly(described.get("owner").getAsString()),
                objectives.get(described.get("objective").getAsString()), true);
            access.set(described.get("value").getAsInt());
            if (described.has("display")) {
                access.display(Component.literal(described.get("display").getAsString()));
            }
            if (described.has("format")) {
                access.numberFormatOverride(numberFormat(described.get("format").getAsString()));
            }
        }
        if (row.has("slots")) {
            for (var entry : row.getAsJsonObject("slots").entrySet()) {
                scoreboard.setDisplayObjective(net.minecraft.world.scores.DisplaySlot.values()[Integer.parseInt(entry.getKey())],
                    objectives.get(entry.getValue().getAsString()));
            }
        }
        if (row.has("teams")) {
            for (var element : row.getAsJsonArray("teams")) {
                var described = element.getAsJsonObject();
                var team = scoreboard.addPlayerTeam(described.get("name").getAsString());
                if (described.has("prefix")) {
                    team.setPlayerPrefix(Component.literal(described.get("prefix").getAsString()));
                }
                if (described.has("suffix")) {
                    team.setPlayerSuffix(Component.literal(described.get("suffix").getAsString()));
                }
                if (described.has("color")) {
                    team.setColor(java.util.Optional.of(net.minecraft.world.scores.TeamColor.values()[described.get("color").getAsInt()]));
                }
                for (var player : described.getAsJsonArray("players")) {
                    scoreboard.addPlayerToTeam(player.getAsString(), team);
                }
            }
        }
        return scoreboard;
    }

    /** The sidebar the scoreboard and the local player's team choose. */
    private static JsonObject sidebar(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            var screen = screen(row.get("width").getAsInt(), row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            set(listener, "scoreboard", scoreboard(row));
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            String local = row.get("local").getAsString();
            set(player, "gameProfile", new com.mojang.authlib.GameProfile(
                java.util.UUID.nameUUIDFromBytes(local.getBytes()), local));
            set(minecraft, "player", player);
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "connection", listener);
            set(minecraft, "level", level);
            var method = net.minecraft.client.gui.Hud.class.getDeclaredMethod("extractScoreboardSidebar",
                GuiGraphicsExtractor.class, net.minecraft.client.DeltaTracker.class);
            method.setAccessible(true);
            method.invoke(screen.hud(), screen.graphics(), null);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The player list over the game: who is listed, how, and the header and footer. */
    private static JsonObject tabList(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            int width = row.get("width").getAsInt();
            var screen = screen(width, row.get("height").getAsInt(), 0, 0);
            var minecraft = screen.minecraft();
            var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
                unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
            var scoreboard = scoreboard(row);
            set(listener, "scoreboard", scoreboard);
            var listed = new it.unimi.dsi.fastutil.objects.ReferenceOpenHashSet<net.minecraft.client.multiplayer.PlayerInfo>();
            set(listener, "listedPlayers", listed);
            set(listener, "onlineMode", false);
            var player = unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class);
            set(player, "connection", listener);
            set(minecraft, "player", player);
            var level = unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientLevel.class);
            set(level, "connection", listener);
            set(minecraft, "level", level);
            for (var playerElement : row.getAsJsonArray("players")) {
                var described = playerElement.getAsJsonObject();
                String name = described.get("name").getAsString();
                var info = new net.minecraft.client.multiplayer.PlayerInfo(new com.mojang.authlib.GameProfile(
                    java.util.UUID.nameUUIDFromBytes(name.getBytes()), name), false);
                set(info, "gameMode", net.minecraft.world.level.GameType.byId(described.get("mode").getAsInt()));
                set(info, "latency", described.get("latency").getAsInt());
                set(info, "tabListOrder", described.get("order").getAsInt());
                if (described.has("display")) {
                    info.setTabListDisplayName(Component.literal(described.get("display").getAsString()));
                }
                listed.add(info);
            }
            var overlay = new net.minecraft.client.gui.components.PlayerTabOverlay(minecraft,
                (net.minecraft.client.gui.Hud) screen.hud());
            if (row.has("header")) {
                overlay.setHeader(Component.literal(row.get("header").getAsString()));
            }
            if (row.has("footer")) {
                overlay.setFooter(Component.literal(row.get("footer").getAsString()));
            }
            overlay.extractRenderState(screen.graphics(), width, scoreboard,
                scoreboard.getDisplayObjective(net.minecraft.world.scores.DisplaySlot.LIST));
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Lighting with no GPU behind it. */
    static final class NoLighting extends com.mojang.blaze3d.platform.Lighting {
        @Override
        public void setupFor(com.mojang.blaze3d.platform.Lighting.Entry entry) {}
    }

    /** An item resolver that draws nothing: slots hold items, which are not recorded. */
    static final class NoItems extends net.minecraft.client.renderer.item.ItemModelResolver {
        NoItems() {
            super(null);
        }

        @Override
        public void updateForTopItem(net.minecraft.client.renderer.item.ItemStackRenderState output,
                net.minecraft.world.item.ItemStack item, net.minecraft.world.item.ItemDisplayContext context,
                net.minecraft.world.level.Level level, net.minecraft.world.entity.ItemOwner owner, int seed) {}
    }

    /**
     * A menu's screen over the game's own container menu: its labels, sprites,
     * name box and what it shows of its data, with slots holding items that
     * draw nothing. After each row's `keys` and `typed` text.
     */
    private static JsonObject containerScreen(JsonObject request) throws Exception {
        var rows = new JsonArray();
        for (var element : request.getAsJsonArray("rows")) {
            var row = element.getAsJsonObject();
            setup();
            MovementOracle.bootstrap();
            var open = MovementOracle.openMenu(row);
            var minecraft = open.scene().minecraft();
            var instance = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
            instance.setAccessible(true);
            instance.set(null, minecraft);
            int width = row.get("width").getAsInt();
            int height = row.get("height").getAsInt();
            var drawn = prepare(minecraft, width, height, row.get("mouse_x").getAsInt(),
                row.get("mouse_y").getAsInt());
            set(minecraft, "itemModelResolver", unsafe.allocateInstance(NoItems.class));
            // The entity models the loom's flag and the book of the enchanting
            // table are baked from.
            var modelManager = unsafe.allocateInstance(net.minecraft.client.resources.model.ModelManager.class);
            set(modelManager, "entityModelSet", net.minecraft.client.model.geom.EntityModelSet.vanilla());
            set(minecraft, "modelManager", modelManager);
            var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
            set(renderer, "lighting", unsafe.allocateInstance(NoLighting.class));
            set(minecraft, "gameRenderer", renderer);
            set(minecraft, "level", open.scene().level());
            set(minecraft, "deltaTracker", new net.minecraft.client.DeltaTracker.Timer(20.0F, 0L, millis -> 50L));
            var menu = open.menu();
            var inventory = open.player().getInventory();
            net.minecraft.client.gui.screens.inventory.AbstractContainerScreen<?> screen =
                switch (row.get("menu").getAsString()) {
                    case "minecraft:anvil" -> new net.minecraft.client.gui.screens.inventory.AnvilScreen(
                        (net.minecraft.world.inventory.AnvilMenu) menu, inventory,
                        Component.translatable("container.repair"));
                    case "minecraft:stonecutter" -> new net.minecraft.client.gui.screens.inventory.StonecutterScreen(
                        (net.minecraft.world.inventory.StonecutterMenu) menu, inventory,
                        Component.translatable("container.stonecutter"));
                    case "minecraft:loom" -> new net.minecraft.client.gui.screens.inventory.LoomScreen(
                        (net.minecraft.world.inventory.LoomMenu) menu, inventory,
                        Component.translatable("container.loom"));
                    case "minecraft:grindstone" -> new net.minecraft.client.gui.screens.inventory.GrindstoneScreen(
                        (net.minecraft.world.inventory.GrindstoneMenu) menu, inventory,
                        Component.translatable("container.grindstone_title"));
                    case "minecraft:enchantment" -> new net.minecraft.client.gui.screens.inventory.EnchantmentScreen(
                        (net.minecraft.world.inventory.EnchantmentMenu) menu, inventory,
                        Component.translatable("container.enchant"));
                    default -> throw new IllegalArgumentException("Unknown menu screen");
                };
            set(screen, "minecraft", minecraft);
            set(screen, "font", font);
            set(screen, "width", width);
            set(screen, "height", height);
            var init = net.minecraft.client.gui.screens.Screen.class.getDeclaredMethod("init");
            init.setAccessible(true);
            init.invoke(screen);
            if (row.has("sync_slot")) {
                // The server's update of a slot, as the screen hears it.
                int slot = row.get("sync_slot").getAsInt();
                if (screen instanceof net.minecraft.world.inventory.ContainerListener listener) {
                    listener.slotChanged(menu, slot, menu.getSlot(slot).getItem());
                } else {
                    // The stonecutter and the loom hear their menu's own update.
                    menu.getSlot(slot).setChanged();
                }
            }
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            var steps = new JsonArray();
            var keys = new ArrayList<String>();
            keys.add("");
            if (row.has("keys")) {
                row.getAsJsonArray("keys").forEach(key -> keys.add(key.getAsString()));
            }
            for (String key : keys) {
                var step = new JsonObject();
                step.addProperty("key", key);
                if (key.startsWith("type:")) {
                    for (char c : key.substring(5).toCharArray()) {
                        screen.charTyped(new net.minecraft.client.input.CharacterEvent(c));
                    }
                }
                var frame = prepare(minecraft, width, height, row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt());
                // The screen's own draw, without the deferred tooltip the game
                // draws last (tooltip text is the item's, which this run has no
                // items to give).
                frame.graphics().nextStratum();
                screen.extractBackground(frame.graphics(), row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt(), 0.0F);
                frame.graphics().nextStratum();
                screen.extractRenderState(frame.graphics(), row.get("mouse_x").getAsInt(),
                    row.get("mouse_y").getAsInt(), 0.0F);
                step.add("ops", record(frame.state()));
                steps.add(step);
            }
            out.add("steps", steps);
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
