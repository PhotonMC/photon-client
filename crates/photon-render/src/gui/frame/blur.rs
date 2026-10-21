use super::*;

/// Where in a frame the background is blurred (`GuiRenderState.blurBeforeThisStratum`): what was
/// drawn before stays under the blur, what comes after is drawn sharp over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuiBlur {
    /// How many vertices, and how many item vertices, the frame held when the blur was asked for.
    pub at: (u32, u32),
    /// The blur's radius in texels (`menuBackgroundBlurriness`).
    pub radius: u32,
}

impl GuiFrame {
    /// `Screen.extractBlurredBackground`: blur everything drawn so far, when `blurriness` is at
    /// least 1. The reference blurs once per frame, so a second request is ignored here.
    pub fn blur_before_this_stratum(&mut self, blurriness: i32) {
        if blurriness >= 1 && self.blur.is_none() {
            self.blur = Some(GuiBlur {
                at: (self.vertices.len() as u32, self.items.len() as u32),
                radius: blurriness as u32,
            });
        }
    }
}

/// A run of the frame's triangles in one blend: the blend, the first vertex and how many.
pub type Batch = (GuiBlend, u32, u32);

/// The batches split at a place in the frame: the runs drawn before it, and those after, with a run
/// that spans it cut in two.
pub fn split_batches(batches: &[Batch], (vertices, items): (u32, u32)) -> (Vec<Batch>, Vec<Batch>) {
    let (mut before, mut after) = (Vec::new(), Vec::new());
    for &(blend, first, count) in batches {
        let at = match blend {
            GuiBlend::Items | GuiBlend::ItemGlint | GuiBlend::Entity => items,
            _ => vertices,
        };
        let end = first + count;
        if end <= at {
            before.push((blend, first, count));
        } else if first >= at {
            after.push((blend, first, count));
        } else {
            before.push((blend, first, at - first));
            after.push((blend, at, end - at));
        }
    }
    (before, after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_over_the_split_is_cut_in_two() {
        let batches = [
            (GuiBlend::Translucent, 0, 12),
            (GuiBlend::Items, 0, 30),
            (GuiBlend::Translucent, 12, 18),
        ];
        let (before, after) = split_batches(&batches, (18, 30));
        assert_eq!(
            before,
            [
                (GuiBlend::Translucent, 0, 12),
                (GuiBlend::Items, 0, 30),
                (GuiBlend::Translucent, 12, 6)
            ]
        );
        assert_eq!(after, [(GuiBlend::Translucent, 18, 12)]);
    }

    #[test]
    fn items_split_by_their_own_count() {
        let batches = [(GuiBlend::Entity, 0, 10), (GuiBlend::Entity, 10, 10)];
        let (before, after) = split_batches(&batches, (999, 10));
        assert_eq!(before, [(GuiBlend::Entity, 0, 10)]);
        assert_eq!(after, [(GuiBlend::Entity, 10, 10)]);
    }

    #[test]
    fn blur_needs_a_blurriness_of_one_and_is_asked_once() {
        let mut frame = GuiFrame::default();
        frame.blur_before_this_stratum(0);
        assert_eq!(frame.blur, None);
        frame.blur_before_this_stratum(5);
        frame.blur_before_this_stratum(9);
        assert_eq!(frame.blur.map(|blur| blur.radius), Some(5));
    }
}
