import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;

/**
 * Test glue only: runs the original client player in a world the test builds.
 *
 * <p>The player here is the reference's own {@code LocalPlayer}, in the
 * reference's own {@code ClientLevel}, with the reference's own keyboard
 * input. Registries and tags come from the vanilla data pack through the same
 * loading path the server uses, and chunks arrive through the client chunk
 * cache's packet entry point. What this file supplies is only what a headless
 * run has no other way to get: the game singleton, an options object, a
 * connection that records what it is asked to send rather than sending it,
 * and a sound manager that plays nothing.
 */
public final class MovementOracle {
    private static boolean bootstrapped;
    private static net.minecraft.core.RegistryAccess.Frozen registries;
    private static sun.misc.Unsafe unsafe;

    /** Records what the player sends instead of putting it on a wire. */
    static final class CapturingConnection extends net.minecraft.network.Connection {
        final java.util.List<net.minecraft.network.protocol.Packet<?>> sent =
            new java.util.ArrayList<>();

        CapturingConnection() {
            super(net.minecraft.network.protocol.PacketFlow.CLIENTBOUND);
        }

        @Override
        public void send(net.minecraft.network.protocol.Packet<?> packet) {
            this.sent.add(packet);
        }
    }

    /** Swallows sounds: movement plays step sounds, and there is no speaker. */
    static final class SilentSoundManager extends net.minecraft.client.sounds.SoundManager {
        private SilentSoundManager() {
            super(null);
        }

        @Override
        public net.minecraft.client.sounds.SoundEngine.PlayResult play(
            net.minecraft.client.resources.sounds.SoundInstance instance) {
            return net.minecraft.client.sounds.SoundEngine.PlayResult.NOT_STARTED;
        }

        @Override
        public void playDelayed(
            net.minecraft.client.resources.sounds.SoundInstance instance, int delay) {}
    }

    /**
     * Breaking a block spawns terrain particles, which look up the block's
     * sprite and tint. A headless run has no atlas, so these stand in: every
     * block's particle sprite is one blank terrain sprite, and particles are
     * dropped when added. Nothing breaking decides reads them back.
     */
    static final class BlankSprite extends net.minecraft.client.renderer.texture.TextureAtlasSprite {
        private BlankSprite() {
            super(null, null, 1, 1, 0, 0, 0);
        }

        @Override
        public com.mojang.blaze3d.platform.Transparency transparency() {
            return new com.mojang.blaze3d.platform.Transparency(false, false);
        }

        @Override
        public net.minecraft.resources.Identifier atlasLocation() {
            return net.minecraft.client.renderer.texture.TextureAtlas.LOCATION_BLOCKS;
        }
    }

    static final class BlankModels extends net.minecraft.client.renderer.block.BlockStateModelSet {
        static net.minecraft.client.renderer.texture.TextureAtlasSprite sprite;

        private BlankModels() {
            super(null, null);
        }

        @Override
        public net.minecraft.client.resources.model.sprite.Material.Baked getParticleMaterial(
            net.minecraft.world.level.block.state.BlockState state) {
            return new net.minecraft.client.resources.model.sprite.Material.Baked(sprite, false);
        }
    }

    static final class BlankModelManager extends net.minecraft.client.resources.model.ModelManager {
        static net.minecraft.client.renderer.block.BlockStateModelSet models;

        private BlankModelManager() {
            super(null, null, null);
        }

        @Override
        public net.minecraft.client.renderer.block.BlockStateModelSet getBlockStateModelSet() {
            return models;
        }
    }

    static final class DroppingParticleEngine extends net.minecraft.client.particle.ParticleEngine {
        private DroppingParticleEngine() {
            super(null, null);
        }

        @Override
        public void add(net.minecraft.client.particle.Particle particle) {}

        @Override
        public net.minecraft.client.particle.Particle createParticle(
            net.minecraft.core.particles.ParticleOptions options,
            double x, double y, double z, double xd, double yd, double zd) {
            return null;
        }
    }

    /** The level tells the renderer which sections to rebuild; nothing draws here. */
    static final class NoRebuilds extends net.minecraft.client.renderer.extract.LevelExtractor {
        private NoRebuilds() {
            super(null, null, null);
        }

        @Override
        public void blockChanged(BlockPos pos, int updateFlags) {}

        @Override
        public void setBlockDirty(BlockPos pos,
            net.minecraft.world.level.block.state.BlockState oldState,
            net.minecraft.world.level.block.state.BlockState newState) {}

        @Override
        public void setSectionDirtyWithNeighbors(int sectionX, int sectionY, int sectionZ) {}

        @Override
        public void setSectionRangeDirty(int minX, int minY, int minZ, int maxX, int maxY, int maxZ) {}

        @Override
        public void setSectionDirty(int sectionX, int sectionY, int sectionZ) {}
    }

    private static Object getField(Object target, String name) throws Exception {
        Class<?> type = target.getClass();
        while (type != null) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(target);
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchFieldException(name);
    }

    static void setField(Object target, String name, Object value) throws Exception {
        Class<?> type = target.getClass();
        while (type != null) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                field.set(target, value);
                return;
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        throw new NoSuchFieldException(name);
    }

    /** Bring the game up once, with the vanilla data pack's registries and tags. */
    static synchronized void bootstrap() throws Exception {
        if (bootstrapped) {
            return;
        }
        registries = Oracle.vanillaRegistries();
        var unsafeField = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
        unsafeField.setAccessible(true);
        unsafe = (sun.misc.Unsafe) unsafeField.get(null);
        bootstrapped = true;
    }

