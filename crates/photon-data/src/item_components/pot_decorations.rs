//! The pot decorations component, which 26.3 changed from a list of item ids (the brick standing
//! for a side with none) to four optional item stack templates. The canonical form is 26.3's;
//! a rewrite for an older version converts to or from the list.

use super::*;

impl Cursor<'_> {
    /// `PotDecorations.STREAM_CODEC`.
    pub fn pot_decorations(&mut self) -> R {
        match self.pots() {
            Pots::Keep => {
                for _ in 0..4 {
                    self.optional(Cursor::template)?;
                }
                Ok(())
            }
            Pots::ToTemplates => self.sherds_to_templates(),
            Pots::ToItems => self.templates_to_sherds(),
        }
    }

    /// An older server's list of items, written as the four templates.
    fn sherds_to_templates(&mut self) -> R {
        let before = self.data;
        let mut wire = Vec::new();
        for _ in 0..self.count()? {
            wire.push(self.varint()?);
        }
        let mut out = Vec::new();
        for side in 0..4 {
            let item = wire.get(side).map(|id| self.mapped(Site::Item, *id));
            match item {
                Some(item) if item != crate::item::ids::BRICK.0 => {
                    out.push(1);
                    photon_core::wire::write_varint(item, &mut out);
                    out.extend([1, 0, 0]);
                }
                _ => out.push(0),
            }
        }
        self.replaced(before, &out);
        Ok(())
    }

    /// The four templates, written as the list of items an older server reads.
    fn templates_to_sherds(&mut self) -> R {
        let before = self.data;
        let mut plain = Cursor::new(self.data);
        let mut items = Vec::new();
        for _ in 0..4 {
            let mut item = None;
            if plain.boolean()? {
                item = Some(plain.varint()?);
                plain.varint()?;
                plain.skip_patch()?;
            }
            items.push(item);
        }
        self.data = plain.data;
        let mut out = Vec::new();
        photon_core::wire::write_varint(4, &mut out);
        for item in items {
            let item = item.unwrap_or(crate::item::ids::BRICK.0);
            photon_core::wire::write_varint(self.mapped(Site::Item, item), &mut out);
        }
        self.replaced(before, &out);
        Ok(())
    }
}
