import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Item entities, projectiles and entity shadows in a scene. */
class MovementEntityOracle extends MovementSupport {
    /**
     * Step the reference's own dropped item through a scene, tick by tick,
     * the way the client level ticks an entity it did not spawn itself: the
     * server's packets set its place and motion, the level does the rest.
     */
    static JsonObject itemEntityTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var listener = scene.listener();
        // The handlers check they run on the packet thread, and read the level.
        setField(scene.minecraft(), "packetProcessor",
            new net.minecraft.network.PacketProcessor(Thread.currentThread()));
        setField(listener, "level", level);
        setField(listener, "minecraft", scene.minecraft());
        // The handlers ask whether the player rides the entity; this one rides nothing.
        setField(scene.minecraft(), "player",
            unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class));
        // A splash in the water asks the renderer's camera whether to show
        // its particles, and the engine to make them; none are drawn here.
        var splashRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(splashRenderer, "mainCamera", new net.minecraft.client.Camera());
        setField(scene.minecraft(), "gameRenderer", splashRenderer);
        setField(scene.minecraft(), "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
        var item = new net.minecraft.world.entity.item.ItemEntity(level,
            Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.DIAMOND),
            Oracle.f64(row, "motion_x"), Oracle.f64(row, "motion_y"), Oracle.f64(row, "motion_z"));
        int entityId = row.get("entity_id").getAsInt();
        item.setId(entityId);
        // `recreateFromPacket`: where the server's relative moves count from.
        item.syncPacketPositionCodec(item.getX(), item.getY(), item.getZ());
        level.addEntity(item);
        if (row.get("on_ground").getAsBoolean()) {
            // `handleMoveEntity` and the teleports hand the packet's flag over.
            item.setOnGround(true);
        }
        var ticks = new JsonArray();
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            // The packets that arrived since the last tick, as the listener
            // handles them.
            for (var element : row.getAsJsonArray("events")) {
                var event = element.getAsJsonObject();
                if (event.get("tick").getAsInt() != tick) {
                    continue;
                }
                boolean onGround = event.has("on_ground") && event.get("on_ground").getAsBoolean();
                switch (event.get("type").getAsString()) {
                    case "move" -> listener.handleMoveEntity(
                        new net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos(entityId,
                            (short) event.get("xa").getAsInt(), (short) event.get("ya").getAsInt(),
                            (short) event.get("za").getAsInt(), onGround));
                    case "sync" -> listener.handleEntityPositionSync(
                        new net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket(entityId,
                            new net.minecraft.world.entity.PositionMoveRotation(
                                new net.minecraft.world.phys.Vec3(Oracle.f64(event, "x"),
                                    Oracle.f64(event, "y"), Oracle.f64(event, "z")),
                                new net.minecraft.world.phys.Vec3(Oracle.f64(event, "motion_x"),
                                    Oracle.f64(event, "motion_y"), Oracle.f64(event, "motion_z")),
                                0.0F, 0.0F),
                            onGround));
                    case "motion" -> listener.handleSetEntityMotion(
                        new net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket(entityId,
                            new net.minecraft.world.phys.Vec3(Oracle.f64(event, "motion_x"),
                                Oracle.f64(event, "motion_y"), Oracle.f64(event, "motion_z"))));
                    default -> throw new IllegalArgumentException("Unknown event " + event);
                }
            }
            level.tickNonPassenger(item);
            var described = new JsonObject();
            described.addProperty("x", Oracle.bits(item.getX()));
            described.addProperty("y", Oracle.bits(item.getY()));
            described.addProperty("z", Oracle.bits(item.getZ()));
            described.addProperty("motion_x", Oracle.bits(item.getDeltaMovement().x));
            described.addProperty("motion_y", Oracle.bits(item.getDeltaMovement().y));
            described.addProperty("motion_z", Oracle.bits(item.getDeltaMovement().z));
            described.addProperty("on_ground", item.onGround());
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    /**
     * A projectile the server added, ticked on the client as the level ticks it: where it goes
     * between the server's packets, against the blocks and the entities the scene holds.
     */
    static JsonObject projectileTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var listener = scene.listener();
        setField(scene.minecraft(), "packetProcessor",
            new net.minecraft.network.PacketProcessor(Thread.currentThread()));
        setField(listener, "level", level);
        setField(listener, "minecraft", scene.minecraft());
        setField(scene.minecraft(), "player",
            unsafe.allocateInstance(net.minecraft.client.player.LocalPlayer.class));
        // A splash in the water asks the camera, bubbles and smoke ask the engine; none is drawn.
        var splashRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(splashRenderer, "mainCamera", new net.minecraft.client.Camera());
        setField(scene.minecraft(), "gameRenderer", splashRenderer);
        setField(scene.minecraft(), "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
        setField(listener, "scoreboard", new net.minecraft.world.scores.Scoreboard());
        setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // What else stands in the scene: the owner, the targets.
        for (var element : row.getAsJsonArray("others")) {
            var other = element.getAsJsonObject();
            var otherType = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                net.minecraft.resources.Identifier.parse(other.get("type").getAsString()));
            var entity = otherType.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            entity.setId(other.get("id").getAsInt());
            entity.snapTo(Oracle.f64(other, "x"), Oracle.f64(other, "y"), Oracle.f64(other, "z"), 0.0F, 0.0F);
            if (entity instanceof net.minecraft.world.entity.LivingEntity living) {
                living.yBodyRot = 0.0F;
                living.yHeadRot = 0.0F;
                living.yBodyRotO = 0.0F;
                living.yHeadRotO = 0.0F;
            }
            level.addEntity(entity);
        }
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("type").getAsString()));
        var projectile = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
            net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
        int projectileId = row.get("entity_id").getAsInt();
        projectile.setId(projectileId);
        projectile.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f32(row, "y_rot"), Oracle.f32(row, "x_rot"));
        projectile.setDeltaMovement(
            Oracle.f64(row, "motion_x"), Oracle.f64(row, "motion_y"), Oracle.f64(row, "motion_z"));
        if (row.has("owner")) {
            // `recreateFromPacket`: the packet's data names the owner.
            var owner = level.getEntity(row.get("owner").getAsInt());
            ((net.minecraft.world.entity.projectile.Projectile) projectile).setOwner(owner);
        }
        if (row.has("dangerous")) {
            ((net.minecraft.world.entity.projectile.hurtingprojectile.WitherSkull) projectile)
                .setDangerous(row.get("dangerous").getAsBoolean());
        }
        projectile.syncPacketPositionCodec(projectile.getX(), projectile.getY(), projectile.getZ());
        level.addEntity(projectile);
        var ticks = new JsonArray();
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            if (!projectile.isRemoved()) {
                level.tickNonPassenger(projectile);
            }
            var described = new JsonObject();
            described.addProperty("present", !projectile.isRemoved());
            if (projectile.isRemoved()) {
                // Gone: nothing more of it to compare.
                ticks.add(described);
                continue;
            }
            described.addProperty("x", Oracle.bits(projectile.getX()));
            described.addProperty("y", Oracle.bits(projectile.getY()));
            described.addProperty("z", Oracle.bits(projectile.getZ()));
            described.addProperty("motion_x", Oracle.bits(projectile.getDeltaMovement().x));
            described.addProperty("motion_y", Oracle.bits(projectile.getDeltaMovement().y));
            described.addProperty("motion_z", Oracle.bits(projectile.getDeltaMovement().z));
            described.addProperty("x_rot", Oracle.bits(projectile.getXRot()));
            described.addProperty("y_rot", Oracle.bits(projectile.getYRot()));
            described.addProperty("in_water", projectile.isInWater());
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    /**
     * Step the reference's own player through a scene, tick by tick.
     *
     * The keys held and the look direction are set before each tick, exactly
     * where the game's frame loop would have left them; then the level ticks
     * the player the way it ticks every entity. After every tick the position, the motion, the flags
     * and every packet the player asked to send are reported, the packets as
     * the bytes their own codec writes.
     */
    /** The renderer's shadow settings, with nothing else of a renderer built. */
    static final class ShadowProbe
        extends net.minecraft.client.renderer.entity.EntityRenderer<
            net.minecraft.world.entity.Entity, net.minecraft.client.renderer.entity.state.EntityRenderState> {
        ShadowProbe() {
            super(null);
        }

        @Override
        public net.minecraft.client.renderer.entity.state.EntityRenderState createRenderState() {
            return new net.minecraft.client.renderer.entity.state.EntityRenderState();
        }
    }

    /** What `ShadowFeatureRenderer` writes for each corner. */
    static final class ShadowCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final JsonArray vertices = new JsonArray();

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            var out = new JsonObject();
            out.addProperty("x", Oracle.bits(x));
            out.addProperty("y", Oracle.bits(y));
            out.addProperty("z", Oracle.bits(z));
            out.addProperty("color", color);
            out.addProperty("u", Oracle.bits(u));
            out.addProperty("v", Oracle.bits(v));
            this.vertices.add(out);
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            throw new UnsupportedOperationException();
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            throw new UnsupportedOperationException();
        }
    }

    /**
     * An entity's shadow over a lit scene: the pieces `extractShadow` finds
     * under it, and the corners `ShadowFeatureRenderer` writes for them.
     */
    /** The scene's level, lit as the client lights loaded chunks, as the game's current level. */
    static Scene litScene(JsonObject row) throws Exception {
        var scene = buildScene(row);
        var level = scene.level();
        var minecraft = scene.minecraft();
        setField(minecraft, "level", level);
        // The level's own light engine, lit as the client lights loaded chunks.
        var extractor = unsafe.allocateInstance(NoRebuilds.class);
        setField(level, "levelExtractor", extractor);
        setField(minecraft, "levelExtractor", extractor);
        var engine = level.getChunkSource().getLightEngine();
        int radius = row.get("chunk_radius").getAsInt();
        var chunks = new java.util.ArrayList<net.minecraft.world.level.ChunkPos>();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                chunks.add(new net.minecraft.world.level.ChunkPos(chunkX, chunkZ));
            }
        }
        for (var chunkPos : chunks) {
            var sections = level.getChunk(chunkPos.x(), chunkPos.z()).getSections();
            for (int index = 0; index < sections.length; index++) {
                if (!sections[index].hasOnlyAir()) {
                    engine.updateSectionStatus(net.minecraft.core.SectionPos.of(
                        chunkPos, level.getSectionYFromSectionIndex(index)), false);
                }
            }
        }
        for (var chunkPos : chunks) {
            engine.setLightEnabled(chunkPos, true);
        }
        for (var chunkPos : chunks) {
            engine.propagateLightSources(chunkPos);
        }
        engine.runLightUpdates();

        return scene;
    }

    static float bits(com.google.gson.JsonElement value) {
        return Float.intBitsToFloat(Integer.parseUnsignedInt(value.getAsString(), 16));
    }

    /**
     * The cardinal light a serialized dimension type decodes to through the
     * codec the client reads the registry with (`DimensionType.NETWORK_CODEC`
     * over NBT), or that it fails to decode: the row's field is absent, a
     * string, or another tag type.
     */
    static JsonObject dimensionCardinalLight(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var tag = new net.minecraft.nbt.CompoundTag();
        tag.putBoolean("has_skylight", true);
        tag.putBoolean("has_ceiling", false);
        tag.putBoolean("has_ender_dragon_fight", false);
        tag.putDouble("coordinate_scale", 1.0);
        tag.putInt("min_y", -64);
        tag.putInt("height", 384);
        tag.putInt("logical_height", 384);
        tag.putString("infiniburn", "#minecraft:infiniburn_overworld");
        tag.putFloat("ambient_light", 0.0F);
        tag.putInt("monster_spawn_light_level", 0);
        tag.putInt("monster_spawn_block_light_limit", 0);
        var cardinal = row.getAsJsonObject("cardinal");
        switch (cardinal.get("kind").getAsString()) {
            case "absent" -> { }
            case "string" -> tag.putString("cardinal_light", cardinal.get("value").getAsString());
            case "int" -> tag.putInt("cardinal_light", cardinal.get("value").getAsInt());
            case "byte" -> tag.putByte("cardinal_light", cardinal.get("value").getAsByte());
            case "list" -> tag.put("cardinal_light", new net.minecraft.nbt.ListTag());
            case "compound" -> tag.put("cardinal_light", new net.minecraft.nbt.CompoundTag());
            default -> throw new IllegalArgumentException("unknown cardinal kind");
        }
        var ops = net.minecraft.resources.RegistryOps.create(
            net.minecraft.nbt.NbtOps.INSTANCE, Oracle.vanillaRegistries());
        var parsed = net.minecraft.world.level.dimension.DimensionType.NETWORK_CODEC.parse(ops, tag);
        var result = new JsonObject();
        if (parsed.result().isPresent()) {
            result.addProperty("decoded", true);
            result.addProperty("cardinal", parsed.result().get().cardinalLightType().getSerializedName());
        } else {
            result.addProperty("decoded", false);
        }
        entry.add("result", result);
        return entry;
    }

    /**
     * Gives the level the nether's cardinal light when the row asks for it.
     * Only the face shade comes from the dimension type, which is what
     * `ClientLevel.cardinalLighting` reads. The overworld's type with the
     * nether's cardinal light keeps the level's height and lighting as built.
     */
    static void useCardinal(net.minecraft.client.multiplayer.ClientLevel level, JsonObject row) throws Exception {
        if (row.has("cardinal") && row.get("cardinal").getAsString().equals("nether")) {
            var base = level.dimensionType();
            var changed = new net.minecraft.world.level.dimension.DimensionType(
                base.hasFixedTime(), base.hasSkyLight(), base.hasCeiling(),
                base.hasEnderDragonFight(), base.coordinateScale(), base.minY(),
                base.height(), base.logicalHeight(), base.infiniburn(), base.ambientLight(),
                base.monsterSettings(), base.skybox(),
                net.minecraft.world.level.CardinalLighting.Type.NETHER, base.attributes(),
                base.timelines(), base.defaultClock());
            setField(level, "dimensionTypeRegistration",
                net.minecraft.core.Holder.direct(changed));
        }
    }

    /**
     * What `FluidRenderer` emits for the fluid block at the row's position:
     * every vertex's place, colour and light, the sprite's own coordinates
     * left out (a blank sprite stands in for the atlas).
     */
    static JsonObject fluidMesh(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = litScene(row);
        var level = scene.level();
        useCardinal(level, row);
        // The models the game bakes, over sprites that report the alpha the row
        // gives them (an opaque one when it says nothing).
        var baker = new SpriteBaker(row.has("sprites") ? row.getAsJsonObject("sprites") : new JsonObject());
        var models = net.minecraft.client.renderer.block.FluidStateModelSet.bake(baker);
        var lava = models.get(net.minecraft.world.level.material.Fluids.LAVA);
        var renderer = new net.minecraft.client.renderer.block.FluidRenderer(
            new net.minecraft.client.renderer.block.FluidStateModelSet(models, lava));
        var pos = new net.minecraft.core.BlockPos(
            row.get("x").getAsInt(), row.get("y").getAsInt(), row.get("z").getAsInt());
        var state = level.getBlockState(pos);
        var vertices = new com.google.gson.JsonArray();
        var builder = new RecordingVertices(vertices);
        var layers = new java.util.TreeSet<String>();
        renderer.tesselate(level, pos, layer -> {
            // Cutout shares the opaque pass here.
            layers.add(layer == net.minecraft.client.renderer.chunk.ChunkSectionLayer.TRANSLUCENT
                ? "translucent" : "opaque");
            return builder;
        }, state, state.getFluidState());
        entry.addProperty("layer", String.join(",", layers));
        // Which vertices there are, not in what order or how often: the
        // surface seen from below is a second quad here, shared vertices there.
        entry.addProperty("quads", vertices.size() / 4);
        var distinct = new java.util.TreeMap<String, com.google.gson.JsonElement>();
        for (var vertex : vertices) {
            var described = vertex.getAsJsonObject();
            distinct.put(described.get("position") + ":" + described.get("color") + ":"
                + described.get("block_light") + ":" + described.get("sky_light"), vertex);
        }
        var sorted = new com.google.gson.JsonArray();
        distinct.values().forEach(sorted::add);
        entry.add("vertices", sorted);
        return entry;
    }

    /**
     * The order translucent faces are drawn in (`VertexSorting.byDistance`
     * over the faces' middles), and which neighbour of a section the camera
     * stands in (`TranslucencyPointOfView`).
     */
    static JsonObject translucentOrder(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var camera = row.getAsJsonArray("camera");
        var centroids = row.getAsJsonArray("centroids");
        var points = new com.mojang.blaze3d.vertex.CompactVectorArray(centroids.size());
        for (int index = 0; index < centroids.size(); index++) {
            var at = centroids.get(index).getAsJsonArray();
            points.set(index, bits(at.get(0)), bits(at.get(1)), bits(at.get(2)));
        }
        var order = com.mojang.blaze3d.vertex.VertexSorting
            .byDistance(bits(camera.get(0)), bits(camera.get(1)), bits(camera.get(2))).sort(points);
        var sorted = new com.google.gson.JsonArray();
        for (int index : order) {
            sorted.add(index);
        }
        entry.add("order", sorted);
        return entry;
    }

    static JsonObject translucentView(JsonObject row) {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var camera = row.getAsJsonArray("camera");
        var section = row.getAsJsonArray("section");
        var view = net.minecraft.client.renderer.chunk.TranslucencyPointOfView.of(
            new net.minecraft.world.phys.Vec3(camera.get(0).getAsDouble(), camera.get(1).getAsDouble(),
                camera.get(2).getAsDouble()),
            net.minecraft.core.SectionPos.asLong(section.get(0).getAsInt(), section.get(1).getAsInt(),
                section.get(2).getAsInt()));
        var fields = new com.google.gson.JsonArray();
        for (var name : new String[] {"x", "y", "z"}) {
            try {
                var field = view.getClass().getDeclaredField(name);
                field.setAccessible(true);
                fields.add(field.getInt(view));
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException(error);
            }
        }
        entry.add("view", fields);
        entry.addProperty("aligned", view.isAxisAligned());
        return entry;
    }

    /** A sprite over real contents, which is what a face bakes its layer from. */
    static final class ContentSprite extends net.minecraft.client.renderer.texture.TextureAtlasSprite {
        ContentSprite(net.minecraft.client.renderer.texture.SpriteContents contents) {
            super(net.minecraft.client.renderer.texture.TextureAtlas.LOCATION_BLOCKS, contents, 16, 16, 0, 0, 0);
        }
    }

    /**
     * The layer a face bakes to (`FaceBakery.computeMaterialTransparency` into
     * `MaterialInfo.of`), over a sprite whose pixels the row gives as alphas:
     * a still image, or a strip of frames played in the row's order.
     */
    static JsonObject faceLayer(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        int width = row.get("width").getAsInt();
        int height = row.get("height").getAsInt();
        int frameWidth = row.get("frame_width").getAsInt();
        int frameHeight = row.get("frame_height").getAsInt();
        var alphas = row.getAsJsonArray("alphas");
        var image = new com.mojang.blaze3d.platform.NativeImage(width, height, true);
        for (int y = 0; y < height; y++) {
            for (int x = 0; x < width; x++) {
                image.setPixel(x, y, alphas.get(y * width + x).getAsInt() << 24 | 0x336699);
            }
        }
        java.util.Optional<net.minecraft.client.resources.metadata.animation.AnimationMetadataSection> animation =
            java.util.Optional.empty();
        if (row.has("playback")) {
            var frames = new java.util.ArrayList<net.minecraft.client.resources.metadata.animation.AnimationFrame>();
            for (var index : row.getAsJsonArray("playback")) {
                frames.add(new net.minecraft.client.resources.metadata.animation.AnimationFrame(
                    index.getAsInt(), java.util.Optional.empty()));
            }
            animation = java.util.Optional.of(
                new net.minecraft.client.resources.metadata.animation.AnimationMetadataSection(
                    java.util.Optional.of(frames), java.util.Optional.of(frameWidth),
                    java.util.Optional.of(frameHeight), 1, false));
        }
        var contents = new net.minecraft.client.renderer.texture.SpriteContents(
            net.minecraft.resources.Identifier.parse("minecraft:probe"),
            new net.minecraft.client.resources.metadata.animation.FrameSize(frameWidth, frameHeight),
            image, animation, java.util.List.of(), java.util.Optional.empty());
        try {
            var material = new net.minecraft.client.resources.model.sprite.Material.Baked(
                new ContentSprite(contents), row.get("force_translucent").getAsBoolean());
            var uvs = row.getAsJsonArray("uvs");
            var method = net.minecraft.client.resources.model.cuboid.FaceBakery.class.getDeclaredMethod(
                "computeMaterialTransparency",
                net.minecraft.client.resources.model.sprite.Material.Baked.class,
                net.minecraft.client.resources.model.cuboid.CuboidFace.UVs.class);
            method.setAccessible(true);
            try {
                var transparency = (com.mojang.blaze3d.platform.Transparency) method.invoke(null, material,
                    new net.minecraft.client.resources.model.cuboid.CuboidFace.UVs(
                        bits(uvs.get(0)), bits(uvs.get(1)), bits(uvs.get(2)), bits(uvs.get(3))));
                entry.addProperty("outcome", "layer");
                entry.addProperty("layer", net.minecraft.client.renderer.chunk.ChunkSectionLayer
                    .byTransparency(transparency).name().toLowerCase(java.util.Locale.ROOT));
            } catch (java.lang.reflect.InvocationTargetException failure) {
                entry.addProperty("outcome", "error");
            }
        } finally {
            contents.close();
        }
        return entry;
    }

    /** A sprite that says how much of it is see-through, whatever its size. */
    static final class AlphaSprite extends net.minecraft.client.renderer.texture.TextureAtlasSprite {
        com.mojang.blaze3d.platform.Transparency transparency;

        AlphaSprite() {
            super(null, null, 1, 1, 0, 0, 0);
        }

        @Override
        public com.mojang.blaze3d.platform.Transparency transparency() {
            return transparency;
        }
    }

    /**
     * Bakes each material to an `AlphaSprite` whose alpha the row names by the
     * sprite's last path part (`translucent` or `transparent`).
     */
    static final class SpriteBaker extends net.minecraft.client.resources.model.sprite.MaterialBaker {
        private final JsonObject alphas;

        SpriteBaker(JsonObject alphas) {
            super(null);
            this.alphas = alphas;
        }

        @Override
        protected net.minecraft.client.resources.model.sprite.Material.Baked bake(
            net.minecraft.client.resources.model.sprite.Material material) {
            try {
                var sprite = (AlphaSprite) unsafe.allocateInstance(AlphaSprite.class);
                var name = material.sprite().getPath().substring("block/".length());
                var alpha = alphas.has(name) ? alphas.get(name).getAsString() : "opaque";
                sprite.transparency = new com.mojang.blaze3d.platform.Transparency(
                    alpha.equals("transparent"), alpha.equals("translucent"));
                return new net.minecraft.client.resources.model.sprite.Material.Baked(
                    sprite, material.forceTranslucent());
            } catch (InstantiationException error) {
                throw new IllegalStateException(error);
            }
        }
    }

    /** Collects each vertex a renderer adds: position bits, colour and packed light. */
    static final class RecordingVertices implements com.mojang.blaze3d.vertex.VertexConsumer {
        private final com.google.gson.JsonArray out;
        private JsonObject vertex;

        RecordingVertices(com.google.gson.JsonArray out) {
            this.out = out;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(float x, float y, float z) {
            vertex = new JsonObject();
            var position = new com.google.gson.JsonArray();
            for (float part : new float[] {x, y, z}) {
                position.add(Integer.toHexString(Float.floatToRawIntBits(part)));
            }
            vertex.add("position", position);
            out.add(vertex);
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setColor(int r, int g, int b, int a) {
            return setColor(a << 24 | r << 16 | g << 8 | b);
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setColor(int color) {
            vertex.addProperty("color", Integer.toUnsignedLong(color));
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setUv(float u, float v) {
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(int u, int v) {
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(int u, int v) {
            vertex.addProperty("block_light", u);
            vertex.addProperty("sky_light", v);
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(float x, float y, float z) {
            return this;
        }

        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(float width) {
            return this;
        }
    }

    /**
     * The client chunk's heightmaps (`ChunkAccess.getHeight`) at the columns a row probes, after
     * blocks are changed the way a block update packet changes them.
     */
    static JsonObject chunkHeightmaps(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var level = buildScene(row).level();
        if (row.has("edits")) {
            for (var element : row.getAsJsonArray("edits")) {
                var described = element.getAsJsonObject();
                var pos = new net.minecraft.core.BlockPos(described.get("x").getAsInt(),
                    described.get("y").getAsInt(), described.get("z").getAsInt());
                level.setServerVerifiedBlockState(pos, state(described), 19);
            }
        }
        var probes = new JsonArray();
        for (var element : row.getAsJsonArray("probes")) {
            var column = element.getAsJsonObject();
            int x = column.get("x").getAsInt();
            int z = column.get("z").getAsInt();
            var chunk = level.getChunk(Math.floorDiv(x, 16), Math.floorDiv(z, 16));
            var heights = new JsonObject();
            for (var type : java.util.List.of(
                net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE,
                net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING,
                net.minecraft.world.level.levelgen.Heightmap.Types.MOTION_BLOCKING_NO_LEAVES)) {
                heights.addProperty(type.name(), chunk.getHeight(type, x, z));
            }
            probes.add(heights);
        }
        entry.add("probes", probes);
        return entry;
    }

    /** `BlockModelLighter` on quads the row describes, in the level the row builds. */
    static JsonObject blockLighting(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = litScene(row);
        var level = scene.level();
        useCardinal(level, row);
        var lighter = new net.minecraft.client.renderer.block.BlockModelLighter();
        var quads = new JsonArray();
        for (var element : row.getAsJsonArray("quads")) {
            var described = element.getAsJsonObject();
            var pos = new net.minecraft.core.BlockPos(described.get("x").getAsInt(),
                described.get("y").getAsInt(), described.get("z").getAsInt());
            var state = level.getBlockState(pos);
            var direction = net.minecraft.core.Direction.byName(described.get("direction").getAsString());
            var corners = described.getAsJsonArray("corners");
            var positions = new org.joml.Vector3f[4];
            for (int corner = 0; corner < 4; corner++) {
                var values = corners.get(corner).getAsJsonArray();
                positions[corner] = new org.joml.Vector3f(bits(values.get(0)), bits(values.get(1)), bits(values.get(2)));
            }
            var material = new net.minecraft.client.resources.model.geometry.BakedQuad.MaterialInfo(
                null, null, null, -1, described.get("shade").getAsBoolean(), 0);
            var quad = new net.minecraft.client.resources.model.geometry.BakedQuad(
                positions[0], positions[1], positions[2], positions[3], 0L, 0L, 0L, 0L, direction, material);
            var instance = new com.mojang.blaze3d.vertex.QuadInstance();
            if (described.get("smooth").getAsBoolean()) {
                lighter.prepareQuadAmbientOcclusion(level, state, pos, quad, instance);
            } else {
                int lightCoords = -1;
                if (described.has("cull") && !described.get("cull").isJsonNull()) {
                    var cull = net.minecraft.core.Direction.byName(described.get("cull").getAsString());
                    var relative = pos.relative(cull);
                    lightCoords = lighter.getLightCoords(state, level, relative);
                }
                lighter.prepareQuadFlat(level, state, pos, lightCoords, quad, instance);
            }
            var vertices = new JsonArray();
            for (int vertex = 0; vertex < 4; vertex++) {
                var out = new JsonObject();
                out.addProperty("grey", instance.getColor(vertex) & 0xFF);
                out.addProperty("light", instance.getLightCoords(vertex));
                vertices.add(out);
            }
            var result = new JsonObject();
            result.add("label", described.get("label"));
            result.add("vertices", vertices);
            quads.add(result);
        }
        entry.add("quads", quads);
        return entry;
    }

    /**
     * `ModelBlockRenderer.tesselateBlock`'s choice between smooth and flat
     * lighting: a block with parts the row describes goes through the real
     * renderer, and each quad it emits is matched against what the lighter
     * gives for smooth and for flat.
     */
    static JsonObject blockSmoothGate(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = litScene(row);
        var level = scene.level();
        var pos = new net.minecraft.core.BlockPos(0, 64, 0);
        var state = level.getBlockState(pos);
        var sources = new java.util.ArrayList<net.minecraft.client.resources.model.geometry.BakedQuad>();
        var parts = new java.util.ArrayList<net.minecraft.client.renderer.block.dispatch.BlockStateModelPart>();
        for (var element : row.getAsJsonArray("parts")) {
            boolean useAmbientOcclusion = element.getAsBoolean();
            var quad = partialUpFace();
            sources.add(quad);
            parts.add(new net.minecraft.client.renderer.block.dispatch.BlockStateModelPart() {
                public java.util.List<net.minecraft.client.resources.model.geometry.BakedQuad> getQuads(
                    net.minecraft.core.Direction direction) {
                    return direction == null ? java.util.List.of(quad) : java.util.List.of();
                }
                public boolean useAmbientOcclusion() {
                    return useAmbientOcclusion;
                }
                public net.minecraft.client.resources.model.sprite.Material.Baked particleMaterial() {
                    return null;
                }
                public int materialFlags() {
                    return 0;
                }
            });
        }
        var model = new net.minecraft.client.renderer.block.dispatch.BlockStateModel() {
            public void collectParts(net.minecraft.util.RandomSource random,
                java.util.List<net.minecraft.client.renderer.block.dispatch.BlockStateModelPart> output) {
                output.addAll(parts);
            }
            public net.minecraft.client.resources.model.sprite.Material.Baked particleMaterial() {
                return null;
            }
            public int materialFlags() {
                return 0;
            }
        };
        var renderer = new net.minecraft.client.renderer.block.ModelBlockRenderer(
            row.get("option").getAsBoolean(), true, null);
        var lighter = new net.minecraft.client.renderer.block.BlockModelLighter();
        var seen = new java.util.ArrayList<int[]>();
        renderer.tesselateBlock((x, y, z, quad, instance) -> {
            var got = new int[8];
            for (int vertex = 0; vertex < 4; vertex++) {
                got[vertex] = instance.getColor(vertex);
                got[4 + vertex] = instance.getLightCoords(vertex);
            }
            seen.add(got);
        }, 0.0F, 0.0F, 0.0F, level, pos, state, model, 0L);
        if (seen.size() != parts.size()) {
            throw new IllegalStateException("the renderer drew " + seen.size() + " quads for " + parts.size() + " parts");
        }
        Boolean smooth = null;
        for (int index = 0; index < seen.size(); index++) {
            var instance = new com.mojang.blaze3d.vertex.QuadInstance();
            lighter.prepareQuadAmbientOcclusion(level, state, pos, sources.get(index), instance);
            boolean isSmooth = matches(seen.get(index), instance);
            lighter.prepareQuadFlat(level, state, pos, -1, sources.get(index), instance);
            boolean isFlat = matches(seen.get(index), instance);
            if (isSmooth == isFlat) {
                throw new IllegalStateException("smooth and flat light the row's quad the same way: " + row.get("label"));
            }
            if (smooth != null && smooth != isSmooth) {
                throw new IllegalStateException("the parts of one block took different lighting");
            }
            smooth = isSmooth;
        }
        entry.addProperty("smooth", smooth != null && smooth);
        return entry;
    }

    /** A lopsided face on top of the block, where smooth and flat lighting differ. */
    static net.minecraft.client.resources.model.geometry.BakedQuad partialUpFace() {
        var positions = new org.joml.Vector3f[] {
            new org.joml.Vector3f(0.0F, 1.0F, 0.0F), new org.joml.Vector3f(0.0F, 1.0F, 0.5F),
            new org.joml.Vector3f(0.5F, 1.0F, 0.5F), new org.joml.Vector3f(0.5F, 1.0F, 0.0F)};
        var material = new net.minecraft.client.resources.model.geometry.BakedQuad.MaterialInfo(
            null, null, null, -1, true, 0);
        return new net.minecraft.client.resources.model.geometry.BakedQuad(
            positions[0], positions[1], positions[2], positions[3], 0L, 0L, 0L, 0L,
            net.minecraft.core.Direction.UP, material);
    }

    static boolean matches(int[] seen, com.mojang.blaze3d.vertex.QuadInstance instance) {
        for (int vertex = 0; vertex < 4; vertex++) {
            if (seen[vertex] != instance.getColor(vertex) || seen[4 + vertex] != instance.getLightCoords(vertex)) {
                return false;
            }
        }
        return true;
    }

    static JsonObject entityShadows(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = litScene(row);
        var level = scene.level();
        var minecraft = scene.minecraft();

        var renderer = (ShadowProbe) unsafe.allocateInstance(ShadowProbe.class);
        setField(renderer, "shadowRadius", Oracle.f32(row, "radius"));
        setField(renderer, "shadowStrength", Oracle.f32(row, "strength"));
        var state = new net.minecraft.client.renderer.entity.state.EntityRenderState();
        state.x = Oracle.f64(row, "x");
        state.y = Oracle.f64(row, "y");
        state.z = Oracle.f64(row, "z");
        state.distanceToCameraSq = Oracle.f64(row, "distance_sq");
        state.isInvisible = row.get("invisible").getAsBoolean();
        scene.options().entityShadows().set(row.get("shadows_option").getAsBoolean());
        var extract = net.minecraft.client.renderer.entity.EntityRenderer.class.getDeclaredMethod(
            "extractShadow", net.minecraft.client.renderer.entity.state.EntityRenderState.class,
            net.minecraft.client.Minecraft.class, net.minecraft.world.level.Level.class);
        extract.setAccessible(true);
        extract.invoke(renderer, state, minecraft, level);

        var pieces = new JsonArray();
        for (var piece : state.shadowPieces) {
            var bounds = piece.shapeBelow().bounds();
            var described = new JsonObject();
            described.addProperty("rx", Oracle.bits(piece.relativeX()));
            described.addProperty("ry", Oracle.bits(piece.relativeY()));
            described.addProperty("rz", Oracle.bits(piece.relativeZ()));
            described.addProperty("min", Oracle.bits((float) bounds.minX) + Oracle.bits((float) bounds.minY)
                + Oracle.bits((float) bounds.minZ));
            described.addProperty("max", Oracle.bits((float) bounds.maxX) + Oracle.bits((float) bounds.maxY)
                + Oracle.bits((float) bounds.maxZ));
            described.addProperty("alpha", Oracle.bits(piece.alpha()));
            pieces.add(described);
        }
        entry.add("pieces", pieces);
        entry.addProperty("radius", Oracle.bits(state.shadowRadius));

        var capture = new ShadowCapture();
        var feature = unsafe.allocateInstance(net.minecraft.client.renderer.feature.ShadowFeatureRenderer.class);
        var prepare = net.minecraft.client.renderer.feature.ShadowFeatureRenderer.class.getDeclaredMethod(
            "prepare", net.minecraft.client.renderer.feature.ShadowFeatureRenderer.Submit.class,
            com.mojang.blaze3d.vertex.VertexConsumer.class);
        prepare.setAccessible(true);
        prepare.invoke(feature, new net.minecraft.client.renderer.feature.ShadowFeatureRenderer.Submit(
            new org.joml.Matrix4f().translation(
                Oracle.f32(row, "tx"), Oracle.f32(row, "ty"), Oracle.f32(row, "tz")),
            state.shadowRadius, state.shadowPieces), capture);
        entry.add("vertices", capture.vertices);
        return entry;
    }
}
