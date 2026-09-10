import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.mojang.math.OctahedralGroup;
import com.mojang.serialization.DataResult;
import com.mojang.serialization.JsonOps;
import it.unimi.dsi.fastutil.longs.LongConsumer;
import java.util.ArrayList;
import java.util.EnumSet;
import java.util.stream.Stream;
import net.minecraft.DetectedVersion;
import net.minecraft.SharedConstants;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.SectionPos;
import net.minecraft.server.Bootstrap;
import net.minecraft.util.Mth;
import net.minecraft.util.RandomSource;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.PositionalRandomFactory;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.block.state.properties.AttachFace;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.phys.shapes.BooleanOp;
import net.minecraft.world.phys.shapes.Shapes;
import net.minecraft.world.phys.shapes.VoxelShape;
import org.joml.Vector3f;

public final class MathOracle {
    public static JsonObject run(JsonObject request) {
        String operation = request.get("operation").getAsString();
        var result = new JsonObject();
        if (operation.equals("vec2")) {
            Vec2 vector = new Vec2(Oracle.f32(request, "x"), Oracle.f32(request, "y"));
            var events = new JsonArray();
            events.add(snapshot(vector));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                vector = switch (step.get("action").getAsString()) {
                    case "normalize" -> vector.normalized();
                    case "scale" -> vector.scale(Oracle.f32(step, "factor"));
                    default -> throw new IllegalArgumentException("Unknown Vec2 action");
                };
                events.add(snapshot(vector));
            }
            result.add("events", events);
        } else if (operation.equals("direction")) {
            Direction facing = Direction.byName(request.get("name").getAsString());
            if (facing == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            var events = new JsonArray();
            events.add(snapshot(facing));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown axis name");
                }
                facing = step.get("clockwise").getAsBoolean()
                    ? facing.getClockWise(axis) : facing.getCounterClockWise(axis);
                events.add(snapshot(facing));
            }
            result.add("events", events);
        } else if (operation.equals("coord")) {
            int value = Integer.parseInt(request.get("value").getAsString());
            String fn = request.get("fn").getAsString();
            int out = switch (fn) {
                case "block_to_section" -> SectionPos.blockToSectionCoord(value);
                case "section_to_block" -> SectionPos.sectionToBlockCoord(value);
                case "section_relative" -> SectionPos.sectionRelative(value);
                default -> throw new IllegalArgumentException("Unknown coord function");
            };
            result.addProperty("value", Integer.toString(out));
            return result;
        } else if (operation.equals("blockpos")) {
            BlockPos pos = blockPos(request);
            var events = new JsonArray();
            events.add(snapshot(pos));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                pos = switch (step.get("action").getAsString()) {
                    case "offset" -> pos.offset(
                        Integer.parseInt(step.get("x").getAsString()),
                        Integer.parseInt(step.get("y").getAsString()),
                        Integer.parseInt(step.get("z").getAsString()));
                    case "min" -> BlockPos.min(pos, blockPos(step));
                    case "max" -> BlockPos.max(pos, blockPos(step));
                    default -> throw new IllegalArgumentException("Unknown BlockPos action");
                };
                events.add(snapshot(pos));
            }
            result.add("events", events);
        } else if (operation.equals("section_relative")) {
            short relative = SectionPos.sectionRelativePos(blockPos(request));
            result.addProperty("relative", Integer.toString(Short.toUnsignedInt(relative)));
            result.addProperty("x", Integer.toString(SectionPos.sectionRelativeX(relative)));
            result.addProperty("y", Integer.toString(SectionPos.sectionRelativeY(relative)));
            result.addProperty("z", Integer.toString(SectionPos.sectionRelativeZ(relative)));
        } else if (operation.equals("vec3_codec")) {
            String mode = request.get("mode").getAsString();
            if (mode.equals("decode")) {
                var input = new JsonArray();
                for (var entry : request.getAsJsonArray("values")) {
                    input.add(Double.longBitsToDouble(Long.parseUnsignedLong(entry.getAsString(), 16)));
                }
                DataResult<Vec3> decoded = Vec3.CODEC.parse(JsonOps.INSTANCE, input);
                result = codecDecodeResult(decoded);
            } else if (mode.equals("encode")) {
                DataResult<JsonElement> encoded = Vec3.CODEC.encodeStart(JsonOps.INSTANCE, vec3(request));
                result = codecEncodeResult(encoded);
            } else {
                throw new IllegalArgumentException("Unknown Vec3 codec mode");
            }
        } else if (operation.equals("chunkpos_codec")) {
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
        } else if (operation.equals("chunkpos_block_coordinates")) {
            ChunkPos pos = chunkPos(request);
            int offset = Integer.parseInt(request.get("offset").getAsString());
            result.add("value", chunkPosBlockCoordinates(pos, offset));
        } else if (operation.equals("chunkpos_region_coordinates")) {
            ChunkPos pos = chunkPos(request);
            int regionX = Integer.parseInt(request.get("region_x").getAsString());
            int regionZ = Integer.parseInt(request.get("region_z").getAsString());
            result.add("value", chunkPosRegionCoordinates(pos, regionX, regionZ));
        } else if (operation.equals("chunkpos_block_positions")) {
            ChunkPos pos = chunkPos(request);
            int blockX = Integer.parseInt(request.get("block_x").getAsString());
            int y = Integer.parseInt(request.get("y").getAsString());
            int blockZ = Integer.parseInt(request.get("block_z").getAsString());
            result.add("value", chunkPosBlockPositions(pos, blockX, y, blockZ, request.getAsJsonArray("contains")));
        } else if (operation.equals("chunkpos_representation")) {
            ChunkPos pos = chunkPos(request);
            int hashX = Integer.parseInt(request.get("hash_x").getAsString());
            int hashZ = Integer.parseInt(request.get("hash_z").getAsString());
            result.add("value", chunkPosRepresentation(pos, hashX, hashZ));
        } else if (operation.equals("chunkpos_from_blockpos")) {
            BlockPos pos = blockPos(request);
            result.add("value", chunkPosFromBlockPos(pos));
        } else if (operation.equals("chunkpos_distances")) {
            ChunkPos pos = chunkPos(request);
            int otherX = Integer.parseInt(request.get("other_x").getAsString());
            int otherZ = Integer.parseInt(request.get("other_z").getAsString());
            result.add("value", chunkPosDistances(pos, otherX, otherZ));
        } else if (operation.equals("chunkpos_from_section_node")) {
            long sectionNode = Long.parseUnsignedLong(request.get("section_node").getAsString(), 16);
            result.addProperty("value", Long.toUnsignedString(ChunkPos.fromSectionNode(sectionNode), 16));
        } else if (operation.equals("chunkpos_range_closed")) {
            result.add("value", chunkPosRangeClosed(request));
        } else if (operation.equals("chunkpos_constant")) {
            result.add("value", chunkPosConstant(request));
        } else if (operation.equals("chunkpos_packed_accessors")) {
            long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
            var value = new JsonObject();
            value.addProperty("x", Integer.toString(ChunkPos.getX(packed)));
            value.addProperty("z", Integer.toString(ChunkPos.getZ(packed)));
            result.add("value", value);
        } else if (operation.equals("sectionpos_packed_accessors")) {
            long packed = Long.parseUnsignedLong(request.get("packed").getAsString(), 16);
            var value = new JsonObject();
            value.addProperty("x", Integer.toString(SectionPos.x(packed)));
            value.addProperty("y", Integer.toString(SectionPos.y(packed)));
            value.addProperty("z", Integer.toString(SectionPos.z(packed)));
            result.add("value", value);
        } else if (operation.equals("sectionpos_block_coordinates")) {
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
        } else if (operation.equals("sectionpos_relative_to_block")) {
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
        } else if (operation.equals("sectionpos_spatial")) {
            SectionPos pos = SectionPos.of(
                Integer.parseInt(request.get("x").getAsString()),
                Integer.parseInt(request.get("y").getAsString()),
                Integer.parseInt(request.get("z").getAsString()));
            var value = new JsonObject();
            value.add("origin", snapshot(pos.origin()));
            value.add("center", snapshot(pos.center()));
            value.add("chunk", snapshot(pos.chunk()));
            result.add("value", value);
        } else if (operation.equals("sectionpos_packed_transformations")) {
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
        } else if (operation.equals("sectionpos_as_long")) {
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
        } else if (operation.equals("sectionpos_offset")) {
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
        } else if (operation.equals("sectionpos_ranges")) {
            result.add("value", sectionPosRanges(request));
        } else if (operation.equals("sectionpos_blocks_inside")) {
            result.add("value", sectionPosBlocksInside(request));
        } else if (operation.equals("sectionpos_around_and_at_block_pos")) {
            result.add("value", sectionPosAroundAndAtBlockPos(request));
        } else if (operation.equals("sectionpos_double_coordinates")) {
            double value = f64(request, "value");
            String fn = request.get("fn").getAsString();
            int out = switch (fn) {
                case "pos_to_section" -> SectionPos.posToSectionCoord(value);
                case "block_to_section" -> SectionPos.blockToSectionCoord(value);
                default -> throw new IllegalArgumentException("Unknown SectionPos double coordinate function");
            };
            result.addProperty("value", Integer.toString(out));
        } else if (operation.equals("sectionpos_of_overloads")) {
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
        } else if (operation.equals("chunkpos_validity")) {
            ChunkPos pos = chunkPos(request);
            result.add("value", chunkPosValidity(pos));
        } else if (operation.equals("mth_integer_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (width.equals("i32")) {
                int input = Integer.parseInt(request.get("input").getAsString());
                int divisor = Integer.parseInt(request.get("divisor").getAsString());
                int value = switch (function) {
                    case "floor_div" -> Mth.floorDiv(input, divisor);
                    case "positive_modulo" -> Mth.positiveModulo(input, divisor);
                    case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                    case "round_toward" -> Mth.roundToward(input, divisor);
                    default -> throw new IllegalArgumentException("Unknown Mth integer function");
                };
                result.addProperty("value", Integer.toString(value));
            } else if (width.equals("i64")) {
                long input = Long.parseLong(request.get("input").getAsString());
                long divisor = Long.parseLong(request.get("divisor").getAsString());
                long value = switch (function) {
                    case "positive_ceil_div" -> Mth.positiveCeilDiv(input, divisor);
                    case "round_toward" -> Mth.roundToward(input, divisor);
                    default -> throw new IllegalArgumentException("Unknown Mth long function");
                };
                result.addProperty("value", Long.toString(value));
            } else {
                throw new IllegalArgumentException("Unknown Mth integer type");
            }
        } else if (operation.equals("mth_integer_basics")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (width.equals("i32")) {
                int input = Integer.parseInt(request.get("input").getAsString());
                if (function.equals("is_multiple")) {
                    result.addProperty("value", Mth.isMultipleOf(input,
                        Integer.parseInt(request.get("divisor").getAsString())));
                } else {
                    int value = switch (function) {
                        case "abs" -> Mth.abs(input);
                        case "clamp" -> Mth.clamp(input,
                            Integer.parseInt(request.get("min").getAsString()),
                            Integer.parseInt(request.get("max").getAsString()));
                        case "wrap_degrees" -> Mth.wrapDegrees(input);
                        default -> throw new IllegalArgumentException("Unknown Mth integer basic function");
                    };
                    result.addProperty("value", Integer.toString(value));
                }
            } else if (width.equals("i64")) {
                long input = Long.parseLong(request.get("input").getAsString());
                if (!function.equals("clamp")) {
                    throw new IllegalArgumentException("Unknown Mth long basic function");
                }
                long value = Mth.clamp(input,
                    Long.parseLong(request.get("min").getAsString()),
                    Long.parseLong(request.get("max").getAsString()));
                result.addProperty("value", Long.toString(value));
            } else {
                throw new IllegalArgumentException("Unknown Mth integer basic type");
            }
        } else if (operation.equals("mth_angle_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (function.equals("pack_degrees")) {
                if (!width.equals("f32")) {
                    throw new IllegalArgumentException("Unknown Mth packDegrees type");
                }
                byte value = Mth.packDegrees(Oracle.f32(request, "input"));
                result.addProperty("value", Byte.toString(value));
            } else if (function.equals("unpack_degrees")) {
                if (!width.equals("i8")) {
                    throw new IllegalArgumentException("Unknown Mth unpackDegrees type");
                }
                byte input = Byte.parseByte(request.get("input").getAsString());
                result.addProperty("bits", Oracle.bits(Mth.unpackDegrees(input)));
            } else if (function.equals("wrap_degrees")) {
                if (!width.equals("i64")) {
                    throw new IllegalArgumentException("Unknown Mth long wrapDegrees type");
                }
                long input = Long.parseLong(request.get("input").getAsString());
                result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(input)));
            } else {
                throw new IllegalArgumentException("Unknown Mth angle function");
            }
        } else if (operation.equals("mth_rounding_helpers")) {
            String width = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (function.equals("ceil")) {
                int value;
                if (width.equals("f32")) {
                    value = Mth.ceil(Oracle.f32(request, "input"));
                } else if (width.equals("f64")) {
                    value = Mth.ceil(f64(request, "input"));
                } else {
                    throw new IllegalArgumentException("Unknown Mth ceil type");
                }
                result.addProperty("value", Integer.toString(value));
            } else if (function.equals("lfloor")) {
                if (!width.equals("f64")) {
                    throw new IllegalArgumentException("Unknown Mth lfloor type");
                }
                result.addProperty("value", Long.toString(Mth.lfloor(f64(request, "input"))));
            } else if (function.equals("ceil_long")) {
                if (!width.equals("f64")) {
                    throw new IllegalArgumentException("Unknown Mth ceilLong type");
                }
                result.addProperty("value", Long.toString(Mth.ceilLong(f64(request, "input"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth rounding function");
            }
        } else if (operation.equals("mth_positive_modulo")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                    Oracle.f32(request, "input"), Oracle.f32(request, "mod"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.positiveModulo(
                    f64(request, "input"), f64(request, "mod"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth positiveModulo type");
            }
        } else if (operation.equals("mth_abs_max")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.absMax(
                    Oracle.f32(request, "first"), Oracle.f32(request, "second"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.absMax(
                    f64(request, "first"), f64(request, "second"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth absMax type");
            }
        } else if (operation.equals("mth_equal")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("equal", Mth.equal(
                    Oracle.f32(request, "first"), Oracle.f32(request, "second")));
            } else if (width.equals("f64")) {
                result.addProperty("equal", Mth.equal(
                    f64(request, "first"), f64(request, "second")));
            } else {
                throw new IllegalArgumentException("Unknown Mth equal type");
            }
        } else if (operation.equals("mth_clamped_lerp")) {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                    Oracle.f32(request, "factor"), Oracle.f32(request, "min"),
                    Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.clampedLerp(
                    f64(request, "factor"), f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth clampedLerp type");
            }
        } else if (operation.equals("mth_next_random")) {
            RandomSource random = RandomSource.create(
                Long.parseLong(request.get("seed").getAsString()));
            String width = request.get("type").getAsString();
            if (width.equals("i32")) {
                result.addProperty("value", Integer.toString(Mth.nextInt(random,
                    Integer.parseInt(request.get("min").getAsString()),
                    Integer.parseInt(request.get("max").getAsString()))));
            } else if (width.equals("f32")) {
                result.addProperty("bits", Oracle.bits(Mth.nextFloat(random,
                    Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
            } else if (width.equals("f64")) {
                result.addProperty("bits", Oracle.bits(Mth.nextDouble(random,
                    f64(request, "min"), f64(request, "max"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth next random type");
            }
        } else if (operation.equals("mth_random_helpers")) {
            RandomSource random = RandomSource.create(
                Long.parseLong(request.get("seed").getAsString()));
            String function = request.get("fn").getAsString();
            if (function.equals("inclusive")) {
                int value = Mth.randomBetweenInclusive(random,
                    Integer.parseInt(request.get("min").getAsString()),
                    Integer.parseInt(request.get("max").getAsString()));
                result.addProperty("value", Integer.toString(value));
                result.addProperty("follow_up", Integer.toString(random.nextInt()));
            } else if (function.equals("between")) {
                float value = Mth.randomBetween(random,
                    Oracle.f32(request, "min"), Oracle.f32(request, "max"));
                result.addProperty("bits", Oracle.bits(value));
                result.addProperty("follow_up_bits", Oracle.bits(random.nextFloat()));
            } else if (function.equals("normal")) {
                float value = Mth.normal(random,
                    Oracle.f32(request, "mean"), Oracle.f32(request, "deviation"));
                result.addProperty("bits", Oracle.bits(value));
                result.addProperty("follow_up_bits", Oracle.bits(random.nextGaussian()));
            } else {
                throw new IllegalArgumentException("Unknown Mth random helper");
            }
        } else if (operation.equals("mth_length")) {
            String type = request.get("type").getAsString();
            String function = request.get("fn").getAsString();
            if (type.equals("f64_2")) {
                double x = f64(request, "x");
                double y = f64(request, "y");
                double value = switch (function) {
                    case "length_squared" -> Mth.lengthSquared(x, y);
                    case "length" -> Mth.length(x, y);
                    default -> throw new IllegalArgumentException("Unknown Mth length function");
                };
                result.addProperty("bits", Oracle.bits(value));
            } else if (type.equals("f32_2")) {
                if (!function.equals("length")) {
                    throw new IllegalArgumentException("Unknown Mth length function");
                }
                result.addProperty("bits", Oracle.bits(Mth.length(
                    Oracle.f32(request, "x"), Oracle.f32(request, "y"))));
            } else if (type.equals("f64_3")) {
                double x = f64(request, "x");
                double y = f64(request, "y");
                double z = f64(request, "z");
                double value = switch (function) {
                    case "length_squared" -> Mth.lengthSquared(x, y, z);
                    case "length" -> Mth.length(x, y, z);
                    default -> throw new IllegalArgumentException("Unknown Mth length function");
                };
                result.addProperty("bits", Oracle.bits(value));
            } else if (type.equals("f32_3")) {
                if (!function.equals("length_squared")) {
                    throw new IllegalArgumentException("Unknown Mth length function");
                }
                result.addProperty("bits", Oracle.bits(Mth.lengthSquared(
                    Oracle.f32(request, "x"), Oracle.f32(request, "y"),
                    Oracle.f32(request, "z"))));
            } else {
                throw new IllegalArgumentException("Unknown Mth length type");
            }
        } else if (operation.equals("mth_quantize")) {
            result.addProperty("value", Integer.toString(Mth.quantize(
                f64(request, "value"), Integer.parseInt(request.get("resolution").getAsString()))));
        } else if (operation.equals("vec3")) {
            Vec3 vector = new Vec3(f64(request, "x"), f64(request, "y"), f64(request, "z"));
            var events = new JsonArray();
            events.add(snapshot(vector));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                vector = switch (step.get("action").getAsString()) {
                    case "normalize" -> vector.normalize();
                    case "scale" -> vector.scale(f64(step, "factor"));
                    case "reverse" -> vector.reverse();
                    case "horizontal" -> vector.horizontal();
                    case "multiply_vec3" -> vector.multiply(vec3(step.getAsJsonObject("scale")));
                    case "multiply_components" -> vector.multiply(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "add_vec3" -> vector.add(vec3(step.getAsJsonObject("delta")));
                    case "add_scalar" -> vector.add(f64(step, "value"));
                    case "add_components" -> vector.add(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "subtract_vec3" -> vector.subtract(vec3(step.getAsJsonObject("delta")));
                    case "subtract_scalar" -> vector.subtract(f64(step, "value"));
                    case "subtract_components" -> vector.subtract(
                        f64(step, "x"), f64(step, "y"), f64(step, "z"));
                    case "vector_to" -> vector.vectorTo(vec3(step.getAsJsonObject("target")));
                    case "cross" -> vector.cross(vec3(step.getAsJsonObject("other")));
                    default -> throw new IllegalArgumentException("Unknown Vec3 action");
                };
                events.add(snapshot(vector));
            }
            result.add("events", events);
        } else if (operation.equals("vec3_dot")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.dot(second)));
        } else if (operation.equals("vec3_distance_to_sqr")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.distanceToSqr(second)));
        } else if (operation.equals("vec3_distance_to")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("value", Oracle.bits(first.distanceTo(second)));
        } else if (operation.equals("vec3_distance_to_sqr_components")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", Oracle.bits(first.distanceToSqr(
                f64(request, "x"), f64(request, "y"), f64(request, "z"))));
        } else if (operation.equals("vec3_horizontal_distances")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("distance", Oracle.bits(first.horizontalDistance()));
            result.addProperty("distance_squared", Oracle.bits(first.horizontalDistanceSqr()));
        } else if (operation.equals("vec3_closer_than")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("position", first.closerThan(second, f64(request, "distance")));
            result.addProperty("xz_y", first.closerThan(
                second, f64(request, "distance_xz"), f64(request, "distance_y")));
        } else if (operation.equals("vec3_offset_random")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            float offset = Oracle.f32(request, "offset");
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("offset_random", snapshot(first.offsetRandom(random, offset)));
            RandomSource randomXZ = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("offset_random_xz", snapshot(first.offsetRandomXZ(randomXZ, offset)));
        } else if (operation.equals("vec3_lerp")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.add("value", snapshot(first.lerp(second, f64(request, "alpha"))));
        } else if (operation.equals("vec3_rotate_clockwise_90")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.rotateClockwise90()));
        } else if (operation.equals("vec3_is_finite")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", first.isFinite());
        } else if (operation.equals("vec3_get")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.addProperty("value", Oracle.bits(first.get(axis)));
        } else if (operation.equals("vec3_with")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
            if (axis == null) {
                throw new IllegalArgumentException("Unknown axis name");
            }
            result.add("value", snapshot(first.with(axis, f64(request, "value"))));
        } else if (operation.equals("vec3_relative")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Direction direction = Direction.byName(request.get("direction").getAsString());
            if (direction == null) {
                throw new IllegalArgumentException("Unknown direction name");
            }
            result.add("value", snapshot(first.relative(direction, f64(request, "distance"))));
        } else if (operation.equals("vec3_align")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            EnumSet<Direction.Axis> axes = EnumSet.noneOf(Direction.Axis.class);
            for (var entry : request.getAsJsonArray("axes")) {
                Direction.Axis axis = Direction.Axis.byName(entry.getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown axis name");
                }
                axes.add(axis);
            }
            result.add("value", snapshot(first.align(axes)));
        } else if (operation.equals("vec3_projected_on")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 onto = vec3(request.getAsJsonObject("onto"));
            result.add("value", snapshot(first.projectedOn(onto)));
        } else if (operation.equals("vec3_to_vector3f")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.toVector3f()));
        } else if (operation.equals("vec3_coordinates")) {
            BlockPos pos = blockPos(request);
            String kind = request.get("kind").getAsString();
            Vec3 value = switch (kind) {
                case "lower_corner" -> Vec3.atLowerCornerOf(pos);
                case "lower_corner_offset" -> Vec3.atLowerCornerWithOffset(
                    pos, f64(request, "offset_x"), f64(request, "offset_y"), f64(request, "offset_z"));
                case "center" -> Vec3.atCenterOf(pos);
                case "bottom_center" -> Vec3.atBottomCenterOf(pos);
                case "up_from_bottom_center" -> Vec3.upFromBottomCenterOf(pos, f64(request, "y_offset"));
                default -> throw new IllegalArgumentException("Unknown Vec3 coordinate helper");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_equality")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            Vec3 second = vec3(request.getAsJsonObject("second"));
            result.addProperty("equals", first.equals(second));
            result.addProperty("first_hash", Integer.toString(first.hashCode()));
            result.addProperty("second_hash", Integer.toString(second.hashCode()));
        } else if (operation.equals("vec3_to_string")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.addProperty("value", first.toString());
        } else if (operation.equals("vec3_rotate")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            float radians = Oracle.f32(request, "radians");
            Vec3 value = switch (request.get("axis").getAsString()) {
                case "x" -> first.xRot(radians);
                case "y" -> first.yRot(radians);
                case "z" -> first.zRot(radians);
                default -> throw new IllegalArgumentException("Unknown Vec3 rotation axis");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_direction_from_rotation")) {
            Vec3 value = switch (request.get("kind").getAsString()) {
                case "components" -> Vec3.directionFromRotation(
                    Oracle.f32(request, "rot_x"), Oracle.f32(request, "rot_y"));
                case "vec2" -> {
                    JsonObject rotation = request.getAsJsonObject("rotation");
                    yield Vec3.directionFromRotation(new Vec2(
                        Oracle.f32(rotation, "x"), Oracle.f32(rotation, "y")));
                }
                default -> throw new IllegalArgumentException("Unknown Vec3 direction rotation kind");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_local_coordinates")) {
            Vec3 value = switch (request.get("kind").getAsString()) {
                case "apply" -> Vec3.applyLocalCoordinatesToRotation(
                    new Vec2(
                        Oracle.f32(request.getAsJsonObject("rotation"), "x"),
                        Oracle.f32(request.getAsJsonObject("rotation"), "y")),
                    vec3(request.getAsJsonObject("direction")));
                case "add" -> vec3(request.getAsJsonObject("first"))
                    .addLocalCoordinates(vec3(request.getAsJsonObject("direction")));
                default -> throw new IllegalArgumentException("Unknown Vec3 local coordinate kind");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_rotation")) {
            Vec3 first = vec3(request.getAsJsonObject("first"));
            result.add("value", snapshot(first.rotation()));
        } else if (operation.equals("vec2_rotate")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 vector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(vector.rotate(f64(request, "angle"))));
        } else if (operation.equals("vec2_dot")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.addProperty("value", Oracle.bits(firstVector.dot(secondVector)));
        } else if (operation.equals("vec2_add_vec2")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.add("value", snapshot(firstVector.add(secondVector)));
        } else if (operation.equals("vec2_add_scalar")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(firstVector.add(Oracle.f32(request, "value"))));
        } else if (operation.equals("vec2_distance_to_sqr")) {
            JsonObject first = request.getAsJsonObject("first");
            JsonObject second = request.getAsJsonObject("second");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            Vec2 secondVector = new Vec2(Oracle.f32(second, "x"), Oracle.f32(second, "y"));
            result.addProperty("value", Oracle.bits(firstVector.distanceToSqr(secondVector)));
        } else if (operation.equals("vec2_negated")) {
            JsonObject first = request.getAsJsonObject("first");
            Vec2 firstVector = new Vec2(Oracle.f32(first, "x"), Oracle.f32(first, "y"));
            result.add("value", snapshot(firstVector.negated()));
        } else if (operation.equals("vec2_equality")) {
            JsonObject firstObject = request.getAsJsonObject("first");
            JsonObject secondObject = request.getAsJsonObject("second");
            Vec2 first = new Vec2(Oracle.f32(firstObject, "x"), Oracle.f32(firstObject, "y"));
            Vec2 second = new Vec2(Oracle.f32(secondObject, "x"), Oracle.f32(secondObject, "y"));
            result.addProperty("equals", first.equals(second));
            result.addProperty("first_hash", Integer.toString(first.hashCode()));
            result.addProperty("second_hash", Integer.toString(second.hashCode()));
        } else if (operation.equals("vec2_constant")) {
            Vec2 value = switch (request.get("name").getAsString()) {
                case "zero" -> Vec2.ZERO;
                case "one" -> Vec2.ONE;
                case "unit_x" -> Vec2.UNIT_X;
                case "neg_unit_x" -> Vec2.NEG_UNIT_X;
                case "unit_y" -> Vec2.UNIT_Y;
                case "neg_unit_y" -> Vec2.NEG_UNIT_Y;
                case "max" -> Vec2.MAX;
                case "min" -> Vec2.MIN;
                default -> throw new IllegalArgumentException("Unknown Vec2 constant");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("vec3_constant")) {
            Vec3 value = switch (request.get("name").getAsString()) {
                case "zero" -> Vec3.ZERO;
                case "x_axis" -> Vec3.X_AXIS;
                case "y_axis" -> Vec3.Y_AXIS;
                case "z_axis" -> Vec3.Z_AXIS;
                default -> throw new IllegalArgumentException("Unknown Vec3 constant");
            };
            result.add("value", snapshot(value));
        } else if (operation.equals("shape")) {
            String kind = request.get("kind").getAsString();
            if (kind.equals("single")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var events = new JsonArray();
                events.add(snapshot(shape));
                for (var entry : request.getAsJsonArray("steps")) {
                    JsonObject step = entry.getAsJsonObject();
                    shape = switch (step.get("op").getAsString()) {
                        case "move" -> shape.move(f64(step, "dx"), f64(step, "dy"), f64(step, "dz"));
                        case "move_vec3" -> shape.move(vec3(step.getAsJsonObject("delta")));
                        case "move_block_pos" -> shape.move(blockPos(step.getAsJsonObject("delta")));
                        default -> throw new IllegalArgumentException("Unknown VoxelShape op");
                    };
                    events.add(snapshot(shape));
                }
                result.add("events", events);
            } else if (kind.equals("join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                BooleanOp op = switch (request.get("op").getAsString()) {
                    case "false" -> BooleanOp.FALSE;
                    case "or" -> BooleanOp.OR;
                    case "and" -> BooleanOp.AND;
                    case "only_first" -> BooleanOp.ONLY_FIRST;
                    case "only_second" -> BooleanOp.ONLY_SECOND;
                    case "not_same" -> BooleanOp.NOT_SAME;
                    case "second" -> BooleanOp.SECOND;
                    case "first" -> BooleanOp.FIRST;
                    default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
                };
                result.add("shape", snapshot(Shapes.join(first, second, op)));
                result.addProperty("join_is_not_empty", Shapes.joinIsNotEmpty(first, second, op));
            } else if (kind.equals("join_many")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("join_many needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                result.add("shape", snapshot(Shapes.or(first, tail)));
            } else if (kind.equals("single_encompassing")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("single_encompassing needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                result.add("shape", snapshot(Shapes.or(first, tail).singleEncompassing()));
            } else if (kind.equals("optimize")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("optimize needs at least one shape");
                }
                VoxelShape shape = voxelShape(entries.get(0).getAsJsonObject());
                for (int i = 1; i < entries.size(); i++) {
                    shape = Shapes.joinUnoptimized(
                        shape,
                        voxelShape(entries.get(i).getAsJsonObject()),
                        BooleanOp.OR
                    );
                }
                var events = new JsonArray();
                events.add(snapshotWithCoords(shape));
                events.add(snapshotWithCoords(shape.optimize()));
                result.add("events", events);
            } else if (kind.equals("edges")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("edges needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var edges = new JsonArray();
                shape.forAllEdges((x1, y1, z1, x2, y2, z2) -> {
                    var edge = new JsonArray();
                    edge.add(Oracle.bits(x1));
                    edge.add(Oracle.bits(y1));
                    edge.add(Oracle.bits(z1));
                    edge.add(Oracle.bits(x2));
                    edge.add(Oracle.bits(y2));
                    edge.add(Oracle.bits(z2));
                    edges.add(edge);
                });
                result.add("edges", edges);
            } else if (kind.equals("join_coords")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                BooleanOp op = switch (request.get("op").getAsString()) {
                    case "or" -> BooleanOp.OR;
                    case "only_first" -> BooleanOp.ONLY_FIRST;
                    case "only_second" -> BooleanOp.ONLY_SECOND;
                    case "and" -> BooleanOp.AND;
                    case "not_same" -> BooleanOp.NOT_SAME;
                    case "first" -> BooleanOp.FIRST;
                    case "second" -> BooleanOp.SECOND;
                    default -> throw new IllegalArgumentException("Unknown VoxelShape boolean op");
                };
                result.add("unoptimized", snapshotWithCoords(Shapes.joinUnoptimized(first, second, op)));
                result.add("optimized", snapshotWithCoords(Shapes.join(first, second, op)));
            } else if (kind.equals("boxes")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("boxes needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var boxes = new JsonArray();
                shape.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
                    var box = new JsonArray();
                    box.add(Oracle.bits(x1));
                    box.add(Oracle.bits(y1));
                    box.add(Oracle.bits(z1));
                    box.add(Oracle.bits(x2));
                    box.add(Oracle.bits(y2));
                    box.add(Oracle.bits(z2));
                    boxes.add(box);
                });
                result.add("boxes", boxes);
            } else if (kind.equals("create_aabb")) {
                AABB aabb = box(request.getAsJsonObject("aabb"));
                result.add("shape", snapshot(Shapes.create(aabb)));
            } else if (kind.equals("create_coords")) {
                JsonObject box = request.getAsJsonObject("box");
                result.add("shape", snapshot(Shapes.create(
                    f64(box, "min_x"), f64(box, "min_y"), f64(box, "min_z"),
                    f64(box, "max_x"), f64(box, "max_y"), f64(box, "max_z"))));
            } else if (kind.equals("infinity")) {
                VoxelShape infinity = Shapes.INFINITY;
                String view = request.get("view").getAsString();
                if (view.equals("snapshot")) {
                    result.add("shape", snapshot(infinity));
                } else if (view.equals("boxes")) {
                    var boxes = new JsonArray();
                    infinity.forAllBoxes((x1, y1, z1, x2, y2, z2) -> {
                        var box = new JsonArray();
                        box.add(Oracle.bits(x1));
                        box.add(Oracle.bits(y1));
                        box.add(Oracle.bits(z1));
                        box.add(Oracle.bits(x2));
                        box.add(Oracle.bits(y2));
                        box.add(Oracle.bits(z2));
                        boxes.add(box);
                    });
                    result.add("boxes", boxes);
                } else if (view.equals("not_same_self")) {
                    result.add("shape", snapshot(Shapes.join(infinity, infinity, BooleanOp.NOT_SAME)));
                } else {
                    throw new IllegalArgumentException("Unknown infinity view");
                }
            } else if (kind.equals("constants")) {
                String name = request.get("name").getAsString();
                if (name.equals("epsilon")) {
                    result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
                } else if (name.equals("big_epsilon")) {
                    result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
                } else if (name.equals("both")) {
                    result.addProperty("epsilon", Oracle.bits(Shapes.EPSILON));
                    result.addProperty("big_epsilon", Oracle.bits(Shapes.BIG_EPSILON));
                } else {
                    throw new IllegalArgumentException("Unknown shape constant");
                }
            } else if (kind.equals("axis_limits")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("axis_limits needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                result.addProperty("min_x", Oracle.bits(shape.min(Direction.Axis.X)));
                result.addProperty("max_x", Oracle.bits(shape.max(Direction.Axis.X)));
                result.addProperty("min_y", Oracle.bits(shape.min(Direction.Axis.Y)));
                result.addProperty("max_y", Oracle.bits(shape.max(Direction.Axis.Y)));
                result.addProperty("min_z", Oracle.bits(shape.min(Direction.Axis.Z)));
                result.addProperty("max_z", Oracle.bits(shape.max(Direction.Axis.Z)));
            } else if (kind.equals("axis_limits_at")) {
                var entries = request.getAsJsonArray("shapes");
                if (entries == null || entries.size() == 0) {
                    throw new IllegalArgumentException("axis_limits_at needs at least one shape");
                }
                VoxelShape first = voxelShape(entries.get(0).getAsJsonObject());
                VoxelShape[] tail = new VoxelShape[entries.size() - 1];
                for (int i = 1; i < entries.size(); i++) {
                    tail[i - 1] = voxelShape(entries.get(i).getAsJsonObject());
                }
                VoxelShape shape = Shapes.or(first, tail);
                var values = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    Direction.Axis axis = Direction.Axis.byName(query.get("axis").getAsString());
                    if (axis == null) {
                        throw new IllegalArgumentException("Unknown shape axis");
                    }
                    double value = query.get("op").getAsString().equals("min")
                        ? shape.min(axis, f64(query, "b"), f64(query, "c"))
                        : shape.max(axis, f64(query, "b"), f64(query, "c"));
                    values.add(Oracle.bits(value));
                }
                result.add("values", values);
            } else if (kind.equals("equal")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                result.addProperty("equal", Shapes.equal(first, second));
            } else if (kind.equals("object_equals")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape sameGeometry = voxelShape(request.getAsJsonObject("same"));
                VoxelShape differentGeometry = voxelShape(request.getAsJsonObject("different"));
                result.addProperty("self", shape.equals(shape));
                result.addProperty("same_geometry", shape.equals(sameGeometry));
                result.addProperty("different_geometry", shape.equals(differentGeometry));
            } else if (kind.equals("object_equals_constants")) {
                VoxelShape empty = Shapes.empty();
                VoxelShape block = Shapes.block();
                VoxelShape infinity = Shapes.INFINITY;
                result.addProperty("empty_self", empty.equals(Shapes.empty()));
                result.addProperty("empty_thin", empty.equals(Shapes.box(0.0, 0.0, 0.0, 0.00000005, 1.0, 1.0)));
                result.addProperty("empty_move", empty.equals(empty.move(1.0, 2.0, 3.0)));
                result.addProperty("block_self", block.equals(Shapes.block()));
                result.addProperty("block_box", block.equals(Shapes.box(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)));
                result.addProperty("block_move", block.equals(block.move(0.0, 0.0, 0.0)));
                result.addProperty("infinity_self", infinity.equals(Shapes.INFINITY));
                result.addProperty("infinity_box", infinity.equals(Shapes.box(
                    Double.NEGATIVE_INFINITY,
                    Double.NEGATIVE_INFINITY,
                    Double.NEGATIVE_INFINITY,
                    Double.POSITIVE_INFINITY,
                    Double.POSITIVE_INFINITY,
                    Double.POSITIVE_INFINITY
                )));
            } else if (kind.equals("to_string")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                result.addProperty("value", shape.toString());
            } else if (kind.equals("rotate")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(shape, rotation)));
            } else if (kind.equals("rotate_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(Shapes.or(first, second), rotation)));
            } else if (kind.equals("rotate_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup rotation = OctahedralGroup.valueOf(request.get("rotation").getAsString());
                result.add("shape", snapshot(Shapes.rotate(
                    shape, rotation, vec3(request.getAsJsonObject("center")))));
            } else if (kind.equals("rotate_horizontal_axis")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateHorizontalAxis(shape);
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal_axis_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateHorizontalAxis(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_axis")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateAllAxis(shape);
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_axis_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var axes = Shapes.rotateAllAxis(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction.Axis axis : Direction.Axis.values()) {
                    if (axes.containsKey(axis)) {
                        var entry = new JsonObject();
                        entry.addProperty("axis", axis.getName());
                        entry.add("shape", snapshot(axes.get(axis)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateHorizontal(shape);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_horizontal_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var directions = Shapes.rotateHorizontal(shape, initial);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateAll(shape);
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_center")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var directions = Shapes.rotateAll(shape, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_all_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var directions = Shapes.rotateAll(shape, initial, vec3(request.getAsJsonObject("center")));
                var snapshots = new JsonArray();
                for (Direction direction : Direction.values()) {
                    if (directions.containsKey(direction)) {
                        var entry = new JsonObject();
                        entry.addProperty("direction", direction.getName());
                        entry.add("shape", snapshot(directions.get(direction)));
                        snapshots.add(entry);
                    }
                }
                result.add("shapes", snapshots);
            } else if (kind.equals("rotate_attach_face")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                var faces = Shapes.rotateAttachFace(shape);
                var faceSnapshots = new JsonArray();
                for (AttachFace face : AttachFace.values()) {
                    var directions = faces.get(face);
                    var directionSnapshots = new JsonArray();
                    for (Direction direction : Direction.values()) {
                        if (directions.containsKey(direction)) {
                            var directionEntry = new JsonObject();
                            directionEntry.addProperty("direction", direction.getName());
                            directionEntry.add("shape", snapshot(directions.get(direction)));
                            directionSnapshots.add(directionEntry);
                        }
                    }
                    var faceEntry = new JsonObject();
                    faceEntry.addProperty("face", face.getSerializedName());
                    faceEntry.add("shapes", directionSnapshots);
                    faceSnapshots.add(faceEntry);
                }
                result.add("faces", faceSnapshots);
            } else if (kind.equals("rotate_attach_face_initial")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("box"));
                OctahedralGroup initial = OctahedralGroup.valueOf(request.get("initial").getAsString());
                var faces = Shapes.rotateAttachFace(shape, initial);
                var faceSnapshots = new JsonArray();
                for (AttachFace face : AttachFace.values()) {
                    var directions = faces.get(face);
                    var directionSnapshots = new JsonArray();
                    for (Direction direction : Direction.values()) {
                        if (directions.containsKey(direction)) {
                            var directionEntry = new JsonObject();
                            directionEntry.addProperty("direction", direction.getName());
                            directionEntry.add("shape", snapshot(directions.get(direction)));
                            directionSnapshots.add(directionEntry);
                        }
                    }
                    var faceEntry = new JsonObject();
                    faceEntry.addProperty("face", face.getSerializedName());
                    faceEntry.add("shapes", directionSnapshots);
                    faceSnapshots.add(faceEntry);
                }
                result.add("faces", faceSnapshots);
            } else if (kind.equals("collide")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                result.addProperty("distance", Oracle.bits(shape.collide(axis, moving, f64(request, "distance"))));
            } else if (kind.equals("collide_many")) {
                AABB moving = box(request.getAsJsonObject("moving"));
                Direction.Axis axis = Direction.Axis.byName(request.get("axis").getAsString());
                if (axis == null) {
                    throw new IllegalArgumentException("Unknown collision axis");
                }
                var shapes = new java.util.ArrayList<VoxelShape>();
                for (var entry : request.getAsJsonArray("shapes")) {
                    shapes.add(voxelShape(entry.getAsJsonObject()));
                }
                result.addProperty("distance", Oracle.bits(Shapes.collide(
                    axis, moving, shapes, f64(request, "distance"))));
            } else if (kind.equals("clip")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else if (kind.equals("clip_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
                var events = new JsonArray();
                for (var entry : request.getAsJsonArray("queries")) {
                    JsonObject query = entry.getAsJsonObject();
                    events.add(snapshot(shape.clip(
                        vec3(query.getAsJsonObject("from")),
                        vec3(query.getAsJsonObject("to")),
                        blockPos(query.getAsJsonObject("pos")))));
                }
                result.add("events", events);
            } else if (kind.equals("closest")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
                result.add("point", closest.isPresent()
                    ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
            } else if (kind.equals("closest_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape shape = Shapes.join(first, second, BooleanOp.OR);
                var closest = shape.closestPointTo(vec3(request.getAsJsonObject("point")));
                result.add("point", closest.isPresent()
                    ? snapshot(closest.get()) : com.google.gson.JsonNull.INSTANCE);
            } else if (kind.equals("face")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown face direction");
                }
                result.add("shape", snapshot(shape.getFaceShape(direction)));
            } else if (kind.equals("face_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown face direction");
                }
                result.add("shape", snapshot(Shapes.or(first, second).getFaceShape(direction)));
            } else if (kind.equals("block_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown occlusion direction");
                }
                result.addProperty("occludes", Shapes.blockOccludes(shape, occluder, direction));
            } else if (kind.equals("block_occludes_join")) {
                VoxelShape first = voxelShape(request.getAsJsonObject("first"));
                VoxelShape second = voxelShape(request.getAsJsonObject("second"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown occlusion direction");
                }
                result.addProperty("occludes", Shapes.blockOccludes(Shapes.or(first, second), occluder, direction));
            } else if (kind.equals("merged_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                Direction direction = Direction.byName(request.get("direction").getAsString());
                if (direction == null) {
                    throw new IllegalArgumentException("Unknown merged occlusion direction");
                }
                result.addProperty("occludes", Shapes.mergedFaceOccludes(shape, occluder, direction));
            } else if (kind.equals("face_occludes")) {
                VoxelShape shape = voxelShape(request.getAsJsonObject("shape"));
                VoxelShape occluder = voxelShape(request.getAsJsonObject("occluder"));
                result.addProperty("occludes", Shapes.faceShapeOccludes(shape, occluder));
            } else {
                throw new IllegalArgumentException("Unknown VoxelShape kind");
            }
        } else if (operation.equals("random")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            var events = new JsonArray();
            events.add(new JsonObject());
            for (var entry : request.getAsJsonArray("steps")) {
                events.add(randomState(random, entry.getAsJsonObject()));
            }
            result.add("events", events);
        } else if (operation.equals("random_fork")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            result.add("prefix_events", randomSteps(random, request.getAsJsonArray("prefix_steps")));
            RandomSource child = random.fork();
            result.add("child_events", randomSteps(child, request.getAsJsonArray("child_steps")));
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        } else if (operation.equals("random_positional")) {
            RandomSource random = RandomSource.create(Long.parseLong(request.get("seed").getAsString()));
            PositionalRandomFactory factory = random.forkPositional();
            var factoryEvents = new JsonArray();
            for (var entry : request.getAsJsonArray("factory_steps")) {
                JsonObject step = entry.getAsJsonObject();
                RandomSource child = switch (step.get("op").getAsString()) {
                    case "at" -> factory.at(step.get("x").getAsInt(), step.get("y").getAsInt(),
                        step.get("z").getAsInt());
                    case "fromHashOf" -> factory.fromHashOf(step.get("name").getAsString());
                    case "fromSeed" -> factory.fromSeed(Long.parseLong(step.get("seed").getAsString()));
                    default -> throw new IllegalArgumentException("Unknown positional random op");
                };
                var state = new JsonObject();
                state.addProperty("op", step.get("op").getAsString());
                state.add("events", randomSteps(child, step.getAsJsonArray("steps")));
                factoryEvents.add(state);
            }
            result.add("factory_events", factoryEvents);
            result.add("parent_events", randomSteps(random, request.getAsJsonArray("parent_steps")));
        } else if (operation.equals("aabb_builder")) {
            AABB.Builder builder = new AABB.Builder();
            var events = new JsonArray();
            var initial = new JsonObject();
            initial.addProperty("defined", builder.isDefined());
            events.add(initial);
            for (var entry : request.getAsJsonArray("points")) {
                JsonObject point = entry.getAsJsonObject();
                builder.include(new Vector3f(
                    Oracle.f32(point, "x"),
                    Oracle.f32(point, "y"),
                    Oracle.f32(point, "z")
                ));
                var state = new JsonObject();
                state.addProperty("defined", builder.isDefined());
                events.add(state);
            }
            result.add("events", events);
            var build = new JsonObject();
            try {
                build.addProperty("outcome", "value");
                build.add("box", snapshot(builder.build()));
            } catch (IllegalStateException error) {
                build.addProperty("outcome", "error");
                build.addProperty("error", "undefined");
                result.addProperty("java_exception", error.getClass().getName());
            }
            result.add("build", build);
        } else if (operation.equals("aabb")) {
            AABB box = box(request);
            var events = new JsonArray();
            events.add(snapshot(box));
            for (var entry : request.getAsJsonArray("steps")) {
                JsonObject step = entry.getAsJsonObject();
                switch (step.get("op").getAsString()) {
                    case "contains" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(
                            f64(step, "x"), f64(step, "y"), f64(step, "z")));
                        events.add(state);
                    }
                    case "intersects" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(box(step.getAsJsonObject("box"))));
                        events.add(state);
                    }
                    case "contains_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.contains(vec3(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "intersects_coords" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z")));
                        events.add(state);
                    }
                    case "intersects_vec3" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            vec3(step.getAsJsonObject("min")),
                            vec3(step.getAsJsonObject("max"))));
                        events.add(state);
                    }
                    case "intersects_block_pos" -> {
                        var state = new JsonObject();
                        state.addProperty("hit", box.intersects(
                            blockPos(step.getAsJsonObject("position"))));
                        events.add(state);
                    }
                    case "collided_along_vector" -> {
                        var boxes = new java.util.ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var state = new JsonObject();
                        state.addProperty("hit", box.collidedAlongVector(
                            vec3(step.getAsJsonObject("vector")), boxes));
                        events.add(state);
                    }
                    case "from_block_pos" -> {
                        box = new AABB(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "from_vec3" -> {
                        box = new AABB(
                            vec3(step.getAsJsonObject("begin")),
                            vec3(step.getAsJsonObject("end")));
                        events.add(snapshot(box));
                    }
                    case "unit_cube_from_lower_corner" -> {
                        box = AABB.unitCubeFromLowerCorner(
                            vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "encapsulating_full_blocks" -> {
                        box = AABB.encapsulatingFullBlocks(
                            blockPos(step.getAsJsonObject("first")),
                            blockPos(step.getAsJsonObject("second")));
                        events.add(snapshot(box));
                    }
                    case "of_bounding_box" -> {
                        JsonObject boundingBox = step.getAsJsonObject("box");
                        box = AABB.of(new BoundingBox(
                            Integer.parseInt(boundingBox.get("min_x").getAsString()),
                            Integer.parseInt(boundingBox.get("min_y").getAsString()),
                            Integer.parseInt(boundingBox.get("min_z").getAsString()),
                            Integer.parseInt(boundingBox.get("max_x").getAsString()),
                            Integer.parseInt(boundingBox.get("max_y").getAsString()),
                            Integer.parseInt(boundingBox.get("max_z").getAsString())));
                        events.add(snapshot(box));
                    }
                    case "expand_vec3" -> {
                        box = box.expandTowards(vec3(step.getAsJsonObject("delta")));
                        events.add(snapshot(box));
                    }
                    case "equals" -> {
                        var state = new JsonObject();
                        state.addProperty("equal", box.equals(box(step.getAsJsonObject("box"))));
                        events.add(state);
                    }
                    case "hash_code" -> {
                        var state = new JsonObject();
                        state.addProperty("hash_code", box.hashCode());
                        events.add(state);
                    }
                    case "to_string" -> {
                        var state = new JsonObject();
                        state.addProperty("value", box.toString());
                        events.add(state);
                    }
                    case "intersect" -> {
                        box = box.intersect(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "minmax" -> {
                        box = box.minmax(box(step.getAsJsonObject("box")));
                        events.add(snapshot(box));
                    }
                    case "move" -> {
                        box = box.move(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "move_block_pos" -> {
                        box = box.move(blockPos(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vec3" -> {
                        box = box.move(vec3(step.getAsJsonObject("position")));
                        events.add(snapshot(box));
                    }
                    case "move_vector3f" -> {
                        JsonObject position = step.getAsJsonObject("position");
                        box = box.move(new Vector3f(
                            Oracle.f32(position, "x"),
                            Oracle.f32(position, "y"),
                            Oracle.f32(position, "z")));
                        events.add(snapshot(box));
                    }
                    case "sizes" -> {
                        var state = new JsonObject();
                        state.addProperty("x_size", Oracle.bits(box.getXsize()));
                        state.addProperty("y_size", Oracle.bits(box.getYsize()));
                        state.addProperty("z_size", Oracle.bits(box.getZsize()));
                        state.addProperty("size", Oracle.bits(box.getSize()));
                        events.add(state);
                    }
                    case "has_nan" -> {
                        var state = new JsonObject();
                        state.addProperty("has_nan", box.hasNaN());
                        events.add(state);
                    }
                    case "positions" -> {
                        var state = new JsonObject();
                        state.add("center", snapshot(box.getCenter()));
                        state.add("bottom_center", snapshot(box.getBottomCenter()));
                        state.add("min_position", snapshot(box.getMinPosition()));
                        state.add("max_position", snapshot(box.getMaxPosition()));
                        events.add(state);
                    }
                    case "min", "max" -> {
                        Direction.Axis axis = Direction.Axis.byName(step.get("axis").getAsString());
                        if (axis == null) {
                            throw new IllegalArgumentException("Unknown axis name");
                        }
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(step.get("op").getAsString().equals("min")
                            ? box.min(axis) : box.max(axis)));
                        events.add(state);
                    }
                    case "distance_point" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            vec3(step.getAsJsonObject("point")))));
                        events.add(state);
                    }
                    case "distance_box" -> {
                        var state = new JsonObject();
                        state.addProperty("value", Oracle.bits(box.distanceToSqr(
                            box(step.getAsJsonObject("box")))));
                        events.add(state);
                    }
                    case "of_size" -> {
                        box = AABB.ofSize(
                            vec3(step.getAsJsonObject("center")),
                            f64(step, "size_x"),
                            f64(step, "size_y"),
                            f64(step, "size_z"));
                        events.add(snapshot(box));
                    }
                    case "set_min_x" -> {
                        box = box.setMinX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_y" -> {
                        box = box.setMinY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_min_z" -> {
                        box = box.setMinZ(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_x" -> {
                        box = box.setMaxX(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_y" -> {
                        box = box.setMaxY(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "set_max_z" -> {
                        box = box.setMaxZ(f64(step, "value"));
                        events.add(snapshot(box));
                    }
                    case "clip" -> {
                        var hit = box.clip(
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")));
                        var state = new JsonObject();
                        if (hit.isPresent()) {
                            state.addProperty("hit", true);
                            state.add("point", snapshotClip(hit.get()));
                        } else {
                            state.addProperty("hit", false);
                        }
                        events.add(state);
                    }
                    case "clip_coords" -> {
                        var hit = AABB.clip(
                            f64(step, "min_x"), f64(step, "min_y"), f64(step, "min_z"),
                            f64(step, "max_x"), f64(step, "max_y"), f64(step, "max_z"),
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")));
                        var state = new JsonObject();
                        if (hit.isPresent()) {
                            state.addProperty("hit", true);
                            state.add("point", snapshotClip(hit.get()));
                        } else {
                            state.addProperty("hit", false);
                        }
                        events.add(state);
                    }
                    case "clip_many" -> {
                        var boxes = new ArrayList<AABB>();
                        for (var boxEntry : step.getAsJsonArray("boxes")) {
                            boxes.add(box(boxEntry.getAsJsonObject()));
                        }
                        var hit = AABB.clip(
                            boxes,
                            vec3(step.getAsJsonObject("from")),
                            vec3(step.getAsJsonObject("to")),
                            blockPos(step.getAsJsonObject("pos")));
                        events.add(snapshot(hit));
                    }
                    case "expand" -> {
                        box = box.expandTowards(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "contract" -> {
                        box = box.contract(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "inflate" -> {
                        box = box.inflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    case "inflate_xyz" -> {
                        box = box.inflate(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    case "deflate" -> {
                        box = box.deflate(f64(step, "amount"));
                        events.add(snapshot(box));
                    }
                    case "deflate_xyz" -> {
                        box = box.deflate(
                            f64(step, "x"), f64(step, "y"), f64(step, "z"));
                        events.add(snapshot(box));
                    }
                    default -> throw new IllegalArgumentException("Unknown AABB op");
                }
            }
            result.add("events", events);
        } else {
            String width = request.get("type").getAsString();
            if (width.equals("f32")) {
                float value = Oracle.f32(request, "value");
                switch (operation) {
                    case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                    case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                    case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                        Oracle.f32(request, "min"), Oracle.f32(request, "max"))));
                    default -> throw new IllegalArgumentException("Unknown math operation");
                }
            } else if (width.equals("f64")) {
                double value = f64(request, "value");
                switch (operation) {
                    case "floor" -> result.addProperty("value", Integer.toString(Mth.floor(value)));
                    case "wrap_degrees" -> result.addProperty("bits", Oracle.bits(Mth.wrapDegrees(value)));
                    case "clamp" -> result.addProperty("bits", Oracle.bits(Mth.clamp(value,
                        f64(request, "min"), f64(request, "max"))));
                    default -> throw new IllegalArgumentException("Unknown math operation");
                }
            } else {
                throw new IllegalArgumentException("Unknown floating-point type");
            }
        }
        return result;
    }

    private static double f64(JsonObject object, String name) {
        return Double.longBitsToDouble(Long.parseUnsignedLong(object.get(name).getAsString(), 16));
    }

    private static AABB box(JsonObject object) {
        return new AABB(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    private static VoxelShape voxelShape(JsonObject object) {
        return Shapes.box(f64(object, "min_x"), f64(object, "min_y"), f64(object, "min_z"),
            f64(object, "max_x"), f64(object, "max_y"), f64(object, "max_z"));
    }

    private static BlockPos blockPos(JsonObject object) {
        return new BlockPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("y").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    private static Vec3 vec3(JsonObject object) {
        return new Vec3(f64(object, "x"), f64(object, "y"), f64(object, "z"));
    }

    private static ChunkPos chunkPos(JsonObject object) {
        return new ChunkPos(
            Integer.parseInt(object.get("x").getAsString()),
            Integer.parseInt(object.get("z").getAsString())
        );
    }

    private static JsonObject chunkPosBlockCoordinates(ChunkPos pos, int offset) {
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

    private static JsonObject chunkPosRegionCoordinates(ChunkPos pos, int regionX, int regionZ) {
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

    private static JsonObject chunkPosBlockPositions(ChunkPos pos, int blockX, int y, int blockZ, JsonArray queries) {
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

    private static JsonObject chunkPosRepresentation(ChunkPos pos, int hashX, int hashZ) {
        var state = new JsonObject();
        state.addProperty("to_string", pos.toString());
        state.addProperty("hash_code", Integer.toString(pos.hashCode()));
        state.addProperty("static_hash", Integer.toString(ChunkPos.hash(hashX, hashZ)));
        return state;
    }

    private static JsonObject chunkPosFromBlockPos(BlockPos pos) {
        var state = new JsonObject();
        state.add("containing", snapshot(ChunkPos.containing(pos)));
        state.addProperty("packed", Long.toUnsignedString(ChunkPos.pack(pos), 16));
        return state;
    }

    private static JsonObject chunkPosDistances(ChunkPos pos, int otherX, int otherZ) {
        ChunkPos other = new ChunkPos(otherX, otherZ);
        var state = new JsonObject();
        state.addProperty("chessboard_chunk", Integer.toString(pos.getChessboardDistance(other)));
        state.addProperty("chessboard_coordinates", Integer.toString(pos.getChessboardDistance(otherX, otherZ)));
        state.addProperty("distance_squared_chunk", Integer.toString(pos.distanceSquared(other)));
        state.addProperty("distance_squared_packed", Integer.toString(pos.distanceSquared(other.pack())));
        return state;
    }

    private static JsonObject chunkPosRangeClosed(JsonObject request) {
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

    private static JsonObject sectionPosRanges(JsonObject request) {
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

    private static JsonObject sectionPosBlocksInside(JsonObject request) {
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

    private static JsonObject sectionPosAroundAndAtBlockPos(JsonObject request) {
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

    private static JsonObject chunkPosConstant(JsonObject request) {
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

    private static JsonObject chunkPosValidity(ChunkPos pos) {
        var output = System.out;
        SharedConstants.setVersion(DetectedVersion.BUILT_IN);
        Bootstrap.bootStrap();
        System.setOut(output);
        var state = new JsonObject();
        state.addProperty("instance", pos.isValid());
        state.addProperty("static", ChunkPos.isValid(pos.x(), pos.z()));
        return state;
    }

    private static JsonObject codecDecodeResult(DataResult<Vec3> decoded) {
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

    private static JsonObject codecEncodeResult(DataResult<JsonElement> encoded) {
        var result = new JsonObject();
        encoded.result().ifPresent(value -> {
            result.addProperty("outcome", "value");
            var values = new JsonArray();
            for (var entry : value.getAsJsonArray()) {
                values.add(Oracle.bits(entry.getAsDouble()));
            }
            result.add("value", values);
        });
        encoded.error().ifPresent(error -> {
            result.addProperty("outcome", "error");
            result.addProperty("error", error.message());
        });
        return result;
    }

    private static JsonObject chunkPosCodecDecodeResult(DataResult<ChunkPos> decoded) {
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

    private static JsonObject chunkPosCodecEncodeResult(DataResult<JsonElement> encoded) {
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

    private static JsonArray randomSteps(RandomSource random, JsonArray entries) {
        var events = new JsonArray();
        for (var entry : entries) {
            events.add(randomState(random, entry.getAsJsonObject()));
        }
        return events;
    }

    private static JsonObject randomState(RandomSource random, JsonObject step) {
        var state = new JsonObject();
        switch (step.get("op").getAsString()) {
            case "nextInt" -> state.addProperty("value", Integer.toString(random.nextInt()));
            case "nextIntBound" -> state.addProperty("value",
                Integer.toString(random.nextInt(step.get("bound").getAsInt())));
            case "nextLong" -> state.addProperty("value", Long.toString(random.nextLong()));
            case "nextBoolean" -> state.addProperty("value", Boolean.toString(random.nextBoolean()));
            case "nextFloat" -> state.addProperty("bits", Oracle.bits(random.nextFloat()));
            case "nextDouble" -> state.addProperty("bits", Oracle.bits(random.nextDouble()));
            case "nextGaussian" -> state.addProperty("bits", Oracle.bits(random.nextGaussian()));
            case "setSeed" -> {
                random.setSeed(Long.parseLong(step.get("seed").getAsString()));
                state.addProperty("op", "setSeed");
            }
            case "nextIntBetweenInclusive" -> state.addProperty("value", Integer.toString(
                random.nextIntBetweenInclusive(step.get("min").getAsInt(), step.get("max").getAsInt())));
            case "nextIntRange" -> state.addProperty("value", Integer.toString(
                random.nextInt(step.get("origin").getAsInt(), step.get("bound").getAsInt())));
            case "triangleDouble" -> state.addProperty("bits", Oracle.bits(
                random.triangle(f64(step, "mean"), f64(step, "spread"))));
            case "triangleFloat" -> state.addProperty("bits", Oracle.bits(
                random.triangle(Oracle.f32(step, "mean"), Oracle.f32(step, "spread"))));
            case "consumeCount" -> {
                random.consumeCount(step.get("rounds").getAsInt());
                state.addProperty("op", "consumeCount");
            }
            default -> throw new IllegalArgumentException("Unknown random op");
        }
        return state;
    }

    private static JsonObject snapshot(AABB box) {
        var state = new JsonObject();
        state.addProperty("min_x", Oracle.bits(box.minX));
        state.addProperty("min_y", Oracle.bits(box.minY));
        state.addProperty("min_z", Oracle.bits(box.minZ));
        state.addProperty("max_x", Oracle.bits(box.maxX));
        state.addProperty("max_y", Oracle.bits(box.maxY));
        state.addProperty("max_z", Oracle.bits(box.maxZ));
        return state;
    }

    private static JsonObject snapshot(VoxelShape shape) {
        var state = new JsonObject();
        state.addProperty("empty", shape.isEmpty());
        var boxes = new JsonArray();
        for (AABB box : shape.toAabbs()) {
            boxes.add(snapshot(box));
        }
        state.add("boxes", boxes);
        if (!shape.isEmpty()) {
            state.add("bounds", snapshot(shape.bounds()));
        }
        return state;
    }

    private static JsonObject snapshotWithCoords(VoxelShape shape) {
        var state = snapshot(shape);
        var coordinates = new JsonObject();
        for (Direction.Axis axis : Direction.Axis.values()) {
            var values = new JsonArray();
            var coords = shape.getCoords(axis);
            for (int i = 0; i < coords.size(); i++) {
                values.add(Oracle.bits(coords.getDouble(i)));
            }
            coordinates.add(axis.getName(), values);
        }
        state.add("coords", coordinates);
        return state;
    }

    private static JsonObject snapshot(BlockPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.getX()));
        state.addProperty("y", Integer.toString(pos.getY()));
        state.addProperty("z", Integer.toString(pos.getZ()));
        return state;
    }

    private static JsonObject snapshot(ChunkPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("z", Integer.toString(pos.z()));
        return state;
    }

    private static JsonObject snapshot(SectionPos pos) {
        var state = new JsonObject();
        state.addProperty("x", Integer.toString(pos.x()));
        state.addProperty("y", Integer.toString(pos.y()));
        state.addProperty("z", Integer.toString(pos.z()));
        state.addProperty("packed", Long.toUnsignedString(pos.asLong(), 16));
        return state;
    }

    private static JsonObject snapshot(Direction facing) {
        var state = new JsonObject();
        state.addProperty("name", facing.getName());
        state.addProperty("data3d", facing.get3DDataValue());
        state.addProperty("data2d", facing.get2DDataValue());
        state.addProperty("opposite", facing.getOpposite().getName());
        state.addProperty("axis", facing.getAxis().getName());
        state.addProperty("axis_direction", facing.getAxisDirection().getName());
        state.addProperty("step_x", facing.getStepX());
        state.addProperty("step_y", facing.getStepY());
        state.addProperty("step_z", facing.getStepZ());
        return state;
    }

    private static JsonObject snapshot(BlockHitResult hit) {
        var state = new JsonObject();
        if (hit == null) {
            state.addProperty("hit", false);
            return state;
        }
        state.addProperty("hit", true);
        state.addProperty("type", hit.getType().name());
        state.add("location", snapshotClip(hit.getLocation()));
        state.addProperty("direction", hit.getDirection().getName());
        state.add("block_pos", snapshot(hit.getBlockPos()));
        state.addProperty("inside", hit.isInside());
        state.addProperty("world_border_hit", hit.isWorldBorderHit());
        return state;
    }

    private static JsonObject snapshot(Vec3 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSqr()));
        return state;
    }

    private static JsonObject snapshot(Vector3f vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("z", Oracle.bits(vector.z));
        return state;
    }

    private static JsonObject snapshotClip(Vec3 point) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(point.x));
        state.addProperty("y", Oracle.bits(point.y));
        state.addProperty("z", Oracle.bits(point.z));
        return state;
    }

    private static JsonObject snapshot(Vec2 vector) {
        var state = new JsonObject();
        state.addProperty("x", Oracle.bits(vector.x));
        state.addProperty("y", Oracle.bits(vector.y));
        state.addProperty("length", Oracle.bits(vector.length()));
        state.addProperty("length_squared", Oracle.bits(vector.lengthSquared()));
        return state;
    }
}
