//! What a minecart does of its own on the client: the rocking of a hit running
//! out (`AbstractMinecart.tick`), the block it carries and how far down, and where
//! the rail under it puts it (`OldMinecartBehavior.getPos`, `getPosOffs`), which its
//! renderer draws it from.

/// `VehicleEntity`'s data ids (`Entity` has eight), then `AbstractMinecart`'s two and the furnace's.
pub const HURT_DATA_ID: u8 = 8;
pub const HURT_DIR_DATA_ID: u8 = 9;
pub const DAMAGE_DATA_ID: u8 = 10;
pub const CUSTOM_DISPLAY_BLOCK_DATA_ID: u8 = 11;
pub const DISPLAY_OFFSET_DATA_ID: u8 = 12;
pub const FUEL_DATA_ID: u8 = 13;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinecartState {
    /// `VehicleEntity.getHurtTime`, `getHurtDir` and `getDamage`.
    pub hurt_time: i32,
    pub hurt_dir: i32,
    pub damage: f32,
    /// `AbstractMinecart.getCustomDisplayBlockState`: the block state the server named, if it did.
    pub(crate) custom_display: Option<u32>,
    /// `AbstractMinecart.getDisplayOffset`, when the server has said (else the type's own).
    pub(crate) display_offset: Option<i32>,
    /// `MinecartFurnace.hasFuel`.
    pub(crate) fuel: bool,
}

// The damage is a sum of finite steps: never NaN.
impl Eq for MinecartState {}

impl Default for MinecartState {
    fn default() -> Self {
        Self {
            hurt_time: 0,
            hurt_dir: 1,
            damage: 0.0,
            custom_display: None,
            display_offset: None,
            fuel: false,
        }
    }
}

impl MinecartState {
    /// `AbstractMinecart.tick`'s start: the hit's time and damage run down.
    pub(crate) fn tick(&mut self) {
        if self.hurt_time > 0 {
            self.hurt_time -= 1;
        }
        if self.damage > 0.0 {
            self.damage -= 1.0;
        }
    }

    /// `AbstractMinecart.getDisplayOffset`: what the server said, else the type's default.
    pub fn display_offset(&self, default: i32) -> i32 {
        self.display_offset.unwrap_or(default)
    }
}

/// The types of minecart and the offset (in sixteenths of a block) their block is lowered by.
pub fn default_display_offset(type_name: &str) -> Option<i32> {
    match type_name {
        "minecraft:minecart"
        | "minecraft:furnace_minecart"
        | "minecraft:tnt_minecart"
        | "minecraft:spawner_minecart"
        | "minecraft:command_block_minecart" => Some(6),
        "minecraft:chest_minecart" => Some(8),
        "minecraft:hopper_minecart" => Some(1),
        _ => None,
    }
}

/// `AbstractMinecart.getDefaultDisplayBlockState`: the block a minecart of this type shows when
/// the server has named none (the furnace's is lit while it has fuel), or none for the plain one.
pub fn default_display_block(type_name: &str, fuel: bool) -> Option<u32> {
    use std::collections::BTreeMap;
    let block = match type_name {
        "minecraft:chest_minecart" => "minecraft:chest",
        "minecraft:furnace_minecart" => "minecraft:furnace",
        "minecraft:hopper_minecart" => "minecraft:hopper",
        "minecraft:tnt_minecart" => "minecraft:tnt",
        "minecraft:spawner_minecart" => "minecraft:spawner",
        "minecraft:command_block_minecart" => "minecraft:command_block",
        _ => return None,
    };
    let block = voxel_data::block::registry::block(block)?;
    let mut chosen = BTreeMap::new();
    match type_name {
        "minecraft:chest_minecart" | "minecraft:furnace_minecart" => {
            chosen.insert("facing".to_owned(), "north".to_owned());
        }
        _ => {}
    }
    if type_name == "minecraft:furnace_minecart" {
        chosen.insert("lit".to_owned(), fuel.to_string());
    }
    block.state_id(&chosen)
}

/// `RailShape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

