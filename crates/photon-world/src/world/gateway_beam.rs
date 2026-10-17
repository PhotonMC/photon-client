//! `TheEndGatewayRenderer.extractRenderState`: the beam a gateway shines while it spawns, and
//! while its cooldown runs.

use photon_core::math::{floor_f32, mth_sin_f64};
use photon_data::block::blocks::id;

use crate::block::piston::Pos;
use crate::block::{self};
use crate::*;

/// `TheEndGatewayBlockEntity.SPAWN_TIME`.
const SPAWN_TIME: f32 = 200.0;
/// `TheEndGatewayBlockEntity.COOLDOWN_TIME`.
const COOLDOWN_TIME: f32 = 40.0;
/// How far the beam reaches while the gateway cools down.
const COOLDOWN_BEAM_DISTANCE: f64 = 50.0;

/// What a gateway's beam is drawn from at a partial tick (`EndGatewayRenderState`).
#[derive(Debug, Clone, PartialEq)]
pub struct GatewayBeamView {
    pub position: Pos,
    /// The beam's texture scale.
    pub scale: f32,
    /// How far the beam reaches above and below the block.
    pub height: i32,
    /// ARGB.
    pub color: u32,
    /// `Math.floorMod(gameTime, 40) + partialTicks`.
    pub animation_time: f32,
}

/// `Mth.clamp` of a float.
fn clamp(value: f32, low: f32, high: f32) -> f32 {
    value.max(low).min(high)
}

impl ClientWorld {
    fn dye_texture_colour(&self, name: &str) -> u32 {
        let index = DYES.iter().position(|candidate| *candidate == name);
        index
            .and_then(photon_data::entity_type::dye_texture_colour)
            .map_or(0, |colour| 0xFF00_0000 | colour)
    }

    /// The beams of the gateways the level ticks.
    pub fn gateway_beam_views(&self, partial: f32) -> Vec<GatewayBeamView> {
        let animation = (self.game_time() as i64).rem_euclid(40) as f32 + partial;
        let bounds = self.dimension();
        let max_y = f64::from(bounds.min_y + bounds.height - 1);
        self.block_tickers
            .positions()
            .filter(|(x, y, z)| {
                block::state_or_air(self.block_state_or_air(*x, *y, *z)).block
                    == id::END_GATEWAY as u16
            })
            .filter_map(|pos| {
                let events = self.block_entity_at(pos.0, pos.1, pos.2)?.events();
                let spawning = events.age < SPAWN_TIME as i64;
                let cooling = events.teleport_cooldown > 0;
                if !spawning && !cooling {
                    return None;
                }
                let percent = if spawning {
                    clamp((events.age as f32 + partial) / SPAWN_TIME, 0.0, 1.0)
                } else {
                    1.0 - clamp(
                        (events.teleport_cooldown as f32 - partial) / COOLDOWN_TIME,
                        0.0,
                        1.0,
                    )
                };
                let distance = if spawning {
                    max_y
                } else {
                    COOLDOWN_BEAM_DISTANCE
                };
                let scale = mth_sin_f64(f64::from(percent * std::f32::consts::PI));
                let height = floor_f32((f64::from(scale) * distance) as f32);
                Some(GatewayBeamView {
                    position: pos,
                    scale,
                    height,
                    color: self.dye_texture_colour(if spawning { "magenta" } else { "purple" }),
                    animation_time: animation,
                })
            })
            .filter(|view| view.height > 0)
            .collect()
    }
}
