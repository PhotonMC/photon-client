//! The reference's render types an entity layer asks for, as the facts the
//! layers and the pipelines they name settle: which texture, which shader
//! switches, how the colour is blended and the depth tested. The renderer
//! draws what these say; the parity run compares them with the reference's
//! own render types.

/// What a render type's pipeline is made of (`RenderPipeline`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PipelineState {
    /// The pipeline's location (`pipeline/eyes`).
    pub location: &'static str,
    /// The shader's switches, sorted, joined by commas.
    pub flags: &'static str,
    /// The shader's values (`ALPHA_CUTOUT=0.1,`), sorted by name.
    pub values: &'static str,
    /// `TRANSLUCENT`, `ADDITIVE` or `NONE`.
    pub blend: &'static str,
    /// The depth test's comparison.
    pub depth_test: &'static str,
    pub depth_write: bool,
    pub cull: bool,
}

/// `RenderPipelines.EYES`: the entity shader with no light, no overlay and no
/// per-face shading, blended and tested against the depth but not written to it.
pub const EYES: PipelineState = PipelineState {
    location: "minecraft:pipeline/eyes",
    flags: "EMISSIVE,NO_CARDINAL_LIGHTING,NO_OVERLAY",
    values: "",
    blend: "TRANSLUCENT",
    depth_test: "GREATER_THAN_OR_EQUAL",
    depth_write: false,
    cull: true,
};

/// `EyesLayer` draws after the layers of the ordinary order (`order(1)`).
pub const EYES_ORDER: i32 = 1;

/// `OverlayTexture.NO_OVERLAY`: the overlay coordinates of a draw that has none.
pub const NO_OVERLAY: i32 = 655_360;

/// The texture (under `textures/`) of the glowing eyes drawn over an entity of
/// this type: `SpiderEyesLayer`, `EnderEyesLayer` and `PhantomEyesLayer`.
pub fn eyes_texture(entity_type: &str) -> Option<&'static str> {
    match entity_type {
        "minecraft:spider" | "minecraft:cave_spider" => Some("entity/spider/spider_eyes"),
        "minecraft:enderman" => Some("entity/enderman/enderman_eyes"),
        "minecraft:phantom" => Some("entity/phantom/phantom_eyes"),
        _ => None,
    }
}

/// `RenderPipelines.ENERGY_SWIRL`: the entity shader with no light, no overlay
/// and no per-face shading, its texture coordinates moved by the layer's
/// offset, cut out below a tenth of alpha, added to what is behind, drawn on
/// both faces and written to the depth.
pub const ENERGY_SWIRL: PipelineState = PipelineState {
    location: "minecraft:pipeline/energy_swirl",
    flags: "APPLY_TEXTURE_MATRIX,EMISSIVE,NO_CARDINAL_LIGHTING,NO_OVERLAY",
    values: "ALPHA_CUTOUT=0.1,",
    blend: "ADDITIVE",
    depth_test: "GREATER_THAN_OR_EQUAL",
    depth_write: true,
    cull: false,
};

/// `CreeperPowerLayer`: the texture the charge is drawn in and the colour it
/// is multiplied by (`-8355712`, 0xFF808080).
pub const POWER_TEXTURE: &str = "entity/creeper/creeper_armor";
pub const POWER_COLOUR: i32 = -8_355_712;

/// `EnergySwirlLayer.submit`'s texture offset at an age: both coordinates
/// `t * 0.01 % 1` (`CreeperPowerLayer.xOffset`).
pub fn swirl_offset(age_in_ticks: f32) -> [f32; 2] {
    let offset = age_in_ticks * 0.01 % 1.0;
    [offset, offset]
}

/// `RenderPipelines.GLINT`: the glint's own shader (the texture moved by the texture matrix, cut
/// out below a tenth of alpha, faded by fog and the glint strength), added to what is behind by
/// its own colour, drawn on both faces over exactly the depth the model wrote.
pub const GLINT: PipelineState = PipelineState {
    location: "minecraft:pipeline/glint",
    flags: "",
    values: "",
    blend: "GLINT",
    depth_test: "EQUAL",
    depth_write: false,
    cull: false,
};

/// `ItemFeatureRenderer.ENCHANTED_GLINT_ARMOR`: the texture (under `textures/`) the glint on armor
/// is drawn in, and the scale of `TextureTransform.ARMOR_ENTITY_GLINT_TEXTURING`.
pub const ARMOR_GLINT_TEXTURE: &str = "misc/enchanted_glint_armor";
pub const ARMOR_GLINT_SCALE: f32 = 0.16;

/// `TextureTransform.setupGlintTexturing`: the matrix (as `Matrix4f.get(float[])` gives it, column
/// by column) that moves the glint's texture at a time (`Util.getMillis`) and glint speed (the
/// option): translated by two slow offsets, turned a tenth of a half turn about z and scaled.
pub fn glint_texture_matrix(millis: i64, glint_speed: f64, scale: f32) -> [f32; 16] {
    let millis = (millis as f64 * glint_speed * 8.0) as i64;
    let layer_offset0 = (millis % 110_000) as f32 / 110_000.0;
    let layer_offset1 = (millis % 30_000) as f32 / 30_000.0;
    let angle = (std::f64::consts::PI / 18.0) as f32;
    let sin = crate::joml::sin(angle);
    let cos = crate::joml::cos_from_sin(sin, angle);
    // translation(-o0, o1, 0), rotateZ keeping it, scale(s) on the three columns.
    [
        cos * scale,
        sin * scale,
        0.0 * scale,
        0.0,
        -sin * scale,
        cos * scale,
        0.0 * scale,
        0.0,
        0.0 * scale,
        0.0 * scale,
        scale,
        0.0,
        -layer_offset0,
        layer_offset1,
        0.0,
        1.0,
    ]
}

/// A glint's texture coordinate for a model's (`TextureMat * vec4(UV0, 0, 1)`).
pub fn glint_uv(matrix: &[f32; 16], [u, v]: [f32; 2]) -> [f32; 2] {
    [
        matrix[0] * u + matrix[4] * v + matrix[12],
        matrix[1] * u + matrix[5] * v + matrix[13],
    ]
}
