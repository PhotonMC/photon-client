import com.google.gson.JsonObject;
import java.util.ArrayList;
import net.minecraft.resources.Identifier;

/**
 * The map a cartography table's first slot holds: its data in the level, and a map renderer whose
 * textures are only names, so the picture a screen draws records which colours it holds.
 */
final class GuiMapOracle extends GuiRecorder {
    /** An atlas of the map decorations: each sprite a whole texture of its own, named for its path. */
    static final class DecorationAtlas extends net.minecraft.client.renderer.texture.TextureAtlas {
        DecorationAtlas() {
            super(null);
        }

        @Override
        public net.minecraft.client.renderer.texture.TextureAtlasSprite getSprite(Identifier location) {
            try {
                var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite) unsafe.allocateInstance(
                    net.minecraft.client.renderer.texture.TextureAtlasSprite.class);
                var contents = unsafe.allocateInstance(net.minecraft.client.renderer.texture.SpriteContents.class);
                set(contents, "additionalMetadata", java.util.List.of());
                set(sprite, "contents", contents);
                set(sprite, "atlasLocation", Identifier.withDefaultNamespace(
                    "textures/gui/map_decoration/" + location.getPath() + ".png"));
                set(sprite, "u0", 0.0F);
                set(sprite, "u1", 1.0F);
                set(sprite, "v0", 0.0F);
                set(sprite, "v1", 1.0F);
                return sprite;
            } catch (Exception error) {
                throw new IllegalStateException(error);
            }
        }
    }

    /** The map textures, named for their place and a hash of the colours the texture would hold. */
    static final class NamedMaps extends net.minecraft.client.resources.MapTextureManager {
        NamedMaps() {
            super(null);
        }

        @Override
        public Identifier prepareMapTexture(net.minecraft.world.level.saveddata.maps.MapId id,
                net.minecraft.world.level.saveddata.maps.MapItemSavedData data) {
            int hash = 0x811C9DC5;
            for (byte packed : data.colors) {
                int argb = net.minecraft.world.level.material.MapColor.getColorFromPackedId(packed);
                for (int shift = 24; shift >= 0; shift -= 8) {
                    hash = (hash ^ ((argb >>> shift) & 0xFF)) * 0x01000193;
                }
            }
            return Identifier.withDefaultNamespace(String.format("map/%d/%08x", id.id(), hash));
        }
    }

    /** Puts the map the row describes (`map`: id, scale, locked, colour runs, decorations) in the level. */
    static void seed(JsonObject row, net.minecraft.client.Minecraft minecraft,
            net.minecraft.client.multiplayer.ClientLevel level) throws Exception {
        if (!row.has("map")) {
            return;
        }
        var described = row.getAsJsonObject("map");
        var data = net.minecraft.world.level.saveddata.maps.MapItemSavedData.createForClient(
            described.get("scale").getAsByte(), described.get("locked").getAsBoolean(),
            net.minecraft.world.level.Level.OVERWORLD);
        for (var element : described.getAsJsonArray("runs")) {
            var run = element.getAsJsonArray();
            int start = run.get(0).getAsInt();
            for (int at = start; at < start + run.get(1).getAsInt(); at++) {
                data.colors[at] = run.get(2).getAsByte();
            }
        }
        var decorations = new ArrayList<net.minecraft.world.level.saveddata.maps.MapDecoration>();
        for (var element : described.getAsJsonArray("decorations")) {
            var decoration = element.getAsJsonObject();
            var type = net.minecraft.core.registries.BuiltInRegistries.MAP_DECORATION_TYPE
                .get(Identifier.withDefaultNamespace(decoration.get("type").getAsString())).orElseThrow();
            decorations.add(new net.minecraft.world.level.saveddata.maps.MapDecoration(type,
                decoration.get("x").getAsByte(), decoration.get("y").getAsByte(),
                decoration.get("rot").getAsByte(), java.util.Optional.empty()));
        }
        data.addClientSideDecorations(decorations);
        level.overrideMapData(new net.minecraft.world.level.saveddata.maps.MapId(described.get("id").getAsInt()), data);
        var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.MapRenderer.class);
        set(renderer, "decorationSprites", unsafe.allocateInstance(DecorationAtlas.class));
        set(renderer, "mapTextureManager", unsafe.allocateInstance(NamedMaps.class));
        set(minecraft, "mapRenderer", renderer);
    }
}
