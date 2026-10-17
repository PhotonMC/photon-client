//! The dimension's data as the particle rows read it: the overworld's clock and the biome a row
//! puts its whole scene in.

use photon_render::sky::{Environment, SkyAmbience};
use photon_world::particles::{Ambience, NoAmbience};

use super::*;

/// The biomes of the data pack, numbered by name: the scene's chunks and the environment agree on
/// the numbers.
pub(super) fn biome_names() -> Result<Vec<String>, String> {
    let mut names: Vec<String> = std::fs::read_dir(data_root().join("minecraft/worldgen/biome"))
        .map_err(|error| error.to_string())?
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().to_string_lossy().into_owned();
            Some(format!("minecraft:{}", name.strip_suffix(".json")?))
        })
        .collect();
    names.sort();
    Ok(names)
}

fn data_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.data/minecraft/assets/jar/data")
}

/// The number of the biome a row's scene is in (the plains where it says nothing).
pub(crate) fn biome_id(row: &Value) -> Result<i32, String> {
    let wanted = row["biome"].as_str().unwrap_or("minecraft:plains");
    let position = biome_names()?
        .iter()
        .position(|name| name == wanted)
        .ok_or_else(|| format!("unknown biome {wanted}"))?;
    i32::try_from(position).map_err(|error| error.to_string())
}

/// What the environment says to a row, at the clock it has been told.
pub(super) struct Environments {
    environment: Option<Environment>,
    clock: i64,
}

impl Environments {
    /// The overworld's data, for a row that sets a biome or a clock; nothing for another.
    pub(super) fn load(row: &Value) -> Result<Self, String> {
        let wanted = row.get("biome").is_some() || row.get("clock").is_some();
        let environment = if wanted {
            let names = biome_names()?;
            let loaded = Environment::load(&data_root(), "minecraft:overworld", &names)
                .ok_or("the overworld's data could not be read")?;
            Some(loaded)
        } else {
            None
        };
        Ok(Self {
            environment,
            clock: row["clock"].as_i64().unwrap_or(0),
        })
    }

    /// An event that reads or sets what the environment says: a `clock` (the overworld's clock
    /// set to the ticks it says) or an `animate_tick` (the blocks about a place pick at random).
    /// The sounds it played; nothing for an event of another kind.
    pub(super) fn run<F: FnMut() -> RandomSource>(
        &mut self,
        event: &Value,
        (engine, world, view): (&mut ParticleEngine, &ClientWorld, &AddView),
        (level_random, particle_random): &mut (&mut RandomSource, &mut F),
    ) -> Result<Option<Vec<String>>, String> {
        match string(event, "type")? {
            "clock" => {
                self.clock = event["ticks"].as_i64().ok_or("the clock needs ticks")?;
                Ok(Some(Vec::new()))
            }
            "animate_tick" => {
                let mut context =
                    LevelEventContext::new(world, view, level_random, &mut *particle_random);
                let ambience = self.ambience();
                context.ambience = &*ambience;
                events::animate_tick(event, engine, &mut context).map(Some)
            }
            _ => Ok(None),
        }
    }

    /// What the attributes say now.
    pub(super) fn ambience(&self) -> Box<dyn Ambience + '_> {
        match &self.environment {
            Some(environment) => Box::new(SkyAmbience::new(environment, |_| self.clock)),
            None => Box::new(NoAmbience),
        }
    }
}
