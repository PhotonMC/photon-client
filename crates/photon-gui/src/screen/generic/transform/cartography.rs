use super::*;
use photon_game::menus::container::{CartographyLayout, MapFacts, map_id_of};
use photon_render::joml::Matrix3x2;

impl GenericScreen {
    /// `CartographyTableScreen.extractBackground` after the background: the error mark, the
    /// picture for the second slot's item and the map in it.
    pub(super) fn cartography_preview(
        &self,
        draw: &mut ScreenDraw<'_, '_>,
        menu: &ContainerMenu,
        (left, top): (i32, i32),
    ) {
        let id = menu
            .slots
            .first()
            .and_then(Option::as_ref)
            .and_then(map_id_of);
        let map = id.and_then(|id| (draw.maps)(id));
        let view = menu.cartography_view(&|_| {
            map.as_ref().map(|map| MapFacts {
                scale: map.scale,
                locked: map.locked,
            })
        });
        let sprite =
            |draw: &mut ScreenDraw<'_, '_>, name: &str, (x, y, w, h): (i32, i32, i32, i32)| {
                draw.frame.sprite(
                    draw.atlas,
                    &format!("container/cartography_table/{name}"),
                    left + x,
                    top + y,
                    w,
                    h,
                );
            };
        let show = |draw: &mut ScreenDraw<'_, '_>, (x, y): (i32, i32), scale: f32| {
            if let Some(map) = &map {
                let mut pose = Matrix3x2::default();
                pose.translate((left + x) as f32, (top + y) as f32);
                pose.scale(scale, scale);
                draw.frame.map(draw.atlas, draw.font, map, &pose);
            }
        };
        for _ in 0..view.error_marks {
            sprite(draw, "error", (35, 31, 28, 21));
        }
        for (name, rect, placed) in pictures(view.layout) {
            sprite(draw, name, *rect);
            if let Some((at, scale)) = placed {
                show(draw, *at, *scale);
            }
        }
    }
}

/// A picture of the table: its sprite and where it goes, and where the map goes over it, with
/// its scale.
type Picture = (
    &'static str,
    (i32, i32, i32, i32),
    Option<((i32, i32), f32)>,
);

/// The pictures of a layout, in the order they are drawn.
fn pictures(layout: CartographyLayout) -> &'static [Picture] {
    match layout {
        CartographyLayout::Scaled => &[("scaled_map", (67, 13, 66, 66), Some(((85, 31), 0.226)))],
        CartographyLayout::Duplicated => &[
            ("duplicated_map", (83, 13, 50, 66), Some(((86, 16), 0.34))),
            ("duplicated_map", (67, 29, 50, 66), Some(((70, 32), 0.34))),
        ],
        CartographyLayout::Locking => &[
            ("map", (67, 13, 66, 66), Some(((71, 17), 0.45))),
            ("locked", (118, 60, 10, 14), None),
        ],
        CartographyLayout::Plain => &[("map", (67, 13, 66, 66), Some(((71, 17), 0.45)))],
    }
}
