import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.SharedConstants;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.server.Bootstrap;
import net.minecraft.tags.TagKey;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.LivingEntity;

class MathPassengerRotationOracle extends MathPassengerPositionOracle {
    static JsonObject clientWorldEntityPassengerRotationResult(JsonObject request) {
        var vehicles = new ArrayList<PassengerPositionBoatFixture>();
        var passengers = new ArrayList<Entity>();
        for (JsonElement rowEntry : request.getAsJsonArray("rows")) {
            JsonObject row = rowEntry.getAsJsonObject();
            JsonObject vehicleInput = row.getAsJsonObject("vehicle");
            if (vehicleInput.get("type_id").getAsInt() != 89) {
                throw new IllegalArgumentException(
                    "Passenger rotation fixture requires an oak boat");
            }
            PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
                vehicleInput.get("entity_id").getAsInt(),
                vec3(vehicleInput.getAsJsonObject("position")),
                Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
                vehicleInput.get("no_gravity").getAsBoolean());
            JsonObject passengerInput = row.getAsJsonObject("passenger");
            int passengerTypeId = passengerInput.get("type_id").getAsInt();
            Entity passenger;
            if (passengerTypeId == 120) {
                passenger = TickSnowballFixture.create(
                    passengerInput.get("entity_id").getAsInt(),
                    vec3(passengerInput.getAsJsonObject("position")),
                    vec3(passengerInput.getAsJsonObject("movement")),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                    passengerInput.get("no_gravity").getAsBoolean());
            } else if (passengerTypeId == 156) {
                passenger = PassengerPositionPlayerFixture.create(
                    passengerInput.get("entity_id").getAsInt(),
                    vec3(passengerInput.getAsJsonObject("position")),
                    vec3(passengerInput.getAsJsonObject("movement")),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                    passengerInput.get("no_gravity").getAsBoolean(),
                    vehicle);
            } else {
                throw new IllegalArgumentException(
                    "Passenger rotation fixture requires snowball or Player passengers");
            }
            setPassengerPositionFixturePassengers(vehicle, List.of(passenger));
            vehicles.add(vehicle);
            passengers.add(passenger);
        }

