use super::*;
use crate::joml::Matrix3x2;

/// A decoration of a map as the GUI draws it (`MapRenderState.MapDecorationRenderState`).
#[derive(Debug, Clone, PartialEq)]
pub struct GuiMapDecoration {
    pub x: i8,
    pub y: i8,
    pub rot: i8,
    /// Whether the decoration type shows on an item frame, which is what the GUI draws.
    pub shows: bool,
    /// The sprite's name in the decorations' atlas, and where it is in the picture it is sampled
    /// from (`[u0, v0, u1, v1]`).
    pub sprite: String,
    pub uv: [f32; 4],
    pub name: Option<Vec<TextRun<'static>>>,
}

/// A map as the GUI draws it: its colours and where its picture is in the maps' image.
#[derive(Debug, Clone, PartialEq)]
pub struct GuiMap {
    pub id: i32,
    /// `MapItemSavedData.scale` and `locked`.
    pub scale: i8,
    pub locked: bool,
    /// The packed colours, row by row.
    pub colors: Vec<u8>,
    /// Where the map's cell is in the maps' image (`[u0, v0, u1, v1]`).
    pub cell: [f32; 4],
    pub decorations: Vec<GuiMapDecoration>,
}

impl GuiMap {
    /// The name the draw log gives the map's texture: its place and a hash of the colours it
    /// holds as the texture holds them (FNV-1a over each colour's ARGB, big end first).
    pub fn texture_name(&self) -> String {
        let mut hash: u32 = 0x811C_9DC5;
        for packed in &self.colors {
            for byte in photon_data::map::packed_color(*packed).to_be_bytes() {
                hash = (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193);
            }
        }
        format!("map/{}/{hash:08x}", self.id)
    }
}

impl GuiFrame {
    /// `GuiGraphicsExtractor.map`: the map's picture over 128 by 128 under `pose`, then the
    /// decorations that show on a frame, each turned and placed by its own pose, with its name
    /// under it.
    pub fn map(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        map: &GuiMap,
        pose: &Matrix3x2,
    ) {
        let name = if self.log.is_some() {
            map.texture_name()
        } else {
            String::new()
        };
        self.map_quad(
            (GuiBlend::Map, &name),
            ((0.0, 1.0, 0.0, 1.0), map.cell),
            (0.0, 0.0, 128.0, 128.0),
            pose,
        );
        for decoration in map.decorations.iter().filter(|decoration| decoration.shows) {
            let (dx, dy) = (
                f32::from(decoration.x) / 2.0 + 64.0,
                f32::from(decoration.y) / 2.0 + 64.0,
            );
            let mut at = *pose;
            at.translate(dx, dy);
            at.rotate(
                (std::f64::consts::PI / 180.0) as f32 * f32::from(decoration.rot) * 360.0 / 16.0,
            );
            at.scale(4.0, 4.0);
            at.translate(-0.125, 0.125);
            let [u0, v0, u1, v1] = decoration.uv;
            self.map_quad(
                (
                    GuiBlend::MapDecoration,
                    &format!("map_decoration/{}", decoration.sprite),
                ),
                ((0.0, 1.0, 1.0, 0.0), [u0, v1, u1, v0]),
                (-1.0, -1.0, 1.0, 1.0),
                &at,
            );
            if let Some(runs) = &decoration.name {
                self.map_name(atlas, font, runs, (dx, dy), pose);
            }
        }
    }

    /// A decoration's name, scaled to fit under it.
    fn map_name(
        &mut self,
        atlas: &GuiAtlas,
        font: &crate::gui::font::Font,
        runs: &[TextRun<'static>],
        (dx, dy): (f32, f32),
        pose: &Matrix3x2,
    ) {
        let width = font.width_of_text_runs(runs) as f32;
        let scale = (25.0 / width).clamp(0.0, 6.0 / 9.0);
        let mut at = *pose;
        at.translate(dx - width * scale / 2.0, dy + 4.0);
        at.scale(scale, scale);
        let mark = self.mark();
        self.styled_text_runs_on(atlas, font, runs, (0, 0), (0xFFFF_FFFF, false, 0x8000_0000));
        self.transform_from(mark, at.m);
    }

    /// One picture of a map screen: its log entry (the texture's name and the part of it the call
    /// names) and its quad (the part of the sampled image it shows), through `pose`.
    fn map_quad(
        &mut self,
        (blend, name): (GuiBlend, &str),
        (log_uv, [u0, v0, u1, v1]): ((f32, f32, f32, f32), [f32; 4]),
        (x0, y0, x1, y1): (f32, f32, f32, f32),
        pose: &Matrix3x2,
    ) {
        let [m00, m01, m10, m11, m20, m21] = pose.m;
        let place = |x: f32, y: f32| (m00 * x + m10 * y + m20, m01 * x + m11 * y + m21);
        if let Some(log) = &mut self.log {
            let (px0, py0) = place(x0, y0);
            let (px1, py1) = place(x1, y1);
            log.push(DrawOp::Texture {
                name: name.to_owned(),
                rect: (px0, py0, px1, py1),
                uv: log_uv,
                colour: 0xFFFF_FFFF,
            });
        }
        let corner = |x: f32, y: f32, uv: [f32; 2]| {
            let (px, py) = place(x, y);
            GuiVertex {
                position: [px, py],
                uv,
                colour: [1.0; 4],
            }
        };
        let corners = [
            corner(x0, y0, [u0, v0]),
            corner(x0, y1, [u0, v1]),
            corner(x1, y1, [u1, v1]),
            corner(x1, y0, [u1, v0]),
        ];
        let first = self.vertices.len() as u32;
        for index in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(corners[index]);
        }
        self.batches.push((blend, first, 6));
    }
}
