//! `extractRenderState` of the test instance block's screen.

use super::*;

impl TestInstanceBlockScreen {
    /// The dimmed game, the widgets in the order they were added, the title and the labels.
    pub fn draw(
        &self,
        (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font),
        mouse: (i32, i32),
        lang: &Lang,
    ) {
        let (width, height) = self.size;
        let left = width / 2 - COLUMN / 2;
        frame.fill_gradient(atlas, (0, 0, width, height), 0xC010_1010, 0xD010_1010);
        self.draw_box((frame, atlas, font), Slot::Id);
        self.draw_info((frame, atlas, font));
        for slot in [Slot::SizeX, Slot::SizeY, Slot::SizeZ] {
            self.draw_box((frame, atlas, font), slot);
        }
        self.rotation.draw(frame, atlas, font, &self.arena, mouse);
        self.entities.draw(frame, atlas, font, &self.arena, mouse);
        for (slot, widget) in [
            (Slot::Reset, &self.reset),
            (Slot::Save, &self.save),
            (Slot::Run, &self.run),
            (Slot::Done, &self.done),
            (Slot::Cancel, &self.cancel),
        ] {
            let highlighted = widget.contains(&self.arena, mouse) || self.focus == slot;
            widget.draw(frame, atlas, font, &self.arena, highlighted);
        }
        let runs = |key: &str| lang.runs(&Component::translatable(key), &Style::default());
        let title = runs("block.minecraft.test_instance_block");
        let title_width = font.width_of_runs(&title);
        frame.styled_text(
            atlas,
            font,
            &title,
            (width / 2 - title_width / 2, 10),
            0xFFFF_FFFF,
            true,
        );
        let label = |frame: &mut GuiFrame, key: &str, x: i32, y: i32| {
            frame.styled_text(atlas, font, &runs(key), (x, y), LABEL_GREY, true);
        };
        label(frame, "test_instance_block.test_id", left, 30);
        label(frame, "test_instance_block.size", left, 150);
        label(
            frame,
            "test_instance_block.rotation",
            self.rectangle(Slot::Rotation).x,
            150,
        );
        label(
            frame,
            "test_instance_block.entities",
            self.rectangle(Slot::Entities).x,
            150,
        );
    }

    /// `EditBox.extractWidgetRenderState` with its border: the sprite, the text and the cursor.
    fn draw_box(&self, (frame, atlas, font): (&mut GuiFrame, &GuiAtlas, &Font), slot: Slot) {
        let Some(index) = box_index(slot) else {
            return;
        };
        let focused = self.focus == slot;
        let sprite = if focused {
            "widget/text_field_highlighted"
        } else {
            "widget/text_field"
        };
        let at = self.rectangle(slot);
        frame.nine_slice(
            atlas,
            sprite,
            (200, 20),
            1,
            false,
            (at.x, at.y, at.width, 20),
        );
        self.edits[index].draw_formatted(
            frame,
            atlas,
            font,
            (at.x + 4, at.y + (20 - 8) / 2),
            focused.then_some(self.focused_at),
            None,
        );
    }
}
