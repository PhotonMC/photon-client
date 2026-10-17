//! `PortalProcessor` and `LocalPlayer.handlePortalTransitionEffect`: a player inside a portal
//! block builds up the effect that spins and clouds the view, and loses it again outside.

use super::*;

/// Which portal block the player is inside; only the nether portal's effect shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalKind {
    Nether,
    End,
}

/// `Entity.portalProcess`, on a client: which portal, and whether the player is inside it this
/// tick (the teleport's own timing belongs to the server).
#[derive(Debug, Clone, Copy)]
struct Process {
    kind: PortalKind,
    inside_this_tick: bool,
}

/// `Entity.portalCooldown`, the process and the effect's intensity, now and a tick ago.
#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    process: Option<Process>,
    cooldown: i32,
    intensity: f32,
    old_intensity: f32,
}

/// `Entity.getDimensionChangingDelay`.
const DIMENSION_CHANGING_DELAY: i32 = 300;

impl LocalPlayer {
    /// `Entity.canUsePortal(false)`: not riding, and alive.
    pub(super) fn can_use_portal(&self) -> bool {
        !self.is_passenger() && self.health > 0.0
    }

    /// `Entity.setAsInsidePortal`, called by a portal block the player is inside.
    pub(super) fn set_as_inside_portal(&mut self, kind: PortalKind) {
        let portal = &mut self.portal;
        if portal.cooldown > 0 {
            portal.cooldown = DIMENSION_CHANGING_DELAY;
        } else {
            match &mut portal.process {
                Some(process) if process.kind == kind => process.inside_this_tick = true,
                other => {
                    *other = Some(Process {
                        kind,
                        inside_this_tick: true,
                    });
                }
            }
        }
    }

    /// The start of `LocalPlayer.aiStep`: the sprint trigger counts down, the portal effect builds
    /// or fades, then the portal cooldown runs down.
    pub(super) fn tick_start_timers(&mut self) {
        if self.sprint_trigger_time > 0 {
            self.sprint_trigger_time -= 1;
        }
        self.tick_portal();
    }

    /// `handlePortalTransitionEffect` and `processPortalCooldown`.
    fn tick_portal(&mut self) {
        let portal = &mut self.portal;
        portal.old_intensity = portal.intensity;
        let mut step = 0.0;
        let confusing = portal
            .process
            .as_mut()
            .filter(|process| process.kind == PortalKind::Nether && process.inside_this_tick);
        if let Some(process) = confusing {
            process.inside_this_tick = false;
            if portal.intensity == 0.0 {
                let pitch = self.random.next_float() * 0.4 + 0.8;
                self.pending_sounds.push(LocalSound {
                    relative: true,
                    delay_ticks: 0,
                    event: "minecraft:block.portal.trigger",
                    source: LocalSoundSource::Ambient,
                    volume: 0.25,
                    pitch,
                    position: (0.0, 0.0, 0.0),
                });
            }
            step = 0.0125;
        } else if portal.intensity > 0.0 {
            step = -0.05;
        }
        let portal = &mut self.portal;
        portal.intensity = (portal.intensity + step).clamp(0.0, 1.0);
        if portal.cooldown > 0 {
            portal.cooldown -= 1;
        }
    }

    /// The portal effect's intensity between the last tick and this one.
    pub fn portal_effect_intensity(&self, partial_tick: f32) -> f32 {
        photon_core::math::mth_lerp_f32(
            partial_tick,
            self.portal.old_intensity,
            self.portal.intensity,
        )
    }

    /// The portal effect's intensity as of the last tick (`GameRenderer.tick` reads this one).
    pub fn portal_effect_intensity_now(&self) -> f32 {
        self.portal.intensity
    }
}
