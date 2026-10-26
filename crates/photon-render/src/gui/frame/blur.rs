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

/// A rectangle `(left, top, right, bottom)`.
pub type ClipRect = (i32, i32, i32, i32);

/// A scissor over item vertices: the first and end of the run, and the rectangle.
pub type ItemScissor = (u32, u32, ClipRect);

/// A run of item vertices cut at the edges of the scissors over it: each piece with the
/// intersection of the scissors that cover it, or none where no scissor does.
pub fn scissor_pieces(
    (first, count): (u32, u32),
    scissors: &[ItemScissor],
) -> Vec<(u32, u32, Option<ClipRect>)> {
    let end = first + count;
    let mut cuts = vec![first, end];
    for &(from, to, _) in scissors {
        cuts.extend(
            [from, to]
                .into_iter()
                .filter(|cut| (first..=end).contains(cut)),
        );
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .map(|pair| {
            let rect = scissors
                .iter()
                .filter(|&&(from, to, _)| from <= pair[0] && pair[1] <= to)
                .map(|&(_, _, rect)| rect)
                .reduce(|a, b| (a.0.max(b.0), a.1.max(b.1), a.2.min(b.2), a.3.min(b.3)));
            (pair[0], pair[1] - pair[0], rect)
        })
        .collect()
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

    #[test]
    fn a_run_is_cut_where_a_scissor_starts_and_ends() {
        let scissors = [(6, 12, (0, 0, 10, 10))];
        assert_eq!(
            scissor_pieces((0, 18), &scissors),
            [(0, 6, None), (6, 6, Some((0, 0, 10, 10))), (12, 6, None)]
        );
        assert_eq!(scissor_pieces((0, 3), &[]), [(0, 3, None)]);
    }
}
