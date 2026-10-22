//! Which block states `BuiltInBlockModels` gives a model of their own, and what each is:
//! the one place that sorts a state into a special renderer, so the entities that draw a block
//! (minecarts, block displays, endermen) and the checks against the reference agree on it.

use photon_core::ids::BlockStateId;

use super::chest::{ChestKind, ChestMaterial};
use super::{copper_golem_statue, shulker_box};

/// The renderer a state's built-in model draws with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialModel {
    /// `EmptyBlockModel`: the air blocks.
    Empty,
    Bell,
    Banner {
        color: &'static str,
        wall: bool,
    },
    Shulker {
        color: Option<&'static str>,
    },
    /// A chest of the material it has without Christmas textures.
    Chest {
        material: ChestMaterial,
        kind: ChestKind,
    },
    Conduit,
    Statue {
        weathering: u8,
        pose: &'static str,
    },
    /// A head or skull of a mob, by the name the skull type gives it.
    Skull {
        kind: String,
    },
    PlayerHead,
    Pot,
    Table,
    /// The end gateway (`portal` is false) or the end portal (true).
    EndCube {
        portal: bool,
    },
    /// The petal blocks: a baked model that is picked by the display context.
    FlowerBed,
}

impl SpecialModel {
    /// The special model of a state, none for a state drawn from its baked model alone.
    pub fn of(state: BlockStateId) -> Option<Self> {
        let name = photon_world::block::state_or_air(state).block().name;
        let short = name.strip_prefix("minecraft:")?;
        Some(match short {
            "air" | "cave_air" | "void_air" => Self::Empty,
            "bell" => Self::Bell,
            "conduit" => Self::Conduit,
            "enchanting_table" => Self::Table,
            "decorated_pot" => Self::Pot,
            "end_gateway" => Self::EndCube { portal: false },
            "end_portal" => Self::EndCube { portal: true },
            "pink_petals" | "wildflowers" => Self::FlowerBed,
            "player_head" | "player_wall_head" => Self::PlayerHead,
            _ => Self::of_family(state, short)?,
        })
    }

    /// The models that come in families of colours, materials or kinds.
    fn of_family(state: BlockStateId, short: &str) -> Option<Self> {
        let property = |key| photon_world::block::rules::property(state, key);
        if short == "shulker_box" {
            return Some(Self::Shulker { color: None });
        }
        if let Some(dye) = short.strip_suffix("_shulker_box") {
            return Some(Self::Shulker {
                color: Some(dye_of(dye)?),
            });
        }
        if short.ends_with("copper_golem_statue") {
            let weathering = ["exposed", "weathered", "oxidized"]
                .iter()
                .position(|stage| short.contains(stage))
                .map_or(0, |index| index as u8 + 1);
            return Some(Self::Statue {
                weathering,
                pose: property("copper_golem_pose")?,
            });
        }
        Self::banner(short)
            .or_else(|| Self::skull(short))
            .or_else(|| Self::chest(state, short))
    }

    /// A banner, on the ground or on a wall.
    fn banner(short: &str) -> Option<Self> {
        let wall = short.ends_with("_wall_banner");
        let dye = short
            .strip_suffix("_wall_banner")
            .or_else(|| short.strip_suffix("_banner"))?;
        Some(Self::Banner {
            color: dye_of(dye)?,
            wall,
        })
    }

    /// A head or skull of one of the skull types.
    fn skull(short: &str) -> Option<Self> {
        let kind = short.replace("_wall", "");
        let kind = kind.trim_end_matches("_head").trim_end_matches("_skull");
        (SKULL_KINDS.contains(&kind) && kind.len() < short.len()).then(|| Self::Skull {
            kind: kind.to_owned(),
        })
    }

    /// A chest of any material, as a half or whole.
    fn chest(state: BlockStateId, short: &str) -> Option<Self> {
        if short != "chest" && !short.ends_with("_chest") {
            return None;
        }
        let kind = match photon_world::block::rules::property(state, "type")
            .and_then(ChestKind::from_name)
        {
            Some(kind) if short != "ender_chest" => kind,
            _ => ChestKind::Single,
        };
        Some(Self::Chest {
            material: chest_material(short),
            kind,
        })
    }

    /// What the reference calls the model, as the check against it prints it.
    pub fn describe(&self, state: BlockStateId) -> String {
        match self {
            Self::Empty => "EmptyBlockModel$Unbaked()".to_owned(),
            Self::Banner { .. }
            | Self::Shulker { .. }
            | Self::Statue { .. }
            | Self::Skull { .. } => self.describe_dressed(),
            Self::Bell
            | Self::Conduit
            | Self::Pot
            | Self::PlayerHead
            | Self::Table
            | Self::EndCube { .. } => self.describe_plain(),
            Self::Chest { material, kind } => describe_chest(*material, *kind),
            Self::FlowerBed => describe_flower_bed(state),
        }
    }

