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
    private static final Map<Integer, Float> ADVANCES = new HashMap<>();
    private static final List<com.mojang.blaze3d.font.GlyphProvider> PROVIDERS = new ArrayList<>();

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
    private static void loadFont(net.minecraft.server.packs.resources.ResourceManager manager, Identifier id)
            throws Exception {
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
                        PROVIDERS.add(unpacked.left().get().load(manager));
                    } catch (Exception skipped) {
                        // the provider needs a resource this run does not have
                    }
                } else {
                    loadFont(manager, unpacked.right().get().id());
                }
            }
        } catch (com.google.gson.JsonParseException | java.io.IOException skipped) {
            // a provider the headless run cannot read
        }
    }

    private static float advance(int codepoint) {
        return ADVANCES.computeIfAbsent(codepoint, cp -> {
            for (var provider : PROVIDERS) {
                var glyph = provider.getGlyph(cp);
                if (glyph != null) {
                    return glyph.info().getAdvance();
                }
            }
            return 7.0F;
        });
    }

    private static synchronized void setup() throws Exception {
        if (font != null) {
            return;
        }
        var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        field.setAccessible(true);
        unsafe = (sun.misc.Unsafe) field.get(null);
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
        loadFont(manager, Identifier.withDefaultNamespace("default"));
        BakedGlyph glyphOf = new BakedGlyph() {
            @Override
            public GlyphInfo info() {
                throw new IllegalStateException("per character");
            }

            @Override
            public TextRenderable.Styled createGlyph(float x, float y, int color, int shadowColor, Style style,
                    float boldOffset, float shadowOffset) {
                return null;
            }
        };
        GlyphSource source = new GlyphSource() {
            @Override
            public BakedGlyph getGlyph(int codepoint) {
                float advance = advance(codepoint);
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
        font = new Font(new Font.Provider() {
            @Override
            public GlyphSource glyphs(FontDescription description) {
                return source;
            }

            @Override
            public net.minecraft.client.gui.font.glyphs.EffectGlyph effect() {
                return null;
            }
        });
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
        var window = unsafe.allocateInstance(com.mojang.blaze3d.platform.Window.class);
        set(window, "guiScaledWidth", width);
        set(window, "guiScaledHeight", height);
        set(minecraft, "window", window);
        var atlasManager = unsafe.allocateInstance(net.minecraft.client.resources.model.sprite.AtlasManager.class);
        var atlas = unsafe.allocateInstance(net.minecraft.client.renderer.texture.TextureAtlas.class);
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
        set(gui, "hud", hud);
        set(minecraft, "gui", gui);
        set(minecraft, "font", font);
        var state = new GuiRenderState();
        return new Screen(minecraft, hud, state, new GuiGraphicsExtractor(minecraft, state, mouseX, mouseY));
    }

    /** What was drawn, in order: rectangles and text, each with its pose applied. */
    static JsonArray record(GuiRenderState state) {
        var ops = new JsonArray();
        state.forEachElement(element -> {
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
        return ops;
    }

    public static JsonObject run(JsonObject request) {
        try {
            return switch (request.get("operation").getAsString()) {
                case "chat" -> chat(request);
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
            for (var messageElement : row.getAsJsonArray("messages")) {
                var message = messageElement.getAsJsonObject();
                set(screen.hud(), "tickCount", message.get("tick").getAsInt());
                chat.addServerSystemMessage(Component.literal(message.get("text").getAsString()));
            }
            set(screen.hud(), "tickCount", row.get("tick").getAsInt());
            chat.extractRenderState(screen.graphics(), font, row.get("tick").getAsInt(), 0, 0,
                net.minecraft.client.gui.components.ChatComponent.DisplayMode.BACKGROUND, false);
            var out = new JsonObject();
            out.addProperty("label", row.get("label").getAsString());
            out.add("ops", record(screen.state()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
