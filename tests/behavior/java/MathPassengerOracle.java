import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.world.entity.Entity;

class MathPassengerOracle extends MathEntityTickOracle {
    static JsonObject clientWorldEntityPassengersResult(JsonObject request) {
        var entities = new ArrayList<PassengerRelationFixtureEntity>();
        for (JsonElement entityEntry : request.getAsJsonArray("entities")) {
            JsonObject entity = entityEntry.getAsJsonObject();
            entities.add(PassengerRelationFixtureEntity.create(entity.get("entity_id").getAsInt()));
        }

        for (JsonElement linkEntry : request.getAsJsonArray("initial_links")) {
            JsonObject link = linkEntry.getAsJsonObject();
            PassengerRelationFixtureEntity vehicle = passengerRelationEntity(
                entities, link.get("vehicle_id").getAsInt());
            if (vehicle == null) {
                throw new IllegalArgumentException("Unknown initial passenger vehicle");
            }
            for (JsonElement passengerEntry : link.getAsJsonArray("passenger_ids")) {
                PassengerRelationFixtureEntity passenger = passengerRelationEntity(
                    entities, passengerEntry.getAsInt());
                if (passenger == null || !passenger.startRiding(vehicle, true, false)) {
                    throw new IllegalArgumentException("Invalid initial passenger relation");
                }
            }
        }

        var states = new JsonArray();
        states.add(passengerRelationState("initial", true, entities));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            PassengerRelationFixtureEntity vehicle = passengerRelationEntity(
                entities, operation.get("vehicle_id").getAsInt());
            boolean accepted = vehicle != null;
            if (vehicle != null) {
                vehicle.ejectPassengers();
                for (JsonElement passengerEntry : operation.getAsJsonArray("passenger_ids")) {
                    PassengerRelationFixtureEntity passenger = passengerRelationEntity(
                        entities, passengerEntry.getAsInt());
                    if (passenger != null) {
                        passenger.startRiding(vehicle, true, false);
                    }
                }
            }
            states.add(passengerRelationState(
                operation.get("label").getAsString(), accepted, entities));
        }

        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static PassengerRelationFixtureEntity passengerRelationEntity(
        List<PassengerRelationFixtureEntity> entities, int entityId
    ) {
        for (PassengerRelationFixtureEntity entity : entities) {
            if (entity.getId() == entityId) {
                return entity;
            }
        }
        return null;
    }

    static JsonObject clientWorldEntityPassengerTickResult(JsonObject request) {
        JsonObject vehicleInput = request.getAsJsonObject("vehicle");
        JsonObject passengerInput = request.getAsJsonObject("passenger");
        JsonObject rootInput = request.getAsJsonObject("root_entity");
        int vehicleId = vehicleInput.get("entity_id").getAsInt();
        int passengerId = passengerInput.get("entity_id").getAsInt();

        PassengerRelationFixtureEntity vehicle = PassengerRelationFixtureEntity.create(
            vehicleId, vec3(vehicleInput.getAsJsonObject("position")));
        TickSnowballFixture passenger = TickSnowballFixture.create(
            passengerId,
            vec3(passengerInput.getAsJsonObject("position")),
            vec3(passengerInput.getAsJsonObject("movement")),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
            passengerInput.get("no_gravity").getAsBoolean());
        TickSnowballFixture root = TickSnowballFixture.create(
            rootInput.get("entity_id").getAsInt(),
            vec3(rootInput.getAsJsonObject("position")),
            vec3(rootInput.getAsJsonObject("movement")),
            Oracle.f32(rootInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(rootInput.getAsJsonObject("rotation"), "x"),
            rootInput.get("no_gravity").getAsBoolean());
        if (!passenger.startRiding(vehicle, true, false)) {
            throw new IllegalArgumentException("Passenger tick fixture relation was rejected");
        }

        var states = new JsonArray();
        states.add(passengerTickState("initial", vehicle, passenger, root));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger tick action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                tickPassengerTree(vehicle, passenger, root);
            }
            states.add(passengerTickState(
                operation.get("label").getAsString(), vehicle, passenger, root));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static void tickPassengerTree(
        PassengerRelationFixtureEntity vehicle,
        TickSnowballFixture passenger,
        TickSnowballFixture root
    ) {
        vehicle.setOldPosAndRot();
        vehicle.tickCount++;
        vehicle.tick();
        if (!passenger.isRemoved() && passenger.getVehicle() == vehicle) {
            passenger.rideTickThroughClientLevel();
        }
        root.tickThroughClientLevel();
    }

    static JsonObject passengerTickState(
        String label,
        PassengerRelationFixtureEntity vehicle,
        TickSnowballFixture passenger,
        TickSnowballFixture root
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        entities.add(tickEntityState(vehicle));
        entities.add(tickSnowballEntityState(passenger));
        entities.add(tickSnowballEntityState(root));
        state.add("entities", entities);
        return state;
    }

    static JsonObject passengerRelationState(
        String label, boolean accepted, List<PassengerRelationFixtureEntity> entities
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", accepted);
        var snapshots = new JsonArray();
        for (PassengerRelationFixtureEntity entity : entities) {
            var snapshot = new JsonObject();
            snapshot.addProperty("entity_id", entity.getId());
            if (entity.getVehicle() == null) {
                snapshot.add("vehicle_id", JsonNull.INSTANCE);
            } else {
                snapshot.addProperty("vehicle_id", entity.getVehicle().getId());
            }
            var passengerIds = new JsonArray();
            for (Entity passenger : entity.getPassengers()) {
                passengerIds.add(passenger.getId());
            }
            snapshot.add("passenger_ids", passengerIds);
            snapshots.add(snapshot);
        }
        state.add("entities", snapshots);
        return state;
    }
}
