import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import io.netty.buffer.Unpooled;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundGameEventPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket;
import net.minecraft.util.Mth;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.level.GameType;

class MathShapePacketOracle extends MathShapeBoxOracle {
    static JsonObject shapeplayerAbilitiesPacketCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            byte[] bytes = hexBytes(input.get("hex").getAsString());
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.wrappedBuffer(bytes));
            ClientboundPlayerAbilitiesPacket packet =
                ClientboundPlayerAbilitiesPacket.STREAM_CODEC.decode(buffer);
            Abilities abilities = new Abilities();
            abilities.invulnerable = packet.isInvulnerable();
            abilities.flying = packet.isFlying();
            abilities.mayfly = packet.canFly();
            abilities.instabuild = packet.canInstabuild();
            abilities.setFlyingSpeed(packet.getFlyingSpeed());
            abilities.setWalkingSpeed(packet.getWalkingSpeed());
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("invulnerable", abilities.invulnerable);
            state.addProperty("flying", abilities.flying);
            state.addProperty("may_fly", abilities.mayfly);
            state.addProperty("instabuild", abilities.instabuild);
            state.addProperty("flying_speed", Oracle.bits(abilities.getFlyingSpeed()));
            state.addProperty("walking_speed", Oracle.bits(abilities.getWalkingSpeed()));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }

    static JsonObject shapeplayerGameModeEventCase(JsonObject request) {
        var result = new JsonObject();
        var states = new JsonArray();
        for (var entry : request.getAsJsonArray("inputs")) {
            JsonObject input = entry.getAsJsonObject();
            byte[] bytes = hexBytes(input.get("hex").getAsString());
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.wrappedBuffer(bytes));
            ClientboundGameEventPacket packet =
                ClientboundGameEventPacket.STREAM_CODEC.decode(buffer);
            if (packet.getEvent() != ClientboundGameEventPacket.CHANGE_GAME_MODE) {
                throw new IllegalArgumentException("Expected change-game-mode event");
            }
            int modeId = Mth.floor(packet.getParam() + 0.5F);
            GameType gameType = GameType.byId(modeId);
            Abilities abilities = new Abilities();
            if (input.has("initial_flying")) {
                abilities.flying = input.get("initial_flying").getAsBoolean();
            }
            if (input.has("initial_flying_speed")) {
                abilities.setFlyingSpeed(Oracle.f32(input, "initial_flying_speed"));
            }
            if (input.has("initial_walking_speed")) {
                abilities.setWalkingSpeed(Oracle.f32(input, "initial_walking_speed"));
            }
            gameType.updatePlayerAbilities(abilities);
            var state = new JsonObject();
            state.addProperty("label", input.get("label").getAsString());
            state.addProperty("param", Oracle.bits(packet.getParam()));
            state.addProperty("mode_id", gameType.getId());
            state.addProperty("mode_name", gameType.getName());
            state.addProperty("is_spectator", gameType == GameType.SPECTATOR);
            state.addProperty("is_creative", gameType.isCreative());
            state.addProperty("is_survival", gameType.isSurvival());
            state.addProperty("may_build", abilities.mayBuild);
            state.addProperty("invulnerable", abilities.invulnerable);
            state.addProperty("flying", abilities.flying);
            state.addProperty("may_fly", abilities.mayfly);
            state.addProperty("instabuild", abilities.instabuild);
            state.addProperty("flying_speed", Oracle.bits(abilities.getFlyingSpeed()));
            state.addProperty("walking_speed", Oracle.bits(abilities.getWalkingSpeed()));
            states.add(state);
        }
        result.add("states", states);
        return result;
    }
}
