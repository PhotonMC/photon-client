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
                case "projectile_ticks" -> rows.add(projectileTicks(element.getAsJsonObject()));
                case "living_entity_ticks" -> rows.add(livingEntityTicks(element.getAsJsonObject()));
                case "entity_shadows" -> rows.add(entityShadows(element.getAsJsonObject()));
                case "entity_pick_table" -> rows.add(entityPickTable(element.getAsJsonObject()));
                case "light_changes" -> rows.add(lightChanges(element.getAsJsonObject()));
                case "sky_flash" -> rows.add(skyFlash(element.getAsJsonObject()));
                case "elytra_state" -> rows.add(elytraState(element.getAsJsonObject()));
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
     * A projectile the server added, ticked on the client as the level ticks it: where it goes
     * between the server's packets, against the blocks and the entities the scene holds.
     */
    private static JsonObject projectileTicks(JsonObject row) throws Exception {
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
        // A sound an entity makes asks the renderer's camera how far it is: none is heard here.
        var soundRenderer = unsafe.allocateInstance(net.minecraft.client.renderer.GameRenderer.class);
        setField(soundRenderer, "mainCamera", new net.minecraft.client.Camera());
        setField(scene.minecraft(), "gameRenderer", soundRenderer);
        // What an entity adds to the air (a glow squid's sparks) is not drawn here.
        setField(scene.minecraft(), "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
        // The server's features: a minecart reads them as it is made (the new behaviour when it enabled `minecart_improvements`).
        setField(listener, "enabledFeatures", row.has("minecart_improvements")
            ? net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS.join(
                net.minecraft.world.flag.FeatureFlagSet.of(net.minecraft.world.flag.FeatureFlags.MINECART_IMPROVEMENTS))
            : net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
        // A boat asks for the teams of what it could push (no team here).
        setField(listener, "scoreboard", new net.minecraft.world.scores.Scoreboard());
        var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
            net.minecraft.resources.Identifier.parse(row.get("entity").getAsString()));
        net.minecraft.world.entity.Entity entity = type.create(level,
            new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
        // What a living entity has beyond an entity's own (a boat is not one).
        var living = entity instanceof net.minecraft.world.entity.LivingEntity livingEntity ? livingEntity : null;
        int entityId = row.get("entity_id").getAsInt();
        entity.setId(entityId);
        entity.snapTo(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"), 0.0F, 0.0F);
        // The constructor turns the head at random; the add-entity packet sets it (here: facing south).
        if (living != null) {
            living.yBodyRot = 0.0F;
            living.yHeadRot = 0.0F;
            living.yBodyRotO = 0.0F;
            living.yHeadRotO = 0.0F;
        }
        entity.syncPacketPositionCodec(entity.getX(), entity.getY(), entity.getZ());
        level.addEntity(entity);
        // Another entity that stays where it is: what a guardian's beam aims at.
        if (row.has("second")) {
            var second = row.getAsJsonObject("second");
            var secondType = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                net.minecraft.resources.Identifier.parse(second.get("entity").getAsString()));
            var other = secondType.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            other.setId(second.get("entity_id").getAsInt());
            other.snapTo(Oracle.f64(second, "x"), Oracle.f64(second, "y"), Oracle.f64(second, "z"), 0.0F, 0.0F);
            if (other instanceof net.minecraft.world.entity.LivingEntity otherLiving) {
                otherLiving.yBodyRot = 0.0F;
                otherLiving.yHeadRot = 0.0F;
                otherLiving.yBodyRotO = 0.0F;
                otherLiving.yHeadRotO = 0.0F;
            }
            other.syncPacketPositionCodec(other.getX(), other.getY(), other.getZ());
            level.addEntity(other);
        }
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
                        new net.minecraft.network.protocol.game.ClientboundHurtAnimationPacket(living));
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
                    case "sync" -> listener.handleEntityPositionSync(
                        new net.minecraft.network.protocol.game.ClientboundEntityPositionSyncPacket(entityId,
                            new net.minecraft.world.entity.PositionMoveRotation(
                                new net.minecraft.world.phys.Vec3(Oracle.f64(event, "x"),
                                    Oracle.f64(event, "y"), Oracle.f64(event, "z")),
                                new net.minecraft.world.phys.Vec3(Oracle.f64(event, "motion_x"),
                                    Oracle.f64(event, "motion_y"), Oracle.f64(event, "motion_z")),
                                0.0F, 0.0F),
                            !event.has("on_ground") || event.get("on_ground").getAsBoolean()));
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
                            living.setHealth(Oracle.f32(event, "health"));
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
                    case "copper_golem_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var golemClass = net.minecraft.world.entity.animal.golem.CopperGolem.class;
                        if (event.has("state")) {
                            data.add(wolfData(golemClass, "COPPER_GOLEM_STATE",
                                net.minecraft.world.entity.animal.golem.CopperGolemState.values()[event.get("state").getAsInt()]));
                        }
                        if (event.has("weather")) {
                            data.add(wolfData(golemClass, "DATA_WEATHER_STATE",
                                net.minecraft.world.level.block.WeatheringCopper.WeatherState.values()[event.get("weather").getAsInt()]));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "entity_motion" -> listener.handleSetEntityMotion(
                        new net.minecraft.network.protocol.game.ClientboundSetEntityMotionPacket(entityId,
                            new net.minecraft.world.phys.Vec3(Oracle.f64(event, "x"), Oracle.f64(event, "y"),
                                Oracle.f64(event, "z"))));
                    case "glow_squid_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.animal.squid.GlowSquid.class,
                            "DATA_DARK_TICKS_REMAINING", event.get("dark_ticks").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "passengers" -> {
                        // The packet as the wire carries it: the vehicle and the ids that ride it.
                        var buffer = new net.minecraft.network.FriendlyByteBuf(io.netty.buffer.Unpooled.buffer());
                        buffer.writeVarInt(event.get("vehicle").getAsInt());
                        var riders = event.getAsJsonArray("riders");
                        var ids = new int[riders.size()];
                        for (int i = 0; i < ids.length; i++) {
                            ids[i] = riders.get(i).getAsInt();
                        }
                        buffer.writeVarIntArray(ids);
                        listener.handleSetEntityPassengersPacket(
                            net.minecraft.network.protocol.game.ClientboundSetPassengersPacket.STREAM_CODEC.decode(buffer));
                    }
                    case "shulker_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var shulkerClass = net.minecraft.world.entity.monster.Shulker.class;
                        if (event.has("face")) {
                            data.add(wolfData(shulkerClass, "DATA_ATTACH_FACE_ID",
                                net.minecraft.core.Direction.from3DDataValue(event.get("face").getAsInt())));
                        }
                        if (event.has("peek")) {
                            data.add(wolfData(shulkerClass, "DATA_PEEK_ID", (byte) event.get("peek").getAsInt()));
                        }
                        if (event.has("color")) {
                            data.add(wolfData(shulkerClass, "DATA_COLOR_ID", (byte) event.get("color").getAsInt()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "sulfur_fuse" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.monster.cubemob.SulfurCube.class, "MAX_FUSE",
                            event.get("fuse").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "head_rotation" -> listener.handleRotateMob(
                        new net.minecraft.network.protocol.game.ClientboundRotateHeadPacket(entity,
                            (byte) event.get("angle").getAsInt()));
                    case "cube_size" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.monster.cubemob.AbstractCubeMob.class, "ID_SIZE",
                            event.get("size").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "baby_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.AgeableMob.class, "DATA_BABY_ID",
                            event.get("baby").getAsBoolean()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "axolotl_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("variant")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_VARIANT",
                                event.get("variant").getAsInt()));
                        }
                        if (event.has("playing_dead")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_PLAYING_DEAD",
                                event.get("playing_dead").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "boat_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var vehicleClass = net.minecraft.world.entity.vehicle.VehicleEntity.class;
                        var boatClass = net.minecraft.world.entity.vehicle.boat.AbstractBoat.class;
                        if (event.has("hurt")) {
                            data.add(wolfData(vehicleClass, "DATA_ID_HURT", event.get("hurt").getAsInt()));
                        }
                        if (event.has("hurt_dir")) {
                            data.add(wolfData(vehicleClass, "DATA_ID_HURTDIR", event.get("hurt_dir").getAsInt()));
                        }
                        if (event.has("damage")) {
                            data.add(wolfData(vehicleClass, "DATA_ID_DAMAGE", (float) event.get("damage").getAsDouble()));
                        }
                        if (event.has("bubble_time")) {
                            data.add(wolfData(boatClass, "DATA_ID_BUBBLE_TIME", event.get("bubble_time").getAsInt()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "flier_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("parrot_variant")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.parrot.Parrot.class, "DATA_VARIANT_ID",
                                event.get("parrot_variant").getAsInt()));
                        }
                        if (event.has("bat_flags")) {
                            data.add(wolfData(net.minecraft.world.entity.ambient.Bat.class, "DATA_ID_FLAGS",
                                (byte) event.get("bat_flags").getAsInt()));
                        }
                        if (event.has("vex_flags")) {
                            data.add(wolfData(net.minecraft.world.entity.monster.Vex.class, "DATA_FLAGS_ID",
                                (byte) event.get("vex_flags").getAsInt()));
                        }
                        if (event.has("allay_dancing")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.allay.Allay.class, "DATA_DANCING",
                                event.get("allay_dancing").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "guardian_random" -> {
                        // What the constructor draws from an unseeded random source (and what
                        // the spikes out of water are drawn from): fixed here.
                        var tail = net.minecraft.world.entity.monster.Guardian.class
                            .getDeclaredField("clientSideTailAnimation");
                        tail.setAccessible(true);
                        tail.setFloat(entity, Oracle.f32(event, "tail"));
                        var tailOld = net.minecraft.world.entity.monster.Guardian.class
                            .getDeclaredField("clientSideTailAnimationO");
                        tailOld.setAccessible(true);
                        tailOld.setFloat(entity, Oracle.f32(event, "tail"));
                        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
                        random.setAccessible(true);
                        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
                    }
                    case "guardian_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var guardianClass = net.minecraft.world.entity.monster.Guardian.class;
                        if (event.has("moving")) {
                            data.add(wolfData(guardianClass, "DATA_ID_MOVING", event.get("moving").getAsBoolean()));
                        }
                        if (event.has("target")) {
                            data.add(wolfData(guardianClass, "DATA_ID_ATTACK_TARGET", event.get("target").getAsInt()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "frog_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var pose = net.minecraft.world.entity.Entity.class.getDeclaredField("DATA_POSE");
                        pose.setAccessible(true);
                        data.add(net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
                            (net.minecraft.network.syncher.EntityDataAccessor<net.minecraft.world.entity.Pose>) pose.get(null),
                            net.minecraft.world.entity.Pose.valueOf(event.get("pose").getAsString())));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "arrow_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("in_ground")) {
                            data.add(wolfData(net.minecraft.world.entity.projectile.arrow.AbstractArrow.class, "IN_GROUND",
                                event.get("in_ground").getAsBoolean()));
                        }
                        if (event.has("color")) {
                            data.add(wolfData(net.minecraft.world.entity.projectile.arrow.Arrow.class, "ID_EFFECT_COLOR",
                                event.get("color").getAsInt()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "minecart_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var vehicle = net.minecraft.world.entity.vehicle.VehicleEntity.class;
                        var cart = net.minecraft.world.entity.vehicle.minecart.AbstractMinecart.class;
                        if (event.has("hurt")) {
                            data.add(wolfData(vehicle, "DATA_ID_HURT", event.get("hurt").getAsInt()));
                        }
                        if (event.has("hurt_dir")) {
                            data.add(wolfData(vehicle, "DATA_ID_HURTDIR", event.get("hurt_dir").getAsInt()));
                        }
                        if (event.has("damage")) {
                            data.add(wolfData(vehicle, "DATA_ID_DAMAGE", (float) event.get("damage").getAsDouble()));
                        }
                        if (event.has("display")) {
                            data.add(wolfData(cart, "DATA_ID_CUSTOM_DISPLAY_BLOCK", java.util.Optional.of(
                                net.minecraft.world.level.block.Block.stateById(event.get("display").getAsInt()))));
                        }
                        if (event.has("no_display")) {
                            data.add(wolfData(cart, "DATA_ID_CUSTOM_DISPLAY_BLOCK", java.util.Optional.empty()));
                        }
                        if (event.has("offset")) {
                            data.add(wolfData(cart, "DATA_ID_DISPLAY_OFFSET", event.get("offset").getAsInt()));
                        }
                        if (event.has("fuel")) {
                            data.add(wolfData(net.minecraft.world.entity.vehicle.minecart.MinecartFurnace.class,
                                "DATA_ID_FUEL", event.get("fuel").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "crystal_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        var crystalClass = net.minecraft.world.entity.boss.enderdragon.EndCrystal.class;
                        if (event.has("target")) {
                            var target = event.getAsJsonArray("target");
                            data.add(wolfData(crystalClass, "DATA_BEAM_TARGET", java.util.Optional.of(
                                new net.minecraft.core.BlockPos(target.get(0).getAsInt(), target.get(1).getAsInt(),
                                    target.get(2).getAsInt()))));
                        }
                        if (event.has("no_target")) {
                            data.add(wolfData(crystalClass, "DATA_BEAM_TARGET", java.util.Optional.empty()));
                        }
                        if (event.has("bottom")) {
                            data.add(wolfData(crystalClass, "DATA_SHOW_BOTTOM", event.get("bottom").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "dragon_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.boss.enderdragon.EnderDragon.class, "DATA_PHASE",
                            event.get("phase").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "move_rot" -> listener.handleMoveEntity(
                        new net.minecraft.network.protocol.game.ClientboundMoveEntityPacket.PosRot(entityId,
                            (short) event.get("xa").getAsInt(), (short) event.get("ya").getAsInt(),
                            (short) event.get("za").getAsInt(), (byte) event.get("y_rot").getAsInt(),
                            (byte) event.get("x_rot").getAsInt(), true));
                    case "wither_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        for (var field : new String[][] {{"target_b", "DATA_TARGET_B"}, {"target_c", "DATA_TARGET_C"},
                                {"invulnerable", "DATA_ID_INV"}}) {
                            if (event.has(field[0])) {
                                data.add(wolfData(net.minecraft.world.entity.boss.wither.WitherBoss.class, field[1],
                                    event.get(field[0]).getAsInt()));
                            }
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "sniffer_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.animal.sniffer.Sniffer.class, "DATA_STATE",
                            net.minecraft.world.entity.animal.sniffer.Sniffer.State.values()[event.get("state").getAsInt()]));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "armadillo_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.animal.armadillo.Armadillo.class, "ARMADILLO_STATE",
                            net.minecraft.world.entity.animal.armadillo.Armadillo.ArmadilloState.values()[event.get("state").getAsInt()]));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "creaking_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        for (var field : new String[][] {{"can_move", "CAN_MOVE"}, {"active", "IS_ACTIVE"},
                                {"tearing_down", "IS_TEARING_DOWN"}}) {
                            if (event.has(field[0])) {
                                data.add(wolfData(net.minecraft.world.entity.monster.creaking.Creaking.class, field[1],
                                    event.get(field[0]).getAsBoolean()));
                            }
                        }
                        if (event.has("home")) {
                            data.add(wolfData(net.minecraft.world.entity.monster.creaking.Creaking.class, "HOME_POS",
                                java.util.Optional.of(new net.minecraft.core.BlockPos(0, 0, 0))));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "warden_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        data.add(wolfData(net.minecraft.world.entity.monster.warden.Warden.class, "CLIENT_ANGER_LEVEL",
                            event.get("anger").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "turtle_data" -> {
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        if (event.has("has_egg")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.turtle.Turtle.class, "HAS_EGG",
                                event.get("has_egg").getAsBoolean()));
                        }
                        if (event.has("laying_egg")) {
                            data.add(wolfData(net.minecraft.world.entity.animal.turtle.Turtle.class, "LAYING_EGG",
                                event.get("laying_egg").getAsBoolean()));
                        }
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "fish_data" -> {
                        // A salmon's size or a pufferfish's puff state, which also resize its box.
                        var data = new java.util.ArrayList<net.minecraft.network.syncher.SynchedEntityData.DataValue<?>>();
                        boolean salmon = event.get("fish").getAsString().equals("salmon");
                        data.add(wolfData(salmon ? net.minecraft.world.entity.animal.fish.Salmon.class
                                : net.minecraft.world.entity.animal.fish.Pufferfish.class,
                            salmon ? "DATA_TYPE" : "PUFF_STATE", event.get("value").getAsInt()));
                        listener.handleSetEntityData(
                            new net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket(entityId, data));
                    }
                    case "squid_speed" -> {
                        // What the constructor draws from an unseeded random source: fixed here.
                        var speed = net.minecraft.world.entity.animal.squid.Squid.class
                            .getDeclaredField("tentacleSpeed");
                        speed.setAccessible(true);
                        speed.setFloat(entity, Oracle.f32(event, "speed"));
                    }
                    case "copper_golem_idle" -> {
                        // What the reference draws from an unseeded random source: fixed here.
                        var random = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
                        random.setAccessible(true);
                        ((net.minecraft.util.RandomSource) random.get(entity)).setSeed(event.get("seed").getAsLong());
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
                    case "minecart_steps" -> {
                        // The steps along the track the server took, with the turns as the wire packs them.
                        var steps = new java.util.ArrayList<net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior.MinecartStep>();
                        for (var stepElement : event.getAsJsonArray("steps")) {
                            var step = stepElement.getAsJsonObject();
                            steps.add(new net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior.MinecartStep(
                                new net.minecraft.world.phys.Vec3(Oracle.f64(step, "x"), Oracle.f64(step, "y"), Oracle.f64(step, "z")),
                                new net.minecraft.world.phys.Vec3(Oracle.f64(step, "mx"), Oracle.f64(step, "my"), Oracle.f64(step, "mz")),
                                net.minecraft.util.Mth.unpackDegrees((byte) step.get("y_rot").getAsInt()),
                                net.minecraft.util.Mth.unpackDegrees((byte) step.get("x_rot").getAsInt()),
                                Oracle.f32(step, "weight")));
                        }
                        listener.handleMinecartAlongTrack(
                            new net.minecraft.network.protocol.game.ClientboundMoveMinecartPacket(entityId, steps));
                    }
                    default -> throw new IllegalArgumentException("Unknown event " + event);
                }
            }
            // The level ticks the entities that are not riding; a rider is ticked by its vehicle.
            level.tickNonPassenger(entity.isPassenger() ? entity.getRootVehicle() : entity);
            var described = new JsonObject();
            described.addProperty("hurt_time", living == null ? 0 : living.hurtTime);
            described.addProperty("death_time", living == null ? 0 : living.deathTime);
            described.addProperty("dead", living != null && living.isDeadOrDying());
            described.addProperty("walk_speed", Oracle.bits(living == null ? 0.0F : living.walkAnimation.speed()));
            described.addProperty("walk_position", Oracle.bits(living == null ? 0.0F : living.walkAnimation.position()));
            if (row.has("extras")) {
                for (var extra : row.getAsJsonArray("extras")) {
                    switch (extra.getAsString()) {
                        case "swing" -> {
                            described.addProperty("swinging", living.swinging);
                            described.addProperty("swing_time", living.swingTime);
                            described.addProperty("off_hand",
                                living.swingingArm == net.minecraft.world.InteractionHand.OFF_HAND);
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("attack_anim_" + partial, Oracle.bits(living.getAttackAnim(partial)));
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
                        case "turtle" -> {
                            var turtle = (net.minecraft.world.entity.animal.turtle.Turtle) entity;
                            described.addProperty("in_water", turtle.isInWater());
                            described.addProperty("on_ground", turtle.onGround());
                            described.addProperty("has_egg", turtle.hasEgg());
                            described.addProperty("laying_egg", turtle.isLayingEgg());
                            described.addProperty("is_on_land", !turtle.isInWater() && turtle.onGround());
                            var box = turtle.getBoundingBox();
                            described.addProperty("max_y", Oracle.bits(box.maxY));
                            described.addProperty("max_x", Oracle.bits(box.maxX));
                        }
                        case "axolotl" -> {
                            var axolotl = (net.minecraft.world.entity.animal.axolotl.Axolotl) entity;
                            described.addProperty("in_water", axolotl.isInWater());
                            described.addProperty("variant", axolotl.getVariant().getId());
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("playing_dead_" + partial, Oracle.bits(axolotl.playingDeadAnimator.getFactor(partial)));
                                described.addProperty("in_water_" + partial, Oracle.bits(axolotl.inWaterAnimator.getFactor(partial)));
                                described.addProperty("on_ground_" + partial, Oracle.bits(axolotl.onGroundAnimator.getFactor(partial)));
                                described.addProperty("moving_" + partial, Oracle.bits(axolotl.movingAnimator.getFactor(partial)));
                            }
                            var states = new Object[][] {
                                {"swim", axolotl.swimAnimationState}, {"walk", axolotl.walkAnimationState},
                                {"walk_under_water", axolotl.walkUnderWaterAnimationState},
                                {"idle_under_water", axolotl.idleUnderWaterAnimationState},
                                {"idle_under_water_on_ground", axolotl.idleUnderWaterOnGroundAnimationState},
                                {"idle_on_ground", axolotl.idleOnGroundAnimationState},
                                {"play_dead", axolotl.playDeadAnimationState},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) axolotl.tickCount) : 0L);
                            }
                        }
                        case "boat" -> {
                            var boat = (net.minecraft.world.entity.vehicle.boat.AbstractBoat) entity;
                            described.addProperty("under_water", boat.isUnderWater());
                            described.addProperty("hurt_time", boat.getHurtTime());
                            described.addProperty("hurt_dir", boat.getHurtDir());
                            described.addProperty("damage", Oracle.bits(boat.getDamage()));
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("bubble_angle_" + partial, Oracle.bits(boat.getBubbleAngle(partial)));
                                for (int side = 0; side < 2; side++) {
                                    described.addProperty("rowing_" + side + "_" + partial,
                                        Oracle.bits(boat.getRowingTime(side, partial)));
                                }
                            }
                        }
                        case "bat" -> {
                            var bat = (net.minecraft.world.entity.ambient.Bat) entity;
                            described.addProperty("resting", bat.isResting());
                            described.addProperty("y", Oracle.bits(bat.getY()));
                            described.addProperty("motion_y", Oracle.bits(bat.getDeltaMovement().y));
                            var states = new Object[][] {{"fly", bat.flyAnimationState}, {"rest", bat.restAnimationState}};
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) bat.tickCount) : 0L);
                            }
                        }
                        case "breeze" -> {
                            var breeze = (net.minecraft.world.entity.monster.breeze.Breeze) entity;
                            var states = new Object[][] {
                                {"idle", breeze.idle}, {"shoot", breeze.shoot}, {"slide", breeze.slide},
                                {"slide_back", breeze.slideBack}, {"inhale", breeze.inhale},
                                {"long_jump", breeze.longJump},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) breeze.tickCount) : 0L);
                            }
                        }
                        case "arrow" -> {
                            var arrow = (net.minecraft.world.entity.projectile.arrow.AbstractArrow) entity;
                            described.addProperty("shake_time", arrow.shakeTime);
                            var inGround = net.minecraft.world.entity.projectile.arrow.AbstractArrow.class.getDeclaredMethod("isInGround");
                            inGround.setAccessible(true);
                            described.addProperty("in_ground", (boolean) inGround.invoke(arrow));
                            described.addProperty("tipped", arrow instanceof net.minecraft.world.entity.projectile.arrow.Arrow plain
                                && plain.getColor() > 0);
                        }
                        case "minecart" -> {
                            var cart = (net.minecraft.world.entity.vehicle.minecart.AbstractMinecart) entity;
                            described.addProperty("hurt_time", cart.getHurtTime());
                            described.addProperty("hurt_dir", cart.getHurtDir());
                            described.addProperty("damage", Oracle.bits(cart.getDamage()));
                            described.addProperty("display_block", net.minecraft.world.level.block.Block.getId(
                                cart.getDisplayBlockState()));
                            described.addProperty("display_offset", cart.getDisplayOffset());
                            described.addProperty("fuse", cart instanceof net.minecraft.world.entity.vehicle.minecart.MinecartTNT tntCart
                                ? tntCart.getFuse() : -1);
                            described.addProperty("x", Oracle.bits(cart.getX()));
                            described.addProperty("y", Oracle.bits(cart.getY()));
                            described.addProperty("z", Oracle.bits(cart.getZ()));
                            described.addProperty("y_rot", Oracle.bits(cart.getYRot()));
                            described.addProperty("x_rot", Oracle.bits(cart.getXRot()));
                            if (cart.getBehavior() instanceof net.minecraft.world.entity.vehicle.minecart.NewMinecartBehavior track) {
                                // The new behaviour: its motion, and where its playback of the steps puts it between two ticks.
                                described.addProperty("motion_x", Oracle.bits(cart.getDeltaMovement().x));
                                described.addProperty("motion_y", Oracle.bits(cart.getDeltaMovement().y));
                                described.addProperty("motion_z", Oracle.bits(cart.getDeltaMovement().z));
                                described.addProperty("playing", track.cartHasPosRotLerp());
                                var samples = new JsonArray();
                                // Unseen ticks are not asked (a place asked for is kept by the reference for the next ask).
                                if (track.cartHasPosRotLerp() && tick >= (row.has("sample_from") ? row.get("sample_from").getAsInt() : 0)) {
                                    for (float partial : new float[] {0.0F, 0.5F}) {
                                        var at = track.getCartLerpPosition(partial);
                                        samples.add(Oracle.bits(at.x) + Oracle.bits(at.y) + Oracle.bits(at.z)
                                            + Oracle.bits(track.getCartLerpXRot(partial)) + Oracle.bits(track.getCartLerpYRot(partial)));
                                    }
                                }
                                described.add("lerp", samples);
                            }
                            // Where the rail puts it, and a little ahead and behind it, at some places
                            // (asked on the last tick only: it is the world that answers).
                            var probes = new JsonArray();
                            if (tick + 1 == row.get("tick_count").getAsInt() && row.has("rail_probes")
                                && cart.getBehavior() instanceof net.minecraft.world.entity.vehicle.minecart.OldMinecartBehavior behavior) {
                                for (var probe : row.getAsJsonArray("rail_probes")) {
                                    var at = probe.getAsJsonArray();
                                    double px = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(0).getAsString(), 16));
                                    double py = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(1).getAsString(), 16));
                                    double pz = Double.longBitsToDouble(Long.parseUnsignedLong(at.get(2).getAsString(), 16));
                                    var found = new JsonArray();
                                    for (var place : new net.minecraft.world.phys.Vec3[] {
                                            behavior.getPos(px, py, pz), behavior.getPosOffs(px, py, pz, 0.3F),
                                            behavior.getPosOffs(px, py, pz, -0.3F)}) {
                                        found.add(place == null ? "none" : Oracle.bits(place.x) + Oracle.bits(place.y) + Oracle.bits(place.z));
                                    }
                                    probes.add(found);
                                }
                            }
                            described.add("rail", probes);
                        }
                        case "end_crystal" -> {
                            var crystal = (net.minecraft.world.entity.boss.enderdragon.EndCrystal) entity;
                            var target = crystal.getBeamTarget();
                            described.addProperty("beam_target", target == null ? "none"
                                : target.getX() + "," + target.getY() + "," + target.getZ());
                            described.addProperty("shows_bottom", crystal.showsBottom());
                        }
                        case "ender_dragon" -> {
                            var dragon = (net.minecraft.world.entity.boss.enderdragon.EnderDragon) entity;
                            // Found a tick in ten, by the entity's own random source: only the last tick says.
                            described.addProperty("nearest_crystal", tick + 1 == row.get("tick_count").getAsInt()
                                ? (dragon.nearestCrystal == null ? 0 : dragon.nearestCrystal.getId()) : -1);
                            described.addProperty("flap_time", Oracle.bits(dragon.flapTime));
                            described.addProperty("old_flap_time", Oracle.bits(dragon.oFlapTime));
                            var samples = new JsonArray();
                            for (int delay = 0; delay < 64; delay++) {
                                var sample = dragon.flightHistory.get(delay);
                                var pair = new JsonObject();
                                pair.addProperty("y", Oracle.bits(sample.y()));
                                pair.addProperty("y_rot", Oracle.bits(sample.yRot()));
                                samples.add(pair);
                            }
                            described.add("history", samples);
                            described.addProperty("y_rot", Oracle.bits(dragon.getYRot()));
                            described.addProperty("dragon_death_time", dragon.dragonDeathTime);
                            var parts = new JsonArray();
                            for (var part : dragon.getSubEntities()) {
                                parts.add(Oracle.bits(part.getX()) + Oracle.bits(part.getY()) + Oracle.bits(part.getZ()));
                            }
                            described.add("parts", parts);
                        }
                        case "wither" -> {
                            var wither = (net.minecraft.world.entity.boss.wither.WitherBoss) entity;
                            described.addProperty("invulnerable", wither.getInvulnerableTicks());
                            for (int i = 0; i < 2; i++) {
                                described.addProperty("x_head_" + i, Oracle.bits(wither.getHeadXRots()[i]));
                                described.addProperty("y_head_" + i, Oracle.bits(wither.getHeadYRots()[i]));
                            }
                            described.addProperty("y_rot", Oracle.bits(wither.getYRot()));
                            described.addProperty("y_body_rot", Oracle.bits(wither.yBodyRot));
                        }
                        case "sniffer" -> {
                            var sniffer = (net.minecraft.world.entity.animal.sniffer.Sniffer) entity;
                            var states = new Object[][] {
                                {"digging", sniffer.diggingAnimationState}, {"sniffing", sniffer.sniffingAnimationState},
                                {"rising", sniffer.risingAnimationState}, {"happy", sniffer.feelingHappyAnimationState},
                                {"scenting", sniffer.scentingAnimationState},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) sniffer.tickCount) : 0L);
                            }
                            described.addProperty("searching", sniffer.isSearching());
                        }
                        case "armadillo" -> {
                            var armadillo = (net.minecraft.world.entity.animal.armadillo.Armadillo) entity;
                            var states = new Object[][] {
                                {"roll_out", armadillo.rollOutAnimationState}, {"roll_up", armadillo.rollUpAnimationState},
                                {"peek", armadillo.peekAnimationState},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) armadillo.tickCount) : 0L);
                            }
                            described.addProperty("hiding", armadillo.shouldHideInShell());
                            described.addProperty("scared", armadillo.isScared());
                            described.addProperty("head_yaw", Oracle.bits(armadillo.getYHeadRot()));
                        }
                        case "creaking" -> {
                            var creaking = (net.minecraft.world.entity.monster.creaking.Creaking) entity;
                            var states = new Object[][] {
                                {"attack", creaking.attackAnimationState},
                                {"invulnerable", creaking.invulnerabilityAnimationState},
                                {"death", creaking.deathAnimationState},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) creaking.tickCount) : 0L);
                            }
                            described.addProperty("can_move", creaking.canMove());
                            described.addProperty("active", creaking.isActive());
                            described.addProperty("tearing_down", creaking.isTearingDown());
                            described.addProperty("heart_bound", creaking.isHeartBound());
                        }
                        case "warden" -> {
                            var warden = (net.minecraft.world.entity.monster.warden.Warden) entity;
                            var states = new Object[][] {
                                {"roar", warden.roarAnimationState}, {"sniff", warden.sniffAnimationState},
                                {"emerge", warden.emergeAnimationState}, {"digging", warden.diggingAnimationState},
                                {"attack", warden.attackAnimationState}, {"sonic_boom", warden.sonicBoomAnimationState},
                            };
                            for (var described2 : states) {
                                var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                described.addProperty(described2[0] + "_started", animation.isStarted());
                                described.addProperty(described2[0] + "_millis",
                                    animation.isStarted() ? animation.getTimeInMillis((float) warden.tickCount) : 0L);
                            }
                            // The pulses at the start and at the end of the tick, as they lerp.
                            described.addProperty("tendril_old", Oracle.bits(warden.getTendrilAnimation(0.0F)));
                            described.addProperty("tendril", Oracle.bits(warden.getTendrilAnimation(1.0F)));
                            described.addProperty("heart_old", Oracle.bits(warden.getHeartAnimation(0.0F)));
                            described.addProperty("heart", Oracle.bits(warden.getHeartAnimation(1.0F)));
                        }
                        case "parrot" -> {
                            var parrot = (net.minecraft.world.entity.animal.parrot.Parrot) entity;
                            described.addProperty("variant", parrot.getVariant().getId());
                            described.addProperty("motion_y", Oracle.bits(parrot.getDeltaMovement().y));
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                float flap = net.minecraft.util.Mth.lerp(partial, parrot.oFlap, parrot.flap);
                                float speed = net.minecraft.util.Mth.lerp(partial, parrot.oFlapSpeed, parrot.flapSpeed);
                                described.addProperty("flap_angle_" + partial,
                                    Oracle.bits((net.minecraft.util.Mth.sin(flap) + 1.0F) * speed));
                            }
                        }
                        case "vex" -> {
                            described.addProperty("charging",
                                ((net.minecraft.world.entity.monster.Vex) entity).isCharging());
                        }
                        case "allay" -> {
                            var allay = (net.minecraft.world.entity.animal.allay.Allay) entity;
                            described.addProperty("dancing", allay.isDancing());
                            described.addProperty("spinning", allay.isSpinning());
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("holding_" + partial, Oracle.bits(allay.getHoldingItemAnimationProgress(partial)));
                                described.addProperty("spin_" + partial, Oracle.bits(allay.getSpinningProgress(partial)));
                            }
                        }
                        case "guardian" -> {
                            var guardian = (net.minecraft.world.entity.monster.Guardian) entity;
                            described.addProperty("in_water", guardian.isInWater());
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                described.addProperty("tail_" + partial, Oracle.bits(guardian.getTailAnimation(partial)));
                                described.addProperty("spikes_" + partial, Oracle.bits(guardian.getSpikesAnimation(partial)));
                            }
                            described.addProperty("attack_time", Oracle.bits(guardian.getClientSideAttackTime()));
                            // The turn of the head and body, the flop and the draws the tick took.
                            described.addProperty("y_rot", Oracle.bits(guardian.getYRot()));
                            described.addProperty("x_rot", Oracle.bits(guardian.getXRot()));
                            described.addProperty("head_y", Oracle.bits(guardian.getYHeadRot()));
                            described.addProperty("body_y", Oracle.bits(guardian.yBodyRot));
                            described.addProperty("on_ground", guardian.onGround());
                            described.addProperty("pos_x", Oracle.bits(guardian.getX()));
                            described.addProperty("pos_y", Oracle.bits(guardian.getY()));
                            described.addProperty("pos_z", Oracle.bits(guardian.getZ()));
                            described.addProperty("motion_x", Oracle.bits(guardian.getDeltaMovement().x));
                            described.addProperty("motion_y", Oracle.bits(guardian.getDeltaMovement().y));
                            described.addProperty("motion_z", Oracle.bits(guardian.getDeltaMovement().z));
                            var randomField = net.minecraft.world.entity.Entity.class.getDeclaredField("random");
                            randomField.setAccessible(true);
                            var source = randomField.get(guardian);
                            var seedField = source.getClass().getDeclaredField("seed");
                            seedField.setAccessible(true);
                            described.addProperty("random_state",
                                ((java.util.concurrent.atomic.AtomicLong) seedField.get(source)).get());
                        }
                        case "frog" -> {
                            described.addProperty("in_water", entity.isInWater());
                            if (entity instanceof net.minecraft.world.entity.animal.frog.Frog frog) {
                                var states = new Object[][] {
                                    {"jump", frog.jumpAnimationState}, {"croak", frog.croakAnimationState},
                                    {"tongue", frog.tongueAnimationState},
                                    {"swim_idle", frog.swimIdleAnimationState},
                                };
                                for (var described2 : states) {
                                    var animation = (net.minecraft.world.entity.AnimationState) described2[1];
                                    described.addProperty(described2[0] + "_started", animation.isStarted());
                                    described.addProperty(described2[0] + "_millis",
                                        animation.isStarted() ? animation.getTimeInMillis((float) frog.tickCount) : 0L);
                                }
                            }
                        }
                        case "dolphin" -> {
                            described.addProperty("max_x", Oracle.bits(entity.getBoundingBox().maxX));
                            var motion = entity.getDeltaMovement();
                            described.addProperty("is_moving", motion.horizontalDistanceSqr() > 1.0E-7);
                            described.addProperty("motion_x", Oracle.bits(motion.x));
                            described.addProperty("motion_z", Oracle.bits(motion.z));
                        }
                        case "fish" -> {
                            described.addProperty("in_water", entity.isInWater());
                            var box = entity.getBoundingBox();
                            described.addProperty("min_x", Oracle.bits(box.minX));
                            described.addProperty("min_y", Oracle.bits(box.minY));
                            described.addProperty("min_z", Oracle.bits(box.minZ));
                            described.addProperty("max_x", Oracle.bits(box.maxX));
                            described.addProperty("max_y", Oracle.bits(box.maxY));
                            described.addProperty("max_z", Oracle.bits(box.maxZ));
                        }
                        case "shulker" -> {
                            var shulker = (net.minecraft.world.entity.monster.Shulker) entity;
                            described.addProperty("face", shulker.getAttachFace().get3DDataValue());
                            described.addProperty("color",
                                shulker.getColor() == null ? 16 : shulker.getColor().getId());
                            described.addProperty("peek_half", Oracle.bits(shulker.getClientPeekAmount(0.5F)));
                            described.addProperty("peek_start", Oracle.bits(shulker.getClientPeekAmount(0.0F)));
                            described.addProperty("peek_end", Oracle.bits(shulker.getClientPeekAmount(1.0F)));
                            described.addProperty("y_rot", Oracle.bits(shulker.getYRot()));
                            described.addProperty("y_body_rot", Oracle.bits(shulker.yBodyRot));
                            described.addProperty("y_body_rot_old", Oracle.bits(shulker.yBodyRotO));
                            described.addProperty("y_head_rot", Oracle.bits(shulker.yHeadRot));
                            described.addProperty("pos_x", Oracle.bits(shulker.getX()));
                            described.addProperty("pos_y", Oracle.bits(shulker.getY()));
                            described.addProperty("pos_z", Oracle.bits(shulker.getZ()));
                            described.addProperty("old_x", Oracle.bits(shulker.xOld));
                            described.addProperty("old_y", Oracle.bits(shulker.yOld));
                            described.addProperty("old_z", Oracle.bits(shulker.zOld));
                            described.addProperty("riding", shulker.isPassenger());
                            var shulkerBox = shulker.getBoundingBox();
                            for (var side : new Object[][] {{"min_x", shulkerBox.minX}, {"min_y", shulkerBox.minY},
                                {"min_z", shulkerBox.minZ}, {"max_x", shulkerBox.maxX}, {"max_y", shulkerBox.maxY},
                                {"max_z", shulkerBox.maxZ}}) {
                                described.addProperty("box_" + side[0], Oracle.bits((double) side[1]));
                            }
                            for (float partial : new float[] {0.0F, 0.5F, 1.0F}) {
                                var offset = shulker.getRenderPosition(partial);
                                described.addProperty("render_" + partial,
                                    offset == null ? "none"
                                        : Oracle.bits(offset.x) + "," + Oracle.bits(offset.y) + "," + Oracle.bits(offset.z));
                            }
                        }
                        case "cube" -> {
                            var cube = (net.minecraft.world.entity.monster.cubemob.AbstractCubeMob) entity;
                            described.addProperty("size", cube.getSize());
                            described.addProperty("squish", Oracle.bits(cube.squish));
                            described.addProperty("old_squish", Oracle.bits(cube.oSquish));
                            described.addProperty("target_squish", Oracle.bits(cube.targetSquish));
                            described.addProperty("squish_half",
                                Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, cube.oSquish, cube.squish)));
                            described.addProperty("y_rot", Oracle.bits(cube.getYRot()));
                            described.addProperty("y_body_rot", Oracle.bits(cube.yBodyRot));
                            described.addProperty("on_ground", cube.onGround());
                            if (cube instanceof net.minecraft.world.entity.monster.cubemob.SulfurCube sulfur) {
                                described.addProperty("fuse", sulfur.getFuse());
                            }
                            var box = cube.getBoundingBox();
                            for (var side : new Object[][] {{"min_x", box.minX}, {"min_y", box.minY}, {"min_z", box.minZ},
                                {"max_x", box.maxX}, {"max_y", box.maxY}, {"max_z", box.maxZ}}) {
                                described.addProperty("box_" + side[0], Oracle.bits((double) side[1]));
                            }
                        }
                        case "squid" -> {
                            var squid = (net.minecraft.world.entity.animal.squid.Squid) entity;
                            described.addProperty("in_water", squid.isInWater());
                            described.addProperty("y_rot", Oracle.bits(squid.getYRot()));
                            described.addProperty("y_body_rot", Oracle.bits(squid.yBodyRot));
                            described.addProperty("tentacle_angle", Oracle.bits(squid.tentacleAngle));
                            described.addProperty("x_body_rot", Oracle.bits(squid.xBodyRot));
                            described.addProperty("z_body_rot", Oracle.bits(squid.zBodyRot));
                            described.addProperty("tentacle_angle_half",
                                Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.oldTentacleAngle, squid.tentacleAngle)));
                            described.addProperty("x_body_rot_half",
                                Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.xBodyRotO, squid.xBodyRot)));
                            described.addProperty("z_body_rot_half",
                                Oracle.bits(net.minecraft.util.Mth.lerp(0.5F, squid.zBodyRotO, squid.zBodyRot)));
                            if (squid instanceof net.minecraft.world.entity.animal.squid.GlowSquid glow) {
                                described.addProperty("dark_ticks", glow.getDarkTicksRemaining());
                            }
                            var motion = squid.getDeltaMovement();
                            described.addProperty("motion_x", Oracle.bits(motion.x));
                            described.addProperty("motion_y", Oracle.bits(motion.y));
                            described.addProperty("motion_z", Oracle.bits(motion.z));
                        }
                        case "copper_golem" -> {
                            var golem = (net.minecraft.world.entity.animal.golem.CopperGolem) entity;
                            var playing = new Object[][] {
                                {"idle", golem.getIdleAnimationState()},
                                {"get_item", golem.getInteractionGetItemAnimationState()},
                                {"get_no_item", golem.getInteractionGetNoItemAnimationState()},
                                {"drop_item", golem.getInteractionDropItemAnimationState()},
                                {"drop_no_item", golem.getInteractionDropNoItemAnimationState()},
                            };
                            for (var animation : playing) {
                                var state = (net.minecraft.world.entity.AnimationState) animation[1];
                                described.addProperty(animation[0] + "_started", state.isStarted());
                                described.addProperty(animation[0] + "_millis", state.getTimeInMillis(golem.tickCount));
                            }
                            described.addProperty("state", golem.getState().ordinal());
                            described.addProperty("weather", golem.getWeatherState().ordinal());
                        }
                        case "attributes" -> {
                            described.addProperty("max_health", Oracle.bits(living.getMaxHealth()));
                            described.addProperty("scale", Oracle.bits(living.getScale()));
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
     * The real `ElytraAnimationState` of a real remote player ticked through a scenario: whether it
     * glides, its motion and whether it sneaks each tick, and the spread of the wings after each.
     */
    private static JsonObject elytraState(final JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var setFlag = net.minecraft.world.entity.Entity.class.getDeclaredMethod("setSharedFlag", int.class, boolean.class);
        setFlag.setAccessible(true);
        {
            var empty = new JsonObject();
            empty.add("blocks", new JsonArray());
            empty.addProperty("chunk_radius", 0);
            var scene = buildScene(empty);
            var player = new net.minecraft.client.player.RemotePlayer(scene.level(),
                new com.mojang.authlib.GameProfile(java.util.UUID.nameUUIDFromBytes(new byte[] {5}), "Glider"));
            var state = new net.minecraft.world.entity.ElytraAnimationState(player);
            var ticks = new JsonArray();
            for (var tickElement : row.getAsJsonArray("ticks")) {
                var tick = tickElement.getAsJsonObject();
                setFlag.invoke(player, 7, tick.get("fall_flying").getAsBoolean());
                player.setPose(tick.get("crouching").getAsBoolean()
                    ? net.minecraft.world.entity.Pose.CROUCHING : net.minecraft.world.entity.Pose.STANDING);
                player.setDeltaMovement(Oracle.f64(tick, "x"), Oracle.f64(tick, "y"), Oracle.f64(tick, "z"));
                state.tick();
                var out = new JsonObject();
                for (var partial : new float[] {0.0F, 0.5F, 1.0F}) {
                    var spread = new JsonArray();
                    spread.add(Oracle.bits(state.getRotX(partial)));
                    spread.add(Oracle.bits(state.getRotY(partial)));
                    spread.add(Oracle.bits(state.getRotZ(partial)));
                    out.add("at_" + partial, spread);
                }
                ticks.add(out);
            }
            entry.add("ticks", ticks);
        }
        return entry;
    }

    /**
     * The sky colour and the sky light factor the real client level's attribute system gives
     * with the flash time set, with the option that hides the flash on or off: where nothing is
     * loaded (plains, no weather, the clocks at zero).
     */
    private static JsonObject skyFlash(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = buildScene(row);
        var level = scene.level();
        setField(scene.minecraft(), "level", level);
        scene.options().hideLightningFlash().set(row.get("hide").getAsBoolean());
        level.setSkyFlashTime(row.get("flash").getAsInt());
        level.environmentAttributes().invalidateTickCache();
        var at = new net.minecraft.world.phys.Vec3(Oracle.f64(row, "x"), Oracle.f64(row, "y"), Oracle.f64(row, "z"));
        int color = level.environmentAttributes().getValue(
            net.minecraft.world.attribute.EnvironmentAttributes.SKY_COLOR, at);
        float factor = level.environmentAttributes().getValue(
            net.minecraft.world.attribute.EnvironmentAttributes.SKY_LIGHT_FACTOR, at);
        entry.addProperty("sky_color", color);
        entry.addProperty("sky_light_factor", Oracle.bits(factor));
        return entry;
    }

    /** One vector as three hex words of its doubles. */
    private static JsonArray vectorBits(net.minecraft.world.phys.Vec3 vector) {
        var bits = new JsonArray();
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.x)));
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.y)));
        bits.add(String.format("%016x", Double.doubleToRawLongBits(vector.z)));
        return bits;
    }

    /**
     * What a leash needs of an entity: whether it can be leashed, where the line is tied to it
     * (the standing camel's, which moves with its sitting animation), whether it is tied by four
     * lines and where, and the same for a holder.
     */
    private static void leashSupport(JsonObject out, net.minecraft.world.entity.Entity entity, String prefix) {
        if (entity instanceof net.minecraft.world.entity.Leashable leashable) {
            out.addProperty(prefix + "leashable", true);
            out.add(prefix + "leash_offset", vectorBits(leashable.getLeashOffset(0.0F)));
            out.addProperty(prefix + "quad", leashable.supportQuadLeash());
            if (leashable.supportQuadLeash()) {
                var offsets = new JsonArray();
                for (var offset : leashable.getQuadLeashOffsets()) {
                    offsets.add(vectorBits(offset));
                }
                out.add(prefix + "quad_offsets", offsets);
            }
        }
        out.addProperty(prefix + "quad_holder", entity.supportQuadLeashAsHolder());
        if (entity.supportQuadLeashAsHolder()) {
            var offsets = new JsonArray();
            for (var offset : entity.getQuadLeashHolderOffsets()) {
                offsets.add(vectorBits(offset));
            }
            out.add(prefix + "quad_holder_offsets", offsets);
        }
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
                // Where each passenger sits (`EntityAttachment.PASSENGER`, not yet turned).
                var seats = new JsonArray();
                for (int seat = 0; seat < 8; seat++) {
                    var point = size.attachments().getNullable(
                        net.minecraft.world.entity.EntityAttachment.PASSENGER, seat, 0.0F);
                    if (point == null) {
                        break;
                    }
                    seats.add(vectorBits(point));
                }
                out.add("passengers", seats);
                leashSupport(out, entity, "");
                if (entity instanceof net.minecraft.world.entity.Mob mob) {
                    mob.setBaby(true);
                    if (mob.isBaby()) {
                        var baby = mob.getDimensions(net.minecraft.world.entity.Pose.STANDING);
                        out.addProperty("baby_width", Oracle.bits(baby.width()));
                        out.addProperty("baby_height", Oracle.bits(baby.height()));
                        out.addProperty("baby_eye_height", Oracle.bits(baby.eyeHeight()));
                        leashSupport(out, entity, "baby_");
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
        // A vehicle the player rides, as the server's passengers packet makes it: the player is put
        // aboard and, on a boat, turned the way the boat faces.
        net.minecraft.world.entity.Entity vehicle = null;
        if (row.has("ride")) {
            var ride = row.getAsJsonObject("ride");
            // A boat asks (and a minecart, which reads the features as it is made) for the teams of what it could push (no team here), and sounds ask the camera.
            setField(listener, "scoreboard", new net.minecraft.world.scores.Scoreboard());
            setField(listener, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
            setField(minecraft, "particleEngine", unsafe.allocateInstance(DroppingParticleEngine.class));
            var type = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE.getValue(
                net.minecraft.resources.Identifier.parse(ride.get("type").getAsString()));
            vehicle = type.create(level, new net.minecraft.world.entity.EntitySpawnRequest(
                net.minecraft.world.entity.EntitySpawnReason.LOAD, true));
            vehicle.setId(ride.get("id").getAsInt());
            if (ride.has("saddled") && ride.get("saddled").getAsBoolean()
                    && vehicle instanceof net.minecraft.world.entity.Mob saddled) {
                saddled.setItemSlot(net.minecraft.world.entity.EquipmentSlot.SADDLE,
                    new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SADDLE));
            }
            if (ride.has("holding")) {
                // What the rider holds in the selected slot (a stick steers a pig).
                var held = net.minecraft.core.registries.BuiltInRegistries.ITEM.getValue(
                    net.minecraft.resources.Identifier.parse(ride.get("holding").getAsString()));
                player.getInventory().setItem(0, new net.minecraft.world.item.ItemStack(held));
            }
            if (vehicle instanceof net.minecraft.world.entity.animal.camel.Camel camel) {
                // The game's clock, and how long ago it last changed its pose (sitting counts negative).
                level.setTimeFromServer(1000L);
                if (ride.has("sit_age")) {
                    camel.setPose(net.minecraft.world.entity.Pose.SITTING);
                    camel.resetLastPoseChangeTick(-(1000L - ride.get("sit_age").getAsLong()));
                } else {
                    camel.resetLastPoseChangeTick(0L);
                }
            }
            if (vehicle instanceof net.minecraft.world.entity.animal.equine.AbstractHorse horse) {
                // A tamed horse in a saddle: the one a player controls.
                horse.setTamed(true);
                if (!ride.has("unsaddled")) {
                    horse.setItemSlot(net.minecraft.world.entity.EquipmentSlot.SADDLE,
                        new net.minecraft.world.item.ItemStack(net.minecraft.world.item.Items.SADDLE));
                }
            }
            vehicle.snapTo(Oracle.f64(ride, "x"), Oracle.f64(ride, "y"), Oracle.f64(ride, "z"),
                Oracle.f32(ride, "y_rot"), 0.0F);
            if (vehicle instanceof net.minecraft.world.entity.LivingEntity living) {
                // The constructor turns the head at random; the add-entity packet sets it.
                float facing = Oracle.f32(ride, "y_rot");
                living.yBodyRot = facing;
                living.yHeadRot = facing;
                living.yBodyRotO = facing;
                living.yHeadRotO = facing;
            }
            if (ride.has("motion_x")) {
                vehicle.setDeltaMovement(
                    Oracle.f64(ride, "motion_x"), Oracle.f64(ride, "motion_y"), Oracle.f64(ride, "motion_z"));
            }
            vehicle.syncPacketPositionCodec(vehicle.getX(), vehicle.getY(), vehicle.getZ());
            level.addEntity(vehicle);
            level.addEntity(player);
            setField(minecraft, "level", level);
            player.startRiding(vehicle, true, false);
            if (vehicle instanceof net.minecraft.world.entity.vehicle.boat.AbstractBoat) {
                player.yRotO = vehicle.getYRot();
                player.setYRot(vehicle.getYRot());
                player.setYHeadRot(vehicle.getYRot());
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
            level.tickNonPassenger(vehicle != null ? vehicle : player);
            var described = describe(player, wire.sent);
            if (vehicle != null) {
                described.add("vehicle", describeVehicle(vehicle));
            }
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

    /** What the level keeps of a vehicle the player rides, after a tick. */
    private static JsonObject describeVehicle(net.minecraft.world.entity.Entity vehicle) throws Exception {
        var out = new JsonObject();
        out.addProperty("x", Oracle.bits(vehicle.getX()));
        out.addProperty("y", Oracle.bits(vehicle.getY()));
        out.addProperty("z", Oracle.bits(vehicle.getZ()));
        out.addProperty("motion_x", Oracle.bits(vehicle.getDeltaMovement().x));
        out.addProperty("motion_y", Oracle.bits(vehicle.getDeltaMovement().y));
        out.addProperty("motion_z", Oracle.bits(vehicle.getDeltaMovement().z));
        out.addProperty("y_rot", Oracle.bits(vehicle.getYRot()));
        out.addProperty("x_rot", Oracle.bits(vehicle.getXRot()));
        out.addProperty("on_ground", vehicle.onGround());
        out.addProperty("horizontal_collision", vehicle.horizontalCollision);
        out.addProperty("vertical_collision", vehicle.verticalCollision);
        out.addProperty("fall_distance", Oracle.bits(vehicle.fallDistance));
        if (vehicle instanceof net.minecraft.world.entity.animal.equine.AbstractHorse) {
            out.addProperty("pending_jump", Oracle.bits((float) getField(vehicle, "playerJumpPendingScale")));
            if (vehicle instanceof net.minecraft.world.entity.animal.camel.Camel camel) {
                out.addProperty("dash_cooldown", camel.getJumpCooldown());
                out.addProperty("camel_sitting", camel.isCamelSitting());
                out.addProperty("dashing", camel.isDashing());
            }
            out.addProperty("y_head_rot", Oracle.bits(((net.minecraft.world.entity.LivingEntity) vehicle).yHeadRot));
        }
        if (vehicle instanceof net.minecraft.world.entity.vehicle.boat.AbstractBoat boat) {
            out.addProperty("status", String.valueOf(getField(boat, "status")));
            out.addProperty("delta_rotation", Oracle.bits((float) getField(boat, "deltaRotation")));
            out.addProperty("paddle_left", boat.getPaddleState(0));
            out.addProperty("paddle_right", boat.getPaddleState(1));
            out.addProperty("water_level", Oracle.bits((double) getField(boat, "waterLevel")));
            out.addProperty("land_friction", Oracle.bits((float) getField(boat, "landFriction")));
            out.addProperty("last_yd", Oracle.bits((double) getField(boat, "lastYd")));
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
