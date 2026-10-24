import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

class EntityDataOracle extends EntityBossOracle {
    static JsonArray floats(final float... values) {
        var array = new JsonArray();
        for (float value : values) {
            array.add(Oracle.bits(value));
        }
        return array;
    }

    /** One part of a layer's definition: its pose, its cubes and its children by name. */
    @SuppressWarnings("unchecked")
    static JsonObject partDefinition(final String name, final Object part) throws Exception {
        var out = new JsonObject();
        out.addProperty("name", name);
        var pose = (net.minecraft.client.model.geom.PartPose) field(part, "partPose");
        out.add("pose", floats(pose.x(), pose.y(), pose.z(), pose.xRot(), pose.yRot(), pose.zRot(),
            pose.xScale(), pose.yScale(), pose.zScale()));
        var cubes = new JsonArray();
        for (var cube : (java.util.List<Object>) field(part, "cubes")) {
            var described = new JsonObject();
            var origin = (org.joml.Vector3fc) field(cube, "origin");
            var dimensions = (org.joml.Vector3fc) field(cube, "dimensions");
            var grow = field(cube, "grow");
            var uv = (net.minecraft.client.model.geom.builders.UVPair) field(cube, "texCoord");
            var scale = (net.minecraft.client.model.geom.builders.UVPair) field(cube, "texScale");
            described.add("origin", floats(origin.x(), origin.y(), origin.z()));
            described.add("dimensions", floats(dimensions.x(), dimensions.y(), dimensions.z()));
            described.add("grow", floats((float) field(grow, "growX"), (float) field(grow, "growY"),
                (float) field(grow, "growZ")));
            described.addProperty("mirror", (boolean) field(cube, "mirror"));
            described.add("uv", floats(uv.u(), uv.v()));
            described.add("uv_scale", floats(scale.u(), scale.v()));
            var faces = new java.util.TreeSet<String>();
            for (var face : (java.util.Set<net.minecraft.core.Direction>) field(cube, "visibleFaces")) {
                faces.add(face.getName());
            }
            var visible = new JsonArray();
            faces.forEach(visible::add);
            described.add("faces", visible);
            cubes.add(described);
        }
        out.add("cubes", cubes);
        var children = new java.util.TreeMap<String, Object>();
        for (var child : ((java.util.Map<String, Object>) field(part, "children")).entrySet()) {
            children.put(child.getKey(), child.getValue());
        }
        var described = new JsonArray();
        for (var child : children.entrySet()) {
            described.add(partDefinition(child.getKey(), child.getValue()));
        }
        out.add("children", described);
        return out;
    }