impl RailShape {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "north_south" => Self::NorthSouth,
            "east_west" => Self::EastWest,
            "ascending_east" => Self::AscendingEast,
            "ascending_west" => Self::AscendingWest,
            "ascending_north" => Self::AscendingNorth,
            "ascending_south" => Self::AscendingSouth,
            "south_east" => Self::SouthEast,
            "south_west" => Self::SouthWest,
            "north_west" => Self::NorthWest,
            "north_east" => Self::NorthEast,
            _ => return None,
        })
    }

    /// `RailShape.isSlope`.
    pub fn is_slope(self) -> bool {
        matches!(
            self,
            Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth
        )
    }

    /// `AbstractMinecart.exits`: the two sides of the block the rail leaves by, as the
    /// blocks beyond them (`(0, -1, 0)` further down for the low end of a slope).
    pub fn exits(self) -> ([i32; 3], [i32; 3]) {
        const X_NEG: [i32; 3] = [-1, 0, 0];
        const X_POS: [i32; 3] = [1, 0, 0];
        const Z_NEG: [i32; 3] = [0, 0, -1];
        const Z_POS: [i32; 3] = [0, 0, 1];
        const X_NEG_BELOW: [i32; 3] = [-1, -1, 0];
        const X_POS_BELOW: [i32; 3] = [1, -1, 0];
        const Z_NEG_BELOW: [i32; 3] = [0, -1, -1];
        const Z_POS_BELOW: [i32; 3] = [0, -1, 1];
        match self {
            Self::NorthSouth => (Z_NEG, Z_POS),
            Self::EastWest => (X_NEG, X_POS),
            Self::AscendingEast => (X_NEG_BELOW, X_POS),
            Self::AscendingWest => (X_NEG, X_POS_BELOW),
            Self::AscendingNorth => (Z_NEG, Z_POS_BELOW),
            Self::AscendingSouth => (Z_NEG_BELOW, Z_POS),
            Self::SouthEast => (Z_POS, X_POS),
            Self::SouthWest => (Z_POS, X_NEG),
            Self::NorthWest => (Z_NEG, X_NEG),
            Self::NorthEast => (Z_NEG, X_POS),
        }
    }
}

/// The shape of the rail a block state is, if it is one (`BaseRailBlock.isRail`).
pub fn rail_shape(state: i32) -> Option<RailShape> {
    let info = crate::block::state_or_air(i64::from(state));
    let block = info.block();
    if !matches!(
        block.name,
        "minecraft:rail"
            | "minecraft:powered_rail"
            | "minecraft:detector_rail"
            | "minecraft:activator_rail"
    ) {
        return None;
    }
    block
        .properties_at(u32::from(info.offset))?
        .into_iter()
        .find(|(name, _)| *name == "shape")
        .and_then(|(_, value)| RailShape::from_name(value))
}

/// Whether a block state is a rail (`BlockTags.RAILS`).
fn is_rail(state: Option<i32>) -> bool {
    state.is_some_and(|state| rail_shape(state).is_some())
}

/// The rail block a position is on or just above: its coordinates and shape (the part
/// `getPos` and `getPosOffs` share).
fn rail_under(
    block_at: &dyn Fn(i32, i32, i32) -> Option<i32>,
    (x, y, z): (f64, f64, f64),
) -> ((i32, i32, i32), Option<RailShape>) {
    let xt = voxel_core::math::floor_f64(x);
    let mut yt = voxel_core::math::floor_f64(y);
    let zt = voxel_core::math::floor_f64(z);
    if is_rail(block_at(xt, yt - 1, zt)) {
        yt -= 1;
    }
    ((xt, yt, zt), block_at(xt, yt, zt).and_then(rail_shape))
}

/// `OldMinecartBehavior.getPos`: where the rail puts a minecart that is at this place, along
/// the rail from one exit to the other and as high as the slope there, if it is on a rail.
pub fn rail_pos(
    block_at: &dyn Fn(i32, i32, i32) -> Option<i32>,
    (x, y, z): (f64, f64, f64),
) -> Option<(f64, f64, f64)> {
    let ((xt, yt, zt), shape) = rail_under(block_at, (x, y, z));
    let (exit0, exit1) = shape?.exits();
    let (xt_f, yt_f, zt_f) = (f64::from(xt), f64::from(yt), f64::from(zt));
    let x0 = xt_f + 0.5 + f64::from(exit0[0]) * 0.5;
    let y0 = yt_f + 0.0625 + f64::from(exit0[1]) * 0.5;
    let z0 = zt_f + 0.5 + f64::from(exit0[2]) * 0.5;
    let x1 = xt_f + 0.5 + f64::from(exit1[0]) * 0.5;
    let y1 = yt_f + 0.0625 + f64::from(exit1[1]) * 0.5;
    let z1 = zt_f + 0.5 + f64::from(exit1[2]) * 0.5;
    let x_d = x1 - x0;
    let y_d = (y1 - y0) * 2.0;
    let z_d = z1 - z0;
    let progress = if x_d == 0.0 {
        z - zt_f
    } else if z_d == 0.0 {
        x - xt_f
    } else {
        let (xx, zz) = (x - x0, z - z0);
        (xx * x_d + zz * z_d) * 2.0
    };
    let mut y_out = y0 + y_d * progress;
    if y_d < 0.0 {
        y_out += 1.0;
    } else if y_d > 0.0 {
        y_out += 0.5;
    }
    Some((x0 + x_d * progress, y_out, z0 + z_d * progress))
}

