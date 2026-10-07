use super::*;

impl DisplayState {
    /// The state of a display of this kind before any data has come.
    pub(crate) fn new(kind: DisplayKind) -> Self {
        Self {
            kind,
            ..Self::default()
        }
    }

    /// `onSyncedDataUpdated` for each field the data holds: what it changes is noted for the tick.
    pub(crate) fn apply_metadata(&mut self, metadata: &[u8]) {
        if let Some(value) = metadata_int_value(metadata, START_DELTA_DATA_ID) {
            self.start_delta = value;
            self.update_start_tick = true;
        }
        if let Some(value) = metadata_int_value(metadata, INTERPOLATION_DURATION_DATA_ID) {
            self.interpolation_duration_data = value;
            self.update_interpolation_duration = true;
        }
        if let Some(value) = metadata_int_value(metadata, POS_ROT_DURATION_DATA_ID) {
            self.pos_rot_duration = value;
        }
        if let Some(value) = floats(metadata, TRANSLATION_DATA_ID, VECTOR3_SERIALIZER_ID) {
            self.translation = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, SCALE_DATA_ID, VECTOR3_SERIALIZER_ID) {
            self.scale = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, LEFT_ROTATION_DATA_ID, QUATERNION_SERIALIZER_ID) {
            self.left_rotation = value;
            self.update_render_state = true;
        }
        if let Some(value) = floats(metadata, RIGHT_ROTATION_DATA_ID, QUATERNION_SERIALIZER_ID) {
            self.right_rotation = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_byte_value(metadata, BILLBOARD_DATA_ID) {
            self.billboard = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_int_value(metadata, BRIGHTNESS_DATA_ID) {
            self.brightness = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, VIEW_RANGE_DATA_ID) {
            self.view_range = value;
        }
        if let Some(value) = metadata_float_value(metadata, SHADOW_RADIUS_DATA_ID) {
            self.shadow_radius = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, SHADOW_STRENGTH_DATA_ID) {
            self.shadow_strength = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_float_value(metadata, WIDTH_DATA_ID) {
            self.width = value;
        }
        if let Some(value) = metadata_float_value(metadata, HEIGHT_DATA_ID) {
            self.height = value;
        }
        if let Some(value) = metadata_int_value(metadata, GLOW_COLOR_DATA_ID) {
            self.glow_color = value;
        }
    }

    /// A text display's data: what changes any of it makes the render state again.
    pub(crate) fn apply_text(&mut self, metadata: &[u8]) {
        if let Some((COMPONENT_SERIALIZER_ID, value)) = find_metadata_field(metadata, TEXT_DATA_ID)
        {
            let mut rest = value;
            if voxel_core::nbt::read_any_nbt(&mut rest).is_ok() {
                self.text.text = value[..value.len() - rest.len()].to_vec();
                self.update_render_state = true;
            }
        }
        if let Some(value) = metadata_int_value(metadata, LINE_WIDTH_DATA_ID) {
            self.text.line_width = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_int_value(metadata, BACKGROUND_DATA_ID) {
            self.text.background = value;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_byte_value(metadata, TEXT_OPACITY_DATA_ID) {
            self.text.opacity = value as i8;
            self.update_render_state = true;
        }
        if let Some(value) = metadata_byte_value(metadata, STYLE_FLAGS_DATA_ID) {
            self.text.flags = value as i8;
            self.update_render_state = true;
        }
    }

    /// A block display's block state data (`DATA_BLOCK_STATE_ID`).
    pub(crate) fn apply_block_state(&mut self, metadata: &[u8]) {
        if let Some((BLOCK_STATE_SERIALIZER_ID, mut value)) =
            find_metadata_field(metadata, BLOCK_STATE_DATA_ID)
            && let Some(id) = voxel_core::wire::read_varint(&mut value)
                .ok()
                .and_then(|id| u32::try_from(id).ok())
        {
            self.block_state = id;
            self.update_render_state = true;
        }
    }

    /// An item display's context data (`DATA_ITEM_DISPLAY_ID`).
    pub(crate) fn apply_item_context(&mut self, metadata: &[u8]) {
        if let Some(value) = metadata_byte_value(metadata, ITEM_DISPLAY_DATA_ID) {
            self.item_display = value;
            self.update_render_state = true;
        }
    }

    /// An item display's item changed (`DATA_ITEM_STACK_ID`).
    pub(crate) fn item_changed(&mut self) {
        self.update_render_state = true;
    }

    /// `Display.createTransformation`.
    pub(super) fn transformation(&self) -> Transformation {
        Transformation {
            translation: self.translation,
            left_rotation: self.left_rotation,
            scale: self.scale,
            right_rotation: self.right_rotation,
        }
    }

    /// The length of the position and turn interpolation (`getPosRotInterpolationDuration`).
    pub fn pos_rot_duration(&self) -> i32 {
        self.pos_rot_duration
    }

    /// `Display.tick`'s client half, but for the position's easing: the notes the data left are
    /// acted on. `tick_count` is the entity's.
    pub(crate) fn tick(&mut self, tick_count: i32) {
        if self.update_start_tick {
            self.update_start_tick = false;
            self.interpolation_start_tick = i64::from(tick_count) + i64::from(self.start_delta);
        }
        if self.update_interpolation_duration {
            self.update_interpolation_duration = false;
            self.interpolation_duration = self.interpolation_duration_data;
        }
        if self.update_render_state {
            self.update_render_state = false;
            let should_interpolate = self.interpolation_duration != 0;
            let progress = f32::from_bits(self.last_progress.get());
            self.render_state = Some(match self.render_state {
                Some(previous) if should_interpolate => {
                    self.interpolated_render_state(&previous, progress)
                }
                _ => self.fresh_render_state(),
            });
            // `updateRenderSubState`.
            match self.kind {
                DisplayKind::Block => self.block_render = Some(self.block_state),
                DisplayKind::Item => self.item_render = Some(self.item_display),
                DisplayKind::Text => {
                    let text = &self.text;
                    self.text_render = Some(match &self.text_render {
                        Some(previous) if should_interpolate => TextRenderState {
                            text: text.text.clone(),
                            line_width: text.line_width,
                            text_opacity: IntInterpolator::Linear(
                                previous.text_opacity.get(progress),
                                i32::from(text.opacity),
                            ),
                            background: IntInterpolator::Color(
                                previous.background.get(progress),
                                text.background,
                            ),
                            flags: text.flags,
                        },
                        _ => TextRenderState {
                            text: text.text.clone(),
                            line_width: text.line_width,
                            text_opacity: IntInterpolator::Constant(i32::from(text.opacity)),
                            background: IntInterpolator::Constant(text.background),
                            flags: text.flags,
                        },
                    });
                    self.text_version = self.text_version.wrapping_add(1);
                }
            }
        }
    }

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

    /// `BlockDisplay.blockRenderState`.
    pub fn block_state(&self) -> Option<u32> {
        self.block_render
    }

    /// `ItemDisplay.itemRenderState`'s `ItemDisplayContext` id.
    pub fn item_context(&self) -> Option<u8> {
        self.item_render
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
