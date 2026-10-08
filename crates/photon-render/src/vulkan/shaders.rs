//! The shaders the pipelines are built from. Their GLSL is in `shaders/` and `build.rs` compiles
//! it to SPIR-V at build time; the scene vertex shaders share `push.glsl` and `lightmap.glsl`.

/// One shader, compiled to SPIR-V.
pub(super) struct Shader(&'static [u8]);

impl Shader {
    /// The SPIR-V words, as a shader module takes them.
    pub(super) fn words(&self) -> Vec<u32> {
        ash::util::read_spv(&mut std::io::Cursor::new(self.0))
            .expect("`build.rs` writes whole SPIR-V words")
    }
}

macro_rules! shader {
    ($file:literal) => {
        Shader(include_bytes!(concat!(env!("OUT_DIR"), "/", $file, ".spv")))
    };
}

pub(super) const SECTION_VERTEX_SHADER: Shader = shader!("section.vert");
pub(super) const VERTEX_SHADER: Shader = shader!("scene.vert");
pub(super) const HELD_VERTEX_SHADER: Shader = shader!("held.vert");
pub(super) const ENTITY_VERTEX_SHADER: Shader = shader!("entity.vert");
pub(super) const ENTITY_FRAGMENT_SHADER: Shader = shader!("entity.frag");
pub(super) const TEXT_VERTEX_SHADER: Shader = shader!("text.vert");
pub(super) const SWIRL_VERTEX_SHADER: Shader = shader!("swirl.vert");
pub(super) const SWIRL_FRAGMENT_SHADER: Shader = shader!("swirl.frag");
pub(super) const GLINT_VERTEX_SHADER: Shader = shader!("glint.vert");
pub(super) const GLINT_FRAGMENT_SHADER: Shader = shader!("glint.frag");
pub(super) const DISSOLVE_FRAGMENT_SHADER: Shader = shader!("dissolve.frag");
pub(super) const RAYS_VERTEX_SHADER: Shader = shader!("rays.vert");
pub(super) const RAYS_FRAGMENT_SHADER: Shader = shader!("rays.frag");
pub(super) const SHADOW_VERTEX_SHADER: Shader = shader!("shadow.vert");
pub(super) const SHADOW_FRAGMENT_SHADER: Shader = shader!("shadow.frag");
pub(super) const TEXT_SEE_THROUGH_VERTEX_SHADER: Shader = shader!("text_see_through.vert");
pub(super) const TEXT_FRAGMENT_SHADER: Shader = shader!("text.frag");
pub(super) const TEXT_SEE_THROUGH_FRAGMENT_SHADER: Shader = shader!("text_see_through.frag");
pub(super) const LINE_VERTEX_SHADER: Shader = shader!("line.vert");
pub(super) const LINE_FRAGMENT_SHADER: Shader = shader!("line.frag");
pub(super) const CRUMBLING_FRAGMENT_SHADER: Shader = shader!("crumbling.frag");
pub(super) const GUI_VERTEX_SHADER: Shader = shader!("gui.vert");
pub(super) const GUI_FRAGMENT_SHADER: Shader = shader!("gui.frag");
pub(super) const GUI_ITEM_VERTEX_SHADER: Shader = shader!("gui_item.vert");
pub(super) const GUI_ITEM_FRAGMENT_SHADER: Shader = shader!("gui_item.frag");
pub(super) const FRAGMENT_SHADER: Shader = shader!("scene.frag");
pub(super) const SKY_VERTEX_SHADER: Shader = shader!("sky.vert");
pub(super) const SKY_FRAGMENT_SHADER: Shader = shader!("sky.frag");
pub(super) const CLOUD_VERTEX_SHADER: Shader = shader!("cloud.vert");
pub(super) const CLOUD_FRAGMENT_SHADER: Shader = shader!("cloud.frag");
pub(super) const WEATHER_FRAGMENT_SHADER: Shader = shader!("weather.frag");
