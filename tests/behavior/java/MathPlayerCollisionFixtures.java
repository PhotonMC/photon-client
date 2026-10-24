import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.List;
import java.util.Locale;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.entity.player.Player;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.LadderBlock;
import net.minecraft.world.level.block.TrapDoorBlock;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.entity.LevelCallback;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;
import sun.misc.Unsafe;

class MathPlayerCollisionFixtures extends MathPassengerFixtures {
    static final class HappyGhastClientLevelFixture extends net.minecraft.client.multiplayer.ClientLevel {
        HappyGhastClientLevelFixture() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        @Override
        public boolean isClientSide() {
            return true;
        }

        @Override
        public net.minecraft.world.flag.FeatureFlagSet enabledFeatures() {
            return net.minecraft.world.flag.FeatureFlagSet.of();
        }
    }

    static final class HappyGhastCollisionFixtureEntity
        extends net.minecraft.world.entity.animal.happyghast.HappyGhast {
        static final net.minecraft.world.level.Level CLIENT_LEVEL = createClientLevel();
        boolean baby;
        boolean alive;
        boolean stillTimeout;
        boolean vehicle;

        HappyGhastCollisionFixtureEntity() {
            super(net.minecraft.world.entity.EntityTypes.HAPPY_GHAST, null);
            this.baby = false;
            this.alive = true;
            this.stillTimeout = false;
            this.vehicle = false;
        }

        static net.minecraft.world.level.Level createClientLevel() {
            try {
                return (net.minecraft.world.level.Level)unsafeInstance()
                    .allocateInstance(HappyGhastClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate client level fixture", error);
            }
        }

        static HappyGhastCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean baby, boolean alive,
            boolean stillTimeout, boolean vehicle
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                HappyGhastCollisionFixtureEntity entity =
                    (HappyGhastCollisionFixtureEntity)unsafe.allocateInstance(
                        HappyGhastCollisionFixtureEntity.class);
                entity.baby = baby;
                entity.alive = alive;
                entity.stillTimeout = stillTimeout;
                entity.vehicle = vehicle;
                entity.setBoundingBox(boundingBox);
                var field = Reflect.field(Entity.class, "position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate happy ghast collision fixture entity", error);
            }
        }

        @Override
        public int getAge() {
            return this.baby ? -1 : 1;
        }

        @Override
        public boolean isAlive() {
            return this.alive;
        }

        @Override
        public boolean isOnStillTimeout() {
            return this.stillTimeout;
        }

        @Override
        public boolean isVehicle() {
            return this.vehicle;
        }

        @Override
        public boolean isPushable() {
            return this.alive;
        }

        @Override
        public net.minecraft.world.level.Level level() {
            return CLIENT_LEVEL;
        }

