use super::*;
use photon_core::java_float::JavaF32;

/// The four Java game-mode values applied to the local player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClientPlayerGameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

/// The six ability values carried by Java's clientbound player-abilities packet,
/// plus the local game-mode-derived build permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientPlayerAbilities {
    pub(crate) invulnerable: bool,
    pub(crate) flying: bool,
    pub(crate) may_fly: bool,
    pub(crate) instabuild: bool,
    pub(crate) may_build: bool,
    pub(crate) flying_speed: JavaF32,
    pub(crate) walking_speed: JavaF32,
}

impl Default for ClientPlayerAbilities {
    fn default() -> Self {
        Self::new(false, false, false, false, 0.05, 0.1)
    }
}

impl ClientPlayerGameMode {
    /// Resolve Java's continuous id mapping, including its survival fallback.
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    /// Resolve the parameter used by Java's change-game-mode event.
    pub fn from_game_event_param(param: f32) -> Self {
        Self::from_id(floor_f32(param + 0.5_f32))
    }

    pub fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Survival => "survival",
            Self::Creative => "creative",
            Self::Adventure => "adventure",
            Self::Spectator => "spectator",
        }
    }

    pub fn is_spectator(self) -> bool {
        matches!(self, Self::Spectator)
    }

    pub fn is_creative(self) -> bool {
        matches!(self, Self::Creative)
    }

    pub fn is_survival(self) -> bool {
        matches!(self, Self::Survival | Self::Adventure)
    }

    /// Apply Java `GameType.updatePlayerAbilities` to an existing ability state.
    ///
    /// Creative deliberately leaves `flying` unchanged, while the other branches
    /// set it exactly as the Java game-mode implementation does.
    pub fn update_player_abilities(
        self,
        mut abilities: ClientPlayerAbilities,
    ) -> ClientPlayerAbilities {
        match self {
            Self::Creative => {
                abilities.may_fly = true;
                abilities.instabuild = true;
                abilities.invulnerable = true;
            }
            Self::Spectator => {
                abilities.may_fly = true;
                abilities.instabuild = false;
                abilities.invulnerable = true;
                abilities.flying = true;
            }
            Self::Survival | Self::Adventure => {
                abilities.may_fly = false;
                abilities.instabuild = false;
                abilities.invulnerable = false;
                abilities.flying = false;
            }
        }
        abilities.may_build = !matches!(self, Self::Adventure | Self::Spectator);
        abilities
    }
}

impl ClientPlayerAbilities {
    pub fn new(
        invulnerable: bool,
        flying: bool,
        may_fly: bool,
        instabuild: bool,
        flying_speed: f32,
        walking_speed: f32,
    ) -> Self {
        Self {
            invulnerable,
            flying,
            may_fly,
            instabuild,
            may_build: true,
            flying_speed: JavaF32::new(flying_speed),
            walking_speed: JavaF32::new(walking_speed),
        }
    }

    pub fn invulnerable(self) -> bool {
        self.invulnerable
    }

    pub fn flying(self) -> bool {
        self.flying
    }

    pub fn may_fly(self) -> bool {
        self.may_fly
    }

    pub fn instabuild(self) -> bool {
        self.instabuild
    }

    pub fn may_build(self) -> bool {
        self.may_build
    }

    pub fn flying_speed(self) -> f32 {
        self.flying_speed.get()
    }

    pub fn walking_speed(self) -> f32 {
        self.walking_speed.get()
    }
}
