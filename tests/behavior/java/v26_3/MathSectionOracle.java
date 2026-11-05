import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.JsonOps;
import it.unimi.dsi.fastutil.longs.LongConsumer;
import java.util.stream.Stream;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.ChunkPos;

class MathSectionOracle extends MathPlayerCollisionFixtures {
    static JsonObject chunkPosBlockCoordinates(ChunkPos pos, int offset) {
        var state = new JsonObject();
        state.addProperty("min_block_x", Integer.toString(pos.getMinBlockX()));
        state.addProperty("min_block_z", Integer.toString(pos.getMinBlockZ()));
        state.addProperty("middle_block_x", Integer.toString(pos.getMiddleBlockX()));
        state.addProperty("middle_block_z", Integer.toString(pos.getMiddleBlockZ()));
        state.addProperty("max_block_x", Integer.toString(pos.getMaxBlockX()));
        state.addProperty("max_block_z", Integer.toString(pos.getMaxBlockZ()));
        state.addProperty("block_x", Integer.toString(pos.getBlockX(offset)));
        state.addProperty("block_z", Integer.toString(pos.getBlockZ(offset)));
        return state;
    }

    static JsonObject chunkPosRegionCoordinates(ChunkPos pos, int regionX, int regionZ) {
        var state = new JsonObject();
        state.addProperty("region_x", Integer.toString(pos.getRegionX()));
        state.addProperty("region_z", Integer.toString(pos.getRegionZ()));
        state.addProperty("packed_region_x", Integer.toString(ChunkPos.getRegionX(pos.pack())));
        state.addProperty("packed_region_z", Integer.toString(ChunkPos.getRegionZ(pos.pack())));
        state.addProperty("region_local_x", Integer.toString(pos.getRegionLocalX()));
        state.addProperty("region_local_z", Integer.toString(pos.getRegionLocalZ()));
        state.add("min_from_region", snapshot(ChunkPos.minFromRegion(regionX, regionZ)));
        state.add("max_from_region", snapshot(ChunkPos.maxFromRegion(regionX, regionZ)));
        return state;
    }

    static JsonObject chunkPosBlockPositions(ChunkPos pos, int blockX, int y, int blockZ, JsonArray queries) {
        var state = new JsonObject();
        state.add("block_at", snapshot(pos.getBlockAt(blockX, y, blockZ)));
        state.add("middle_block_position", snapshot(pos.getMiddleBlockPosition(y)));
        state.add("world_position", snapshot(pos.getWorldPosition()));
        var contains = new JsonArray();
        for (var entry : queries) {
            contains.add(pos.contains(blockPos(entry.getAsJsonObject())));
        }
        state.add("contains", contains);
        return state;
    }

    static JsonObject chunkPosRepresentation(ChunkPos pos, int hashX, int hashZ) {
        var state = new JsonObject();
        state.addProperty("to_string", pos.toString());
        state.addProperty("hash_code", Integer.toString(pos.hashCode()));
        state.addProperty("static_hash", Integer.toString(ChunkPos.hash(hashX, hashZ)));
        return state;
    }

    static JsonObject chunkPosFromBlockPos(BlockPos pos) {
        var state = new JsonObject();
        state.add("containing", snapshot(ChunkPos.containing(pos)));
        state.addProperty("packed", Long.toUnsignedString(ChunkPos.pack(pos), 16));
        return state;
    }

    static JsonObject chunkPosDistances(ChunkPos pos, int otherX, int otherZ) {
        ChunkPos other = new ChunkPos(otherX, otherZ);
        var state = new JsonObject();
        state.addProperty("chessboard_chunk", Integer.toString(pos.getChessboardDistance(other)));
        state.addProperty("chessboard_coordinates", Integer.toString(pos.getChessboardDistance(otherX, otherZ)));
        state.addProperty("distance_squared_chunk", Integer.toString(pos.distanceSquared(other)));
        state.addProperty("distance_squared_packed", Integer.toString(pos.distanceSquared(other.pack())));
        return state;
    }

