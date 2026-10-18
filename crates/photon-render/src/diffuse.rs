//! The level's two diffuse lights (`Lighting.updateLevel`): the first points the
//! same way in every dimension, the second points up in the default dimension and
//! down in the Nether. The reference keeps them in one shared buffer the shaders
//! read; the vertices lit on the CPU read them from here, set once a frame.

use photon_world::CardinalLightType;
use std::sync::atomic::{AtomicBool, Ordering};

static NETHER: AtomicBool = AtomicBool::new(false);

/// Choose the lights for the frames that follow, from the dimension type.
pub fn set_level(kind: CardinalLightType) {
    NETHER.store(kind == CardinalLightType::Nether, Ordering::Relaxed);
}

/// The level's two light directions, before they are normalised.
pub fn level_directions() -> [[f32; 3]; 2] {
    directions(NETHER.load(Ordering::Relaxed))
}

fn directions(nether: bool) -> [[f32; 3]; 2] {
    [
        [0.2, 1.0, -0.7],
        [-0.2, if nether { -1.0 } else { 1.0 }, 0.7],
    ]
}

#[cfg(test)]
mod tests {
    use super::directions;

    #[test]
    fn the_nether_turns_the_second_light_downwards() {
        assert_eq!(directions(false)[1], [-0.2, 1.0, 0.7]);
        assert_eq!(directions(true)[1], [-0.2, -1.0, 0.7]);
        assert_eq!(directions(true)[0], directions(false)[0]);
    }
}
