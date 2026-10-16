//! Entity animation scenarios: flyers.

use crate::movement::entities::prelude::*;

fn flyers_1() -> Vec<Value> {
    vec![
        // A bat hangs (still, snapped to the block above it, one animation) or flies (the other); a
        // vex reads its charging flag; an allay's arms rise to what it holds, and its dance spins
        // and sways on a cycle of fifty-five ticks.
        // A parrot beats its wings faster in the air (a rider or one on the ground slows them) and
        // sinks slowly while it falls there; its variant is read from its data.
        living("parrot-on-the-ground", "parrot", 30, vec![on_ground(1, true)]).extras(&["parrot"]).row(),
        living("parrot-flies", "parrot", 40, vec![on_ground(1, false)]).extras(&["parrot"]).row(),
        living("parrot-lands", "parrot", 40, vec![on_ground(1, false), on_ground(15, true)]).extras(&["parrot"]).row(),
        living("parrot-falls", "parrot", 30, vec![on_ground(1, false), json!({"tick": 3, "type": "entity_motion", "x": f64_bits(0.0), "y": f64_bits(-0.5), "z": f64_bits(0.0)})]).extras(&["parrot"]).row(),
        living("parrot-variant", "parrot", 10, vec![json!({"tick": 2, "type": "flier_data", "parrot_variant": 3})]).extras(&["parrot"]).row(),
    ]
}