    static JsonObject chunkPosRangeClosed(JsonObject request) {
        Stream<ChunkPos> positions;
        if (request.get("mode").getAsString().equals("center-radius")) {
            ChunkPos center = chunkPos(request);
            int radius = Integer.parseInt(request.get("radius").getAsString());
            positions = ChunkPos.rangeClosed(center, radius);
        } else if (request.get("mode").getAsString().equals("from-to")) {
            ChunkPos from = new ChunkPos(
                Integer.parseInt(request.get("from_x").getAsString()),
                Integer.parseInt(request.get("from_z").getAsString())
            );
            ChunkPos to = new ChunkPos(
                Integer.parseInt(request.get("to_x").getAsString()),
                Integer.parseInt(request.get("to_z").getAsString())
            );
            positions = ChunkPos.rangeClosed(from, to);
        } else {
            throw new IllegalArgumentException("Unknown ChunkPos range mode");
        }
        var values = new JsonArray();
        positions.forEach(pos -> values.add(snapshot(pos)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    static JsonObject sectionPosRanges(JsonObject request) {
        Stream<SectionPos> positions;
        switch (request.get("mode").getAsString()) {
            case "between" -> positions = SectionPos.betweenClosedStream(
                Integer.parseInt(request.get("min_x").getAsString()),
                Integer.parseInt(request.get("min_y").getAsString()),
                Integer.parseInt(request.get("min_z").getAsString()),
                Integer.parseInt(request.get("max_x").getAsString()),
                Integer.parseInt(request.get("max_y").getAsString()),
                Integer.parseInt(request.get("max_z").getAsString()));
            case "cube" -> positions = SectionPos.cube(
                SectionPos.of(
                    Integer.parseInt(request.get("x").getAsString()),
                    Integer.parseInt(request.get("y").getAsString()),
                    Integer.parseInt(request.get("z").getAsString())),
                Integer.parseInt(request.get("radius").getAsString()));
            case "around-chunk" -> positions = SectionPos.aroundChunk(
                new ChunkPos(
                    Integer.parseInt(request.get("chunk_x").getAsString()),
                    Integer.parseInt(request.get("chunk_z").getAsString())),
                Integer.parseInt(request.get("radius").getAsString()),
                Integer.parseInt(request.get("min_section").getAsString()),
                Integer.parseInt(request.get("max_section").getAsString()));
            default -> throw new IllegalArgumentException("Unknown SectionPos range mode");
        }
        var values = new JsonArray();
        positions.forEach(pos -> values.add(snapshot(pos)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    static JsonObject sectionPosBlocksInside(JsonObject request) {
        SectionPos pos = SectionPos.of(
            Integer.parseInt(request.get("x").getAsString()),
            Integer.parseInt(request.get("y").getAsString()),
            Integer.parseInt(request.get("z").getAsString()));
        var values = new JsonArray();
        pos.blocksInside().forEach(block -> values.add(snapshot(block)));
        var state = new JsonObject();
        state.add("positions", values);
        return state;
    }

    static JsonObject sectionPosAroundAndAtBlockPos(JsonObject request) {
        var values = new JsonArray();
        LongConsumer sectionConsumer = sectionNode -> values.add(Long.toUnsignedString(sectionNode, 16));
        switch (request.get("mode").getAsString()) {
            case "coordinates" -> SectionPos.aroundAndAtBlockPos(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()),
                sectionConsumer);
            case "blockpos" -> SectionPos.aroundAndAtBlockPos(blockPos(request), sectionConsumer);
            case "packed" -> SectionPos.aroundAndAtBlockPos(
                Long.parseUnsignedLong(request.get("block_node").getAsString(), 16),
                sectionConsumer);
            default -> throw new IllegalArgumentException("Unknown SectionPos around-and-at mode");
        }
        var state = new JsonObject();
        state.add("section_nodes", values);
        return state;
    }

    static JsonObject chunkPosConstant(JsonObject request) {
        var state = new JsonObject();
        switch (request.get("name").getAsString()) {
            case "zero" -> state.add("chunk_pos", snapshot(ChunkPos.ZERO));
            case "invalid_chunk_pos" -> state.addProperty("packed", Long.toUnsignedString(ChunkPos.INVALID_CHUNK_POS, 16));
            case "region_size" -> state.addProperty("value", Integer.toString(ChunkPos.REGION_SIZE));
            case "region_max_index" -> state.addProperty("value", Integer.toString(ChunkPos.REGION_MAX_INDEX));
            default -> throw new IllegalArgumentException("Unknown ChunkPos constant");
        }
        return state;
    }

    static JsonObject chunkPosValidity(ChunkPos pos) {
        var output = System.out;
        SharedConstants.setVersion(TEST_VERSION);
        Bootstrap.bootStrap();
        System.setOut(output);
        var state = new JsonObject();
        state.addProperty("instance", pos.isValid());
        state.addProperty("static", ChunkPos.isValid(pos.x(), pos.z()));
        return state;
    }

    static JsonObject chunkPosCodecDecodeResult(DataResult<ChunkPos> decoded) {
        var result = new JsonObject();
        decoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            result.add("value", snapshot(value));
        });
        decoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
            error.partialValue().ifPresent(value -> result.add("partial", snapshot(value)));
        });
        return result;
    }

    static JsonObject chunkPosCodecEncodeResult(DataResult<JsonElement> encoded) {
        var result = new JsonObject();
        encoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            var values = new JsonArray();
            for (var entry : value.getAsJsonArray()) {
                values.add(entry.getAsInt());
            }
            result.add("value", values);
        });
        encoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
        });
        return result;
    }

    static JsonObject sectionRelativeCase(JsonObject request) {
        var result = new JsonObject();
        short relative = SectionPos.sectionRelativePos(blockPos(request));
        result.addProperty("relative", Integer.toString(Short.toUnsignedInt(relative)));
        result.addProperty("x", Integer.toString(SectionPos.sectionRelativeX(relative)));
        result.addProperty("y", Integer.toString(SectionPos.sectionRelativeY(relative)));
        result.addProperty("z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
        return result;
    }

    static JsonObject chunkposCodecCase(JsonObject request) {
        var result = new JsonObject();
        String mode = request.get("mode").getAsString();
        if (mode.equals("decode")) {
            var input = new JsonArray();
            for (var entry : request.getAsJsonArray("values")) {
                input.add(Integer.parseInt(entry.getAsString()));
            }
            DataResult<ChunkPos> decoded = ChunkPos.CODEC.parse(JsonOps.INSTANCE, input);
            result = chunkPosCodecDecodeResult(decoded);
        } else if (mode.equals("encode")) {
            DataResult<JsonElement> encoded = ChunkPos.CODEC.encodeStart(JsonOps.INSTANCE, chunkPos(request));
            result = chunkPosCodecEncodeResult(encoded);
        } else {
            throw new IllegalArgumentException("Unknown ChunkPos codec mode");
        }
        return result;
    }

    static JsonObject chunkposBlockCoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        int offset = Integer.parseInt(request.get("offset").getAsString());
        result.add("value", chunkPosBlockCoordinates(pos, offset));
        return result;
    }

    static JsonObject chunkposRegionCoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        int regionX = Integer.parseInt(request.get("region_x").getAsString());
        int regionZ = Integer.parseInt(request.get("region_z").getAsString());
        result.add("value", chunkPosRegionCoordinates(pos, regionX, regionZ));
        return result;
    }

    static JsonObject chunkposBlockPositionsCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        int blockX = Integer.parseInt(request.get("block_x").getAsString());
        int y = Integer.parseInt(request.get("y").getAsString());
        int blockZ = Integer.parseInt(request.get("block_z").getAsString());
        result.add("value", chunkPosBlockPositions(pos, blockX, y, blockZ, request.getAsJsonArray("contains")));
        return result;
    }

    static JsonObject chunkposRepresentationCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        int hashX = Integer.parseInt(request.get("hash_x").getAsString());
        int hashZ = Integer.parseInt(request.get("hash_z").getAsString());
        result.add("value", chunkPosRepresentation(pos, hashX, hashZ));
        return result;
    }

    static JsonObject chunkposFromBlockposCase(JsonObject request) {
        var result = new JsonObject();
        BlockPos pos = blockPos(request);
        result.add("value", chunkPosFromBlockPos(pos));
        return result;
    }

    static JsonObject chunkposDistancesCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        int otherX = Integer.parseInt(request.get("other_x").getAsString());
        int otherZ = Integer.parseInt(request.get("other_z").getAsString());
        result.add("value", chunkPosDistances(pos, otherX, otherZ));
        return result;
    }

    static JsonObject chunkposFromSectionNodeCase(JsonObject request) {
        var result = new JsonObject();
        long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
        result.addProperty("value", Long.toUnsignedString(ChunkPos.fromSectionNode(sectionNode), 16));
        return result;
    }

    static JsonObject chunkposRangeClosedCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", chunkPosRangeClosed(request));
        return result;
    }

    static JsonObject chunkposConstantCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", chunkPosConstant(request));
        return result;
    }

    static JsonObject chunkposPackedAccessorsCase(JsonObject request) {
        var result = new JsonObject();
        long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
        var value = new JsonObject();
        value.addProperty("x", Integer.toString(ChunkPos.getX(packed)));
        value.addProperty("z", Integer.toString(ChunkPos.getZ(packed)));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposPackedAccessorsCase(JsonObject request) {
        var result = new JsonObject();
        long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
        var value = new JsonObject();
        value.addProperty("x", Integer.toString(SectionPos.x(packed)));
        value.addProperty("y", Integer.toString(SectionPos.y(packed)));
        value.addProperty("z", Integer.toString(SectionPos.z(packed)));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposBlockCoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        SectionPos pos = SectionPos.of(
            Integer.parseInt(request.get("x").getAsString()),
            Integer.parseInt(request.get("y").getAsString()),
            Integer.parseInt(request.get("z").getAsString()));
        var value = new JsonObject();
        value.addProperty("min_block_x", Integer.toString(pos.minBlockX()));
        value.addProperty("min_block_y", Integer.toString(pos.minBlockY()));
        value.addProperty("min_block_z", Integer.toString(pos.minBlockZ()));
        value.addProperty("max_block_x", Integer.toString(pos.maxBlockX()));
        value.addProperty("max_block_y", Integer.toString(pos.maxBlockY()));
        value.addProperty("max_block_z", Integer.toString(pos.maxBlockZ()));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposRelativeToBlockCase(JsonObject request) {
        var result = new JsonObject();
        SectionPos pos = SectionPos.of(
            Integer.parseInt(request.get("x").getAsString()),
            Integer.parseInt(request.get("y").getAsString()),
            Integer.parseInt(request.get("z").getAsString()));
        short relative = Short.parseShort(request.get("relative").getAsString());
        var value = new JsonObject();
        value.addProperty("relative_x", Integer.toString(SectionPos.sectionRelativeX(relative)));
        value.addProperty("relative_y", Integer.toString(SectionPos.sectionRelativeY(relative)));
        value.addProperty("relative_z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
        value.addProperty("block_x", Integer.toString(pos.relativeToBlockX(relative)));
        value.addProperty("block_y", Integer.toString(pos.relativeToBlockY(relative)));
        value.addProperty("block_z", Integer.toString(pos.relativeToBlockZ(relative)));
        value.add("block_position", snapshot(pos.relativeToBlockPos(relative)));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposSpatialCase(JsonObject request) {
        var result = new JsonObject();
        SectionPos pos = SectionPos.of(
            Integer.parseInt(request.get("x").getAsString()),
            Integer.parseInt(request.get("y").getAsString()),
            Integer.parseInt(request.get("z").getAsString()));
        var value = new JsonObject();
        value.add("origin", snapshot(pos.origin()));
        value.add("center", snapshot(pos.center()));
        value.add("chunk", snapshot(pos.chunk()));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposPackedTransformationsCase(JsonObject request) {
        var result = new JsonObject();
        long blockNode = Long.parseUnsignedLong(request.get("block_node").getAsString(), 16);
        long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
        int x = Integer.parseInt(request.get("x").getAsString());
        int z = Integer.parseInt(request.get("z").getAsString());
        var value = new JsonObject();
        value.addProperty("block_to_section", Long.toUnsignedString(SectionPos.blockToSection(blockNode), 16));
        value.addProperty("zero_node", Long.toUnsignedString(SectionPos.getZeroNode(sectionNode), 16));
        value.addProperty("zero_node_coordinates", Long.toUnsignedString(SectionPos.getZeroNode(x, z), 16));
        value.addProperty("section_to_chunk", Long.toUnsignedString(SectionPos.sectionToChunk(sectionNode), 16));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposAsLongCase(JsonObject request) {
        var result = new JsonObject();
        long blockNode = Long.parseUnsignedLong(request.get("block_node").getAsString(), 16);
        int x = Integer.parseInt(request.get("x").getAsString());
        int y = Integer.parseInt(request.get("y").getAsString());
        int z = Integer.parseInt(request.get("z").getAsString());
        int sectionX = Integer.parseInt(request.get("section_x").getAsString());
        int sectionY = Integer.parseInt(request.get("section_y").getAsString());
        int sectionZ = Integer.parseInt(request.get("section_z").getAsString());
        var value = new JsonObject();
        value.addProperty("from_block_pos", Long.toUnsignedString(SectionPos.asLong(BlockPos.of(blockNode)), 16));
        value.addProperty("coordinates", Long.toUnsignedString(SectionPos.asLong(x, y, z), 16));
        value.addProperty("instance", Long.toUnsignedString(SectionPos.of(sectionX, sectionY, sectionZ).asLong(), 16));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposOffsetCase(JsonObject request) {
        var result = new JsonObject();
        long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
        Direction direction = Direction.byName(request.get("direction").getAsString());
        if (direction == null) {
            throw new IllegalArgumentException("Unknown SectionPos offset direction");
        }
        int stepX = Integer.parseInt(request.get("step_x").getAsString());
        int stepY = Integer.parseInt(request.get("step_y").getAsString());
        int stepZ = Integer.parseInt(request.get("step_z").getAsString());
        int x = Integer.parseInt(request.get("x").getAsString());
        int y = Integer.parseInt(request.get("y").getAsString());
        int z = Integer.parseInt(request.get("z").getAsString());
        SectionPos instance = SectionPos.of(x, y, z).offset(stepX, stepY, stepZ);
        var value = new JsonObject();
        value.addProperty("packed_direction", Long.toUnsignedString(SectionPos.offset(sectionNode, direction), 16));
        value.addProperty("packed_steps", Long.toUnsignedString(SectionPos.offset(sectionNode, stepX, stepY, stepZ), 16));
        value.addProperty("instance_x", Integer.toString(instance.x()));
        value.addProperty("instance_y", Integer.toString(instance.y()));
        value.addProperty("instance_z", Integer.toString(instance.z()));
        value.addProperty("instance_packed", Long.toUnsignedString(instance.asLong(), 16));
        result.add("value", value);
        return result;
    }

    static JsonObject sectionposRangesCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", sectionPosRanges(request));
        return result;
    }

    static JsonObject sectionposBlocksInsideCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", sectionPosBlocksInside(request));
        return result;
    }

    static JsonObject sectionposAroundAndAtBlockPosCase(JsonObject request) {
        var result = new JsonObject();
        result.add("value", sectionPosAroundAndAtBlockPos(request));
        return result;
    }

    static JsonObject sectionposDoubleCoordinatesCase(JsonObject request) {
        var result = new JsonObject();
        double value = f64(request, "value");
        String fn = request.get("fn").getAsString();
        int out = switch (fn) {
            case "pos_to_section" -> SectionPos.posToSectionCoord(value);
            case "block_to_section" -> SectionPos.blockToSectionCoord(value);
            default -> throw new IllegalArgumentException("Unknown SectionPos double coordinate function");
        };
        result.addProperty("value", Integer.toString(out));
        return result;
    }

    static JsonObject sectionposOfOverloadsCase(JsonObject request) {
        var result = new JsonObject();
        SectionPos value = switch (request.get("mode").getAsString()) {
            case "blockpos" -> SectionPos.of(blockPos(request));
            case "chunkpos" -> SectionPos.of(chunkPos(request),
                Integer.parseInt(request.get("section_y").getAsString()));
            case "position" -> SectionPos.of(vec3(request));
            case "packed" -> SectionPos.of(Long.parseUnsignedLong(
                request.get("section_node").getAsString(), 16));
            default -> throw new IllegalArgumentException("Unknown SectionPos of overload mode");
        };
        result.add("value", snapshot(value));
        return result;
    }

    static JsonObject chunkposValidityCase(JsonObject request) {
        var result = new JsonObject();
        ChunkPos pos = chunkPos(request);
        result.add("value", chunkPosValidity(pos));
        return result;
    }
}