    /// The renderers that carry nothing of the state.
    fn describe_plain(&self) -> String {
        match self {
            Self::Bell => wrap("BellSpecialRenderer$Unbaked()", false),
            Self::Conduit => wrap("ConduitSpecialRenderer$Unbaked()", true),
            Self::Pot => wrap("DecoratedPotSpecialRenderer$Unbaked()", true),
            Self::PlayerHead => wrap("PlayerHeadSpecialRenderer$Unbaked()", true),
            Self::Table => wrap(
                "BookSpecialRenderer$Unbaked(openAngle=0.0,page1=0.0,page2=0.0)",
                true,
            ),
            Self::EndCube { portal } => {
                let effect = if *portal { "portal" } else { "gateway" };
                wrap(
                    &format!("EndCubeSpecialRenderer$Unbaked(effect={effect})"),
                    *portal,
                )
            }
            _ => String::new(),
        }
    }

    /// The renderers that carry a colour, texture or kind.
    fn describe_dressed(&self) -> String {
        match self {
            Self::Banner { color, wall } => {
                let attachment = if *wall { "wall" } else { "ground" };
                wrap(
                    &format!(
                        "BannerSpecialRenderer$Unbaked(baseColor={color},attachment={attachment})"
                    ),
                    true,
                )
            }
            Self::Shulker { color } => wrap(
                &format!(
                    "ShulkerBoxSpecialRenderer$Unbaked(texture={},openness=0.0)",
                    sprite(&shulker_box::texture(*color))
                ),
                true,
            ),
            Self::Statue { weathering, pose } => wrap(
                &format!(
                    "CopperGolemStatueSpecialRenderer$Unbaked(texture=minecraft:textures/{}.png,pose={pose})",
                    copper_golem_statue::texture(*weathering)
                ),
                true,
            ),
            Self::Skull { kind } => wrap(
                &format!(
                    "SkullSpecialRenderer$Unbaked(kind={kind},textureOverride=none,animation=0.0)"
                ),
                true,
            ),
            _ => String::new(),
        }
    }
}

/// The skull types a block is named for (`SkullBlock.Types`), the player's apart.
const SKULL_KINDS: [&str; 6] = [
    "skeleton",
    "wither_skeleton",
    "zombie",
    "creeper",
    "piglin",
    "dragon",
];

/// `SpecialBlockModelWrapper.Unbaked` around a renderer.
fn wrap(renderer: &str, transformed: bool) -> String {
    let transformation = if transformed { "transform" } else { "none" };
    format!("SpecialBlockModelWrapper$Unbaked(model={renderer},transformation={transformation})")
}

/// A chest: the Christmas texture where `IsXmas` says so, for the materials that have it.
fn describe_chest(material: ChestMaterial, kind: ChestKind) -> String {
    let chest_type = match kind {
        ChestKind::Single => "single",
        ChestKind::Left => "left",
        ChestKind::Right => "right",
    };
    let renderer = |material: ChestMaterial| {
        wrap(
            &format!(
                "ChestSpecialRenderer$Unbaked(texture={},openness=0.0,chestType={chest_type})",
                sprite(&material.texture(kind))
            ),
            true,
        )
    };
    if material.has_christmas_variant() {
        format!(
            "ConditionalBlockModel$Unbaked(transformation=none,property=IsXmas,onTrue={},onFalse={})",
            renderer(ChestMaterial::Christmas),
            renderer(material)
        )
    } else {
        renderer(material)
    }
}

/// The petals: the baked model, turned by the display context.
fn describe_flower_bed(state: BlockStateId) -> String {
    let model = photon_world::block::rules::serialize_state(state);
    let baked = |transformation: &str| {
        format!("BlockStateModelWrapper$Unbaked(model={model},transformation={transformation})")
    };
    format!(
        "SelectBlockModel$Unbaked(transformation=none,unbakedSwitch=SelectBlockModel$UnbakedSwitch(property=DisplayContext(),cases=[SelectBlockModel$SwitchCase(values=[BlockDisplayContext],model={})]),fallback={})",
        baked("transform"),
        baked("none")
    )
}

/// A texture's name as the reference prints a sprite: its last path segment.
fn sprite(texture: &str) -> String {
    format!(
        "minecraft:{}",
        texture.rsplit('/').next().unwrap_or(texture)
    )
}

fn dye_of(name: &str) -> Option<&'static str> {
    photon_world::DYES.iter().copied().find(|dye| *dye == name)
}

/// `ChestRenderer.getChestMaterial` without Christmas textures, from the block's short name.
pub fn chest_material(short: &str) -> ChestMaterial {
    match short.strip_prefix("waxed_").unwrap_or(short) {
        "ender_chest" => ChestMaterial::Ender,
        "copper_chest" => ChestMaterial::CopperUnaffected,
        "exposed_copper_chest" => ChestMaterial::CopperExposed,
        "weathered_copper_chest" => ChestMaterial::CopperWeathered,
        "oxidized_copper_chest" => ChestMaterial::CopperOxidized,
        "trapped_chest" => ChestMaterial::Trapped,
        _ => ChestMaterial::Regular,
    }
}
