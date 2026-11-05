import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;

class MathPassengerPositionOracle extends MathPassengerOracle {
    static JsonObject clientWorldEntityPassengerPositionResult(JsonObject request) {
        JsonObject vehicleInput = request.getAsJsonObject("vehicle");
        int vehicleId = vehicleInput.get("entity_id").getAsInt();
        int vehicleTypeId = vehicleInput.get("type_id").getAsInt();
        if (vehicleTypeId != 89) {
            throw new IllegalArgumentException("Passenger position fixture requires an oak boat");
        }
        PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
            vehicleId,
            vec3(vehicleInput.getAsJsonObject("position")),
            Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
            vehicleInput.get("no_gravity").getAsBoolean());
        var passengers = new ArrayList<TickSnowballFixture>();
        for (JsonElement passengerEntry : request.getAsJsonArray("passengers")) {
            JsonObject passengerInput = passengerEntry.getAsJsonObject();
            if (passengerInput.get("type_id").getAsInt() != 120) {
                throw new IllegalArgumentException(
                    "Passenger position fixture requires snowball passengers");
            }
            passengers.add(TickSnowballFixture.create(
                passengerInput.get("entity_id").getAsInt(),
                vec3(passengerInput.getAsJsonObject("position")),
                vec3(passengerInput.getAsJsonObject("movement")),
                Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                passengerInput.get("no_gravity").getAsBoolean()));
        }
        vehicle.setFixturePassengers(resolvePassengerIds(
            passengers, request.getAsJsonArray("initial_passenger_ids")));

        var states = new JsonArray();
        states.add(passengerPositionState("initial", vehicle, passengers));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            String action = operation.get("action").getAsString();
            if (action.equals("set-passengers-and-tick")) {
                vehicle.setFixturePassengers(resolvePassengerIds(
                    passengers, operation.getAsJsonArray("passenger_ids")));
            } else if (!action.equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger position action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                tickPassengerPositionTree(vehicle, passengers);
            }
            states.add(passengerPositionState(
                operation.get("label").getAsString(), vehicle, passengers));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPassengerPlayerPositionResult(JsonObject request) {
        JsonObject vehicleInput = request.getAsJsonObject("vehicle");
        int vehicleId = vehicleInput.get("entity_id").getAsInt();
        if (vehicleInput.get("type_id").getAsInt() != 89) {
            throw new IllegalArgumentException("Passenger player position fixture requires an oak boat");
        }
        PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
            vehicleId,
            vec3(vehicleInput.getAsJsonObject("position")),
            Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
            vehicleInput.get("no_gravity").getAsBoolean());
        JsonObject passengerInput = request.getAsJsonObject("passenger");
        if (passengerInput.get("type_id").getAsInt() != 156) {
            throw new IllegalArgumentException(
                "Passenger player position fixture requires a Player passenger");
        }
        PassengerPositionPlayerFixture passenger = PassengerPositionPlayerFixture.create(
            passengerInput.get("entity_id").getAsInt(),
            vec3(passengerInput.getAsJsonObject("position")),
            vec3(passengerInput.getAsJsonObject("movement")),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
            passengerInput.get("no_gravity").getAsBoolean(),
            vehicle);
        vehicle.setFixturePassengers(List.of(passenger));

