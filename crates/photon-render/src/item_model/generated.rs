use super::*;

/// What an item's tints read of its components (`DyedItemColor`, `PotionContents`, `MapItemColor`,
/// `FireworkExplosion`); the game's item values answer it.
pub trait ItemTints {
    fn dyed_color(&self) -> Option<i32>;
    /// The colour of the potion contents, or `default` without any.
    fn potion_color_or(&self, default: i32) -> i32;
    fn map_color(&self) -> Option<i32>;
    fn firework_colors(&self) -> Vec<i32>;
}

/// A tint's colour, ARGB. `grass` reads the grass colour map.
pub fn tint_colour(
    tint: &TintSource,
    grass: &dyn Fn(f64, f64) -> u32,
    values: &dyn ItemTints,
) -> u32 {
    match *tint {
        TintSource::Constant(rgb) | TintSource::Default(rgb) => 0xFF00_0000 | rgb,
        TintSource::Component { source, default } => {
            let default = default as i32;
            0xFF00_0000
                | (match source {
                    ComponentTint::Dye => values.dyed_color().unwrap_or(default),
                    ComponentTint::Potion => values.potion_color_or(default),
                    ComponentTint::MapColor => values.map_color().unwrap_or(default),
                    ComponentTint::Firework => {
                        let colors = values.firework_colors();
                        match colors.len() {
                            0 => default,
                            1 => colors[0],
                            count => {
                                let total = |shift: i32| {
                                    colors
                                        .iter()
                                        .map(|color| color >> shift & 0xFF)
                                        .sum::<i32>()
                                        / count as i32
                                };
                                total(16) << 16 | total(8) << 8 | total(0)
                            }
                        }
                    }
                } as u32
                    & 0xFF_FFFF)
        }
        TintSource::Grass {
            temperature,
            downfall,
        } => 0xFF00_0000 | grass(f64::from(temperature), f64::from(downfall)),
    }
}

/// The elements the item model generator extrudes a layered flat item
/// into: each layer's front and back, then a thin side face along every
/// edge between an opaque and a transparent pixel. `layer_alpha` answers a
/// layer's texture alpha and size.
pub fn generated_elements(
    textures: &crate::model::TextureSlots,
    layer_alpha: &dyn Fn(&Identifier) -> Option<LayerAlpha>,
) -> Vec<crate::model::ModelElement> {
    use crate::block::geometry::{FaceUvs, Quadrant};
    use crate::model::{ModelElement, ModelFace};
    use photon_core::direction::Direction;
    let mut elements = Vec::new();
    let element =
        |from: [f32; 3], to: [f32; 3], facing: Direction, slot: &str, layer: i32, uvs: FaceUvs| {
            ModelElement {
                from,
                to,
                rotation: None,
                shade: true,
                light_emission: 0,
                faces: vec![(
                    facing,
                    ModelFace {
                        cull_for_direction: None,
                        tint_index: layer,
                        texture: format!("#{slot}"),
                        uvs: Some(uvs),
                        rotation: Quadrant::R0,
                    },
                )],
            }
        };
    for (layer, slot) in ["layer0", "layer1", "layer2", "layer3", "layer4"]
        .iter()
        .enumerate()
    {
        let Some(material) = textures.material(slot) else {
            break;
        };
        let layer = layer as i32;
        let (from, to) = ([0.0, 0.0, 7.5], [16.0, 16.0, 8.5]);
        elements.push(element(
            from,
            to,
            Direction::South,
            slot,
            layer,
            FaceUvs {
                min_u: 0.0,
                min_v: 0.0,
                max_u: 16.0,
                max_v: 16.0,
            },
        ));
        elements.push(element(
            from,
            to,
            Direction::North,
            slot,
            layer,
            FaceUvs {
                min_u: 16.0,
                min_v: 0.0,
                max_u: 0.0,
                max_v: 16.0,
            },
        ));
        let Some((alpha, width, height)) = layer_alpha(&material.sprite) else {
            continue;
        };
        let x_scale = 16.0 / width as f32;
        let y_scale = 16.0 / height as f32;
        for (facing, x, y) in side_faces(&alpha, width, height) {
            let (x, y) = (x as f32, y as f32);
            let u0 = x + 0.1;
            let u1 = x + 1.0 - 0.1;
            let (v0, v1) = if matches!(facing, SideDirection::Up | SideDirection::Down) {
                (y + 0.1, y + 1.0 - 0.1)
            } else {
                (y + 1.0 - 0.1, y + 0.1)
            };
            let (mut start_x, mut start_y, mut end_x, mut end_y) = (x, y, x, y);
            match facing {
                SideDirection::Up => end_x += 1.0,
                SideDirection::Down => {
                    end_x += 1.0;
                    start_y += 1.0;
                    end_y += 1.0;
                }
                SideDirection::Left => end_y += 1.0,
                SideDirection::Right => {
                    start_x += 1.0;
                    end_x += 1.0;
                    end_y += 1.0;
                }
            }
            start_x *= x_scale;
            end_x *= x_scale;
            start_y = 16.0 - start_y * y_scale;
            end_y = 16.0 - end_y * y_scale;
            let (from, to) = match facing {
                SideDirection::Up => ([start_x, start_y, 7.5], [end_x, start_y, 8.5]),
                SideDirection::Down => ([start_x, end_y, 7.5], [end_x, end_y, 8.5]),
                SideDirection::Left => ([start_x, start_y, 7.5], [start_x, end_y, 8.5]),
                SideDirection::Right => ([end_x, start_y, 7.5], [end_x, end_y, 8.5]),
            };
            elements.push(element(
                from,
                to,
                facing.direction(),
                slot,
                layer,
                FaceUvs {
                    min_u: u0 * x_scale,
                    min_v: v0 * y_scale,
                    max_u: u1 * x_scale,
                    max_v: v1 * y_scale,
                },
            ));
        }
    }
    elements
}