        @Override
        public net.minecraft.world.item.ItemStack getMainHandItem() {
            return net.minecraft.world.item.ItemStack.EMPTY;
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }
    }

    static final class PlayerCollisionFixtureEntity
        extends net.minecraft.server.level.ServerPlayer {
        static final net.minecraft.server.level.ServerLevel CLIMBABLE_LEVEL =
            ClimbableClientLevelFixture.createClientLevel();
        boolean spectator;
        boolean alive;
        boolean onClimbable;
        boolean flying;
        boolean fallFlying;
        boolean useClimbableContext;
        BlockState inBlockState;

        PlayerCollisionFixtureEntity() {
            super(null, null, null, null);
        }

        static PlayerCollisionFixtureEntity create(AABB boundingBox, Vec3 position) {
            return create(boundingBox, position, false, true, false, false, null);
        }

        static PlayerCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean spectator, boolean alive,
            boolean onClimbable, boolean flying
        ) {
            return create(boundingBox, position, spectator, alive, onClimbable, flying, null);
        }

        static PlayerCollisionFixtureEntity create(
            AABB boundingBox, Vec3 position, boolean spectator, boolean alive,
            boolean onClimbable, boolean flying, JsonObject climbableContext
        ) {
            try {
                Unsafe unsafe = unsafeInstance();
                PlayerCollisionFixtureEntity entity =
                    (PlayerCollisionFixtureEntity)unsafe.allocateInstance(
                        PlayerCollisionFixtureEntity.class);
                entity.spectator = spectator;
                entity.alive = alive;
                entity.onClimbable = onClimbable;
                entity.flying = flying;
                entity.fallFlying = climbableContext != null
                    && climbableContext.has("fall_flying")
                    && climbableContext.get("fall_flying").getAsBoolean();
                entity.useClimbableContext = climbableContext != null;
                entity.setBoundingBox(boundingBox);
                var field = Reflect.field(Entity.class, "position");
                unsafe.putObject(entity, unsafe.objectFieldOffset(field), position);
                if (climbableContext != null) {
                    entity.inBlockState = createClimbableBlockState(climbableContext);
                    var abilitiesField = Reflect.field(net.minecraft.world.entity.player.Player.class, "abilities");
                    var abilities = new net.minecraft.world.entity.player.Abilities();
                    abilities.flying = flying;
                    unsafe.putObject(entity, unsafe.objectFieldOffset(abilitiesField), abilities);
                }
                return entity;
            } catch (ReflectiveOperationException error) {
                throw new IllegalStateException("Unable to allocate player collision fixture entity", error);
            }
        }

        static BlockState createClimbableBlockState(JsonObject context) {
            String kind = context.get("kind").getAsString();
            if (kind.equals("trapdoor_ladder")) {
                boolean open = context.has("trapdoor_open")
                    && context.get("trapdoor_open").getAsBoolean();
                Direction trapdoorFacing = Direction.valueOf(
                    context.get("trapdoor_facing").getAsString().toUpperCase(Locale.ROOT));
                Direction ladderFacing = Direction.valueOf(
                    context.get("ladder_facing").getAsString().toUpperCase(Locale.ROOT));
                boolean belowIsLadder = !context.has("below_is_ladder")
                    || context.get("below_is_ladder").getAsBoolean();
                BlockState belowState = belowIsLadder
                    ? Blocks.LADDER.defaultBlockState().setValue(LadderBlock.FACING, ladderFacing)
                    : Blocks.AIR.defaultBlockState();
                ClimbableClientLevelFixture.setBelowState(belowState);
                return Blocks.OAK_TRAPDOOR.defaultBlockState()
                    .setValue(TrapDoorBlock.OPEN, open)
                    .setValue(TrapDoorBlock.FACING, trapdoorFacing);
            } else if (kind.equals("climbable_tag")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                bindClimbableTag(state.getBlock());
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else if (kind.equals("glide_through_tag")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                bindClimbableAndGlideThroughTags(state.getBlock());
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else if (kind.equals("update_tags_packet")) {
                int stateId = context.get("state_id").getAsInt();
                BlockState state = Block.stateById(stateId);
                applyUpdateTagsPacket(context);
                ClimbableClientLevelFixture.setBelowState(Blocks.AIR.defaultBlockState());
                return state;
            } else {
                throw new IllegalArgumentException("Unknown climbable context kind");
            }
        }

        static void applyUpdateTagsPacket(JsonObject context) {
            java.util.Map<net.minecraft.resources.Identifier, it.unimi.dsi.fastutil.ints.IntList> tags =
                new java.util.HashMap<>();
            JsonObject blockTags = context.getAsJsonObject("block_tags");
            for (var entry : blockTags.entrySet()) {
                it.unimi.dsi.fastutil.ints.IntList values = new it.unimi.dsi.fastutil.ints.IntArrayList();
                for (JsonElement value : entry.getValue().getAsJsonArray()) {
                    values.add(value.getAsInt());
                }
                tags.put(net.minecraft.resources.Identifier.parse(entry.getKey()), values);
            }
            var networkPayload = new net.minecraft.tags.TagNetworkSerialization.NetworkPayload(tags);
            java.util.Map<net.minecraft.resources.ResourceKey<? extends net.minecraft.core.Registry<?>>,
                net.minecraft.tags.TagNetworkSerialization.NetworkPayload> registries =
                new java.util.HashMap<>();
            registries.put(BuiltInRegistries.BLOCK.key(), networkPayload);
            var packet = new net.minecraft.network.protocol.common.ClientboundUpdateTagsPacket(registries);
            var blockPayload = packet.getTags().get(BuiltInRegistries.BLOCK.key());
            BuiltInRegistries.BLOCK.prepareTagReload(
                blockPayload.resolve(BuiltInRegistries.BLOCK)).apply();
        }

        static void bindClimbableAndGlideThroughTags(Block block) {
            java.util.List<net.minecraft.core.Holder<Block>> climbableValues = block == Blocks.VINE
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.List<net.minecraft.core.Holder<Block>> glideThroughValues = block == Blocks.VINE
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.Map<net.minecraft.tags.TagKey<Block>, java.util.List<net.minecraft.core.Holder<Block>>> tags =
                java.util.Map.of(
                    net.minecraft.tags.BlockTags.CLIMBABLE, climbableValues,
                    net.minecraft.tags.BlockTags.CAN_GLIDE_THROUGH, glideThroughValues);
            BuiltInRegistries.BLOCK.prepareTagReload(
                new net.minecraft.tags.TagLoader.LoadResult<>(
                    net.minecraft.core.registries.Registries.BLOCK, tags)).apply();
        }

        static void bindClimbableTag(Block block) {
            java.util.List<net.minecraft.core.Holder<Block>> values = block == Blocks.LADDER
                ? java.util.List.of(BuiltInRegistries.BLOCK.wrapAsHolder(block))
                : java.util.List.of();
            java.util.Map<net.minecraft.tags.TagKey<Block>, java.util.List<net.minecraft.core.Holder<Block>>> tags =
                java.util.Map.of(net.minecraft.tags.BlockTags.CLIMBABLE, values);
            BuiltInRegistries.BLOCK.prepareTagReload(
                new net.minecraft.tags.TagLoader.LoadResult<>(
                    net.minecraft.core.registries.Registries.BLOCK, tags)).apply();
        }

        @Override
        public boolean isPassengerOfSameVehicle(Entity other) {
            return false;
        }

        @Override
        public boolean isAlive() {
            return this.alive;
        }

        @Override
        public boolean isSpectator() {
            return this.spectator;
        }

        @Override
        public boolean onClimbable() {
            return this.useClimbableContext
                ? super.onClimbable()
                : this.flying ? false : this.onClimbable;
        }

        @Override
        public BlockState getInBlockState() {
            return this.useClimbableContext
                ? this.inBlockState
                : Blocks.AIR.defaultBlockState();
        }

        @Override
        public boolean isFallFlying() {
            return this.fallFlying;
        }

        @Override
        public net.minecraft.server.level.ServerLevel level() {
            return CLIMBABLE_LEVEL;
        }

        @Override
        public boolean isShiftKeyDown() {
            return false;
        }

        @Override
        public net.minecraft.world.item.ItemStack getMainHandItem() {
            return net.minecraft.world.item.ItemStack.EMPTY;
        }
    }

    static final class ClimbableClientLevelFixture
        extends net.minecraft.server.level.ServerLevel {
        static BlockState belowState;
        static boolean clientSide;
        static boolean hasChunk = true;
        static int minY = -64;

        ClimbableClientLevelFixture() {
            super(null, null, null, null, null, null, false, 0L, null, false);
        }

        static net.minecraft.server.level.ServerLevel createClientLevel() {
            try {
                return (net.minecraft.server.level.ServerLevel)unsafeInstance()
                    .allocateInstance(ClimbableClientLevelFixture.class);
            } catch (InstantiationException error) {
                throw new IllegalStateException("Unable to allocate climbable level fixture", error);
            }
        }

        static void setBelowState(BlockState state) {
            belowState = state;
        }

        static void setTravelState(
            boolean clientSideState, boolean hasChunkState, int minimumY) {
            clientSide = clientSideState;
            hasChunk = hasChunkState;
            minY = minimumY;
        }

        @Override
        public BlockState getBlockState(BlockPos pos) {
            return pos.getY() == 0 && belowState != null
                ? belowState
                : Blocks.AIR.defaultBlockState();
        }

        @Override
        public FluidState getFluidState(BlockPos pos) {
            return Blocks.AIR.defaultBlockState().getFluidState();
        }

        @Override
        public boolean isClientSide() {
            return clientSide;
        }

        @Override
        public boolean hasChunkAt(BlockPos pos) {
            return hasChunk;
        }

        @Override
        public int getMinY() {
            return minY;
        }
    }

    static final class NoopEntityCallbacks implements LevelCallback<Entity> {
        @Override
        public void onCreated(Entity entity) {
        }

        @Override
        public void onDestroyed(Entity entity) {
        }

        @Override
        public void onTickingStart(Entity entity) {
        }

        @Override
        public void onTickingEnd(Entity entity) {
        }

        @Override
        public void onTrackingStart(Entity entity) {
        }

        @Override
        public void onTrackingEnd(Entity entity) {
        }

        @Override
        public void onSectionChange(Entity entity) {
        }
    }
}
