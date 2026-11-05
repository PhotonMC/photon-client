package com.mojang.blaze3d.audio;

import net.minecraft.world.phys.Vec3;

/**
 * Test glue only: stands in for a class of the reference's sound device layer (see `Channel`). It
 * holds where the audio listener is and which way it looks, as the code that reads it asks.
 */
public class ListenerTransform {
    private final Vec3 position;
    private final Vec3 forward;
    private final Vec3 up;

    public ListenerTransform() {
        this(Vec3.ZERO, new Vec3(0.0, 0.0, -1.0), new Vec3(0.0, 1.0, 0.0));
    }

    public ListenerTransform(final Vec3 position, final Vec3 forward, final Vec3 up) {
        this.position = position;
        this.forward = forward;
        this.up = up;
    }

    public Vec3 position() {
        return this.position;
    }

    public Vec3 forward() {
        return this.forward;
    }

    public Vec3 up() {
        return this.up;
    }

    /** The side the listener's right ear is on: forward crossed with up. */
    public Vec3 right() {
        return this.forward.cross(this.up);
    }
}
