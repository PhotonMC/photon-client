/** Every read and write of a private field of the game goes through here, so a version with other names changes one place. */
final class Reflect {
    private Reflect() {}

    /** The unsafe singleton, for objects built without a constructor. */
    static sun.misc.Unsafe unsafe() {
        try {
            var field = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            field.setAccessible(true);
            return (sun.misc.Unsafe) field.get(null);
        } catch (ReflectiveOperationException error) {
            throw new IllegalStateException(error);
        }
    }

    static Object get(Object target, String name) throws Exception {
        var field = find(target.getClass(), name);
        return field.get(target);
    }

    static void set(Object target, String name, Object value) throws Exception {
        var field = find(target.getClass(), name);
        field.set(target, value);
    }

    /** A field declared by `owner` (or above it), read from `target`. */
    static Object get(Object target, Class<?> owner, String name) throws Exception {
        return find(owner, name).get(target);
    }

    static void set(Object target, Class<?> owner, String name, Object value) throws Exception {
        find(owner, name).set(target, value);
    }

    /** A method declared by `owner`, called on `target` (null for a static one). */
    static Object invoke(Object target, Class<?> owner, String name, Class<?>[] types, Object... args)
            throws Exception {
        var method = owner.getDeclaredMethod(name, types);
        method.setAccessible(true);
        return method.invoke(target, args);
    }

    static <T> T construct(Class<T> type, Class<?>[] types, Object... args) throws Exception {
        var constructor = type.getDeclaredConstructor(types);
        constructor.setAccessible(true);
        return constructor.newInstance(args);
    }

    /** A static field, written with the value's own type (an int stays an int). */
    static void setStatic(Class<?> owner, String name, Object value) throws Exception {
        var field = find(owner, name);
        field.set(null, value);
    }

    private static java.lang.reflect.Field find(Class<?> start, String name) throws NoSuchFieldException {
        for (Class<?> type = start; type != null; type = type.getSuperclass()) {
            try {
                var field = type.getDeclaredField(name);
                field.setAccessible(true);
                return field;
            } catch (NoSuchFieldException missing) {
                // look in the class above
            }
        }
        throw new NoSuchFieldException(name);
    }
}
