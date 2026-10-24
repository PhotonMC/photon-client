package org.lwjgl.system;

import java.nio.IntBuffer;
import sun.misc.Unsafe;

/** Minimal test glue for the native integer buffer read used by PreeditEvent. */
public final class MemoryUtil {
    private static final Unsafe UNSAFE = unsafeInstance();

    private MemoryUtil() {
    }

    public static IntBuffer memIntBuffer(long address, int size) {
        int[] values = new int[size];
        for (int i = 0; i < size; i++) {
            values[i] = UNSAFE.getInt(address + (long)i * Integer.BYTES);
        }
        return IntBuffer.wrap(values);
    }

    private static Unsafe unsafeInstance() {
        try {
            var field = Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (Unsafe)field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException("Unable to access native memory for test glue", error);
        }
    }
}