        var states = new JsonArray();
        states.add(passengerRotationState("initial", vehicles, passengers));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger rotation action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (int index = 0; index < vehicles.size(); index++) {
                    PassengerPositionBoatFixture vehicle = vehicles.get(index);
                    vehicle.setOldPosAndRot();
                    vehicle.tickCount++;
                    vehicle.tick();
                    Entity passenger = passengers.get(index);
                    if (passenger instanceof TickSnowballFixture snowball) {
                        snowball.rideTickThroughClientLevel();
                        vehicle.positionRider(passenger);
                    } else if (passenger instanceof PassengerPositionPlayerFixture player) {
                        player.setOldPosAndRot();
                        player.tickCount++;
                        player.rideTick();
                    } else {
                        throw new IllegalStateException("Unknown passenger rotation fixture");
                    }
                }
            }
            states.add(passengerRotationState(
                operation.get("label").getAsString(), vehicles, passengers));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPassengerBoatTurnExemptionResult(
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
        bindBreezeCanTurnInBoatsTag();

        JsonObject vehicleInput = request.getAsJsonObject("vehicle");
        if (vehicleInput.get("type_id").getAsInt() != 89) {
            throw new IllegalArgumentException(
                "Passenger boat turn exemption fixture requires an oak boat");
        }
        PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
            vehicleInput.get("entity_id").getAsInt(),
            vec3(vehicleInput.getAsJsonObject("position")),
            Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
            vehicleInput.get("no_gravity").getAsBoolean());

        JsonObject passengerInput = request.getAsJsonObject("passenger");
        if (passengerInput.get("type_id").getAsInt() != 17) {
            throw new IllegalArgumentException(
                "Passenger boat turn exemption fixture requires a Breeze");
        }
        PassengerPositionBreezeFixture passenger = PassengerPositionBreezeFixture.create(
            passengerInput.get("entity_id").getAsInt(),
            vec3(passengerInput.getAsJsonObject("position")),
            vec3(passengerInput.getAsJsonObject("movement")),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
            Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
            passengerInput.get("no_gravity").getAsBoolean(),
            vehicle);
        setPassengerPositionFixturePassengers(vehicle, List.of(passenger));

        var states = new JsonArray();
        states.add(passengerRotationState(
            "initial", List.of(vehicle), List.of(passenger)));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger boat turn action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                vehicle.setOldPosAndRot();
                vehicle.tickCount++;
                vehicle.tick();
                passenger.setOldPosAndRot();
                passenger.tickCount++;
                passenger.rideTick();
            }
            states.add(passengerRotationState(
                operation.get("label").getAsString(),
                List.of(vehicle),
                List.of(passenger)));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPassengerAnimalRotationResult(
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
        var vehicles = new ArrayList<PassengerPositionBoatFixture>();
        var animals = new ArrayList<PassengerPositionCowFixture>();
        var fillers = new ArrayList<TickSnowballFixture>();
        for (JsonElement rowEntry : request.getAsJsonArray("rows")) {
            JsonObject row = rowEntry.getAsJsonObject();
            JsonObject vehicleInput = row.getAsJsonObject("vehicle");
            if (vehicleInput.get("type_id").getAsInt() != 89) {
                throw new IllegalArgumentException(
                    "Passenger animal rotation fixture requires an oak boat");
            }
            PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
                vehicleInput.get("entity_id").getAsInt(),
                vec3(vehicleInput.getAsJsonObject("position")),
                Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
                vehicleInput.get("no_gravity").getAsBoolean());

            JsonObject animalInput = row.getAsJsonObject("animal");
            if (animalInput.get("type_id").getAsInt() != 30) {
                throw new IllegalArgumentException(
                    "Passenger animal rotation fixture requires a Cow");
            }
            PassengerPositionCowFixture animal = PassengerPositionCowFixture.create(
                animalInput.get("entity_id").getAsInt(),
                vec3(animalInput.getAsJsonObject("position")),
                vec3(animalInput.getAsJsonObject("movement")),
                Oracle.f32(animalInput.getAsJsonObject("rotation"), "y"),
                Oracle.f32(animalInput.getAsJsonObject("rotation"), "x"),
                animalInput.get("no_gravity").getAsBoolean(),
                vehicle);

            JsonObject fillerInput = row.getAsJsonObject("filler");
            if (fillerInput.get("type_id").getAsInt() != 120) {
                throw new IllegalArgumentException(
                    "Passenger animal rotation fixture requires a Snowball filler");
            }
            TickSnowballFixture filler = TickSnowballFixture.create(
                fillerInput.get("entity_id").getAsInt(),
                vec3(fillerInput.getAsJsonObject("position")),
                vec3(fillerInput.getAsJsonObject("movement")),
                Oracle.f32(fillerInput.getAsJsonObject("rotation"), "y"),
                Oracle.f32(fillerInput.getAsJsonObject("rotation"), "x"),
                fillerInput.get("no_gravity").getAsBoolean());
            setPassengerPositionFixturePassengers(vehicle, List.of(animal, filler));
            vehicles.add(vehicle);
            animals.add(animal);
            fillers.add(filler);
        }

        var states = new JsonArray();
        states.add(passengerAnimalRotationState(
            "initial", vehicles, animals, fillers));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("tick")) {
                throw new IllegalArgumentException("Unknown passenger animal rotation action");
            }
            for (int tick = 0; tick < operation.get("ticks").getAsInt(); tick++) {
                for (int index = 0; index < vehicles.size(); index++) {
                    PassengerPositionBoatFixture vehicle = vehicles.get(index);
                    vehicle.setOldPosAndRot();
                    vehicle.tickCount++;
                    vehicle.tick();
                    PassengerPositionCowFixture animal = animals.get(index);
                    animal.setOldPosAndRot();
                    animal.tickCount++;
                    animal.rideTick();
                    TickSnowballFixture filler = fillers.get(index);
                    filler.rideTickThroughClientLevel();
                    vehicle.positionRider(filler);
                }
            }
            states.add(passengerAnimalRotationState(
                operation.get("label").getAsString(), vehicles, animals, fillers));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static JsonObject clientWorldEntityPassengerTurnResult(JsonObject request) {
        var output = System.out;
        var errorOutput = System.err;
        try {
            SharedConstants.setVersion(TEST_VERSION);
            Bootstrap.bootStrap();
        } finally {
            System.setOut(output);
            System.setErr(errorOutput);
        }
        var vehicles = new ArrayList<PassengerPositionBoatFixture>();
        var passengers = new ArrayList<Entity>();
        for (JsonElement rowEntry : request.getAsJsonArray("rows")) {
            JsonObject row = rowEntry.getAsJsonObject();
            JsonObject vehicleInput = row.getAsJsonObject("vehicle");
            if (vehicleInput.get("type_id").getAsInt() != 89) {
                throw new IllegalArgumentException(
                    "Passenger turn fixture requires an oak boat");
            }
            PassengerPositionBoatFixture vehicle = PassengerPositionBoatFixture.create(
                vehicleInput.get("entity_id").getAsInt(),
                vec3(vehicleInput.getAsJsonObject("position")),
                Oracle.f32(vehicleInput.getAsJsonObject("rotation"), "y"),
                vehicleInput.get("no_gravity").getAsBoolean());

            JsonObject passengerInput = row.getAsJsonObject("passenger");
            int passengerTypeId = passengerInput.get("type_id").getAsInt();
            Entity passenger;
            if (passengerTypeId == 156) {
                passenger = PassengerPositionPlayerFixture.create(
                    passengerInput.get("entity_id").getAsInt(),
                    vec3(passengerInput.getAsJsonObject("position")),
                    vec3(passengerInput.getAsJsonObject("movement")),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                    passengerInput.get("no_gravity").getAsBoolean(),
                    vehicle);
            } else if (passengerTypeId == 17) {
                passenger = PassengerPositionBreezeFixture.create(
                    passengerInput.get("entity_id").getAsInt(),
                    vec3(passengerInput.getAsJsonObject("position")),
                    vec3(passengerInput.getAsJsonObject("movement")),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "y"),
                    Oracle.f32(passengerInput.getAsJsonObject("rotation"), "x"),
                    passengerInput.get("no_gravity").getAsBoolean(),
                    vehicle);
            } else {
                throw new IllegalArgumentException(
                    "Passenger turn fixture requires a Player or Breeze passenger");
            }
            setPassengerPositionFixturePassengers(vehicle, List.of(passenger));
            vehicles.add(vehicle);
            passengers.add(passenger);
        }

        var states = new JsonArray();
        states.add(passengerRotationState("initial", vehicles, passengers));
        for (JsonElement operationEntry : request.getAsJsonArray("operations")) {
            JsonObject operation = operationEntry.getAsJsonObject();
            if (!operation.get("action").getAsString().equals("turn")) {
                throw new IllegalArgumentException("Unknown passenger turn action");
            }
            double mouseX = operation.get("mouse_x").getAsDouble();
            double mouseY = operation.get("mouse_y").getAsDouble();
            for (Entity passenger : passengers) {
                passenger.turn(mouseX, mouseY);
            }
            states.add(passengerRotationState(
                operation.get("label").getAsString(), vehicles, passengers));
        }
        var result = new JsonObject();
        result.add("states", states);
        return result;
    }

    static void bindBreezeCanTurnInBoatsTag() {
        java.util.Map<net.minecraft.tags.TagKey<net.minecraft.world.entity.EntityType<?>>,
            java.util.List<net.minecraft.core.Holder<net.minecraft.world.entity.EntityType<?>>>> tags =
            java.util.Map.of(
                net.minecraft.tags.EntityTypeTags.CAN_TURN_IN_BOATS,
                java.util.List.of(
                    BuiltInRegistries.ENTITY_TYPE.wrapAsHolder(
                        net.minecraft.world.entity.EntityTypes.BREEZE)));
        BuiltInRegistries.ENTITY_TYPE.prepareTagReload(
            new net.minecraft.tags.TagLoader.LoadResult<>(
                net.minecraft.core.registries.Registries.ENTITY_TYPE, tags)).apply();
    }

    static JsonObject passengerRotationState(
        String label,
        List<PassengerPositionBoatFixture> vehicles,
        List<Entity> passengers
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (int index = 0; index < vehicles.size(); index++) {
            entities.add(tickEntityState(vehicles.get(index)));
            entities.add(passengerRotationEntityState(passengers.get(index)));
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject passengerAnimalRotationState(
        String label,
        List<PassengerPositionBoatFixture> vehicles,
        List<PassengerPositionCowFixture> animals,
        List<TickSnowballFixture> fillers
    ) {
        var state = new JsonObject();
        state.addProperty("label", label);
        state.addProperty("accepted", true);
        var entities = new JsonArray();
        for (int index = 0; index < vehicles.size(); index++) {
            entities.add(tickEntityState(vehicles.get(index)));
            entities.add(passengerRotationEntityState(animals.get(index)));
            entities.add(tickSnowballEntityState(fillers.get(index)));
        }
        state.add("entities", entities);
        return state;
    }

    static JsonObject passengerRotationEntityState(Entity entity) {
        if (entity instanceof TickSnowballFixture snowball) {
            return tickSnowballEntityState(snowball);
        }
        var state = tickEntityState(entity);
        if (entity instanceof LivingEntity living) {
            state.addProperty("head_rotation", Oracle.bits(living.yHeadRot));
            state.addProperty("previous_head_rotation", Oracle.bits(living.yHeadRotO));
            state.addProperty("body_rotation_y", Oracle.bits(living.yBodyRot));
            state.addProperty("previous_body_rotation_y", Oracle.bits(living.yBodyRotO));
        }
        return state;
    }
}
