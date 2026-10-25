//! `EffectsInInventory`: the active effects listed to the right of the survival and creative
//! inventories, each with its icon, name and time left.

use photon_data::item::effect::MobEffect;
use photon_game::text::{Component, Run, Style};
use photon_world::ClientMobEffectState;

use super::ScreenDraw;
use super::confirm::head_by_width;
use super::container::ContainerLayout;
use crate::hud::effects::{compare, entry, sprite};
use crate::{FontRuns, FrameRuns};

/// The tooltip a cut entry under the mouse asks for, which the screen draws after everything.
pub type EffectsTooltip = Option<Vec<Vec<Run>>>;

/// The left edge of the list and the room it has: two units right of the image.
fn room(layout: &ContainerLayout) -> (i32, i32) {
    let x0 = layout.left() + layout.image.0 + 2;
    (x0, layout.width - x0)
}

/// `canSeeEffects`: there are 32 units free to the right of the image.
pub fn can_see(layout: &ContainerLayout) -> bool {
    room(layout).1 >= 32
}

/// `extractRenderState`: the effects in order down from the image's top, squeezed together past
/// five, each as wide as its name and time where there is room and only its icon where not.
pub fn draw(
    draw: &mut ScreenDraw<'_, '_>,
    layout: &ContainerLayout,
    mouse: (f64, f64),
) -> EffectsTooltip {
    let (x0, available) = room(layout);
    let effects = draw.viewer.effects;
    if effects.is_empty() || available < 32 {
        return None;
    }
    let max_width = if available >= 120 { available - 7 } else { 32 };
    let count = effects.len() as i32;
    let y_step = if count > 5 { 132 / (count - 1) } else { 33 };
    let mut sorted = effects.to_vec();
    sorted.sort_by(compare);
    let mouse = (mouse.0 as i32, mouse.1 as i32);
    let mut tooltip = None;
    for (index, effect) in sorted.iter().enumerate() {
        let y0 = layout.top() + index as i32 * y_step;
        let asked = draw_entry(draw, effect, (x0, y0, y_step), max_width, mouse);
        // `setTooltipForNextFrame`: the first tooltip asked for in a frame is the one drawn.
        tooltip = tooltip.or(asked);
    }
    tooltip
}

/// The tooltip [`draw`] returned, unless the screen leaves its widgets' tooltips out.
pub fn draw_tooltip(
    draw: &mut ScreenDraw<'_, '_>,
    layout: &ContainerLayout,
    tooltip: EffectsTooltip,
    mouse: (f64, f64),
) {
    if let Some(lines) = tooltip.filter(|_| draw.widget_tooltips) {
        layout.tooltip(draw, &lines, mouse);
    }
}

/// One entry: its background as wide as its texts allow, the texts (the name cut short to fit),
/// and its icon. An entry whose name is cut, or that has no room for it, asks for a tooltip.
fn draw_entry(
    draw: &mut ScreenDraw<'_, '_>,
    effect: &ClientMobEffectState,
    (x0, y0, y_step): (i32, i32, i32),
    max_width: i32,
    mouse: (i32, i32),
) -> EffectsTooltip {
    let entry = entry(effect)?;
    let (name, duration) = texts(draw, entry, effect);
    let name_width = draw.font.width_of_runs(&name);
    let duration_width = draw.font.width_of_runs(&duration);
    let width = max_width.min((32 + name_width + 7).max(32 + duration_width + 7));
    let background = if effect.is_ambient() {
        "container/inventory/effect_background_ambient"
    } else {
        "container/inventory/effect_background"
    };
    draw.frame.sprite(draw.atlas, background, x0, y0, width, 32);
    let max_text = width - 32 - 7;
    let compact = max_text <= 0 || name_width > max_text;
    if max_text > 0 {
        let shown = if compact {
            clip(draw, &name, max_text)
        } else {
            name.clone()
        };
        let (atlas, font) = (draw.atlas, draw.font);
        draw.frame
            .styled_text(atlas, font, &shown, x0 + 32, y0 + 7, 0xFFFF_FFFF, true);
        draw.frame
            .styled_text(atlas, font, &duration, x0 + 32, y0 + 16, 0xFF80_8080, true);
    }
    let over = (x0..=x0 + width).contains(&mouse.0) && (y0..=y0 + y_step).contains(&mouse.1);
    let tooltip = (compact && over).then(|| vec![name, duration]);
    draw.frame
        .sprite(draw.atlas, &sprite(entry), x0 + 7, y0 + 7, 18, 18);
    tooltip
}

/// `getEffectName`, with the level from II to X, and `MobEffectUtil.formatDuration` at the
/// server's tick rate.
fn texts(
    draw: &ScreenDraw<'_, '_>,
    entry: &MobEffect,
    effect: &ClientMobEffectState,
) -> (Vec<Run>, Vec<Run>) {
    let mut name = Component::translatable(entry.description_id());
    let amplifier = effect.amplifier();
    if (1..=9).contains(&amplifier) {
        let level = Component::translatable(format!("enchantment.level.{}", amplifier + 1));
        name = name.append(Component::literal(" ")).append(level);
    }
    let duration =
        photon_game::tooltip::format_duration(effect.duration_ticks(), 1.0, draw.viewer.tick_rate);
    let style = Style::default();
    (
        draw.lang.runs(&name, &style),
        draw.lang.runs(&duration, &style),
    )
}

/// `ComponentRenderUtils.clipText`: as much of the text as fits before an ellipsis.
fn clip(draw: &ScreenDraw<'_, '_>, text: &[Run], width: i32) -> Vec<Run> {
    let ellipsis = draw
        .lang
        .runs(&Component::literal("..."), &Style::default());
    let mut head = head_by_width(text, draw.font, width - draw.font.width_of_runs(&ellipsis));
    head.extend(ellipsis);
    head
}
