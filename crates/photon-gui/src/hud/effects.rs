//! The active effects' icons at the top right of the HUD (`Hud.extractEffects`), and the order the
//! HUD and the inventory list effects in.

use std::cmp::Ordering;

use photon_core::math::mth_cos_f64;
use photon_data::item::effect::{EffectCategory, MobEffect};
use photon_data::item::mob_effects::MOB_EFFECTS;
use photon_render::gui::{GuiAtlas, GuiFrame};
use photon_world::ClientMobEffectState;

/// The effect's registry entry.
pub fn entry(effect: &ClientMobEffectState) -> Option<&'static MobEffect> {
    usize::try_from(effect.effect_id())
        .ok()
        .and_then(|id| MOB_EFFECTS.get(id))
}

/// `Hud.getMobEffectSprite`.
pub fn sprite(entry: &MobEffect) -> String {
    format!("mob_effect/{}", entry.name.trim_start_matches("minecraft:"))
}

/// `MobEffectInstance.compareTo`: ambient effects last, then infinite ones, then by duration and
/// colour.
pub fn compare(a: &ClientMobEffectState, b: &ClientMobEffectState) -> Ordering {
    let colour = |effect| entry(effect).map_or(0, |entry| entry.color);
    let (a_ticks, b_ticks) = (a.duration_ticks(), b.duration_ticks());
    // Two long ambient effects (beacons that keep renewing them) go by colour alone, so their
    // order holds still as their durations are topped up.
    if (a_ticks <= 32147 || b_ticks <= 32147) && !(a.is_ambient() && b.is_ambient()) {
        a.is_ambient()
            .cmp(&b.is_ambient())
            .then((a_ticks == -1).cmp(&(b_ticks == -1)))
            .then(a_ticks.cmp(&b_ticks))
            .then(colour(a).cmp(&colour(b)))
    } else {
        a.is_ambient()
            .cmp(&b.is_ambient())
            .then(colour(a).cmp(&colour(b)))
    }
}

/// `Hud.extractEffects`: the effects that show an icon, last in order first, beneficial ones along
/// the top from the right and the others in the row below. Photon has no demo mode, whose notice
/// would push them down.
pub fn draw(frame: &mut GuiFrame, atlas: &GuiAtlas, width: i32, effects: &[ClientMobEffectState]) {
    let mut sorted = effects.to_vec();
    sorted.sort_by(|a, b| compare(b, a));
    let (mut beneficial, mut others) = (0, 0);
    for effect in sorted.iter().filter(|effect| effect.shows_icon()) {
        let Some(entry) = entry(effect) else {
            continue;
        };
        let (x, y) = if entry.category == EffectCategory::Beneficial {
            beneficial += 1;
            (width - 25 * beneficial, 1)
        } else {
            others += 1;
            (width - 25 * others, 27)
        };
        let mut alpha = 1.0_f32;
        if effect.is_ambient() {
            frame.sprite(atlas, "hud/effect_background_ambient", x, y, 24, 24);
        } else {
            frame.sprite(atlas, "hud/effect_background", x, y, 24, 24);
            if effect.ends_within(200) {
                // The last ten seconds fade the icon and blink it, faster as it runs out.
                let remaining = effect.duration_ticks();
                let used_seconds = 10 - remaining / 20;
                let angle = remaining as f32 * std::f32::consts::PI / 5.0;
                alpha = (remaining as f32 / 10.0 / 5.0 * 0.5).clamp(0.0, 0.5)
                    + mth_cos_f64(f64::from(angle))
                        * (used_seconds as f32 / 10.0 * 0.25).clamp(0.0, 0.25);
                alpha = alpha.clamp(0.0, 1.0);
            }
        }
        // `ARGB.white`.
        let colour = ((alpha * 255.0).floor() as u32) << 24 | 0x00FF_FFFF;
        frame.sprite_tinted(atlas, &sprite(entry), (x + 3, y + 3, 18, 18), colour);
    }
}
