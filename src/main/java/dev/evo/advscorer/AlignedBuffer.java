package dev.evo.advscorer;

import java.lang.reflect.Field;
import java.nio.Buffer;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import sun.misc.Unsafe;

public class AlignedBuffer {
    private static Unsafe UNSAFE = getUnsafe();

    private static final long BUFFER_ADDRESS_FIELD_OFFSET = getAddressFieldOffset();

    private static Unsafe getUnsafe() {
        try {
            Field theUnsafeField = Unsafe.class.getDeclaredField("theUnsafe");
            theUnsafeField.setAccessible(true);
            return (Unsafe) theUnsafeField.get(null);
        } catch (NoSuchFieldException | IllegalAccessException ex) {
            throw new RuntimeException(ex);
        }
    }

    private static long getAddressFieldOffset() {
        try {
            return UNSAFE.objectFieldOffset(Buffer.class.getDeclaredField("address"));
        } catch (NoSuchFieldException ex) {
            throw new RuntimeException(ex);
        }
    }

    private static long getDirectArrayAddress(ByteBuffer buffer) {
        return UNSAFE.getLong(buffer, BUFFER_ADDRESS_FIELD_OFFSET);
    }

    public static ByteBuffer create(int size, int align) {
        assert align > 0;
        assert align % 2 == 0;

        ByteBuffer buf = ByteBuffer.allocateDirect(size + align);
        // System.out.println(buf);
        long bufAddr = getDirectArrayAddress(buf);
        // System.out.println(String.format("%x", bufAddr));
        int overAlignedOffset = (int) (bufAddr % align);
        int offset = 0;
        if (overAlignedOffset > 0) {
            offset = align - overAlignedOffset;
        }
        // System.out.println(String.format("Offset: %s", offset));
        buf.position(offset);
        return buf.slice().order(ByteOrder.nativeOrder());
    }
}
