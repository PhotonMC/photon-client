/** Every read and write of a private field of the game goes through here, so a version with other names changes one place. */
final class Reflect {
    private Reflect() {}

    static Object get(Object target, String name) throws Exception {
        var field = find(target.getClass(), name);
        return field.get(target);
    }

    static void set(Object target, String name, Object value) throws Exception {
        var field = find(target.getClass(), name);
        field.set(target, value);
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
