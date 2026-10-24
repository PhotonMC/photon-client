import com.google.gson.JsonObject;

/** The entity a living-entity row ticks, with the data values its events build. */
class MovementLivingRow extends MovementEntityOracle {
    /** The entity under test and what a row's events act on. */
    record LivingRow(
        JsonObject row,
        Scene scene,
        net.minecraft.client.multiplayer.ClientLevel level,
        net.minecraft.client.multiplayer.ClientPacketListener listener,
        net.minecraft.world.entity.Entity entity,
        net.minecraft.world.entity.LivingEntity living,
        int entityId) {}

    /** A bee's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> beeData(
        final String field, final Object value) throws Exception {
        var accessor = Reflect.field(net.minecraft.world.entity.animal.bee.Bee.class, field);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }

    /** A wolf's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> wolfData(
        final Class<?> owner, final String field, final Object value) throws Exception {
        var accessor = Reflect.field(owner, field);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }

    /** A creeper's data value, built from the reference's own accessor. */
    @SuppressWarnings({"rawtypes", "unchecked"})
    static net.minecraft.network.syncher.SynchedEntityData.DataValue<?> creeperData(
        final String field, final Object value) throws Exception {
        var accessor = Reflect.field(net.minecraft.world.entity.monster.Creeper.class, field);
        return net.minecraft.network.syncher.SynchedEntityData.DataValue.create(
            (net.minecraft.network.syncher.EntityDataAccessor) accessor.get(null), value);
    }
}