    /**
     * The attributes every living entity type starts with (`DefaultAttributes`: the base value of
     * each attribute its supplier has), in entity type order, and each attribute's range.
     */
    static JsonObject entityAttributes() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        var types = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        var attributes = net.minecraft.core.registries.BuiltInRegistries.ATTRIBUTE;
        for (var type : types) {
            if (!net.minecraft.world.entity.ai.attributes.DefaultAttributes.hasSupplier(
                (net.minecraft.world.entity.EntityType) type)) {
                continue;
            }
            var supplier = net.minecraft.world.entity.ai.attributes.DefaultAttributes.getSupplier(
                (net.minecraft.world.entity.EntityType) type);
            var out = new JsonObject();
            out.addProperty("label", types.getKey(type).toString());
            out.addProperty("kind", "type");
            out.addProperty("id", types.getId(type));
            var list = new JsonArray();
            var holders = new java.util.ArrayList<net.minecraft.core.Holder.Reference<
                net.minecraft.world.entity.ai.attributes.Attribute>>(attributes.listElements().toList());
            holders.sort(java.util.Comparator.comparingInt(holder -> attributes.getId(holder.value())));
            for (var holder : holders) {
                if (supplier.hasAttribute(holder)) {
                    var pair = new JsonObject();
                    pair.addProperty("name", holder.key().identifier().toString());
                    pair.addProperty("base", Oracle.bits(supplier.getBaseValue(holder)));
                    list.add(pair);
                }
            }
            out.add("attributes", list);
            rows.add(out);
        }
        for (var attribute : attributes) {
            var out = new JsonObject();
            out.addProperty("label", attributes.getKey(attribute).toString());
            out.addProperty("kind", "range");
            out.addProperty("id", attributes.getId(attribute));
            out.addProperty("default", Oracle.bits(attribute.getDefaultValue()));
            if (attribute instanceof net.minecraft.world.entity.ai.attributes.RangedAttribute ranged) {
                out.addProperty("min", Oracle.bits(ranged.getMinValue()));
                out.addProperty("max", Oracle.bits(ranged.getMaxValue()));
            }
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The entity type registry: each type's id and name, in id order. */
    static JsonObject entityTypes() {
        Oracle.vanillaRegistries();
        var registry = net.minecraft.core.registries.BuiltInRegistries.ENTITY_TYPE;
        var rows = new JsonArray();
        for (var type : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(type).toString());
            out.addProperty("id", registry.getId(type));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** The cat, chicken, cow, frog, pig and zombie nautilus variants the game's data pack holds: their textures and model. */
    static JsonObject entityVariants() {
        var registries = Oracle.vanillaRegistries();
        var rows = new java.util.TreeMap<String, JsonObject>();
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.PIG_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "pig", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.COW_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "cow", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.CHICKEN_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "chicken", holder.key().identifier().toString(), variant.modelAndTexture().asset().id(),
                variant.babyTexture().id(), variant.modelAndTexture().model().name());
        });
        // A cat's variant has no model family of its own: the model is the cat's.
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.CAT_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "cat", holder.key().identifier().toString(), variant.adultAssetInfo().id(),
                variant.babyAssetInfo().id(), "normal");
        });
        // A frog's variant has one texture and no baby (a frog is never one).
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.FROG_VARIANT).listElements().forEach(holder -> {
            variants(rows, "frog", holder.key().identifier().toString(), holder.value().assetInfo().id(), null, "normal");
        });
        registries.lookupOrThrow(net.minecraft.core.registries.Registries.ZOMBIE_NAUTILUS_VARIANT).listElements().forEach(holder -> {
            var variant = holder.value();
            variants(rows, "zombie_nautilus", holder.key().identifier().toString(),
                variant.modelAndTexture().asset().id(), null, variant.modelAndTexture().model().getSerializedName());
        });
        var list = new JsonArray();
        rows.values().forEach(list::add);
        var result = new JsonObject();
        result.add("rows", list);
        return result;
    }

    /** The wolf variants of the game's data pack: the six textures each wears (grown and baby; wild, tame, angry). */
    static JsonObject wolfVariants() {
        var rows = new java.util.TreeMap<String, JsonObject>();
        Oracle.vanillaRegistries().lookupOrThrow(net.minecraft.core.registries.Registries.WOLF_VARIANT)
            .listElements().forEach(holder -> {
                var variant = holder.value();
                var out = new JsonObject();
                String name = holder.key().identifier().toString();
                out.addProperty("label", name);
                out.addProperty("name", name);
                var adult = variant.adultInfo();
                var baby = variant.babyInfo();
                out.addProperty("wild", adult.wild().id().toString());
                out.addProperty("tame", adult.tame().id().toString());
                out.addProperty("angry", adult.angry().id().toString());
                out.addProperty("baby_wild", baby.wild().id().toString());
                out.addProperty("baby_tame", baby.tame().id().toString());
                out.addProperty("baby_angry", baby.angry().id().toString());
                rows.put(name, out);
            });
        var list = new JsonArray();
        rows.values().forEach(list::add);
        var result = new JsonObject();
        result.add("rows", list);
        return result;
    }

    /**
     * The map's data tables: what each of the 256 packed colours is (`MapColor.getColorFromPackedId`)
     * and the decoration types of the registry in the order the server numbers them.
     */
    static JsonObject mapData() {
        var rows = new JsonArray();
        var colors = new JsonObject();
        colors.addProperty("label", "colors");
        var packed = new JsonArray();
        for (int id = 0; id < 256; id++) {
            packed.add(net.minecraft.world.level.material.MapColor.getColorFromPackedId(id));
        }
        colors.add("colors", packed);
        rows.add(colors);
        var registry = net.minecraft.core.registries.BuiltInRegistries.MAP_DECORATION_TYPE;
        for (var type : registry) {
            var out = new JsonObject();
            out.addProperty("label", registry.getKey(type).toString());
            out.addProperty("id", registry.getId(type));
            out.addProperty("asset", type.assetId().toString());
            out.addProperty("show_on_item_frame", type.showOnItemFrame());
            out.addProperty("map_color", type.mapColor());
            out.addProperty("track_count", type.trackCount());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    static JsonObject paintingVariants() {
        var rows = new java.util.TreeMap<String, JsonObject>();
        Oracle.vanillaRegistries().lookupOrThrow(net.minecraft.core.registries.Registries.PAINTING_VARIANT)
            .listElements().forEach(holder -> {
                var variant = holder.value();
                var out = new JsonObject();
                String name = holder.key().identifier().toString();
                out.addProperty("label", name);
                out.addProperty("name", name);
                out.addProperty("asset", variant.assetId().toString());
                out.addProperty("width", variant.width());
                out.addProperty("height", variant.height());
                rows.put(name, out);
            });
        var list = new JsonArray();
        rows.values().forEach(list::add);
        var result = new JsonObject();
        result.add("rows", list);
        return result;
    }

    static void variants(
        final java.util.Map<String, JsonObject> rows, final String kind, final String name,
        final net.minecraft.resources.Identifier asset, final net.minecraft.resources.Identifier baby,
        final String model) {
        var out = new JsonObject();
        out.addProperty("label", kind + ":" + name);
        out.addProperty("kind", kind);
        out.addProperty("name", name);
        out.addProperty("asset", asset.toString());
        out.addProperty("baby_asset", baby == null ? "" : baby.toString());
        out.addProperty("model", model.toLowerCase());
        rows.put(kind + ":" + name, out);
    }

    /**
     * Where the variant sits among a pig's and a cow's data (the accessor
     * their classes define last but for the sound), and which serializer
     * carries it.
     */
    static JsonObject entityVariantFields() throws Exception {
        Oracle.vanillaRegistries();
        var rows = new JsonArray();
        Object[][] fields = {
            {net.minecraft.world.entity.animal.pig.Pig.class, "DATA_VARIANT_ID", "pig"},
            {net.minecraft.world.entity.animal.cow.Cow.class, "DATA_VARIANT_ID", "cow"},
            {net.minecraft.world.entity.ExperienceOrb.class, "DATA_VALUE", "experienceorb"},
            {net.minecraft.world.entity.monster.piglin.Piglin.class, "DATA_BABY_ID", "piglin-baby"},
            {net.minecraft.world.entity.monster.piglin.Piglin.class, "DATA_IS_DANCING", "piglin-dancing"},
            {net.minecraft.world.entity.Avatar.class, "DATA_PLAYER_MAIN_HAND", "avatar-main-hand"},
            {net.minecraft.world.entity.animal.sheep.Sheep.class, "DATA_WOOL_ID", "sheep-wool"},
            {net.minecraft.world.entity.animal.chicken.Chicken.class, "DATA_VARIANT_ID", "chicken"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_CLIENT_FLAGS", "armor-stand-flags"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_HEAD_POSE", "armor-stand-head"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_BODY_POSE", "armor-stand-body"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_LEFT_ARM_POSE", "armor-stand-left-arm"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_RIGHT_ARM_POSE", "armor-stand-right-arm"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_LEFT_LEG_POSE", "armor-stand-left-leg"},
            {net.minecraft.world.entity.decoration.ArmorStand.class, "DATA_RIGHT_LEG_POSE", "armor-stand-right-leg"},
            {net.minecraft.world.entity.npc.villager.Villager.class, "DATA_VILLAGER_DATA", "villager-data"},
            {net.minecraft.world.entity.npc.villager.AbstractVillager.class, "DATA_UNHAPPY_COUNTER", "villager-unhappy"},
            {net.minecraft.world.entity.animal.equine.AbstractHorse.class, "DATA_ID_FLAGS", "horse-flags"},
            {net.minecraft.world.entity.animal.equine.Horse.class, "DATA_ID_TYPE_VARIANT", "horse-variant"},
            {net.minecraft.world.entity.animal.equine.AbstractChestedHorse.class, "DATA_ID_CHEST", "horse-chest"},
            {net.minecraft.world.entity.animal.equine.Llama.class, "DATA_VARIANT_ID", "llama-variant"},
            {net.minecraft.world.entity.animal.camel.Camel.class, "DASH", "camel-dash"},
            {net.minecraft.world.entity.animal.camel.Camel.class, "LAST_POSE_CHANGE_TICK", "camel-last-pose"},
            {net.minecraft.world.entity.TamableAnimal.class, "DATA_FLAGS_ID", "tamable-flags"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_INTERESTED_ID", "wolf-interested"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_COLLAR_COLOR", "wolf-collar"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_ANGER_END_TIME", "wolf-anger-end"},
            {net.minecraft.world.entity.animal.wolf.Wolf.class, "DATA_VARIANT_ID", "wolf-variant"},
            {net.minecraft.world.entity.animal.polarbear.PolarBear.class, "DATA_STANDING_ID", "polar-bear-standing"},
            {net.minecraft.world.entity.monster.Strider.class, "DATA_SUFFOCATING", "strider-suffocating"},
            {net.minecraft.world.entity.monster.hoglin.Hoglin.class, "DATA_IMMUNE_TO_ZOMBIFICATION", "hoglin-immune"},
            {net.minecraft.world.entity.animal.golem.CopperGolem.class, "DATA_WEATHER_STATE", "copper-golem-weather"},
            {net.minecraft.world.entity.animal.golem.CopperGolem.class, "COPPER_GOLEM_STATE", "copper-golem-state"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "UNHAPPY_COUNTER", "panda-unhappy"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "SNEEZE_COUNTER", "panda-sneeze"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "EAT_COUNTER", "panda-eat"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "MAIN_GENE_ID", "panda-main-gene"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "HIDDEN_GENE_ID", "panda-hidden-gene"},
            {net.minecraft.world.entity.animal.panda.Panda.class, "DATA_ID_FLAGS", "panda-flags"},
            {net.minecraft.world.entity.animal.fox.Fox.class, "DATA_TYPE_ID", "fox-variant"},
            {net.minecraft.world.entity.animal.fox.Fox.class, "DATA_FLAGS_ID", "fox-flags"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "DATA_VARIANT_ID", "cat-variant"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "IS_LYING", "cat-lying"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "RELAX_STATE_ONE", "cat-relax"},
            {net.minecraft.world.entity.animal.feline.Cat.class, "DATA_COLLAR_COLOR", "cat-collar"},
            {net.minecraft.world.entity.animal.bee.Bee.class, "DATA_FLAGS_ID", "bee-flags"},
            {net.minecraft.world.entity.animal.bee.Bee.class, "DATA_ANGER_END_TIME", "bee-anger-end"},
            {net.minecraft.world.entity.animal.rabbit.Rabbit.class, "DATA_TYPE_ID", "rabbit-type"},
            {net.minecraft.world.entity.animal.goat.Goat.class, "DATA_HAS_LEFT_HORN", "goat-left-horn"},
            {net.minecraft.world.entity.animal.goat.Goat.class, "DATA_HAS_RIGHT_HORN", "goat-right-horn"},
            {net.minecraft.world.entity.animal.cow.MushroomCow.class, "DATA_TYPE", "mooshroom-type"},
            {net.minecraft.world.entity.animal.golem.SnowGolem.class, "DATA_PUMPKIN_ID", "snow-golem-pumpkin"},
            {net.minecraft.world.entity.monster.EnderMan.class, "DATA_CARRY_STATE", "enderman-carry"},
            {net.minecraft.world.entity.monster.EnderMan.class, "DATA_CREEPY", "enderman-creepy"},
            {net.minecraft.world.entity.monster.zombie.ZombieVillager.class, "DATA_VILLAGER_DATA", "zombie-villager-data"},
            {net.minecraft.world.entity.raid.Raider.class, "IS_CELEBRATING", "raider-celebrating"},
            {net.minecraft.world.entity.monster.illager.SpellcasterIllager.class, "DATA_SPELL_CASTING_ID", "spellcaster-casting"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_IS_POWERED", "creeper-powered"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_SWELL_DIR", "creeper-swell-dir"},
            {net.minecraft.world.entity.monster.Creeper.class, "DATA_IS_IGNITED", "creeper-ignited"},
            {net.minecraft.world.entity.animal.frog.Frog.class, "DATA_VARIANT_ID", "frog-variant"},
            {net.minecraft.world.entity.animal.nautilus.ZombieNautilus.class, "DATA_VARIANT_ID", "zombie-nautilus-variant"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_HURT", "boat-hurt"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_HURTDIR", "boat-hurt-dir"},
            {net.minecraft.world.entity.vehicle.VehicleEntity.class, "DATA_ID_DAMAGE", "boat-damage"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_PADDLE_LEFT", "boat-paddle-left"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_PADDLE_RIGHT", "boat-paddle-right"},
            {net.minecraft.world.entity.vehicle.boat.AbstractBoat.class, "DATA_ID_BUBBLE_TIME", "boat-bubble-time"},
            {net.minecraft.world.entity.animal.parrot.Parrot.class, "DATA_VARIANT_ID", "parrot-variant"},
            {net.minecraft.world.entity.ambient.Bat.class, "DATA_ID_FLAGS", "bat-flags"},
            {net.minecraft.world.entity.monster.Vex.class, "DATA_FLAGS_ID", "vex-flags"},
            {net.minecraft.world.entity.animal.allay.Allay.class, "DATA_DANCING", "allay-dancing"},
            {net.minecraft.world.entity.monster.Ghast.class, "DATA_IS_CHARGING", "ghast-charging"},
            {net.minecraft.world.entity.monster.Phantom.class, "ID_SIZE", "phantom-size"},
            {net.minecraft.world.entity.monster.Guardian.class, "DATA_ID_MOVING", "guardian-moving"},
            {net.minecraft.world.entity.monster.Guardian.class, "DATA_ID_ATTACK_TARGET", "guardian-attack-target"},
            {net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_VARIANT", "axolotl-variant"},
            {net.minecraft.world.entity.animal.axolotl.Axolotl.class, "DATA_PLAYING_DEAD", "axolotl-playing-dead"},
            {net.minecraft.world.entity.animal.turtle.Turtle.class, "HAS_EGG", "turtle-has-egg"},
            {net.minecraft.world.entity.animal.turtle.Turtle.class, "LAYING_EGG", "turtle-laying-egg"},
            {net.minecraft.world.entity.animal.fish.Salmon.class, "DATA_TYPE", "salmon-type"},
            {net.minecraft.world.entity.animal.fish.TropicalFish.class, "DATA_ID_TYPE_VARIANT", "tropical-fish-variant"},
            {net.minecraft.world.entity.animal.fish.Pufferfish.class, "PUFF_STATE", "pufferfish-puff-state"},
        };
        for (var described : fields) {
            var field = Reflect.field(((Class<?>) described[0]), (String) described[1]);
            var accessor = (net.minecraft.network.syncher.EntityDataAccessor<?>) field.get(null);
            var out = new JsonObject();
            out.addProperty("label", (String) described[2]);
            out.addProperty("data_id", accessor.id());
            out.addProperty("serializer",
                net.minecraft.network.syncher.EntityDataSerializers.getSerializedId(accessor.serializer()));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /**
     * Every entity data serializer by the id the wire carries, named by the
     * field that holds it in `EntityDataSerializers`.
     */
    static JsonObject entityDataSerializers() throws Exception {
        Oracle.vanillaRegistries();
        var byId = new java.util.TreeMap<Integer, String>();
        for (var field : net.minecraft.network.syncher.EntityDataSerializers.class.getDeclaredFields()) {
            if (!java.lang.reflect.Modifier.isStatic(field.getModifiers())
                || !net.minecraft.network.syncher.EntityDataSerializer.class.isAssignableFrom(field.getType())) {
                continue;
            }
            Reflect.open(field);
            var serializer = (net.minecraft.network.syncher.EntityDataSerializer<?>) field.get(null);
            int id = net.minecraft.network.syncher.EntityDataSerializers.getSerializedId(serializer);
            if (id >= 0) {
                byId.put(id, field.getName());
            }
        }
        var rows = new JsonArray();
        for (var entry : byId.entrySet()) {
            var out = new JsonObject();
            out.addProperty("label", entry.getValue());
            out.addProperty("id", entry.getKey());
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }

    /** Every model layer the game builds: its texture size and the tree of its parts. */
    static JsonObject modelLayers() throws Exception {
        // The layers are made from registries (decorated pot patterns): the game's bootstrap first.
        Oracle.vanillaRegistries();
        var layers = new java.util.TreeMap<String, Object>();
        for (var entry : net.minecraft.client.model.geom.LayerDefinitions.createRoots().entrySet()) {
            layers.put(entry.getKey().toString(), entry.getValue());
        }
        var rows = new JsonArray();
        for (var entry : layers.entrySet()) {
            var layer = entry.getValue();
            var mesh = field(layer, "mesh");
            var material = field(layer, "material");
            var out = new JsonObject();
            out.addProperty("label", entry.getKey());
            out.addProperty("texture_width", (int) field(material, "xTexSize"));
            out.addProperty("texture_height", (int) field(material, "yTexSize"));
            var root = ((net.minecraft.client.model.geom.builders.MeshDefinition) mesh).getRoot();
            out.add("root", partDefinition("root", root));
            rows.add(out);
        }
        var result = new JsonObject();
        result.add("rows", rows);
        return result;
    }
}