fn flyers_2() -> Vec<Value> {
    vec![
        // A breeze runs its idle animation from its first tick; its pose starts the shooting,
        // inhaling and sliding ones (stopping the idle, inhaling, jumping and shooting ones), a
        // long jump runs on its own tick, and ending a slide starts the slide back.
        living("breeze-idles", "breeze", 30, vec![]).extras(&["breeze"]).row(),
        living(
            "breeze-shoots",
            "breeze",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "SHOOTING"}),
                json!({"tick": 20, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
        living(
            "breeze-inhales",
            "breeze",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "INHALING"}),
                json!({"tick": 15, "type": "frog_data", "pose": "SHOOTING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
        living("breeze-slides", "breeze", 40, vec![json!({"tick": 5, "type": "frog_data", "pose": "SLIDING"})])
            .extras(&["breeze"])
            .row(),
        living(
            "breeze-slides-and-stops",
            "breeze",
            50,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "SLIDING"}),
                json!({"tick": 20, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
    ]
}

fn flyers_3() -> Vec<Value> {
    vec![
        living(
            "breeze-slides-again",
            "breeze",
            60,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "SLIDING"}),
                json!({"tick": 20, "type": "frog_data", "pose": "STANDING"}),
                json!({"tick": 30, "type": "frog_data", "pose": "SLIDING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
        living(
            "breeze-jumps",
            "breeze",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "LONG_JUMPING"}),
                json!({"tick": 18, "type": "frog_data", "pose": "STANDING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
        living(
            "breeze-shoots-again",
            "breeze",
            40,
            vec![
                json!({"tick": 5, "type": "frog_data", "pose": "SHOOTING"}),
                json!({"tick": 12, "type": "frog_data", "pose": "SHOOTING"}),
            ],
        )
        .extras(&["breeze"])
        .row(),
        // A wither's side heads turn after the entities its data names (at most 40 degrees up or down
        // and 10 sideways a tick) or after its body; it settles its upward motion and turns to face
        // fast horizontal motion; the invulnerable ticks come from the data.
        living("wither-idle", "wither", 20, vec![]).extras(&["wither"]).player_x(30.5).row(),
        living(
            "wither-looks-at-a-target",
            "wither",
            40,
            vec![json!({"tick": 2, "type": "wither_data", "target_b": 41, "target_c": 41, "invulnerable": 150})],
        )
        .extras(&["wither"])
        .player_x(30.5)
        .beside("pig", 11.5, 66.0, 5.5)
        .row(),
    ]
}

fn flyers_4() -> Vec<Value> {
    vec![
        living("wither-loses-its-target", "wither", 40, vec![json!({"tick": 2, "type": "wither_data", "target_b": 41}), json!({"tick": 14, "type": "wither_data", "target_b": 0})]).extras(&["wither"]).player_x(30.5).beside("pig", -9.5, 60.0, -7.5).row(),
        living("wither-target-above", "wither", 30, vec![json!({"tick": 2, "type": "wither_data", "target_c": 41})]).extras(&["wither"]).player_x(30.5).beside("pig", 0.5, 74.0, 0.5).row(),
        living("wither-turns-with-its-motion", "wither", 30, vec![json!({"tick": 3, "type": "entity_motion", "x": f64_bits(0.4), "y": f64_bits(0.5), "z": f64_bits(-0.3)})]).extras(&["wither"]).player_x(30.5).row(),
        living("wither-slow-motion", "wither", 30, vec![json!({"tick": 3, "type": "entity_motion", "x": f64_bits(0.1), "y": f64_bits(0.5), "z": f64_bits(0.1)})]).extras(&["wither"]).player_x(30.5).row(),
        // An ender dragon beats its wings faster the slower it moves, steadily while its phase has
        // it sit (the phase it starts in), and records its height and turn each tick into its flight
        // history (sixty-four samples) before its position eases to the last packet's; a dying
        // one does neither.
        // A minecart's hit and damage run down a tick at a time; the block it carries is the one the
        // server named or its type's own (a furnace's lit while it has fuel), lowered by the offset
        // the server said or the type's own; its position and turn ease to the last packet's over three
        // ticks (a move, a move with a turn, a teleport), else its turns are kept under a whole turn;
        // and the rail under a place (and a little ahead and behind it) puts it where the rail leads.
        living("minecart-idle", "minecart", 20, vec![]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
    ]
}

fn flyers_5() -> Vec<Value> {
    vec![
        living(
            "minecart-hit",
            "minecart",
            20,
            vec![json!({"tick": 2, "type": "minecart_data", "hurt": 10, "hurt_dir": -1, "damage": 25.5})],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
        living("minecart-chest", "chest_minecart", 6, vec![])
            .extras(&["minecart"])
            .player_x(30.5)
            .set("chunk_radius", json!(1))
            .row(),
        living("minecart-hopper", "hopper_minecart", 6, vec![])
            .extras(&["minecart"])
            .player_x(30.5)
            .set("chunk_radius", json!(1))
            .row(),
        living("minecart-tnt", "tnt_minecart", 6, vec![])
            .extras(&["minecart"])
            .player_x(30.5)
            .set("chunk_radius", json!(1))
            .row(),
        living(
            "minecart-furnace",
            "furnace_minecart",
            12,
            vec![
                json!({"tick": 4, "type": "minecart_data", "fuel": true}),
                json!({"tick": 8, "type": "minecart_data", "fuel": false}),
            ],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
    ]
}

fn flyers_6() -> Vec<Value> {
    vec![
        living("minecart-spawner-command", "spawner_minecart", 6, vec![])
            .extras(&["minecart"])
            .player_x(30.5)
            .set("chunk_radius", json!(1))
            .row(),
        living(
            "minecart-tnt-is-lit",
            "tnt_minecart",
            100,
            vec![json!({"tick": 3, "type": "entity_event", "event": 70})],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
        living(
            "minecart-tnt-is-lit-twice",
            "tnt_minecart",
            40,
            vec![
                json!({"tick": 3, "type": "entity_event", "event": 70}),
                json!({"tick": 20, "type": "entity_event", "event": 70}),
            ],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
        living(
            "minecart-names-its-block",
            "minecart",
            12,
            vec![
                json!({"tick": 2, "type": "minecart_data", "display": 1, "offset": 3}),
                json!({"tick": 7, "type": "minecart_data", "no_display": true}),
            ],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
        living(
            "minecart-chest-offset-and-block",
            "chest_minecart",
            8,
            vec![json!({"tick": 2, "type": "minecart_data", "display": 1})],
        )
        .extras(&["minecart"])
        .player_x(30.5)
        .set("chunk_radius", json!(1))
        .row(),
    ]
}

fn flyers_7() -> Vec<Value> {
    vec![
        living("minecart-moves", "minecart", 24, vec![json!({"tick": 2, "type": "move", "xa": 4096, "ya": 0, "za": 0}), json!({"tick": 8, "type": "move", "xa": 1000, "ya": 2000, "za": -3000}), json!({"tick": 9, "type": "move", "xa": 500, "ya": 0, "za": 500})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("minecart-moves-and-turns", "minecart", 30, vec![json!({"tick": 2, "type": "move_rot", "xa": 4096, "ya": 0, "za": 0, "y_rot": 64, "x_rot": 0}), json!({"tick": 8, "type": "move_rot", "xa": 1000, "ya": 0, "za": -3000, "y_rot": 200, "x_rot": 10}), json!({"tick": 14, "type": "move_rot", "xa": 0, "ya": 0, "za": 0, "y_rot": 250, "x_rot": -10})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("minecart-is-teleported", "minecart", 12, vec![json!({"tick": 2, "type": "sync", "x": f64_bits(5.5), "y": f64_bits(65.0), "z": f64_bits(-2.5), "motion_x": f64_bits(0.0), "motion_y": f64_bits(0.0), "motion_z": f64_bits(0.0)})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("minecart-rails", "minecart", 4, vec![]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("blocks", json!(cat![(-2_i64..6).flat_map(|x| (-2_i64..4).map(move |z| (x, z))).map(|(x, z)| json!({"x": x, "y": 63, "z": z, "block": "minecraft:stone"})).collect::<Vec<_>>(), vec![json!({"x": 0, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "east_west"}}), json!({"x": 1, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_east"}}), json!({"x": 2, "y": 65, "z": 0, "block": "minecraft:powered_rail", "properties": {"shape": "east_west", "powered": "true"}}), json!({"x": 0, "y": 64, "z": 1, "block": "minecraft:detector_rail", "properties": {"shape": "north_south"}}), json!({"x": 0, "y": 64, "z": 2, "block": "minecraft:activator_rail", "properties": {"shape": "ascending_south"}}), json!({"x": 3, "y": 64, "z": 1, "block": "minecraft:rail", "properties": {"shape": "south_east"}}), json!({"x": 4, "y": 64, "z": 1, "block": "minecraft:rail", "properties": {"shape": "north_west"}}), json!({"x": 3, "y": 64, "z": 2, "block": "minecraft:rail", "properties": {"shape": "north_east"}}), json!({"x": 4, "y": 64, "z": 2, "block": "minecraft:rail", "properties": {"shape": "south_west"}}), json!({"x": 4, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_west"}}), json!({"x": 5, "y": 64, "z": 0, "block": "minecraft:rail", "properties": {"shape": "ascending_north"}})]])).set("rail_probes", json!([(0.5, 64.0, 0.5), (0.2, 64.1, 0.5), (0.9, 64.0, 0.5), (1.5, 64.2, 0.5), (1.1, 64.0, 0.5), (2.5, 65.0, 0.5), (0.5, 64.0, 1.5), (0.5, 64.0, 2.2), (0.5, 64.0, 2.9), (3.4, 64.0, 1.6), (4.6, 64.0, 1.4), (3.2, 64.0, 2.8), (4.1, 64.0, 2.2), (4.5, 64.0, 0.5), (5.5, 64.0, 0.4), (0.5, 65.0, 0.5), (-1.5, 64.0, 0.5), (0.5, 66.5, 0.5), (3.5, 65.0, 1.5)].into_iter().map(|(x, y, z)| vec![f64_bits(x), f64_bits(y), f64_bits(z)]).collect::<Vec<_>>())).row(),
        // A display entity keeps nothing until some data arrives; then its render state is made from the
        // data (the transformation of the four parts, billboard, brightness, shadow, glow, and the
        // block or item it shows). While the interpolation duration is not zero a change eases from where
        // the last had got to (the turns by the shorter arc, linear where they are the same), over the
        // duration from the start tick (the delay after it was sent); the position and the turn ease over
        // the teleport duration, and are put there at once while it is zero.
        living("display-quiet", "block_display", 6, vec![]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
    ]
}

fn flyers_8() -> Vec<Value> {
    vec![
        living("display-block-static", "block_display", 8, vec![display_data(2, json!({"block_state": 1, "translation": [0.5, 0.25, -1.0], "scale": [2.0, 1.0, 0.5], "left_rotation": [0.0, 0.25881904, 0.0, 0.9659258], "right_rotation": [0.0, 0.0, 0.25881904, 0.9659258], "billboard": 3, "brightness": 10486000, "glow": 16746496, "shadow_radius": 0.75, "shadow_strength": 0.5, "view_range": 2.0, "width": 1.5, "height": 2.0}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("display-eases", "block_display", 30, vec![display_data(2, json!({"block_state": 1, "translation": [0.0, 0.0, 0.0], "scale": [1.0, 1.0, 1.0], "shadow_radius": 0.0})), display_data(6, json!({"duration": 10, "start_delta": 0, "translation": [1.0, 2.0, 3.0], "scale": [2.0, 2.0, 2.0], "left_rotation": [0.0, 0.7071068, 0.0, 0.7071068], "shadow_radius": 1.0})), display_data(13, json!({"duration": 8, "start_delta": 2, "translation": [-1.0, 0.0, 0.5], "shadow_radius": 0.25, "shadow_strength": 0.5})), display_data(24, json!({"duration": 0, "translation": [0.0, 0.0, 0.0]}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("display-turns-the-other-way", "block_display", 20, vec![display_data(2, json!({"block_state": 1, "left_rotation": [0.0, 0.0, 0.0, 1.0]})), display_data(5, json!({"duration": 6, "start_delta": 0, "left_rotation": [0.0, 0.8660254, 0.0, -0.5], "right_rotation": [0.0, 0.0, 0.0, 1.0]}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("display-moves", "block_display", 30, vec![display_data(2, json!({"block_state": 1})), json!({"tick": 3, "type": "sync", "x": f64_bits(5.5), "y": f64_bits(65.0), "z": f64_bits(-2.5), "motion_x": f64_bits(0.0), "motion_y": f64_bits(0.0), "motion_z": f64_bits(0.0)}), display_data(6, json!({"pos_rot": 6})), json!({"tick": 8, "type": "sync", "x": f64_bits(10.5), "y": f64_bits(66.0), "z": f64_bits(4.5), "motion_x": f64_bits(0.0), "motion_y": f64_bits(0.0), "motion_z": f64_bits(0.0)}), json!({"tick": 12, "type": "move_rot", "xa": 4096, "ya": 0, "za": 0, "y_rot": 64, "x_rot": 0})]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        // A text display's text, line width, background, opacity and flags make its render state; the
        // opacity eases in whole numbers and the background by channels, over the same duration.
        living("display-text-static", "text_display", 8, vec![display_data(2, json!({"text": "Hello\nworld", "line_width": 120, "background": 1090486272, "text_opacity": -128, "style_flags": 13}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
    ]
}

fn flyers_9() -> Vec<Value> {
    vec![
        living("display-text-eases", "text_display", 24, vec![display_data(2, json!({"text": "a", "background": -2146426832, "text_opacity": 100})), display_data(5, json!({"duration": 10, "start_delta": 0, "background": 16777215, "text_opacity": -1, "line_width": 40})), display_data(9, json!({"duration": 6, "start_delta": 0, "text_opacity": 20}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        living("display-item", "item_display", 14, vec![display_data(2, json!({"item": "minecraft:diamond_sword", "item_display": 8})), display_data(8, json!({"item_display": 5}))]).extras(&["display"]).player_x(30.5).set("chunk_radius", json!(1)).row(),
        // A minecart of the new behaviour (the server enabled `minecart_improvements`) is not eased to
        // the place a packet gives: a move or a teleport puts it there at once, and it plays back the
        // steps the server sent along the track over three ticks, weighted by how far each went
        // (a batch that arrives while one plays waits; one of no weight is played at once, and its
        // answer is kept for the next as the reference keeps it), turning by the short way round.
        living("minecart-new-idle", "minecart", 6, vec![]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).row(),
        living("minecart-new-steps", "minecart", 24, vec![json!({"tick": 2, "type": "minecart_steps", "steps": [cart_step((0.75, 64.1, 0.5), (0.1, 0.0, 0.0), (64, 0), 0.25), cart_step((1.0, 64.1, 0.5), (0.2, 0.0, 0.0), (64, 0), 0.5), cart_step((1.5, 64.6, 0.5), (0.3, 0.1, 0.0), (70, -32), 0.5)]})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).row(),
        living("minecart-new-batches", "minecart", 30, vec![json!({"tick": 1, "type": "minecart_steps", "steps": [cart_step((1.0, 64.1, 0.5), (0.25, 0.0, 0.0), (64, 0), 1.0)]}), json!({"tick": 2, "type": "minecart_steps", "steps": [cart_step((1.5, 64.1, 0.5), (0.25, 0.0, 0.0), (120, 10), 0.4)]}), json!({"tick": 3, "type": "minecart_steps", "steps": [cart_step((2.0, 64.1, 0.5), (0.25, 0.0, 0.0), (-120, 100), 0.6)]}), json!({"tick": 12, "type": "minecart_steps", "steps": [cart_step((2.0, 64.1, 0.5), (0.0, 0.0, 0.0), (-120, 100), 1.0)]})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).row(),
    ]
}

fn flyers_10() -> Vec<Value> {
    vec![
        living("minecart-new-no-weight", "minecart", 14, vec![json!({"tick": 2, "type": "minecart_steps", "steps": [cart_step((2.5, 65.1, 0.5), (0.0, 0.0, 0.0), (32, 0), 0.0)]}), json!({"tick": 3, "type": "minecart_steps", "steps": [cart_step((3.5, 65.1, 1.5), (0.0, 0.0, 0.0), (96, 8), 0.0)]}), json!({"tick": 5, "type": "minecart_steps", "steps": [cart_step((4.5, 65.1, 1.5), (0.0, 0.0, 0.0), (96, 8), 0.0), cart_step((5.5, 65.1, 1.5), (0.0, 0.0, 0.0), (100, 8), 0.0)]})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).row(),
        // No frame is drawn between the ticks (the cart is out of sight): the place the last tick asked for
        // at no weight is the one the next tick gets.
        living("minecart-new-no-weight-unseen", "minecart", 14, vec![json!({"tick": 2, "type": "minecart_steps", "steps": [cart_step((2.5, 65.1, 0.5), (0.0, 0.0, 0.0), (32, 0), 0.0)]}), json!({"tick": 3, "type": "minecart_steps", "steps": [cart_step((3.5, 65.1, 1.5), (0.0, 0.0, 0.0), (96, 8), 0.0)]}), json!({"tick": 4, "type": "minecart_steps", "steps": [cart_step((4.5, 65.1, 1.5), (0.0, 0.0, 0.0), (96, 8), 0.0)]}), json!({"tick": 6, "type": "minecart_steps", "steps": [cart_step((5.5, 65.1, 1.5), (0.0, 0.0, 0.0), (96, 8), 1.0)]})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).set("sample_from", json!(5)).row(),
        living("minecart-new-is-moved", "minecart", 24, vec![json!({"tick": 2, "type": "move", "xa": 4096, "ya": 0, "za": 0}), json!({"tick": 4, "type": "move_rot", "xa": 1000, "ya": 2000, "za": -3000, "y_rot": 200, "x_rot": 10}), json!({"tick": 8, "type": "move_rot", "xa": 0, "ya": 0, "za": 0, "y_rot": 250, "x_rot": -100}), json!({"tick": 12, "type": "sync", "x": f64_bits(5.5), "y": f64_bits(65.0), "z": f64_bits(-2.5), "motion_x": f64_bits(0.0), "motion_y": f64_bits(0.0), "motion_z": f64_bits(0.0)}), json!({"tick": 14, "type": "entity_motion", "x": f64_bits(0.4), "y": f64_bits(0.0), "z": f64_bits(-0.3)})]).extras(&["minecart"]).player_x(30.5).set("chunk_radius", json!(1)).set("minecart_improvements", json!(true)).row(),
        living("dragon-hovers", "ender_dragon", 30, vec![]).extras(&["ender_dragon"]).player_x(30.5).row(),
        living("dragon-flies", "ender_dragon", 50, vec![json!({"tick": 2, "type": "dragon_data", "phase": 0}), json!({"tick": 3, "type": "entity_motion", "x": f64_bits(0.3), "y": f64_bits(0.2), "z": f64_bits(-0.4)})]).extras(&["ender_dragon"]).player_x(30.5).row(),
    ]
}

fn flyers_11() -> Vec<Value> {
    vec![
        living("dragon-flies-and-turns", "ender_dragon", 90, cat![vec![json!({"tick": 2, "type": "dragon_data", "phase": 1}), json!({"tick": 3, "type": "entity_motion", "x": f64_bits(1.1), "y": f64_bits(-0.1), "z": f64_bits(0.2)})], (5_i64..80).step_by(3).map(|t| json!({"tick": t, "type": "move_rot", "xa": 300, "ya": if t < 40 { 40 } else { -60 }, "za": 100, "y_rot": (t * 3).rem_euclid(256), "x_rot": 0})).collect::<Vec<_>>()]).extras(&["ender_dragon"]).player_x(30.5).row(),
        living("dragon-lands", "ender_dragon", 40, vec![json!({"tick": 2, "type": "dragon_data", "phase": 3})]).extras(&["ender_dragon"]).player_x(30.5).row(),
        living("dragon-dies-hovering", "ender_dragon", 40, vec![json!({"tick": 6, "type": "death"})]).extras(&["ender_dragon"]).player_x(30.5).row(),
        living("dragon-dies", "ender_dragon", 60, vec![json!({"tick": 2, "type": "dragon_data", "phase": 0}), json!({"tick": 3, "type": "entity_motion", "x": f64_bits(0.3), "y": f64_bits(0.2), "z": f64_bits(-0.4)}), json!({"tick": 20, "type": "death"})]).extras(&["ender_dragon"]).player_x(30.5).row(),
        living("bat-flies", "bat", 30, vec![]).extras(&["bat"]).row(),
    ]
}

fn flyers_12() -> Vec<Value> {
    vec![
        living("bat-hangs", "bat", 30, vec![json!({"tick": 2, "type": "flier_data", "bat_flags": 1})])
            .extras(&["bat"])
            .row(),
        living(
            "bat-wakes",
            "bat",
            40,
            vec![
                json!({"tick": 2, "type": "flier_data", "bat_flags": 1}),
                json!({"tick": 20, "type": "flier_data", "bat_flags": 0}),
            ],
        )
        .extras(&["bat"])
        .row(),
        living("vex-charges", "vex", 10, vec![json!({"tick": 2, "type": "flier_data", "vex_flags": 1})])
            .extras(&["vex"])
            .row(),
        living("allay-hovers", "allay", 20, vec![]).extras(&["allay"]).row(),
        living(
            "allay-takes-an-item",
            "allay",
            30,
            vec![json!({"tick": 3, "type": "equipment", "item": "minecraft:bamboo"})],
        )
        .extras(&["allay"])
        .row(),
    ]
}

fn flyers_13() -> Vec<Value> {
    vec![
        living(
            "allay-lets-go",
            "allay",
            30,
            vec![
                json!({"tick": 2, "type": "equipment", "item": "minecraft:bamboo"}),
                json!({"tick": 12, "type": "equipment", "item": null}),
            ],
        )
        .extras(&["allay"])
        .row(),
        living("allay-dances", "allay", 130, vec![json!({"tick": 3, "type": "flier_data", "allay_dancing": true})])
            .extras(&["allay"])
            .row(),
        living(
            "allay-stops-dancing",
            "allay",
            100,
            vec![
                json!({"tick": 3, "type": "flier_data", "allay_dancing": true}),
                json!({"tick": 40, "type": "flier_data", "allay_dancing": false}),
            ],
        )
        .extras(&["allay"])
        .row(),
        living("bee-quiet", "bee", 6, vec![]).extras(&["bee"]).row(),
        living("bee-rolls", "bee", 20, vec![json!({"tick": 2, "type": "bee_data", "flags": 2})]).extras(&["bee"]).row(),
    ]
}

fn flyers_14() -> Vec<Value> {
    vec![
        living(
            "bee-rolls-then-stops",
            "bee",
            30,
            vec![
                json!({"tick": 1, "type": "bee_data", "flags": 2}),
                json!({"tick": 9, "type": "bee_data", "flags": 0}),
            ],
        )
        .extras(&["bee"])
        .row(),
        // A bee with nectar drops nectar particles at random, which the headless client cannot draw.
        living("bee-stung", "bee", 6, vec![json!({"tick": 1, "type": "bee_data", "flags": 4})]).extras(&["bee"]).row(),
        living("bee-angry", "bee", 12, vec![json!({"tick": 1, "type": "bee_data", "anger_end": 5})])
            .extras(&["bee"])
            .row(),
        living("bee-angry-for-a-long-time", "bee", 12, vec![json!({"tick": 1, "type": "bee_data", "anger_end": 500})])
            .extras(&["bee"])
            .row(),
    ]
}

fn flyers_15() -> Vec<Value> {
    // The idle spin starts 200 to 240 ticks after the wait is drawn (from a source the reference
    // does not seed: fixed), plays for 3.5 seconds and, ten ticks after it started, draws again.
    [4, 100, 205, 215, 235, 255, 290, 330, 470, 700]
        .into_iter()
        .map(|ticks| {
            living(
                format!("copper-golem-idle-{ticks}"),
                "copper_golem",
                ticks,
                vec![json!({"tick": 0, "type": "copper_golem_idle", "seed": 5})],
            )
            .extras(&["copper_golem"])
            .row()
        })
        .collect::<Vec<_>>()
}

pub fn rows() -> Vec<Value> {
    [
        flyers_1(),
        flyers_2(),
        flyers_3(),
        flyers_4(),
        flyers_5(),
        flyers_6(),
        flyers_7(),
        flyers_8(),
        flyers_9(),
        flyers_10(),
        flyers_11(),
        flyers_12(),
        flyers_13(),
        flyers_14(),
        flyers_15(),
    ]
    .concat()
}
