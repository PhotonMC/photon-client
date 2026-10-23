//! Trap doors beside blocks that give a redstone signal.

use super::*;

type Neighbour = (String, &'static str, Vec<(&'static str, &'static str)>);

/// Neighbours that may power a place, each with the states worth telling apart.
fn signal_neighbours() -> Vec<Neighbour> {
    let mut list: Vec<Neighbour> = Vec::new();
    list.push(("redstone-block".into(), "redstone_block", vec![]));
    list.push(("redstone-wire-0".into(), "redstone_wire", vec![("power", "0")]));
    for powered in ["true", "false"] {
        for (face, facing) in
            [("floor", "east"), ("ceiling", "west"), ("wall", "east"), ("wall", "west"), ("wall", "north")]
        {
            let state = vec![("face", face), ("facing", facing), ("powered", powered)];
            list.push((format!("lever-{face}-{facing}-{powered}"), "lever", state.clone()));
            list.push((format!("button-{face}-{facing}-{powered}"), "stone_button", state));
        }
        list.push((format!("plate-{powered}"), "stone_pressure_plate", vec![("powered", powered)]));
        for facing in ["north", "east", "south", "west", "up", "down"] {
            list.push((
                format!("observer-{facing}-{powered}"),
                "observer",
                vec![("facing", facing), ("powered", powered)],
            ));
        }
        for facing in ["north", "east", "south", "west"] {
            let state = vec![("facing", facing), ("powered", powered)];
            list.push((format!("repeater-{facing}-{powered}"), "repeater", state.clone()));
            // A powered comparator gives what its block entity holds, which the rules leave out.
            if powered == "false" {
                list.push((format!("comparator-{facing}-{powered}"), "comparator", state));
            }
        }
    }
    for lit in ["true", "false"] {
        list.push((format!("torch-{lit}"), "redstone_torch", vec![("lit", lit)]));
        for facing in ["north", "east", "south", "west"] {
            list.push((
                format!("wall-torch-{facing}-{lit}"),
                "redstone_wall_torch",
                vec![("facing", facing), ("lit", lit)],
            ));
        }
    }
    for power in ["0", "7", "15"] {
        list.push((format!("weighted-plate-{power}"), "light_weighted_pressure_plate", vec![("power", power)]));
    }
    list
}

/// Trap doors beside blocks that give a signal, straight or through a block that carries it.
pub(super) fn signal_rows() -> Vec<Value> {
    let trapdoors = ["oak_trapdoor", "iron_trapdoor", "copper_trapdoor"];
    let ground = block([0, 63, 0], "stone", &[]);
    let mut rows = Vec::new();
    for (label, name, properties) in signal_neighbours() {
        for (side, at) in [("east", [1, 64, 0]), ("above", [0, 65, 0]), ("north", [0, 64, -1])] {
            let world = vec![ground.clone(), block(at, name, &properties)];
            rows.push(row(&format!("{label}-{side}"), &trapdoors, world, &ON_TOP));
        }
        // Through a solid block beside the place, with the source beyond, under or over it.
        for (side, source) in [("far", [2, 64, 0]), ("under", [1, 63, 0]), ("over", [1, 65, 0])] {
            let world = vec![ground.clone(), block([1, 64, 0], "stone", &[]), block(source, name, &properties)];
            rows.push(row(&format!("{label}-through-{side}"), &trapdoors, world, &ON_TOP));
        }
    }
    rows
}
