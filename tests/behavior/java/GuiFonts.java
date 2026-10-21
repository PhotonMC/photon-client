import java.util.ArrayList;
import net.minecraft.client.gui.font.glyphs.BakedGlyph;
import net.minecraft.network.chat.Component;
import java.lang.reflect.Field;
import net.minecraft.client.gui.Font;
import net.minecraft.network.chat.FontDescription;
import com.mojang.blaze3d.font.GlyphInfo;
import net.minecraft.client.gui.font.providers.GlyphProviderDefinition;
import net.minecraft.client.gui.GlyphSource;
import java.util.HashMap;
import net.minecraft.resources.Identifier;
import java.util.List;
import java.util.Map;
import java.util.Set;
import net.minecraft.network.chat.Style;
import net.minecraft.client.gui.font.TextRenderable;

/** Fonts for the headless run: providers, advances and glyph sources from the game's own files. */
class GuiFonts {
    static sun.misc.Unsafe unsafe;

    static Font font;
    /** The game's own assets, once loaded: what screens read texts from. */
    static net.minecraft.server.packs.resources.ResourceManager resources;

    /** Each font's providers, and the advances asked of them. */
    static final Map<String, List<com.mojang.blaze3d.font.GlyphProvider>> FONTS = new HashMap<>();

    static final Map<String, Map<Integer, Float>> ADVANCES = new HashMap<>();

    static final Map<String, GlyphSource> SOURCES = new HashMap<>();

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
        for (Class<?> type = target.getClass(); type != null; type = type.getSuperclass()) {
            try {
                Field field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(target);
            } catch (NoSuchFieldException ignored) {
                // keep looking up the class chain
            }
        }
        throw new NoSuchFieldException(name);
    }

    /**
     * `LogoRenderer` picks its "Minceraft" easter egg from an unseeded random source, one
     * logo in ten thousand; a capture holds it off so every run draws the same logo.
     */
    static <T> T plainLogo(T screen) throws Exception {
        set(get(screen, "logoRenderer"), "showEasterEgg", false);
        return screen;
    }

    /** Loads `font/<id>.json` the way the font manager does, bitmap and space providers only. */
    static void loadFont(net.minecraft.server.packs.resources.ResourceManager manager, Identifier id,
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

    static float advance(String font, int codepoint) {
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
    static GlyphSource glyphSource(String fontId) {
        return new GlyphSource() {
            @Override
            public BakedGlyph getGlyph(int codepoint) {
                float advance = advance(fontId, codepoint);
                // A bitmap glyph has the box its provider gives it, as the game bakes it;
                // a glyph of only an advance has none.
                for (var provider : FONTS.get(fontId)) {
                    var glyph = provider.getGlyph(codepoint);
                    if (glyph == null) {
                        continue;
                    }
                    if (glyph.getClass().getName().endsWith("BitmapProvider$Glyph")) {
                        try {
                            float scale = ((Number) get(glyph, "scale")).floatValue();
                            int width = ((Number) get(glyph, "width")).intValue();
                            int height = ((Number) get(glyph, "height")).intValue();
                            int ascent = ((Number) get(glyph, "ascent")).intValue();
                            float up = 7.0F - ascent;
                            return new net.minecraft.client.gui.font.glyphs.BakedSheetGlyph(glyph.info(), null, null,
                                0.0F, 0.0F, 0.0F, 0.0F, 0.0F, width * scale, up, up + height * scale);
                        } catch (Exception error) {
                            throw new IllegalStateException(error);
                        }
                    }
                    break;
                }
                return new BakedGlyph() {
                    @Override
                    public GlyphInfo info() {
                        return GlyphInfo.simple(advance);
                    }

                    @Override
                    public TextRenderable.Styled createGlyph(float x, float y, int color, int shadowColor,
                            Style style, float boldOffset, float shadowOffset) {
                        return null;
                    }
                };
            }

            @Override
            public BakedGlyph getRandomGlyph(net.minecraft.util.RandomSource random, int width) {
                return getGlyph('?');
            }
        };
    }

    /** The reference's default font, for oracles that measure text outside a screen. */
    static Font loadedFont() throws Exception {
        setup();
        return font;
    }

    static synchronized void setup() throws Exception {
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
        resources = manager;
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
                // An underline or a strikethrough: a box, as a glyph's.
                return (x0, y0, x1, y1, depth, color, shadowColor, shadowOffset) ->
                    new GuiRecorder.Effect(x0, y0, x1, y1, color, shadowColor, shadowOffset);
            }
        });
    }
}