    public static JsonObject run(JsonObject request) throws Exception {
        bootstrap();
        String operation = request.get("operation").getAsString();
        var rows = new JsonArray();
        for (var element : request.get("rows").getAsJsonArray()) {
            switch (operation) {
                case "local_player_ticks" -> rows.add(localPlayerTicks(element.getAsJsonObject()));
                case "item_entity_ticks" -> rows.add(itemEntityTicks(element.getAsJsonObject()));
                case "living_entity_ticks" -> rows.add(livingEntityTicks(element.getAsJsonObject()));
                case "entity_shadows" -> rows.add(entityShadows(element.getAsJsonObject()));
                case "entity_pick_table" -> rows.add(entityPickTable(element.getAsJsonObject()));
                case "light_changes" -> rows.add(lightChanges(element.getAsJsonObject()));
                case "container_clicks" -> rows.add(containerClicks(element.getAsJsonObject()));
                default -> throw new IllegalArgumentException(
                    "Unknown movement operation: " + operation);
            }
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    private static net.minecraft.world.level.block.state.BlockState state(JsonObject described) {
        var block = net.minecraft.core.registries.BuiltInRegistries.BLOCK.getValue(
            net.minecraft.resources.Identifier.parse(described.get("block").getAsString()));
        var state = block.defaultBlockState();
        if (described.has("properties")) {
            for (var entry : described.getAsJsonObject("properties").entrySet()) {
                state = withProperty(state, entry.getKey(), entry.getValue().getAsString());
            }
        }
        return state;
    }

    @SuppressWarnings({"unchecked", "rawtypes"})
    private static net.minecraft.world.level.block.state.BlockState withProperty(
        net.minecraft.world.level.block.state.BlockState state, String name, String value) {
        var property = state.getBlock().getStateDefinition().getProperty(name);
        if (property == null) {
            throw new IllegalArgumentException("No property " + name);
        }
        var parsed = property.getValue(value)
            .orElseThrow(() -> new IllegalArgumentException("Bad value " + value));
        return state.setValue((net.minecraft.world.level.block.state.properties.Property) property,
            (Comparable) parsed);
    }

    /** A client level with the scene's blocks, and the game around it. */
    record Scene(
        net.minecraft.client.multiplayer.ClientPacketListener listener,
        CapturingConnection wire,
        net.minecraft.client.multiplayer.ClientLevel level,
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.Options options) {}

    static Scene buildScene(JsonObject row) throws Exception {
        var listener = (net.minecraft.client.multiplayer.ClientPacketListener)
            unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
        setField(listener, "registryAccess", registries);
        setField(listener, "clockManager", new net.minecraft.client.ClientClockManager());
        setField(listener, "localGameProfile", new com.mojang.authlib.GameProfile(
            java.util.UUID.nameUUIDFromBytes(new byte[] {1}), "Parity"));
        var wire = new CapturingConnection();
        setField(listener, "connection", wire);
        setField(listener, "clientLoaded", true);

        var dimensionType = registries.lookupOrThrow(
            net.minecraft.core.registries.Registries.DIMENSION_TYPE).getOrThrow(
            net.minecraft.world.level.dimension.BuiltinDimensionTypes.OVERWORLD);
        var level = new net.minecraft.client.multiplayer.ClientLevel(
            listener,
            new net.minecraft.client.multiplayer.ClientLevel.ClientLevelData(
                net.minecraft.world.Difficulty.PEACEFUL, false, false),
            net.minecraft.world.level.Level.OVERWORLD, dimensionType, 8, 8, null, false, 0L, 63);

        var minecraft = (net.minecraft.client.Minecraft)
            unsafe.allocateInstance(net.minecraft.client.Minecraft.class);
        var instanceField = net.minecraft.client.Minecraft.class.getDeclaredField("instance");
        instanceField.setAccessible(true);
        instanceField.set(null, minecraft);
        setField(level, "minecraft", minecraft);
        var workDir = java.nio.file.Files.createTempDirectory("oracle-options").toFile();
        var options = new net.minecraft.client.Options(minecraft, workDir);
        options.autoJump().set(row.has("auto_jump") && row.get("auto_jump").getAsBoolean());
        setField(minecraft, "options", options);
        setField(minecraft, "gui", unsafe.allocateInstance(net.minecraft.client.gui.Gui.class));
        // `hasControlDown` asks the window's keys: a window with no keys down.
        var window = unsafe.allocateInstance(com.mojang.blaze3d.platform.Window.class);
        setField(window, "handle", 1L);
        setField(minecraft, "window", window);
        setField(minecraft, "tutorial", new net.minecraft.client.tutorial.Tutorial(minecraft, options));
        setField(minecraft, "soundManager", unsafe.allocateInstance(SilentSoundManager.class));

        // Chunks go in through the client cache's packet entry point, as the
        // sections the reference's own section code builds and writes.
        var blocks = new java.util.HashMap<Long, net.minecraft.world.level.block.state.BlockState>();
        for (var element : row.get("blocks").getAsJsonArray()) {
            var described = element.getAsJsonObject();
            blocks.put(BlockPos.asLong(described.get("x").getAsInt(), described.get("y").getAsInt(),
                described.get("z").getAsInt()), state(described));
        }
        int radius = row.get("chunk_radius").getAsInt();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                var buffer = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
                for (int index = 0; index < level.getSectionsCount(); index++) {
                    var section = new net.minecraft.world.level.chunk.LevelChunkSection(
                        level.palettedContainerFactory());
                    int sectionMinY = level.getMinY() + index * 16;
                    for (int x = 0; x < 16; x++) {
                        for (int y = 0; y < 16; y++) {
                            for (int z = 0; z < 16; z++) {
                                var state = blocks.get(BlockPos.asLong(
                                    chunkX * 16 + x, sectionMinY + y, chunkZ * 16 + z));
                                if (state != null) {
                                    section.setBlockState(x, y, z, state);
                                }
                            }
                        }
                    }
                    section.write(buffer);
                }
                level.getChunkSource().replaceWithPacketData(
                    chunkX, chunkZ, buffer, java.util.Map.of(), output -> {});
            }
        }

        return new Scene(listener, wire, level, minecraft, options);
    }

