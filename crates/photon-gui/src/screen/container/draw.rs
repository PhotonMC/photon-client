//! Drawing what the container screens share: the carried stack and a slot.

use super::{ContainerLayout, ContainerMouse, SlotPreview, SlotView, Stack};
use crate::{ItemSlots, screen::ScreenDraw};

impl ContainerMouse {
    /// `extractCarriedItem`: the stack at the mouse, less what a drag over
    /// more than one slot would leave.
    pub(crate) fn draw_carried(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        carried: Option<&Stack>,
        mouse: (f64, f64),
    ) {
        let Some(carried) = carried else {
            return;
        };
        let (count, text) = self.carried_count(carried);
        let (x, y) = (mouse.0 as i32 - 8, mouse.1 as i32 - 8);
        if count > 0 || text.is_some() {
            let shown = text.map(|text| (text, 0xFFFF_FF55));
            draw.frame.item_with_count(
                draw.atlas, draw.font, draw.item, carried, count, shown, x, y,
            );
        }
    }

    /// `extractSlot`: a drag's preview count, an empty slot's icon, or the
    /// stack and its count, seeded by the slot's `x + y * imageWidth`.
    pub(crate) fn draw_slot(
        &mut self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &impl SlotView,
        index: usize,
        layout: &ContainerLayout,
        (slot_x, slot_y): (i32, i32),
        empty_icon: Option<&str>,
    ) {
        let (x, y) = (layout.left() + slot_x, layout.top() + slot_y);
        let seed = slot_x + slot_y * layout.image.0;
        let mut shown = menu
            .stack(index)
            .map(|stack| (stack.clone(), stack.item_count()));
        let mut text = None;
        let mut preview = false;
        let yellow;
        match self.slot_preview(index, menu) {
            Some(SlotPreview::Hidden) => return,
            Some(SlotPreview::Fill {
                stack,
                count,
                limit,
            }) => {
                preview = true;
                if let Some(limit) = limit {
                    yellow = limit;
                    text = Some((yellow.as_str(), 0xFFFF_FF55));
                }
                shown = Some((stack, count)).filter(|&(_, count)| count > 0);
            }
            None => {}
        }
        if shown.is_none()
            && let Some(icon) = empty_icon
        {
            draw.frame.sprite(draw.atlas, icon, x, y, 16, 16);
            return;
        }
        if preview {
            draw.frame
                .fill(draw.atlas, (x, y, x + 16, y + 16), 0x80FF_FFFF);
        }
        if let Some((stack, count)) = shown {
            draw.frame.item_with_count_seeded(
                draw.atlas,
                draw.font,
                draw.item,
                &stack,
                (count, text),
                (x, y, seed),
            );
        }
    }
}
