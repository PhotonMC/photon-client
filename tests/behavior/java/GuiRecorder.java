import java.util.ArrayList;
import net.minecraft.client.renderer.state.gui.BlitRenderState;
import net.minecraft.client.renderer.state.gui.ColoredRectangleRenderState;
import net.minecraft.network.chat.Component;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.renderer.state.gui.GuiRenderState;
import java.util.HashMap;
import net.minecraft.resources.Identifier;
import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.List;
import java.util.Map;
import net.minecraft.network.chat.Style;
import net.minecraft.client.gui.font.TextRenderable;

/** Stubs for a headless GUI, and the recorder that keeps what a screen draws. */
class GuiRecorder extends GuiFonts {
    /** The sprites asked of the GUI atlas, each with a texture coordinate of its own to tell it by. */
    static final Map<Float, String> SPRITE_NAMES = new HashMap<>();

    static final Map<String, Float> SPRITE_COORDINATES = new HashMap<>();

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

        @Override
        public boolean isActive() {
            return false;
        }
    }

    /** A glyph's box: all the bounds the game asks of a drawn glyph. */
    record Box(float left, float top, float right, float bottom, Style style)
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

    /** An underline or strikethrough: what the game asks of the effect glyph, and its bounds. */
    record Effect(float x0, float y0, float x1, float y1, int color, int shadowColor, float shadowOffset)
            implements TextRenderable {
        public float left() {
            return x0;
        }

        public float top() {
            return y0;
        }

        public float right() {
            return x1 + (shadowColor != 0 ? shadowOffset : 0.0F);
        }

        public float bottom() {
            return y1 + (shadowColor != 0 ? shadowOffset : 0.0F);
        }

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

    static net.minecraft.network.chat.MessageSignature signature(int id) {
        var bytes = new byte[256];
        java.util.Arrays.fill(bytes, (byte) id);
        return new net.minecraft.network.chat.MessageSignature(bytes);
    }

    static net.minecraft.client.multiplayer.chat.GuiMessageTag tag(String name) {
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

    /// The rectangles a text draws for its underlines and strikethrough, as the font prepared them:
    /// where, in what colour and with which shadow colour (zero for none).
    static JsonArray effects(net.minecraft.client.renderer.state.gui.GuiTextRenderState text) throws Exception {
        var out = new JsonArray();
        text.ensurePrepared().visit(new net.minecraft.client.gui.Font.GlyphVisitor() {
            @Override
            public void acceptEffect(net.minecraft.client.gui.font.TextRenderable effect) {
                var box = (Effect) effect;
                var rect = new JsonObject();
                rect.addProperty("x0", box.x0());
                rect.addProperty("y0", box.y0());
                rect.addProperty("x1", box.x1());
                rect.addProperty("y1", box.y1());
                rect.addProperty("color", box.color());
                rect.addProperty("shadowColor", box.shadowColor());
                out.add(rect);
            }
        });
        return out;
    }

    static JsonArray record(GuiRenderState state) {
        var ops = new JsonArray();
        state.forEachElement(element -> {
            if (element instanceof BlitRenderState blit) {
                var op = new JsonObject();
                var view = blit.textureSetup().texure0() instanceof NamedView named ? named.name : "";
                if (view.equals("textures/atlas/gui.png")) {
                    op.addProperty("op", "sprite");
                    op.addProperty("name", SPRITE_NAMES.get(blit.u0()));
                } else if (view.equals("textures/atlas/blocks.png")) {
                    // A sprite of the block atlas, which is laid out differently here: named, not placed.
                    op.addProperty("op", "sprite");
                    op.addProperty("name", "block_atlas");
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
                        run.addProperty("underlined", style.isUnderlined());
                        run.addProperty("strikethrough", style.isStrikethrough());
                        run.addProperty("obfuscated", style.isObfuscated());
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
                op.add("effects", effects(text));
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

    static String flatten(JsonArray runs) {
        var text = new StringBuilder();
        runs.forEach(run -> text.append(run.getAsJsonObject().get("text").getAsString()));
        return text.toString();
    }

    /** A formatted sequence as plain text. */
    static String plain(net.minecraft.util.FormattedCharSequence sequence) {
        var text = new StringBuilder();
        sequence.accept((index, style, codepoint) -> {
            text.appendCodePoint(codepoint);
            return true;
        });
        return text.toString();
    }

    /** A suggestion source with no game behind it: it says whether the server was asked. */
    static final class OracleSource extends net.minecraft.client.multiplayer.ClientSuggestionProvider {
        boolean asked;
        final java.util.List<String> players;
        final java.util.List<String> teams;
        java.util.List<String> selected = java.util.List.of();

        OracleSource(java.util.List<String> players, java.util.List<String> teams, boolean selectors) {
            super(null, null, permission -> selectors
                || !permission.equals(net.minecraft.server.permissions.Permissions.COMMANDS_ENTITY_SELECTORS));
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
            return selected;
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

    /** The dispatcher a `ClientboundCommandsPacket` payload makes, as the client builds it. */
    @SuppressWarnings("unchecked")
    static com.mojang.brigadier.CommandDispatcher<net.minecraft.client.multiplayer.ClientSuggestionProvider>
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
}