/// `OldMinecartBehavior.getPosOffs`: [`rail_pos`] this far along the rail (either way) from here.
pub fn rail_pos_offset(
    block_at: &dyn Fn(i32, i32, i32) -> Option<i32>,
    (x, y, z): (f64, f64, f64),
    offset: f64,
) -> Option<(f64, f64, f64)> {
    let ((xt, yt, zt), shape) = rail_under(block_at, (x, y, z));
    let shape = shape?;
    let mut y = f64::from(yt);
    if shape.is_slope() {
        y = f64::from(yt + 1);
    }
    let (exit0, exit1) = shape.exits();
    let mut x_d = f64::from(exit1[0] - exit0[0]);
    let mut z_d = f64::from(exit1[2] - exit0[2]);
    let dd = (x_d * x_d + z_d * z_d).sqrt();
    x_d /= dd;
    z_d /= dd;
    let x = x + x_d * offset;
    let z = z + z_d * offset;
    let (floor_x, floor_z) = (
        voxel_core::math::floor_f64(x) - xt,
        voxel_core::math::floor_f64(z) - zt,
    );
    if exit0[1] != 0 && floor_x == exit0[0] && floor_z == exit0[2] {
        y += f64::from(exit0[1]);
    } else if exit1[1] != 0 && floor_x == exit1[0] && floor_z == exit1[2] {
        y += f64::from(exit1[1]);
    }
    rail_pos(block_at, (x, y, z))
}

#[cfg(test)]
mod tests {
    use crate::{DimensionBounds, EntityInput};
    use voxel_core::binary::Uuid;

    /// A teleport eases a minecart to its target over three ticks, as an entity with an
    /// interpolation handler does, instead of setting it at once.
    #[test]
    fn a_minecart_is_eased_to_a_teleport_over_three_ticks() {
        let mut world = crate::ClientWorld::new(DimensionBounds::default());
        let kind = voxel_data::entity_type::id("minecraft:minecart").expect("a minecart");
        assert!(world.apply_entity(EntityInput {
            entity_id: 5,
            uuid: Uuid::new(0, 5),
            type_id: kind,
            x: 0.5,
            y: 64.0,
            z: 0.5,
            movement: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            data: 0,
        }));
        world.entities.get_mut(&5).expect("the minecart").ticking = true;
        assert!(world.apply_entity_position_sync(5, (5.5, 65.0, -2.5), (0.0, 0.0)));
        // The target's chunk is not loaded here, which stops an entity ticking: tick it all the same.
        world.entities.get_mut(&5).expect("the minecart").ticking = true;
        let mut seen = Vec::new();
        for _ in 0..4 {
            world.tick_entities();
            seen.push(world.entity_at(5).expect("the minecart").position().0);
        }
        assert!((seen[0] - 2.1666666).abs() < 1e-6, "{seen:?}");
        assert!(
            (seen[2] - 5.5).abs() < 1e-9 && seen[3] == seen[2],
            "{seen:?}"
        );
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    /// Each type that carries a block names one: the furnace's lit by its fuel, the chest facing north.
    #[test]
    fn the_blocks_the_types_carry_are_named() {
        for name in [
            "minecraft:chest_minecart",
            "minecraft:furnace_minecart",
            "minecraft:hopper_minecart",
            "minecraft:tnt_minecart",
            "minecraft:spawner_minecart",
            "minecraft:command_block_minecart",
        ] {
            assert!(default_display_block(name, false).is_some(), "{name}");
        }
        assert_eq!(default_display_block("minecraft:minecart", false), None);
        assert_ne!(
            default_display_block("minecraft:furnace_minecart", false),
            default_display_block("minecraft:furnace_minecart", true)
        );
    }
}