    /**
     * Step the reference's own dropped item through a scene, tick by tick,
     * the way the client level ticks an entity it did not spawn itself: the
     * server's packets set its place and motion, the level does the rest.
     */
    private static JsonObject itemEntityTicks(JsonObject row) throws Exception {
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
        private ShadowProbe() {
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
    private static JsonObject entityShadows(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
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

    /**
     * A living entity of the reference through the packets that hurt or kill
     * it and the client ticks after them: `hurtTime`, `deathTime`, whether
     * its health is spent, and the limbs' walk animation.
     */
    private static JsonObject livingEntityTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        var listener = scene.listener();
        setField(scene.minecraft(), "packetProcessor",
            new net.minecraft.network.PacketProcessor(Thread.currentThread()));
        setField(listener, "level", level);
        setField(listener, "minecraft", scene.minecraft());
        // The level pushes entities against the client's player, and lets a
        // dying one fall over only near it: one a few blocks away.
        var player = new net.minecraft.client.player.LocalPlayer(
            scene.minecraft(), level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        player.snapTo(row.has("player_x") ? Oracle.f64(row, "player_x") : 6.5, 64.0, 0.5, 0.0F, 0.0F);
        setField(scene.minecraft(), "player", player);
        setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("entity").getAsString()));
        var entity = (net.minecraft.world.entity.LivingEntity) type.create(level,
            new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
        int entityId = row.get("entity_id").getAsInt();
        entity.setId(entityId);
        entity.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"), 0.0F, 0.0F);
        entity.syncPacketPositionCodec(entity.getX(), entity.getY(), entity.getZ());
        level.addEntity(entity);
        var ticks = new JsonArray();
        for (int tick = 0; tick < row.get("tick_count").getAsInt(); tick++) {
            for (var element : row.getAsJsonArray("events")) {
                var event = element.getAsJsonObject();
                if (event.get("tick").getAsInt() != tick) {
                    continue;
                }
                switch (event.get("type").getAsString()) {
                    case "damage" -> listener.handleDamageEvent(
                        new net.minecraft.network.protocol.game.ClientboundDamageEventPacket(
                            entity, level.damageSources().generic()));
                    case "hurt" -> listener.handleHurtAnimation(
                        new net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket(entity));
                    case "death" -> listener.handleEntityEvent(
                        new net.minecraft.network.protocol.game.ClientboundEntityEventPacket(entity, (byte) 3));
                    case "move" -> listener.handleMoveEntity(
                        new net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.Pos(entityId,
                            (short) event.get("xa").getAsInt(), (short) event.get("ya").getAsInt(),
                            (short) event.get("za").getAsInt(),
                            !event.has("on_ground") || event.get("on_ground").getAsBoolean()));
                    case "animate" -> listener.handleAnimate(
                        new net.minecraft.network.protocol.game.ClientboundAnimatePacket(
                            entity, event.get("action").getAsInt()));
                    case "creeper_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("swell_dir")) {
                            data.add(creeperData("DATA_SWELL_DIR", event.get("swell_dir").getAsInt()));
                        }
                        if (event.has("ignited")) {
                            data.add(creeperData("DATA_IS_IGNITED", event.get("ignited").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "game_time" -> level.setTimeFromServer(event.get("time").getAsLong());
                    case "camel_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var camel = net.minecraft.world.entity.animal.camel.Camel.class;
                        if (event.has("dash")) {
                            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                                net.minecraft.world.entity.animal.camel.Camel.DASH, event.get("dash").getAsBoolean()));
                        }
                        if (event.has("last_pose")) {
                            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                                net.minecraft.world.entity.animal.camel.Camel.LAST_POSE_CHANGE_TICK,
                                event.get("last_pose").getAsLong()));
                        }
                        if (event.has("pose")) {
                            var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
                            pose.setAccessible(true);
                            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                                (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
                                net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "horse_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var flags = net.minecraft.world.entity.animal.equine.AbstractHorse.class.getDeclaredField("DATA_ID_FLAGS");
                        flags.setAccessible(true);
                        data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                            (net.minecraft.network.syncher.EntityDataAccessor<Byte>) flags.get(null),
                            (byte) event.get("flags").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "seed" -> {
                        // What the reference draws from an unseeded random source: fixed here.
                        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
                        random.setAccessible(true);
                        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
                    }
                    case "bee_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("flags")) {
                            data.add(beeData("DATA_FLAGS_ID", (byte) event.get("flags").getAsInt()));
                        }
                        if (event.has("anger_end")) {
                            data.add(beeData("DATA_ANGER_END_TIME", event.get("anger_end").getAsLong()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "wolf_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("flags")) {
                            data.add(wolfData(net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID",
                                (byte) event.get("flags").getAsInt()));
                        }
                        if (event.has("interested")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_INTERESTED_ID",
                                event.get("interested").getAsBoolean()));
                        }
                        if (event.has("collar")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_COLLAR_COLOR",
                                event.get("collar").getAsInt()));
                        }
                        if (event.has("anger_end")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_ANGER_END_TIME",
                                event.get("anger_end").getAsLong()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                        if (event.has("health")) {
                            entity.setHealth(Oracle.f32(event, "health"));
                        }
                    }
                    case "weather" -> {
                        level.setRainLevel(Oracle.f32(event, "rain"));
                        level.setThunderLevel(Oracle.f32(event, "thunder"));
                    }
                    case "equipment" -> {
                        var stack = event.get("item").isJsonNull()
                            ? net.minecraft.world.item.ItemStack.EMPTY
                            : new net.minecraft.world.item.ItemStack(net.minecraft.core.registries.BuiltInRegistries.ITEM
                                .getValue(net.minecraft.resources.Identifier.parse(event.get("item").getAsString())));
                        listener.handleSetEquipment(new net.minecraft.network.protocol.game.ClientboundSetEquipmentPacket(
                            entityId, java.util.List.of(com.mojang.datafixers.util.Pair.of(
                                net.minecraft.world.entity.EquipmentSlot.MAINHAND, stack))));
                    }
                    case "mob_flags" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.Mob.class, "DATA_MOB_FLAGS_ID",
                            (byte) event.get("flags").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "strider_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.monster.Strider.class, "DATA_SUFFOCATING",
                            event.get("suffocating").getAsBoolean()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "panda_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var pandaClass = net.minecraft.world.entity.animal.panda.Panda.class;
                        for (var counter : new String[][] {{"unhappy", "UNHAPPY_COUNTER"}, {"sneeze", "SNEEZE_COUNTER"}, {"eat", "EAT_COUNTER"}}) {
                            if (event.has(counter[0])) {
                                data.add(wolfData(pandaClass, counter[1], event.get(counter[0]).getAsInt()));
                            }
                        }
                        for (var gene : new String[][] {{"main_gene", "MAIN_GENE_ID"}, {"hidden_gene", "HIDDEN_GENE_ID"}, {"flags", "DATA_ID_FLAGS"}}) {
                            if (event.has(gene[0])) {
                                data.add(wolfData(pandaClass, gene[1], (byte) event.get(gene[0]).getAsInt()));
                            }
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "fox_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var foxClass = net.minecraft.world.entity.animal.fox.Fox.class;
                        if (event.has("variant")) {
                            data.add(wolfData(foxClass, "DATA_TYPE_ID", event.get("variant").getAsInt()));
                        }
                        if (event.has("flags")) {
                            data.add(wolfData(foxClass, "DATA_FLAGS_ID", (byte) event.get("flags").getAsInt()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "polar_bear_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.animal.polarbear.PolarBear.class, "DATA_STANDING_ID",
                            event.get("standing").getAsBoolean()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "cat_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var catClass = net.minecraft.world.entity.animal.feline.Cat.class;
                        if (event.has("flags")) {
                            data.add(wolfData(net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID",
                                (byte) event.get("flags").getAsInt()));
                        }
                        if (event.has("lying")) {
                            data.add(wolfData(catClass, "IS_LYING", event.get("lying").getAsBoolean()));
                        }
                        if (event.has("relaxed")) {
                            data.add(wolfData(catClass, "RELAX_STATE_ONE", event.get("relaxed").getAsBoolean()));
                        }
                        if (event.has("collar")) {
                            data.add(wolfData(catClass, "DATA_COLLAR_COLOR", event.get("collar").getAsInt()));
                        }
                        if (event.has("pose")) {
                            var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
                            pose.setAccessible(true);
                            data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                                (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
                                net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "rabbit_idle" -> {
                        // What the reference draws from an unseeded random source: fixed here.
                        var rabbit = (net.minecraft.world.entity.animal.rabbit.Rabbit) entity;
                        var timeout = net.minecraft.world.entity.animal.rabbit.Rabbit.class
                            .getDeclaredField("idleAnimationTimeout");
                        timeout.setAccessible(true);
                        timeout.setInt(rabbit, event.get("timeout").getAsInt());
                        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
                        random.setAccessible(true);
                        ((net.minecraft.util.RandomSource) random.get(rabbit)).setSeed(event.get("seed").getAsLong());
                    }
                    case "attributes" -> {
                        var instances = new java.util.ArrayList<net.minecraft.world.entity.ai.attributes.AttributeInstance>();
                        for (var item : event.getAsJsonArray("attributes")) {
                            var changed = item.getAsJsonObject();
                            var holder = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE.get(
                                net.minecraft.resources.Identifier.parse(changed.get("name").getAsString())).orElseThrow();
                            var instance = new net.minecraft.world.entity.ai.attributes.AttributeInstance(holder, ignored -> {});
                            instance.setBaseValue(Oracle.f64(changed, "base"));
                            for (var modifier : changed.getAsJsonArray("modifiers")) {
                                var described = modifier.getAsJsonObject();
                                instance.addTransientModifier(new net.minecraft.world.entity.ai.attributes.AttributeModifier(
                                    net.minecraft.resources.Identifier.parse(described.get("id").getAsString()),
                                    Oracle.f64(described, "amount"),
                                    net.minecraft.world.entity.ai.attributes.AttributeModifier.Operation.BY_ID.apply(
                                        described.get("operation").getAsInt())));
                            }
                            instances.add(instance);
                        }
                        listener.handleUpdateAttributes(
                            new net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket(entityId, instances));
                    }
                    case "entity_event" -> listener.handleEntityEvent(
                        new net.minecraft.network.protocol.game.ClientboundEntityEventPacket(
                            entity, (byte) event.get("event").getAsInt()));
                    default -> throw new IllegalArgumentException("Unknown event " + event);
                }
            }
            level.tickNonPassenger(entity);
            var described = new JsonObject();
            described.addProperty("hurt_time", entity.hurtTime);
            described.addProperty("death_time", entity.deathTime);
            described.addProperty("dead", entity.isDeadOrDying());
            described.addProperty("walk_speed", Oracle.bits(entity.walkAnimation.speed()));
            described.addProperty("walk_position", Oracle.bits(entity.walkAnimation.position()));
            if (row.has("extras")) {
                for (var extra : row.getAsJsonArray("extras")) {
                    switch (extra.getAsString()) {
                        case "swing" -> {
                            described.addProperty("swinging", entity.swinging);
                            described.addProperty("swing_time", entity.swingTime);
                            described.addProperty("off_hand",
                                entity.swingingArm == net.minecraft.world.InteractionHand.OFF_HAND);
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("attack_anim_" + partial, Oracle.bits(entity.getAttackAnim(partial)));
                            }
                        }
                        case "sheep" -> {
                            var sheep = (net.minecraft.world.entity.animal.sheep.Sheep) entity;
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("eat_position_" + partial,
                                    Oracle.bits(sheep.getHeadEatPositionScale(partial)));
                                described.addProperty("eat_angle_" + partial,
                                    Oracle.bits(sheep.getHeadEatAngleScale(partial)));
                            }
                        }
                        case "chicken" -> {
                            var chicken = (net.minecraft.world.entity.animal.chicken.Chicken) entity;
                            described.addProperty("flap", Oracle.bits(chicken.flap));
                            described.addProperty("flap_speed", Oracle.bits(chicken.flapSpeed));
                            described.addProperty("old_flap", Oracle.bits(chicken.oFlap));
                            described.addProperty("old_flap_speed", Oracle.bits(chicken.oFlapSpeed));
                        }
                        case "camel" -> {
                            var camel = (net.minecraft.world.entity.animal.camel.Camel) entity;
                            var states = new net.minecraft.world.entity.AnimationState[] {
                                camel.sitAnimationState, camel.sitPoseAnimationState, camel.sitUpAnimationState,
                                camel.idleAnimationState, camel.dashAnimationState};
                            String[] names = {"sit", "sit_pose", "sit_up", "idle", "dash"};
                            for (int i = 0; i < names.length; i++) {
                                described.addProperty(names[i] + "_started", states[i].isStarted());
                                described.addProperty(names[i] + "_millis",
                                    states[i].isStarted() ? states[i].getTimeInMillis(camel.tickCount) : 0L);
                            }
                            described.addProperty("dashing", camel.isDashing());
                        }
                        case "horse" -> {
                            var horse = (net.minecraft.world.entity.animal.equine.AbstractHorse) entity;
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("eat_" + partial, Oracle.bits(horse.getEatAnim(partial)));
                                described.addProperty("stand_" + partial, Oracle.bits(horse.getStandAnim(partial)));
                                described.addProperty("mouth_" + partial, Oracle.bits(horse.getMouthAnim(partial)));
                            }
                            described.addProperty("tail_counter", horse.tailCounter);
                        }
                        case "bee" -> {
                            var bee = (net.minecraft.world.entity.animal.bee.Bee) entity;
                            described.addProperty("roll_0.0", Oracle.bits(bee.getRollAmount(0.0F)));
                            described.addProperty("roll_0.5", Oracle.bits(bee.getRollAmount(0.5F)));
                            described.addProperty("stung", bee.hasStung());
                            described.addProperty("nectar", bee.hasNectar());
                            described.addProperty("angry", bee.isAngry());
                        }
                        case "wolf" -> {
                            var wolf = (net.minecraft.world.entity.animal.wolf.Wolf) entity;
                            described.addProperty("angry", wolf.isAngry());
                            described.addProperty("sitting", wolf.isInSittingPose());
                            described.addProperty("tame", wolf.isTame());
                            described.addProperty("tail_angle", Oracle.bits(wolf.getTailAngle()));
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("head_roll_" + partial, Oracle.bits(wolf.getHeadRollAngle(partial)));
                                described.addProperty("shake_" + partial, Oracle.bits(wolf.getShakeAnim(partial)));
                            }
                        }
                        case "panda" -> {
                            var panda = (net.minecraft.world.entity.animal.panda.Panda) entity;
                            described.addProperty("variant", panda.getVariant().getId());
                            described.addProperty("unhappy_counter", panda.getUnhappyCounter());
                            described.addProperty("sneeze_counter", panda.getSneezeCounter());
                            described.addProperty("sneezing", panda.isSneezing());
                            described.addProperty("eating", panda.isEating());
                            described.addProperty("sitting", panda.isSitting());
                            described.addProperty("rolling", panda.isRolling());
                            described.addProperty("on_back", panda.isOnBack());
                            described.addProperty("roll_counter", panda.rollCounter);
                            described.addProperty("scared", panda.isScared());
                            described.addProperty("x_rot", Oracle.bits(panda.getXRot()));
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("sit_" + partial, Oracle.bits(panda.getSitAmount(partial)));
                                described.addProperty("back_" + partial, Oracle.bits(panda.getLieOnBackAmount(partial)));
                                described.addProperty("roll_" + partial, Oracle.bits(panda.getRollAmount(partial)));
                            }
                        }
                        case "fox" -> {
                            var fox = (net.minecraft.world.entity.animal.fox.Fox) entity;
                            described.addProperty("variant", fox.getVariant().getId());
                            described.addProperty("sitting", fox.isSitting());
                            described.addProperty("crouching", fox.isCrouching());
                            described.addProperty("sleeping", fox.isSleeping());
                            described.addProperty("faceplanted", fox.isFaceplanted());
                            described.addProperty("pouncing", fox.isPouncing());
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("head_roll_" + partial, Oracle.bits(fox.getHeadRollAngle(partial)));
                                described.addProperty("crouch_" + partial, Oracle.bits(fox.getCrouchAmount(partial)));
                            }
                        }
                        case "strider" -> described.addProperty("suffocating",
                            ((net.minecraft.world.entity.monster.Strider) entity).isSuffocating());
                        case "ravager" -> {
                            var ravager = (net.minecraft.world.entity.monster.Ravager) entity;
                            described.addProperty("attack_tick", ravager.getAttackTick());
                            described.addProperty("stunned_tick", ravager.getStunnedTick());
                            described.addProperty("roar_tick", ravager.getRoarTick());
                        }
                        case "hoglin" -> described.addProperty("attack_ticks",
                            ((net.minecraft.world.entity.monster.hoglin.HoglinBase) entity).getAttackAnimationRemainingTicks());
                        case "polar_bear" -> {
                            var bear = (net.minecraft.world.entity.animal.polarbear.PolarBear) entity;
                            described.addProperty("standing", bear.isStanding());
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("stand_" + partial, Oracle.bits(bear.getStandingAnimationScale(partial)));
                            }
                        }
                        case "cat" -> {
                            var cat = (net.minecraft.world.entity.animal.feline.Cat) entity;
                            described.addProperty("sitting", cat.isInSittingPose());
                            described.addProperty("tame", cat.isTame());
                            described.addProperty("crouching", cat.isCrouching());
                            described.addProperty("on_player", cat.isLyingOnTopOfSleepingPlayer());
                            for (float partial : new float[] {0.0F, 0.5F}) {
                                described.addProperty("lie_" + partial, Oracle.bits(cat.getLieDownAmount(partial)));
                                described.addProperty("lie_tail_" + partial, Oracle.bits(cat.getLieDownAmountTail(partial)));
                                described.addProperty("relax_" + partial, Oracle.bits(cat.getRelaxStateOneAmount(partial)));
                            }
                        }
                        case "rabbit" -> {
                            var rabbit = (net.minecraft.world.entity.animal.rabbit.Rabbit) entity;
                            described.addProperty("hop_started", rabbit.hopAnimationState.isStarted());
                            described.addProperty("idle_started", rabbit.idleHeadTiltAnimationState.isStarted());
                            described.addProperty("hop_millis", rabbit.hopAnimationState.getTimeInMillis(rabbit.tickCount));
                            described.addProperty("idle_millis",
                                rabbit.idleHeadTiltAnimationState.getTimeInMillis(rabbit.tickCount));
                            described.addProperty("jump_completion", Oracle.bits(rabbit.getJumpCompletion(0.5F)));
                        }
                        case "attributes" -> {
                            described.addProperty("max_health", Oracle.bits(entity.getMaxHealth()));
                            described.addProperty("scale", Oracle.bits(entity.getScale()));
                        }
                        case "goat" -> {
                            var goat = (net.minecraft.world.entity.animal.goat.Goat) entity;
                            described.addProperty("ramming", Oracle.bits(goat.getRammingXHeadRot()));
                        }
                        case "iron_golem" -> {
                            var golem = (net.minecraft.world.entity.animal.golem.IronGolem) entity;
                            described.addProperty("attack_tick", golem.getAttackAnimationTick());
                            described.addProperty("offer_flower_tick", golem.getOfferFlowerTick());
                        }
                        case "creeper" -> {
                            var creeper = (net.minecraft.world.entity.monster.Creeper) entity;
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("swelling_" + partial, Oracle.bits(creeper.getSwelling(partial)));
                            }
                        }
                        default -> throw new IllegalArgumentException("Unknown extra " + extra);
                    }
                }
            }
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    /** A bee's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> beeData(
        final String field, final Object value) throws Exception {
        var accessor = net.minecraft.world.entity.animal.bee.Bee.class.getDeclaredField(field);
        accessor.setAccessible(true);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }

    /** A wolf's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> wolfData(
        final Class<?> owner, final String field, final Object value) throws Exception {
        var accessor = owner.getDeclaredField(field);
        accessor.setAccessible(true);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }

    /** A creeper's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    private static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> creeperData(
        final String field, final Object value) throws Exception {
        var accessor = net.minecraft.world.entity.monster.Creeper.class.getDeclaredField(field);
        accessor.setAccessible(true);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }

    /**
     * Every entity type as a new entity of it answers the crosshair:
     * `isPickable`, `getPickRadius`, `canBePickedFromInside`, and its size in
     * the standing pose, a baby's too where the type has babies. A type that
     * makes no entity here (the player) has no row of answers.
     */
    private static JsonObject entityPickTable(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        setField(scene.minecraft(), "level", level);
        setField(scene.listener(), "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // Teams are looked up by entities that build boss bars.
        setField(scene.listener(), "scoreboard", new net.minecraft.world.scores.Scoreboard());
        var types = new JsonArray();
        var registry = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        for (var type : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(type).toString());
            out.addProperty("id", registry.getId(type));
            net.minecraft.world.entity.Entity entity = null;
            try {
                entity = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                    net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            } catch (Throwable failure) {
                var where = failure.getStackTrace().length > 0 ? failure.getStackTrace()[0] : null;
                out.addProperty("error", failure.getClass().getSimpleName() + " at " + where);
            }
            out.addProperty("created", entity != null);
            if (entity != null) {
                out.addProperty("pickable", entity.isPickable());
                out.addProperty("pick_radius", Oracle.bits(entity.getPickRadius()));
                out.addProperty("from_inside", entity.canBePickedFromInside());
                out.addProperty("monster", entity instanceof net.minecraft.world.entity.monster.Monster);
                var size = entity.getDimensions(net.minecraft.world.entity.Pose.STANDING);
                out.addProperty("width", Oracle.bits(size.width()));
                out.addProperty("height", Oracle.bits(size.height()));
                out.addProperty("eye_height", Oracle.bits(size.eyeHeight()));
                if (entity instanceof net.minecraft.world.entity.Mob mob) {
                    mob.setBaby(true);
                    if (mob.isBaby()) {
                        var baby = mob.getDimensions(net.minecraft.world.entity.Pose.STANDING);
                        out.addProperty("baby_width", Oracle.bits(baby.width()));
                        out.addProperty("baby_height", Oracle.bits(baby.height()));
                        out.addProperty("baby_eye_height", Oracle.bits(baby.eyeHeight()));
                    }
                }
            }
            types.add(out);
        }
        entry.add("types", types);
        return entry;
    }

    private static JsonObject localPlayerTicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());

        var scene = buildScene(row);
        var listener = scene.listener();
        var wire = scene.wire();
        var level = scene.level();
        var minecraft = scene.minecraft();
        var options = scene.options();

        var player = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        // The server hands out the entity id at login; the scenario names it,
        // because the sprint command packet carries it.
        player.setId(row.get("entity_id").getAsInt());
        setField(minecraft, "player", player);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(listener.getLocalGameProfile(), false);
        var gameType = net.minecraft.world.level.GameType.byName(row.get("game_mode").getAsString());
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, gameType);
        setField(player, "playerInfo", info);
        var gameMode = new net.minecraft.client.multiplayer.MultiPlayerGameMode(minecraft, listener);
        gameMode.setLocalMode(gameType);
        gameMode.adjustPlayer(player);
        setField(minecraft, "gameMode", gameMode);
        var renderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        var camera = new net.minecraft.client.Camera();
        setField(renderer, "mainCamera", camera);
        setField(minecraft, "gameRenderer", renderer);
        camera.setEntity(player);
        player.input = new net.minecraft.client.player.KeyboardInput(options);
        player.snapTo(
            Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"),
            Oracle.f32(row, "y_rot"), Oracle.f32(row, "x_rot"));
        // The constructor turns a new living entity to a random facing; the
        // scene starts with head and body where the player looks instead.
        player.yHeadRot = player.getYRot();
        player.yHeadRotO = player.getYRot();
        player.yBodyRot = player.getYRot();
        player.yBodyRotO = player.getYRot();

        boolean breaking = row.has("breaking");
        if (breaking) {
            // The client's own player is one of its level's entities, and
            // placing a block checks the level's entities for room.
            level.addEntity(player);
            setField(minecraft, "level", level);
            var sprite = (net.minecraft.client.renderer.texture.TextureAtlasSprite)
                unsafe.allocateInstance(BlankSprite.class);
            BlankModels.sprite = sprite;
            BlankModelManager.models = (net.minecraft.client.renderer.block.BlockStateModelSet)
                unsafe.allocateInstance(BlankModels.class);
            setField(minecraft, "modelManager", unsafe.allocateInstance(BlankModelManager.class));
            setField(minecraft, "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
            setField(minecraft, "blockColors", net.minecraft.client.color.block.BlockColors.createDefault());
            setField(level, "levelExtractor", unsafe.allocateInstance(NoRebuilds.class));
            setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
            var breaking_ = row.getAsJsonObject("breaking");
            if (breaking_.has("main_hand")) {
                var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(breaking_.get("main_hand").getAsString()));
                int count = breaking_.has("main_hand_count") ? breaking_.get("main_hand_count").getAsInt() : 1;
                player.getInventory().setItem(0, new net.minecraft.world.item.ItemStack(item, count));
            }
            if (breaking_.has("off_hand")) {
                var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(breaking_.get("off_hand").getAsString()));
                player.getInventory().setItem(40, new net.minecraft.world.item.ItemStack(item));
            }
            setField(getField(minecraft, "gameRenderer"), "itemInHandRenderer",
                unsafe.allocateInstance(net.minecraft.client.renderer.ItemInHandRenderer.class));
            if (breaking_.has("effects")) {
                for (var effectElement : breaking_.getAsJsonArray("effects")) {
                    var effect = effectElement.getAsJsonObject();
                    var holder = net.minecraft.core.registries.BuiltInRegistries.MOB_EFFECT.get(
                        net.minecraft.resources.Identifier.parse(effect.get("effect").getAsString()))
                        .orElseThrow();
                    player.forceAddEffect(new net.minecraft.world.effect.MobEffectInstance(
                        holder, 1000000, effect.get("amplifier").getAsInt()), null);
                }
            }
        }
        if (row.has("entities")) {
            // Entities the server told of: made as the type makes them, put
            // where the scene says and added to the level.
            for (var element : row.getAsJsonArray("entities")) {
                var described = element.getAsJsonObject();
                var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                    net.minecraft.resources.Identifier.parse(described.get("type").getAsString()));
                var entity = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                    net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
                entity.setId(described.get("id").getAsInt());
                entity.snapTo(Oracle.f64(described, "x"), Oracle.f64(described, "y"),
                    Oracle.f64(described, "z"), 0.0F, 0.0F);
                level.addEntity(entity);
            }
        }
        var startAttack = net.minecraft.client.Minecraft.class.getDeclaredMethod("startAttack");
        startAttack.setAccessible(true);
        var continueAttack = net.minecraft.client.Minecraft.class.getDeclaredMethod(
            "continueAttack", boolean.class);
        continueAttack.setAccessible(true);

        var ticks = new JsonArray();
        for (var element : row.get("ticks").getAsJsonArray()) {
            var input = element.getAsJsonObject();
            options.keyUp.setDown(input.get("forward").getAsBoolean());
            options.keyDown.setDown(input.get("backward").getAsBoolean());
            options.keyLeft.setDown(input.get("left").getAsBoolean());
            options.keyRight.setDown(input.get("right").getAsBoolean());
            options.keyJump.setDown(input.get("jump").getAsBoolean());
            options.keyShift.setDown(input.get("sneak").getAsBoolean());
            options.keySprint.setDown(input.get("sprint").getAsBoolean());
            if (input.has("y_rot")) {
                player.setYRot(Oracle.f32(input, "y_rot"));
            }
            if (input.has("x_rot")) {
                player.setXRot(Oracle.f32(input, "x_rot"));
            }
            wire.sent.clear();
            if (input.has("attack")) {
                attack(minecraft, level, input, startAttack, continueAttack);
            }
            // The level ticks its entities, not the entity itself: it saves
            // the old position and rotation and counts the tick first, and
            // both are read during the tick.
            level.tickNonPassenger(player);
            var described = describe(player, wire.sent);
            if (input.has("attack")) {
                described.add("breaking", breakingState(minecraft, level, gameMode, player,
                    input.getAsJsonObject("attack")));
            }
            ticks.add(described);
        }
        entry.addProperty("outcome", "ticked");
        entry.add("ticks", ticks);
        return entry;
    }

    /**
     * One tick of what `Minecraft.runTick` does before the level ticks the
     * player: the server's block packets first, then the attack key as
     * `handleKeybinds` spends it, then the miss wait counting down. The aim
     * is given, not picked: the scene names the block face or a miss.
     */
    private static void attack(
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.multiplayer.ClientLevel level,
        JsonObject input,
        java.lang.reflect.Method startAttack,
        java.lang.reflect.Method continueAttack) throws Exception {
        var attack = input.getAsJsonObject("attack");
        if (attack.has("server")) {
            for (var element : attack.getAsJsonArray("server")) {
                var event = element.getAsJsonObject();
                if (event.has("ack")) {
                    level.handleBlockChangedAck(event.get("ack").getAsInt());
                } else {
                    level.setServerVerifiedBlockState(
                        new BlockPos(event.get("x").getAsInt(), event.get("y").getAsInt(),
                            event.get("z").getAsInt()),
                        state(event), 19);
                }
            }
        }
        // `Minecraft.tick`: the right click delay counts down first.
        int rightClickDelay = (int) getField(minecraft, "rightClickDelay");
        if (rightClickDelay > 0) {
            setField(minecraft, "rightClickDelay", rightClickDelay - 1);
        }
        if (attack.has("pick")) {
            // `Minecraft.pick`: where the crosshair lands, as the player picks it.
            minecraft.hitResult = minecraft.player.raycastHitResult(1.0F, minecraft.player);
        } else if (attack.has("target")) {
            var target = attack.getAsJsonObject("target");
            var pos = new BlockPos(target.get("x").getAsInt(), target.get("y").getAsInt(),
                target.get("z").getAsInt());
            var face = net.minecraft.core.Direction.byName(target.get("face").getAsString());
            var location = target.has("location")
                ? new net.minecraft.world.phys.Vec3(
                    Oracle.f64(target.getAsJsonObject("location"), "x"),
                    Oracle.f64(target.getAsJsonObject("location"), "y"),
                    Oracle.f64(target.getAsJsonObject("location"), "z"))
                : net.minecraft.world.phys.Vec3.atCenterOf(pos);
            minecraft.hitResult = new net.minecraft.world.phys.BlockHitResult(
                location, face, pos, target.has("inside") && target.get("inside").getAsBoolean());
        } else {
            minecraft.hitResult = net.minecraft.world.phys.BlockHitResult.miss(
                net.minecraft.world.phys.Vec3.ZERO, net.minecraft.core.Direction.UP, BlockPos.ZERO);
        }
        for (int click = 0; click < attack.get("clicks").getAsInt(); click++) {
            startAttack.invoke(minecraft);
        }
        if (attack.has("use")) {
            var use = attack.getAsJsonObject("use");
            var startUseItem = net.minecraft.client.Minecraft.class.getDeclaredMethod("startUseItem");
            startUseItem.setAccessible(true);
            for (int click = 0; click < use.get("clicks").getAsInt(); click++) {
                startUseItem.invoke(minecraft);
            }
            // The pick item key is spent after the use clicks, before a held use.
            if (attack.has("pick_item")) {
                var pickItem = net.minecraft.client.Minecraft.class.getDeclaredMethod("pickBlockOrEntity");
                pickItem.setAccessible(true);
                for (int click = 0; click < attack.getAsJsonObject("pick_item").get("clicks").getAsInt(); click++) {
                    pickItem.invoke(minecraft);
                }
            }
            if (use.get("down").getAsBoolean() && (int) getField(minecraft, "rightClickDelay") == 0
                && !minecraft.player.isUsingItem()) {
                startUseItem.invoke(minecraft);
            }
        }
        continueAttack.invoke(minecraft, attack.get("down").getAsBoolean());
        int missTime = (int) getField(minecraft, "missTime");
        if (missTime > 0) {
            setField(minecraft, "missTime", missTime - 1);
        }
    }

    /** What breaking left behind: the game mode's own state and the level's. */
    private static JsonObject breakingState(
        net.minecraft.client.Minecraft minecraft,
        net.minecraft.client.multiplayer.ClientLevel level,
        net.minecraft.client.multiplayer.MultiPlayerGameMode gameMode,
        net.minecraft.client.player.LocalPlayer player,
        JsonObject attack) throws Exception {
        var out = new JsonObject();
        out.addProperty("destroying", gameMode.isDestroying());
        out.addProperty("progress", Oracle.bits((float) getField(gameMode, "destroyProgress")));
        out.addProperty("destroy_ticks", Oracle.bits((float) getField(gameMode, "destroyTicks")));
        out.addProperty("delay", (int) getField(gameMode, "destroyDelay"));
        out.addProperty("miss_time", (int) getField(minecraft, "missTime"));
        var destroying = (it.unimi.dsi.fastutil.ints.Int2ObjectMap<?>) getField(level, "destroyingBlocks");
        var entry = (net.minecraft.server.level.BlockDestructionProgress) destroying.get(player.getId());
        out.addProperty("stage", entry == null ? -1 : entry.getProgress());
        var handler = getField(level, "blockStatePredictionHandler");
        out.addProperty("sequence", (int) getField(handler, "currentSequenceNr"));
        out.addProperty("attack_strength_ticker", (int) getField(player, "attackStrengthTicker"));
        out.addProperty("right_click_delay", (int) getField(minecraft, "rightClickDelay"));
        out.addProperty("main_hand_count", player.getMainHandItem().getCount());
        out.addProperty("off_hand_count", player.getOffhandItem().getCount());
        var states = new JsonArray();
        for (var element : attack.getAsJsonArray("watch")) {
            var watched = element.getAsJsonObject();
            states.add(net.minecraft.world.level.block.Block.getId(level.getBlockState(new BlockPos(
                watched.get("x").getAsInt(), watched.get("y").getAsInt(), watched.get("z").getAsInt()))));
        }
        out.add("watched_states", states);
        return out;
    }

    /**
     * Clicks in the player's own inventory menu, as the inventory screen
     * sends them: `MultiPlayerGameMode.handleContainerInput` runs the menu's
     * click as the client predicts it and sends the click packet. After
     * every click the packets sent, as their codecs write them, every slot
     * and the carried stack are reported. Stacks carry no components, so
     * their hashes are never asked for.
     */
    /** A menu and the game around it: the scene, the player and the menu, slots and data set. */
    record OpenMenu(Scene scene, net.minecraft.client.player.LocalPlayer player,
                    net.minecraft.world.inventory.AbstractContainerMenu menu, int containerId) {}

    static OpenMenu openMenu(JsonObject row) throws Exception {
        var scene = buildScene(row);
        var listener = scene.listener();
        var minecraft = scene.minecraft();
        var level = scene.level();
        setField(listener, "decoratedHashOpsGenerator",
            (net.minecraft.network.HashedPatchMap.HashGenerator) component -> 0);
        var player = new net.minecraft.client.player.LocalPlayer(
            minecraft, level, listener, null, null,
            net.minecraft.world.entity.player.Input.EMPTY, false, null);
        player.setId(1);
        setField(minecraft, "player", player);
        var info = new net.minecraft.client.multiplayer.PlayerInfo(listener.getLocalGameProfile(), false);
        var gameType = net.minecraft.world.level.GameType.byName(row.get("game_mode").getAsString());
        var setMode = net.minecraft.client.multiplayer.PlayerInfo.class.getDeclaredMethod(
            "setGameMode", net.minecraft.world.level.GameType.class);
        setMode.setAccessible(true);
        setMode.invoke(info, gameType);
        setField(player, "playerInfo", info);
        var gameMode = new net.minecraft.client.multiplayer.MultiPlayerGameMode(minecraft, listener);
        gameMode.setLocalMode(gameType);
        gameMode.adjustPlayer(player);
        setField(minecraft, "gameMode", gameMode);
        setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        player.getInventory().setSelectedSlot(row.get("selected").getAsInt());
        net.minecraft.world.inventory.AbstractContainerMenu menu = player.inventoryMenu;
        int containerId = 0;
        if (row.has("menu")) {
            // A menu the server opened: built the way the client builds it,
            // over the player's own inventory.
            containerId = row.get("container_id").getAsInt();
            // A furnace menu reads the items its recipes accept from the
            // server's recipe sets, and what burns from the client's fuel
            // values.
            setField(listener, "fuelValues", net.minecraft.world.level.block.entity.FuelValues
                .vanillaBurnTimes(Oracle.vanillaRegistries(), net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS));
            var accepted = new java.util.ArrayList<net.minecraft.world.item.crafting.Ingredient>();
            if (row.has("furnace_inputs")) {
                for (var name : row.getAsJsonArray("furnace_inputs")) {
                    accepted.add(net.minecraft.world.item.crafting.Ingredient.of(
                        net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                            net.minecraft.resources.Identifier.parse(name.getAsString()))));
                }
            }
            var inputs = net.minecraft.world.item.crafting.RecipePropertySet.create(accepted);
            var sets = new java.util.HashMap<net.minecraft.resources.ResourceKey<
                net.minecraft.world.item.crafting.RecipePropertySet>,
                net.minecraft.world.item.crafting.RecipePropertySet>();
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.FURNACE_INPUT, inputs);
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.BLAST_FURNACE_INPUT, inputs);
            sets.put(net.minecraft.world.item.crafting.RecipePropertySet.SMOKER_INPUT, inputs);
            if (row.has("recipe_sets")) {
                for (var setEntry : row.getAsJsonObject("recipe_sets").entrySet()) {
                    var members = new java.util.ArrayList<net.minecraft.world.item.crafting.Ingredient>();
                    for (var name : setEntry.getValue().getAsJsonArray()) {
                        members.add(net.minecraft.world.item.crafting.Ingredient.of(
                            net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                                net.minecraft.resources.Identifier.parse(name.getAsString()))));
                    }
                    sets.put(net.minecraft.resources.ResourceKey.create(
                            net.minecraft.world.item.crafting.RecipePropertySet.TYPE_KEY,
                            net.minecraft.resources.Identifier.parse(setEntry.getKey())),
                        net.minecraft.world.item.crafting.RecipePropertySet.create(members));
                }
            }
            var cutter = new java.util.ArrayList<net.minecraft.world.item.crafting.SelectableRecipe.SingleInputEntry<
                net.minecraft.world.item.crafting.StonecutterRecipe>>();
            if (row.has("stonecutter_inputs")) {
                for (var name : row.getAsJsonArray("stonecutter_inputs")) {
                    cutter.add(new net.minecraft.world.item.crafting.SelectableRecipe.SingleInputEntry<>(
                        net.minecraft.world.item.crafting.Ingredient.of(
                            net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                                net.minecraft.resources.Identifier.parse(name.getAsString()))),
                        new net.minecraft.world.item.crafting.SelectableRecipe<>(
                            net.minecraft.world.item.crafting.display.SlotDisplay.AnyFuel.INSTANCE,
                            java.util.Optional.empty())));
                }
            }
            setField(listener, "recipes", new net.minecraft.client.multiplayer.ClientRecipeContainer(
                sets, new net.minecraft.world.item.crafting.SelectableRecipe.SingleInputSet<>(cutter)));
            var type = net.minecraft.core.registries.BuiltInRegistries.MENU.getValue(
                net.minecraft.resources.Identifier.parse(row.get("menu").getAsString()));
            if (row.has("xp_level")) {
                player.experienceLevel = row.get("xp_level").getAsInt();
            }
            menu = type.create(containerId, player.getInventory());
            player.containerMenu = menu;
        }
        for (var element : row.getAsJsonArray("slots")) {
            var slot = element.getAsJsonObject();
            menu.getSlot(slot.get("slot").getAsInt()).set(stack(slot));
        }
        if (row.has("carried")) {
            menu.setCarried(stack(row.getAsJsonObject("carried")));
        }
        // What the server last sent of the menu's data, after the slots are
        // set, which would make a menu work its own out.
        if (row.has("data")) {
            var values = row.getAsJsonArray("data");
            for (int index = 0; index < values.size(); index++) {
                menu.setData(index, values.get(index).getAsInt());
            }
        }
        setField(menu, "stateId", row.get("state_id").getAsInt());
        return new OpenMenu(scene, player, menu, containerId);
    }

    private static JsonObject containerClicks(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var open = openMenu(row);
        var scene = open.scene();
        var listener = scene.listener();
        var minecraft = scene.minecraft();
        var player = open.player();
        var menu = open.menu();
        int containerId = open.containerId();
        var gameMode = minecraft.gameMode;
        var wire = scene.wire();
        var clicks = new JsonArray();
        for (var element : row.getAsJsonArray("clicks")) {
            var click = element.getAsJsonObject();
            wire.sent.clear();
            gameMode.handleContainerInput(containerId, click.get("slot").getAsInt(), click.get("button").getAsInt(),
                net.minecraft.world.inventory.ContainerInput.valueOf(click.get("type").getAsString()), player);
            var described = new JsonObject();
            var packets = new JsonArray();
            for (var packet : wire.sent) {
                var packetJson = new JsonObject();
                packetJson.addProperty("type", packet.type().id().toString());
                packetJson.addProperty("bytes", encode(packet));
                packets.add(packetJson);
            }
            described.add("sent", packets);
            var slots = new JsonArray();
            for (var slot : menu.slots) {
                slots.add(describeStack(slot.getItem()));
            }
            described.add("slots", slots);
            described.add("carried", describeStack(menu.getCarried()));
            described.addProperty("selected", player.getInventory().getSelectedSlot());
            clicks.add(described);
        }
        entry.addProperty("outcome", "clicked");
        entry.add("clicks", clicks);
        return entry;
    }

    private static net.minecraft.world.item.ItemStack stack(JsonObject described) {
        var item = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
            net.minecraft.resources.Identifier.parse(described.get("item").getAsString()));
        return new net.minecraft.world.item.ItemStack(item, described.get("count").getAsInt());
    }

    private static com.google.gson.JsonElement describeStack(net.minecraft.world.item.ItemStack stack) {
        if (stack.isEmpty()) {
            return com.google.gson.JsonNull.INSTANCE;
        }
        var out = new JsonObject();
        out.addProperty("item", net.minecraft.core.registries.BuiltInRegistries.ITEM.getKey(stack.getItem()).toString());
        out.addProperty("count", stack.getCount());
        return out;
    }

    /**
     * The client level's own light engine, lit from nothing and then kept
     * current through block updates.
     *
     * The level's light starts the way the server first lights a chunk: every
     * section that holds blocks is marked, every column is switched on, and
     * the light sources are spread. Each event then arrives as a block update
     * from the server would, and the engine runs as the frame loop runs it.
     * After the start and after every event, the block and sky light of a box
     * are reported, as the level's light getters read them.
     */
    private static JsonObject lightChanges(JsonObject row) throws Exception {
        var scene = buildScene(row);
        var level = scene.level();
        var extractor = unsafe.allocateInstance(NoRebuilds.class);
        setField(level, "levelExtractor", extractor);
        setField(scene.minecraft(), "levelExtractor", extractor);
        var engine = level.getChunkSource().getLightEngine();
        int radius = row.get("chunk_radius").getAsInt();
        var chunks = new java.util.ArrayList<net.minecraft.world.level.ChunkPos>();
        for (int chunkX = -radius; chunkX <= radius; chunkX++) {
            for (int chunkZ = -radius; chunkZ <= radius; chunkZ++) {
                chunks.add(new net.minecraft.world.level.ChunkPos(chunkX, chunkZ));
            }
        }
        for (var chunkPos : chunks) {
            var chunk = level.getChunk(chunkPos.x(), chunkPos.z());
            var sections = chunk.getSections();
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

        var box = row.getAsJsonObject("box");
        var snapshots = new JsonArray();
        snapshots.add(lightSnapshot(engine, box));
        for (var element : row.get("events").getAsJsonArray()) {
            var event = element.getAsJsonObject();
            level.setServerVerifiedBlockState(
                new BlockPos(event.get("x").getAsInt(), event.get("y").getAsInt(), event.get("z").getAsInt()),
                state(event), 19);
            engine.runLightUpdates();
            snapshots.add(lightSnapshot(engine, box));
        }
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        entry.addProperty("outcome", "lit");
        entry.add("snapshots", snapshots);
        return entry;
    }

    /** Block then sky light of every position in the box, y, then z, then x, as hex digits. */
    private static JsonObject lightSnapshot(
        net.minecraft.world.level.lighting.LevelLightEngine engine, JsonObject box) {
        var out = new JsonObject();
        for (var layer : net.minecraft.world.level.LightLayer.values()) {
            var listener = engine.getLayerListener(layer);
            var values = new StringBuilder();
            for (int y = 0; y < box.get("size_y").getAsInt(); y++) {
                for (int z = 0; z < box.get("size_z").getAsInt(); z++) {
                    for (int x = 0; x < box.get("size_x").getAsInt(); x++) {
                        values.append(Integer.toHexString(listener.getLightValue(new BlockPos(
                            box.get("x").getAsInt() + x, box.get("y").getAsInt() + y,
                            box.get("z").getAsInt() + z))));
                    }
                }
            }
            out.addProperty(layer == net.minecraft.world.level.LightLayer.BLOCK ? "block" : "sky",
                values.toString());
        }
        return out;
    }

    private static JsonObject describe(
        net.minecraft.client.player.LocalPlayer player,
        java.util.List<net.minecraft.network.protocol.Packet<?>> sent) throws Exception {
        var tick = new JsonObject();
        tick.addProperty("x", Oracle.bits(player.getX()));
        tick.addProperty("y", Oracle.bits(player.getY()));
        tick.addProperty("z", Oracle.bits(player.getZ()));
        tick.addProperty("motion_x", Oracle.bits(player.getDeltaMovement().x));
        tick.addProperty("motion_y", Oracle.bits(player.getDeltaMovement().y));
        tick.addProperty("motion_z", Oracle.bits(player.getDeltaMovement().z));
        tick.addProperty("y_rot", Oracle.bits(player.getYRot()));
        tick.addProperty("x_rot", Oracle.bits(player.getXRot()));
        tick.addProperty("on_ground", player.onGround());
        tick.addProperty("horizontal_collision", player.horizontalCollision);
        tick.addProperty("vertical_collision", player.verticalCollision);
        tick.addProperty("minor_horizontal_collision", player.minorHorizontalCollision);
        tick.addProperty("sprinting", player.isSprinting());
        tick.addProperty("shift_key_down", player.isShiftKeyDown());
        tick.addProperty("pose", player.getPose().name());
        tick.addProperty("fall_distance", Oracle.bits(player.fallDistance));
        tick.addProperty("flying", player.getAbilities().flying);
        // What the view bob reads, half way through the next frame.
        var avatar = player.avatarState();
        tick.addProperty("bob_walk", Oracle.bits(avatar.getBackwardsInterpolatedWalkDistance(0.5F)));
        tick.addProperty("bob", Oracle.bits(avatar.getInterpolatedBob(0.5F)));
        // What the player model is posed from, half way through the frame.
        tick.addProperty("body_rot", Oracle.bits(net.minecraft.util.Mth.rotLerp(0.5F, player.yBodyRotO, player.yBodyRot)));
        tick.addProperty("head_rot", Oracle.bits(net.minecraft.util.Mth.rotLerp(0.5F, player.yHeadRotO, player.yHeadRot)));
        tick.addProperty("walk_position", Oracle.bits(player.walkAnimation.position(0.5F)));
        tick.addProperty("walk_speed", Oracle.bits(player.walkAnimation.speed(0.5F)));
        tick.addProperty("swim_amount", Oracle.bits(player.getSwimAmount(0.5F)));
        tick.addProperty("hand_x_bob", Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, player.xBobO, player.xBob)));
        tick.addProperty("hand_y_bob", Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, player.yBobO, player.yBob)));
        var packets = new JsonArray();
        for (var packet : sent) {
            var described = new JsonObject();
            described.addProperty("type", packet.type().id().toString());
            described.addProperty("bytes", encode(packet));
            packets.add(described);
        }
        tick.add("sent", packets);
        return tick;
    }

    /** A packet as the bytes its own codec writes, without the packet id. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    private static String encode(net.minecraft.network.protocol.Packet<?> packet) throws Exception {
        Class<?> type = packet.getClass();
        java.lang.reflect.Field codecField = null;
        while (type != null && codecField == null) {
            try {
                codecField = type.getDeclaredField("STREAM_CODEC");
            } catch (NoSuchFieldException missing) {
                type = type.getSuperclass();
            }
        }
        if (codecField == null) {
            throw new IllegalStateException("No codec on " + packet.getClass());
        }
        codecField.setAccessible(true);
        var codec = (net.minecraft.network.codec.StreamCodec) codecField.get(null);
        var buffer = new net.minecraft.network.RegistryFriendlyByteBuf(
            io.netty.buffer.Unpooled.buffer(), registries);
        codec.encode(buffer, packet);
        var bytes = new byte[buffer.readableBytes()];
        buffer.readBytes(bytes);
        return java.util.HexFormat.of().formatHex(bytes);
    }
}
