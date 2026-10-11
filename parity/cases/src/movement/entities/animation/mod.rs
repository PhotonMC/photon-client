//! Entity animation scenarios, by the kind of entity.

mod animals;
mod aquatic;
mod bosses;
mod cubes;
mod culling;
mod farm;
mod flyers;
mod objects;
mod pets;
mod swimmers;

use serde_json::Value;

pub fn rows() -> Vec<Value> {
    [
        animals::rows(),
        pets::rows(),
        farm::rows(),
        aquatic::rows(),
        swimmers::rows(),
        bosses::rows(),
        cubes::rows(),
        culling::rows(),
        flyers::rows(),
        objects::rows(),
    ]
    .concat()
}
