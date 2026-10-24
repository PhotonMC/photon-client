import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntitySupport {
    /** Keeps each vertex's position, texture coordinates and normal. */
    static final class Capture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final java.util.List<String> vertices = new java.util.ArrayList<>();

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            this.vertices.add(Oracle.bits(x) + Oracle.bits(y) + Oracle.bits(z) + Oracle.bits(u) + Oracle.bits(v)
                + Oracle.bits(nx) + Oracle.bits(ny) + Oracle.bits(nz));
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

    static Object field(final Object owner, final String name) throws Exception {
        return Reflect.get(owner, name);
    }

    static final sun.misc.Unsafe UNSAFE = unsafe();

    static sun.misc.Unsafe unsafe() {
        try {
            var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (sun.misc.Unsafe) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }

    static net.minecraft.core.Rotations rotations(final JsonArray bits) {
        return new net.minecraft.core.Rotations(
            Float.intBitsToFloat((int) Long.parseLong(bits.get(0).getAsString(), 16)),
            Float.intBitsToFloat((int) Long.parseLong(bits.get(1).getAsString(), 16)),
            Float.intBitsToFloat((int) Long.parseLong(bits.get(2).getAsString(), 16)));
    }

    /** A block model part with no quads: enough for a block model state to submit something. */
    static net.minecraft.client.renderer.block.dispatch.BlockStateModelPart emptyPart() {
        return (net.minecraft.client.renderer.block.dispatch.BlockStateModelPart) java.lang.reflect.Proxy.newProxyInstance(
            EntityOracle.class.getClassLoader(),
            new Class<?>[] {net.minecraft.client.renderer.block.dispatch.BlockStateModelPart.class},
            (proxy, method, args) -> {
                var type = method.getReturnType();
                if (type == java.util.List.class) {
                    return java.util.List.of();
                }
                if (type == boolean.class) {
                    return false;
                }
                return type == int.class ? 0 : null;
            });
    }

    /** Makes a block model state non-empty, with a part that draws nothing. */
    static void fillBlock(final net.minecraft.client.renderer.block.BlockModelRenderState block) {
        block.setupModel(new org.joml.Matrix4f(), false).add(emptyPart());
    }

    /** Takes each vertex a beam's geometry builds, one chained call at a time. */
    static final class BeamCapture implements com.mojang.blaze3d.vertex.VertexConsumer {
        final JsonArray out;
        float[] position;
        float u;
        float v;
        int color;
        int light;
        float[] normal;

        BeamCapture(final JsonArray out) {
            this.out = out;
        }

        void flush() {
            if (this.position == null) {
                return;
            }
            this.out.add(Oracle.bits(this.position[0]) + Oracle.bits(this.position[1]) + Oracle.bits(this.position[2])
                + Oracle.bits(this.u) + Oracle.bits(this.v) + String.format("%08x%08x", this.color, this.light)
                + (this.normal == null ? "000000000000000000000000"
                    : Oracle.bits(this.normal[0]) + Oracle.bits(this.normal[1]) + Oracle.bits(this.normal[2])));
            this.position = null;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer addVertex(final float x, final float y, final float z) {
            this.flush();
            this.position = new float[] {x, y, z};
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int r, final int g, final int b, final int a) {
            this.color = (a << 24) | (r << 16) | (g << 8) | b;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setColor(final int color) {
            this.color = color;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv(final float u, final float v) {
            this.u = u;
            this.v = v;
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv1(final int u, final int v) {
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setUv2(final int u, final int v) {
            this.light = (v << 16) | (u & 0xFFFF);
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setNormal(final float x, final float y, final float z) {
            this.normal = new float[] {x, y, z};
            return this;
        }

        @Override
        public com.mojang.blaze3d.vertex.VertexConsumer setLineWidth(final float width) {
            return this;
        }

        @Override
        public void addVertex(
            final float x, final float y, final float z, final int color, final float u, final float v,
            final int overlayCoords, final int lightCoords, final float nx, final float ny, final float nz) {
            throw new UnsupportedOperationException();
        }
    }

    static net.minecraft.world.phys.Vec3 vec3(final JsonArray bits) {
        return new net.minecraft.world.phys.Vec3(
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(0).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(1).getAsString(), 16)),
            Double.longBitsToDouble(Long.parseUnsignedLong(bits.get(2).getAsString(), 16)));
    }

    /** A layer of the table baked, as the game bakes it. */
    static net.minecraft.client.model.geom.ModelPart bakedLayer(final String id) {
        Oracle.vanillaRegistries();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            if (entry.getKey().toString().equals(id)) {
                return entry.getValue().bakeRoot();
            }
        }
        throw new IllegalArgumentException("No layer " + id);
    }

    /** Every quad of a posed model, sorted, as the part renderer writes them. */
    static JsonObject posedQuads(final String label, final net.minecraft.client.model.Model<?> model) {
        var capture = new Capture();
        model.root().render(new com.mojang.blaze3d.vertex.PoseStack(), capture, 0, 0);
        var quads = new java.util.ArrayList<String>();
        for (int i = 0; i < capture.vertices.size(); i += 4) {
            quads.add(String.join("", capture.vertices.subList(i, i + 4)));
        }
        java.util.Collections.sort(quads);
        var out = new JsonObject();
        out.addProperty("label", label);
        var array = new JsonArray();
        quads.forEach(array::add);
        out.add("quads", array);
        return out;
    }

    static void humanoidFields(final net.minecraft.client.renderer.entity.state.HumanoidRenderState state, final JsonObject row) {
        state.yRot = Oracle.f32(row, "head_y_rot");
        state.xRot = Oracle.f32(row, "x_rot");
        state.walkAnimationPos = Oracle.f32(row, "walk_position");
        state.walkAnimationSpeed = Oracle.f32(row, "walk_speed");
        state.ageInTicks = Oracle.f32(row, "age_in_ticks");
        state.attackTime = Oracle.f32(row, "attack_time");
        state.swimAmount = Oracle.f32(row, "swim_amount");
    }
}
