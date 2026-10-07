use super::*;

impl DisplayState {
    /// `createFreshRenderState`.
    pub(super) fn fresh_render_state(&self) -> RenderState {
        RenderState {
            transformation: TransformationInterpolator::Constant(self.transformation()),
            billboard: Billboard::from_id(self.billboard),
            brightness_override: self.brightness,
            shadow_radius: FloatInterpolator::Constant(self.shadow_radius),
            shadow_strength: FloatInterpolator::Constant(self.shadow_strength),
            glow_color_override: self.glow_color,
        }
    }

    /// `createInterpolatedRenderState`: from where the last one had got to.
    pub(super) fn interpolated_render_state(
        &self,
        previous: &RenderState,
        progress: f32,
    ) -> RenderState {
        RenderState {
            transformation: TransformationInterpolator::Slerp(
                previous.transformation.get(progress),
                self.transformation(),
            ),
            billboard: Billboard::from_id(self.billboard),
            brightness_override: self.brightness,
            shadow_radius: FloatInterpolator::Linear(
                previous.shadow_radius.get(progress),
                self.shadow_radius,
            ),
            shadow_strength: FloatInterpolator::Linear(
                previous.shadow_strength.get(progress),
                self.shadow_strength,
            ),
            glow_color_override: self.glow_color,
        }
    }

    /// `calculateInterpolationProgress`: how far the easing of the render state has gone at a
    /// part tick (and it is remembered for the next render state to ease from).
    pub fn interpolation_progress(&self, tick_count: i32, partial: f32) -> f32 {
        let duration = self.interpolation_duration;
        if duration <= 0 {
            return 1.0;
        }
        let since_update = (i64::from(tick_count) - self.interpolation_start_tick) as f32;
        let progress = mth_inverse_lerp_f32(since_update + partial, 0.0, duration as f32);
        let progress = if progress < 0.0 {
            0.0
        } else {
            progress.min(1.0)
        };
        self.last_progress.set(progress.to_bits());
        progress
    }

    /// `Display.renderState`: none until some data has changed.
    pub fn render_state(&self) -> Option<&RenderState> {
        self.render_state.as_ref()
    }

    /// `TextDisplay.textRenderState`, and how many times it has been made (the lines split from the
    /// text are good until it changes).
    pub fn text_render_state(&self) -> Option<(&TextRenderState, u32)> {
        self.text_render
            .as_ref()
            .map(|state| (state, self.text_version))
    }

    /// `Display.shouldRenderAtSqrDistance`: whether it is near enough to be drawn (`view_scale` is
    /// `Entity.getViewScale`).
    pub fn should_render_at_sqr_distance(&self, distance_sqr: f64, view_scale: f64) -> bool {
        let range = f64::from(self.view_range) * 64.0 * view_scale;
        distance_sqr < range * range
    }

    /// `Display.getWidth` and `getHeight`.
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

/// `Display.TextDisplay.TextRenderState`: the text (a component, as the network carried it), how
/// wide its lines are, how opaque and what background, and the style flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRenderState {
    pub text: Vec<u8>,
    pub line_width: i32,
    pub text_opacity: IntInterpolator,
    pub background: IntInterpolator,
    pub flags: i8,
}

/// `Display.RenderState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderState {
    pub transformation: TransformationInterpolator,
    pub billboard: Billboard,
    pub brightness_override: i32,
    pub shadow_radius: FloatInterpolator,
    pub shadow_strength: FloatInterpolator,
    pub glow_color_override: i32,
}
