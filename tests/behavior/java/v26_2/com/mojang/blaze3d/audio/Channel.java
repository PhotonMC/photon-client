package com.mojang.blaze3d.audio;

import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.sounds.AudioStream;
import net.minecraft.world.phys.Vec3;

/**
 * Test glue only: stands in for the reference's OpenAL source so the sound engine can run with no
 * sound device. It keeps no sound of the original; it writes down each call the engine makes of it.
 */
public class Channel {
    /** Every call, in order, across all the channels of a scenario. */
    public static final List<String> LOG = new ArrayList<>();
    private static int created;
    private final int id;
    private boolean stopped;
    private boolean paused;

    static Channel create() {
        return new Channel(created++);
    }

    /** Forget the last scenario's channels and calls. */
    public static void reset() {
        LOG.clear();
        created = 0;
    }

    private Channel(final int id) {
        this.id = id;
    }

    public int id() {
        return this.id;
    }

    /** The sound ending by itself, which the real source does when its buffer runs out. */
    public void finish() {
        this.stopped = true;
    }

    private void log(final String call) {
        LOG.add("channel" + this.id + " " + call);
    }

    public void destroy() {
        this.log("destroy");
    }

    public void play() {
        this.log("play");
    }

    public void pause() {
        if (!this.stopped && !this.paused) {
            this.paused = true;
            this.log("pause");
        }
    }

    public void unpause() {
        if (this.paused) {
            this.paused = false;
            this.log("unpause");
        }
    }

    public void stop() {
        if (!this.stopped) {
            this.stopped = true;
            this.log("stop");
        }
    }

    public boolean playing() {
        return !this.stopped && !this.paused;
    }

    public boolean stopped() {
        return this.stopped;
    }

    public void setSelfPosition(final Vec3 newPosition) {
        this.log("position " + bits(newPosition.x) + " " + bits(newPosition.y) + " " + bits(newPosition.z));
    }

    public void setPitch(final float pitch) {
        this.log("pitch " + Integer.toHexString(Float.floatToIntBits(pitch)));
    }

    public void setLooping(final boolean looping) {
        this.log("looping " + looping);
    }

    public void setVolume(final float volume) {
        this.log("volume " + Integer.toHexString(Float.floatToIntBits(volume)));
    }

    public void disableAttenuation() {
        this.log("attenuation none");
    }

    public void linearAttenuation(final float maxDistance) {
        this.log("attenuation linear " + Integer.toHexString(Float.floatToIntBits(maxDistance)));
    }

    public void setRelative(final boolean relative) {
        this.log("relative " + relative);
    }

    public void attachStaticBuffer(final SoundBuffer buffer) {
        this.log("static buffer");
    }

    public void attachBufferStream(final AudioStream stream) {
        this.log("stream buffer");
    }

    public void updateStream() {
    }

    private static String bits(final double value) {
        return Long.toHexString(Double.doubleToLongBits(value));
    }
}
