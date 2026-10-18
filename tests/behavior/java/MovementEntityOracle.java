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

    /** `BlockModelLighter` on quads the row describes, in the level the row builds. */
    static JsonObject blockLighting(JsonObject row) throws Exception {
        var entry = new JsonObject();
        entry.addProperty("label", row.get("label").getAsString());
        var scene = litScene(row);
        var level = scene.level();
        if (row.has("cardinal") && row.get("cardinal").getAsString().equals("nether")) {
            // Only the face shade comes from the dimension type, which is what
            // `ClientLevel.cardinalLighting` reads. The overworld's type with
            // the nether's cardinal light keeps the level's height and lighting
            // as built.
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
