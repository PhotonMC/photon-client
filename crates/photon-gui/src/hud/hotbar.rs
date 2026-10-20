//! The hotbar (`Hud.extractItemHotbar`): its sprites, the item of each slot and the off hand's,
//! and the attack indicator beside it.

use super::*;

pub(super) fn draw(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &photon_render::gui::font::Font),
    (width, height): (i32, i32),
    input: &HudInput,
    draw_item: &mut ItemDrawer<'_>,
) {
    let t = GuiBlend::Translucent;
    let centre = width / 2;
    frame.sprite_blend(atlas, t, "hud/hotbar", (centre - 91, height - 22, 182, 22));
    frame.sprite_blend(
        atlas,
        t,
        "hud/hotbar_selection",
        (
            centre - 91 - 1 + input.selected_slot * 20,
            height - 22 - 1,
            24,
            23,
        ),
    );
    // The off hand is on the left of a right-handed player, which
    // is the default main arm.
    if input.offhand.is_some() {
        frame.sprite_blend(
            atlas,
            t,
            "hud/hotbar_offhand_left",
            (centre - 91 - 29, height - 23, 29, 24),
        );
    }
    slot_items(frame, (atlas, font), (centre, height), input, draw_item);
    attack_indicator(frame, atlas, (centre, height), input);
}

/// `extractSlot` for each hotbar slot, then the off hand: the item, its bar and cooldown, then
/// its count when it is not one.
fn slot_items(
    frame: &mut GuiFrame,
    (atlas, font): (&GuiAtlas, &photon_render::gui::font::Font),
    (centre, height): (i32, i32),
    input: &HudInput,
    draw_item: &mut ItemDrawer<'_>,
) {
    let y = height - 16 - 3;
    let slots = (0..9)
        .map(|i| (&input.hotbar[i], centre - 90 + i as i32 * 20 + 2))
        .chain(std::iter::once((&input.offhand, centre - 91 - 26)));
    for (slot, (stack, x)) in slots.enumerate() {
        let Some(stack) = stack else {
            continue;
        };
        // `getUseItem() == itemStack`: the selected slot is the main hand's stack.
        let using = input.item_use.and_then(|(off_hand, ticks)| {
            let is_used = if off_hand {
                slot == 9
            } else {
                slot as i32 == input.selected_slot
            };
            is_used.then_some(ticks)
        });
        frame.item_in_use(draw_item, stack, (x as f32, y as f32), using);
        frame.item_bar(atlas, stack, x, y);
        frame.item_cooldown(atlas, stack, x, y);
        if stack.item_count() != 1 {
            let text = stack.item_count().to_string();
            frame.text_with_shadow(
                atlas,
                font,
                &text,
                x + 19 - 2 - font.width(&text),
                y + 6 + 3,
                0xFFFF_FFFF,
            );
        }
    }
}

/// The indicator beside the hotbar, on the side of the off hand (the main arm is the right one).
fn attack_indicator(
    frame: &mut GuiFrame,
    atlas: &GuiAtlas,
    (centre, height): (i32, i32),
    input: &HudInput,
) {
    let t = GuiBlend::Translucent;
    if input.attack_indicator == AttackIndicator::Hotbar && input.attack_strength < 1.0 {
        let progress = (input.attack_strength * 19.0) as i32;
        let (x, y) = (centre + 91 + 6, height - 20);
        frame.sprite_blend(
            atlas,
            t,
            "hud/hotbar_attack_indicator_background",
            (x, y, 18, 18),
        );
        frame.sprite_region_blend(
            atlas,
            t,
            "hud/hotbar_attack_indicator_progress",
            (18, 18),
            (0, 18 - progress),
            (x, y + 18 - progress, 18, progress),
        );
    }
}
