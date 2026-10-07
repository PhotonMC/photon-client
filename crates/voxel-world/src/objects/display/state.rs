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

    /// `BlockDisplay.blockRenderState`.
    pub fn block_state(&self) -> Option<u32> {
        self.block_render
    }

    /// `ItemDisplay.itemRenderState`'s `ItemDisplayContext` id.
    pub fn item_context(&self) -> Option<u8> {
        self.item_render
    }
}

/// A display's data and what it has made of it.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayState {
    pub(super) kind: DisplayKind,
    pub(super) start_delta: i32,
    pub(super) interpolation_duration_data: i32,
    pub(super) pos_rot_duration: i32,
    pub(super) translation: [f32; 3],
    pub(super) scale: [f32; 3],
    pub(super) left_rotation: [f32; 4],
    pub(super) right_rotation: [f32; 4],
    pub(super) billboard: u8,
    pub(super) brightness: i32,
    pub(super) view_range: f32,
    pub(super) shadow_radius: f32,
    pub(super) shadow_strength: f32,
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) glow_color: i32,
    pub(super) block_state: u32,
    pub(super) item_display: u8,
    pub(super) interpolation_start_tick: i64,
    pub(super) interpolation_duration: i32,
    /// `lastProgress`, as its bits: asking for a progress remembers it.
    pub(super) last_progress: Cell<u32>,
    pub(super) update_render_state: bool,
    pub(super) update_start_tick: bool,
    pub(super) update_interpolation_duration: bool,
    pub(super) render_state: Option<RenderState>,
    /// `BlockDisplay.blockRenderState`: the block it shows.
    pub(super) block_render: Option<u32>,
    /// `ItemDisplay.itemRenderState`'s context (the stack is the entity's).
    pub(super) item_render: Option<u8>,
    /// A text display's data, and the render state made of it, with how many times it was made
    /// (the lines split from its text are kept until then).
    pub(super) text: TextData,
    pub(super) text_render: Option<TextRenderState>,
    pub(super) text_version: u32,
}

/// A text display's own data (`Display.TextDisplay`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TextData {
    pub(super) text: Vec<u8>,
    pub(super) line_width: i32,
    pub(super) background: i32,
    pub(super) opacity: i8,
    pub(super) flags: i8,
}

impl Default for TextData {
    fn default() -> Self {
        Self {
            // `Component.empty()`: a string tag with no text.
            text: vec![8, 0, 0],
            line_width: 200,
            background: 1_073_741_824,
            opacity: -1,
            flags: 0,
        }
    }
}

// The data is made of finite numbers: never NaN.
impl Eq for DisplayState {}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            kind: DisplayKind::Block,
            start_delta: 0,
            interpolation_duration_data: 0,
            pos_rot_duration: 0,
            translation: [0.0; 3],
            scale: [1.0; 3],
            left_rotation: [0.0, 0.0, 0.0, 1.0],
            right_rotation: [0.0, 0.0, 0.0, 1.0],
            billboard: 0,
            brightness: NO_BRIGHTNESS_OVERRIDE,
            view_range: 1.0,
            shadow_radius: 0.0,
            shadow_strength: 1.0,
            width: 0.0,
            height: 0.0,
            glow_color: -1,
            block_state: 0,
            item_display: 0,
            interpolation_start_tick: -2_147_483_648,
            interpolation_duration: 0,
            last_progress: Cell::new(0.0_f32.to_bits()),
            update_render_state: false,
            update_start_tick: false,
            update_interpolation_duration: false,
            render_state: None,
            block_render: None,
            item_render: None,
            text: TextData::default(),
            text_render: None,
            text_version: 0,
        }
    }
}

/// A vector of `N` floats a data value holds, if the field is one.
pub(super) fn floats<const N: usize>(
    metadata: &[u8],
    data_id: u8,
    serializer: i32,
) -> Option<[f32; N]> {
    let (found, value) = find_metadata_field(metadata, data_id)?;
    if found != serializer {
        return None;
    }
    let mut out = [0.0; N];
    for (index, slot) in out.iter_mut().enumerate() {
        *slot = f32::from_be_bytes(value.get(index * 4..index * 4 + 4)?.try_into().ok()?);
    }
    Some(out)
}