        var states = new JsonArray();
        states.add(passengerPlayerPositionState("initial", vehicle, passenger));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger player position action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                vehicle.setOldPosAndRot();
                vehicle.tickCount++;
                vehicle.tick();
                if (vehicle.hasPassenger(passenger)) {
                    passenger.setOldPosAndRot();
                    passenger.tickCount++;
                    passenger.rideTick();
                }
            }
            states.add(passengerPlayerPositionState(
                operation.get("label").getAsString(), vehicle, passenger));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPassengerBoatFamilyPositionResult(
        JsonObject request
    ) {
        var vehicles = new ArrayList<net.minecraft.world.entity.vehicle.boat.AbstractBoat>();
        var passengers = new ArrayList<TickSnowballFixture>();
        for (JsonElement rowEntry : request.getAsJsonArray("rows")) {
            JsonObject row = rowEntry.getAsJsonObject();
            JsonObject vehicleInput = row.getAsJsonObject("vehicle");
            int vehicleTypeId = vehicleInput.get("type_id").getAsInt();
            net.minecraft.world.entity.vehicle.boat.AbstractBoat vehicle =
                createPassengerPositionBoat(
                    vehicleTypeId,
                    vehicleInput.get("entity_id").getAsInt(),
                    vec3(vehicleInput.getAsJsonObject("position")),
                    Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
                    vehicleInput.get("no_gravity").getAsBoolean());
            JsonObject passengerInput = row.getAsJsonObject("passenger");
            if (passengerInput.get("type_id").getAsInt() != 120) {
                throw new IllegalArgumentException(
                    "Boat family position fixture requires snowball passengers");
            }
            TickSnowballFixture passenger = TickSnowballFixture.create(
                passengerInput.get("entity_id").getAsInt(),
                vec3(passengerInput.getAsJsonObject("position")),
                vec3(passengerInput.getAsJsonObject("movement")),
                Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                passengerInput.get("no_gravity").getAsBoolean());
            setPassengerPositionFixturePassengers(vehicle, List.of(passenger));
            vehicles.add(vehicle);
            passengers.add(passenger);
        }

        var states = new JsonArray();
        states.add(boatFamilyPositionState("initial", vehicles, passengers));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown boat family position action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (int index = 0; index < vehicles.size(); index++) {
                    var vehicle = vehicles.get(index);
                    vehicle.setOldPosAndRot();
                    vehicle.tickCount++;
                    vehicle.tick();
                    TickSnowballFixture passenger = passengers.get(index);
                    passenger.rideTickThroughClientLevel();
                    vehicle.positionRider(passenger);
                }
            }
            states.add(boatFamilyPositionState(
                operation.get("label").getAsString(), vehicles, passengers));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static net.minecraft.world.entity.vehicle.boat.AbstractBoat createPassengerPositionBoat(
        int typeId, int entityId, Vec3 position, float yRot, boolean noGravity
    ) {
        return switch (typeId) {
            case 8 -> PassengerPositionChestRaftFixture.create(
                entityId, position, yRot, noGravity);
            case 9 -> PassengerPositionRaftFixture.create(entityId, position, yRot, noGravity);
            case 89 -> PassengerPositionBoatFixture.create(
                entityId, position, yRot, noGravity);
            case 90 -> PassengerPositionChestBoatFixture.create(
                entityId, position, yRot, noGravity);
            default -> throw new IllegalArgumentException(
                "Unknown boat family position vehicle type");
        };
    }

    static JsonObject clientWorldEntityPassengerAnimalPositionResult(
        JsonObject request
    ) {
        var output = System.out;
        var errorOutput = System.err;
        try {
            SharedConstants.setVersion(TEST_VERSION);
            Bootstrap.bootStrap();
        } finally {
            System.setOut(output);
            System.setErr(errorOutput);
        }

        JsonObject vehicleInput = request.getAsJsonObject("vehicle");
        if (vehicleInput.get("type_id").getAsInt() != 89) {
            throw new IllegalArgumentException(
                "Passenger animal position fixture requires an oak boat");
        }
        PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
            vehicleInput.get("entity_id").getAsInt(),
            vec3(vehicleInput.getAsJsonObject("position")),
            Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
            vehicleInput.get("no_gravity").getAsBoolean());

        JsonObject animalInput = request.getAsJsonObject("animal");
        if (animalInput.get("type_id").getAsInt() != 30) {
            throw new IllegalArgumentException(
                "Passenger animal position fixture requires a Cow");
        }
        PassengerPositionCowFixture animal = PassengerPositionCowFixture.create(
            animalInput.get("entity_id").getAsInt(),
            vec3(animalInput.getAsJsonObject("position")),
            vec3(animalInput.getAsJsonObject("movement")),
            Oracle.f32(animalInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(animalInput.getAsJsonObject("rotation"), "x"),
            animalInput.get("no_gravity").getAsBoolean(),
            vehicle);
        setPassengerPositionFixturePassengers(vehicle, List.of(animal));

        var states = new JsonArray();
        states.add(passengerAnimalPositionState(
            "initial", vehicle, animal));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger animal position action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                vehicle.setOldPosAndRot();
                vehicle.tickCount++;
                vehicle.tick();
                animal.setOldPosAndRot();
                animal.tickCount++;
                animal.rideTick();
            }
            states.add(passengerAnimalPositionState(
                operation.get("label").getAsString(), vehicle, animal));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static List<TickSnowballFixture> resolvePassengerIds(
        List<TickSnowballFixture> passengers, JsonArray passengerIds
    ) {
        var resolved = new ArrayList<TickSnowballFixture>();
        for (JsonElement passengerEntry : passengerIds) {
            int passengerId = passengerEntry.getAsInt();
            TickSnowballFixture passenger = passengers.stream()
                .filter(candidate -> candidate.entityId() == passengerId)
                .findFirst()
                .orElseThrow(() -> new IllegalArgumentException(
                    "Unknown passenger position fixture entity"));
            resolved.add(passenger);
        }
        return resolved;
    }

    static void tickPassengerPositionTree(
        PassengerPositionBoatFixture vehicle,
        List<TickSnowballFixture> allPassengers
    ) {
        vehicle.setOldPosAndRot();
        vehicle.tickCount++;
        vehicle.tick();
        for (Entity passengerEntity : vehicle.getPassengers()) {
            TickSnowballFixture passenger = (TickSnowballFixture)passengerEntity;
            passenger.rideTickThroughClientLevel();
            vehicle.positionRider(passenger);
        }
        for (TickSnowballFixture passenger : allPassengers) {
            if (!vehicle.hasPassenger(passenger)) {
                passenger.tickThroughClientLevel();
            }
        }
    }

    static JsonObject passengerPositionState(
        String label,
        PassengerPositionBoatFixture vehicle,
        List<TickSnowballFixture> passengers
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        entities.add(tickEntityState(vehicle));
        for (TickSnowballFixture passenger : passengers) {
            entities.add(tickSnowballEntityState(passenger));
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject passengerPlayerPositionState(
        String label,
        PassengerPositionBoatFixture vehicle,
        PassengerPositionPlayerFixture passenger
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        entities.add(tickEntityState(vehicle));
        entities.add(tickEntityState(passenger));
        state.add("entities", entities);
        return state;
    }

    static JsonObject boatFamilyPositionState(
        String label,
        List<net.minecraft.world.entity.vehicle.boat.AbstractBoat> vehicles,
        List<TickSnowballFixture> passengers
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (int index = 0; index < vehicles.size(); index++) {
            entities.add(tickEntityState(vehicles.get(index)));
            entities.add(tickSnowballEntityState(passengers.get(index)));
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject passengerAnimalPositionState(
        String label,
        PassengerPositionBoatFixture vehicle,
        PassengerPositionCowFixture animal
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        entities.add(tickEntityState(vehicle));
        entities.add(tickEntityState(animal));
        state.add("entities", entities);
        return state;
    }
}
