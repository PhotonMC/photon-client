use super::*;

impl GuiFrame {
    /// `GuiGraphics.entity`: an entity's triangles, in GUI units with depth
    /// towards the viewer, textured from the skin.
    pub fn entity(&mut self, vertices: &[crate::gui::items::GuiItemVertex]) {
        if vertices.is_empty() {
            return;
        }
        let first = self.items.len() as u32;
        self.items.extend_from_slice(vertices);
        self.batches
            .push((GuiBlend::Entity, first, vertices.len() as u32));
    }

    /// `GuiGraphics.item`: one item's triangles into the slot at `(x, y)`.
    pub fn item(
        &mut self,
        draw: &mut ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        x: i32,
        y: i32,
    ) {
        self.item_at(draw, stack, x as f32, y as f32);
    }

    /// An item's triangles into the slot at a place the pose has put between pixels, as a toast
    /// does while it slides.
    pub fn item_at(
        &mut self,
        draw: &mut ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        x: f32,
        y: f32,
    ) {
        let first = self.items.len() as u32;
        let drawn = draw(stack, x, y, &mut self.items) as u32;
        let added = self.items.len() as u32 - first;
        if added == 0 {
            return;
        }
        match self.batches.last_mut() {
            Some((GuiBlend::Items, _, count)) => *count += drawn,
            _ => self.batches.push((GuiBlend::Items, first, drawn)),
        }
        // The glint is drawn right after its item, before the next item's depth is cleared.
        if added > drawn {
            self.batches
                .push((GuiBlend::ItemGlint, first + drawn, added - drawn));
        }
    }

    /// An item drawn under a pose that scales by `scale` and moves by `origin` (a toast's slide
    /// and the recipe toast's smaller station): the item is drawn at `(x, y)` in the scaled
    /// space, and what it added is carried over to the screen, its glint with it.
    pub fn item_scaled(
        &mut self,
        draw: &mut ItemDrawer<'_>,
        stack: &photon_world::ClientItemStackState,
        (x, y): (f32, f32),
        (scale, origin): (f32, (f32, f32)),
    ) {
        let first = self.items.len();
        self.item_at(draw, stack, x, y);
        for vertex in &mut self.items[first..] {
            vertex.position[0] = origin.0 + scale * vertex.position[0];
            vertex.position[1] = origin.1 + scale * vertex.position[1];
        }
    }
}
