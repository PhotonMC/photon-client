//! The text on the signs near the camera (`StandingSignRenderer`, `HangingSignRenderer`); the
//! boards themselves are block models.

use photon_core::math::Vec3;
use photon_gui::sign_text::{SignFace, sign_text_submits};
use photon_net::connection::PlaySession;
use photon_render::block_entity::sign::{OUTLINE_DISTANCE_SQR, SignKind, sign_text_vertices};
use photon_render::joml::Pose;
use photon_render::scene::Vertex;

use crate::overlays::{FrameView, block_light};

/// The sign being written on: its block, whether its front, and its lines as they stand.
pub type SignEditing<'a> = Option<((i32, i32, i32), bool, &'a [String; 4])>;

/// The triangles of the text of every sign near the camera; `first_person` whether the camera
/// is the player's eyes, `editing` the sign being written on, its face and its lines as they
/// stand (`AbstractSignEditScreen.setMessage` sets them on the block entity as they are typed).
pub fn sign_texts(
    session: &PlaySession,
    (lang, font, atlas): (
        &photon_game::text::Lang,
        &photon_render::gui::font::Font,
        &photon_render::gui::GuiAtlas,
    ),
    view: FrameView,
    (first_person, editing): (bool, SignEditing<'_>),
) -> Vec<Vertex> {
    let world = session.world();
    let player = session.player();
    let scoping = first_person && player.is_some_and(|player| player.is_scoping());
    let mut vertices = Vec::new();
    for (x, y, z) in crate::objects::block_entity_items::in_view(world, view) {
        let Some(sign) = world.sign_view((x, y, z)) else {
            continue;
        };
        let kind = if sign.hanging {
            SignKind::Hanging
        } else {
            SignKind::Standing
        };
        // `isOutlineVisible`: the camera's entity within sixteen blocks of the sign's centre.
        let centre = Vec3::new(f64::from(x) + 0.5, f64::from(y) + 0.5, f64::from(z) + 0.5);
        let near = player.is_some_and(|player| {
            let at = player.position();
            (at.x - centre.x).powi(2) + (at.y - centre.y).powi(2) + (at.z - centre.z).powi(2)
                < OUTLINE_DISTANCE_SQR
        });
        let (mut front, mut back) = photon_game::sign::sign_texts(sign.tag);
        if let Some((_, is_front, lines)) = editing.filter(|(at, ..)| *at == (x, y, z)) {
            let face = if is_front { &mut front } else { &mut back };
            // A line typed over turns plain; the others keep their style.
            for (message, line) in face.messages.iter_mut().zip(lines) {
                if lang.plain(message) != *line {
                    *message = photon_game::text::Component::literal(line);
                }
            }
        }
        // The player's own text is not filtered: `isTextFilteringEnabled` is an online account's.
        let lines = |text: &photon_game::sign::SignText| {
            text.messages(false)
                .clone()
                .map(|line| lang.runs(&line, &photon_game::text::Style::default()))
        };
        let (front_lines, back_lines) = (lines(&front), lines(&back));
        let faces = [(&front_lines, &front), (&back_lines, &back)].map(|(lines, text)| SignFace {
            lines,
            color: text.color,
            glowing: text.has_glowing_text,
        });
        let mut base = Pose::default();
        base.translate(
            (x - view.origin[0]) as f32,
            (y - view.origin[1]) as f32,
            (z - view.origin[2]) as f32,
        );
        let submits = sign_text_submits(
            &base,
            kind,
            (&kind.transformations(sign.wall, sign.rotation), faces),
            (scoping || near, block_light(world, (x, y, z))),
            font,
        );
        vertices.extend(sign_text_vertices(&submits, (font, atlas)));
    }
    vertices
}
