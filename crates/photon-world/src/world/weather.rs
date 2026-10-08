use crate::*;
use photon_core::java_float::JavaF32;

impl ClientWorld {
    pub fn game_time(&self) -> u64 {
        self.game_time
    }

    /// `Level.setRainLevel` and `setThunderLevel`: the weather the server sent
    /// (the level starts dry).
    pub fn set_weather(&mut self, rain_level: f32, thunder_level: f32) {
        self.rain_level = JavaF32::new(rain_level);
        self.thunder_level = JavaF32::new(thunder_level);
    }

    /// `Level.isThundering`: the level can have weather and the thunder,
    /// scaled by the rain, is above nine tenths.
    pub fn is_thundering(&self) -> bool {
        let environment = &self.dimension_environment;
        let can_have_weather = environment.has_skylight
            && !environment.has_ceiling
            && self.dimension_key != "minecraft:the_end";
        can_have_weather && self.thunder_level.get() * self.rain_level.get() > 0.9
    }

    /// `ClientLevel.setTimeFromServer`: the game time the server sent.
    pub fn set_game_time(&mut self, game_time: u64) {
        self.game_time = game_time;
    }

    pub fn tick(&mut self) {
        self.world_border.tick();
        self.block_animations.tick();
        self.tick_skulls();
        self.game_time = self.game_time.wrapping_add(1);
        let local_player_is_passenger = self.local_player.as_ref().is_some_and(|player| {
            self.entity_vehicle_id(player.id).is_some_and(|vehicle_id| {
                self.entity_passengers(vehicle_id)
                    .is_some_and(|ids| ids.contains(&player.id))
            })
        });
        if !local_player_is_passenger {
            self.tick_local_player(None);
        }
        self.tick_entities();
        self.tick_sky_flash();
    }

    /// `ClientLevel.tick`'s share: the sky flash counts down.
    pub fn tick_sky_flash(&mut self) {
        if self.sky_flash_time > 0 {
            self.sky_flash_time -= 1;
        }
    }

    /// `ClientLevel.skyFlashTime`, without the option that hides the flash.
    pub fn sky_flash_time(&self) -> i32 {
        self.sky_flash_time
    }
}
