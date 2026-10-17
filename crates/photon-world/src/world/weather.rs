use crate::*;
use photon_core::java_float::JavaF32;

impl ClientWorld {
    pub fn game_time(&self) -> u64 {
        self.level.game_time
    }

    /// `Level.setRainLevel` and `setThunderLevel`: the weather the server sent
    /// (the level starts dry).
    pub fn set_weather(&mut self, rain_level: f32, thunder_level: f32) {
        self.weather.rain_level = JavaF32::new(rain_level);
        self.weather.thunder_level = JavaF32::new(thunder_level);
    }

    /// `Level.canHaveWeather`: a sky to rain from, and no ceiling or end of the world over it.
    fn can_have_weather(&self) -> bool {
        let environment = &self.level.dimension_environment;
        environment.has_skylight
            && !environment.has_ceiling
            && self.level.dimension_key != "minecraft:the_end"
    }

    /// `Level.isRaining`: the level can have weather and its rain is above a fifth.
    pub fn is_raining(&self) -> bool {
        self.can_have_weather() && self.weather.rain_level.get() > 0.2
    }

    /// `Level.isThundering`: the level can have weather and the thunder,
    /// scaled by the rain, is above nine tenths.
    pub fn is_thundering(&self) -> bool {
        self.can_have_weather()
            && self.weather.thunder_level.get() * self.weather.rain_level.get() > 0.9
    }

    /// `ClientLevel.setTimeFromServer`: the game time the server sent.
    pub fn set_game_time(&mut self, game_time: u64) {
        self.level.game_time = game_time;
    }

    pub fn tick(&mut self) {
        self.world_border.tick();
        self.block_animations.tick();
        self.tick_skulls();
        self.level.game_time = self.level.game_time.wrapping_add(1);
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
        if self.weather.sky_flash_time > 0 {
            self.weather.sky_flash_time -= 1;
        }
    }

    /// `ClientLevel.skyFlashTime`, without the option that hides the flash.
    pub fn sky_flash_time(&self) -> i32 {
        self.weather.sky_flash_time
    }
}
