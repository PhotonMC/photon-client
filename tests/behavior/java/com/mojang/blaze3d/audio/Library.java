package com.mojang.blaze3d.audio;

import java.util.Set;
import org.jspecify.annotations.Nullable;

/**
 * Test glue only: the channel pools of the reference's sound library with no sound device behind them
 * (see `Channel`). The pools count channels as the reference's own do: one more than the limit is refused.
 */
public class Library {
    private Library.ChannelPool staticChannels = new Library.CountingChannelPool(0);
    private Library.ChannelPool streamingChannels = new Library.CountingChannelPool(0);

    public Library() {
    }

    /** A pool of the given size, for static or streaming sounds. */
    public void setPools(final int staticCount, final int streamingCount) {
        this.staticChannels = new Library.CountingChannelPool(staticCount);
        this.streamingChannels = new Library.CountingChannelPool(streamingCount);
    }

    public @Nullable Channel acquireChannel(final Library.Pool pool) {
        return (pool == Library.Pool.STREAMING ? this.streamingChannels : this.staticChannels).acquire();
    }

    public void releaseChannel(final Channel channel) {
        if (!this.staticChannels.release(channel) && !this.streamingChannels.release(channel)) {
            throw new IllegalStateException("Tried to release unknown channel");
        }
    }

    public enum Pool {
        STATIC,
        STREAMING;
    }

    interface ChannelPool {
        @Nullable Channel acquire();

        boolean release(Channel channel);
    }

    private static class CountingChannelPool implements Library.ChannelPool {
        private final int limit;
        private final Set<Channel> activeChannels = java.util.Collections.newSetFromMap(new java.util.IdentityHashMap<>());

        CountingChannelPool(final int limit) {
            this.limit = limit;
        }

        @Override
        public @Nullable Channel acquire() {
            if (this.activeChannels.size() >= this.limit) {
                return null;
            }
            Channel channel = Channel.create();
            this.activeChannels.add(channel);
            return channel;
        }

        @Override
        public boolean release(final Channel channel) {
            if (!this.activeChannels.remove(channel)) {
                return false;
            }
            channel.destroy();
            return true;
        }
    }
}
