//! The text of the world a frame draws.

use super::*;
use crate::objects::displays;
use photon_render::scene::Vertex;

impl Play {
    /// The text of the world as triangles: name tags, text displays and signs: seen through
    /// blocks, tested against the depth, and the front layer of the outlined lines.
    pub(super) fn world_texts(
        &self,
        camera: &overlays::TagCamera,
        view: overlays::FrameView,
    ) -> (Vec<Vertex>, Vec<Vertex>, Vec<Vertex>) {
        let (font, atlas) = self.gui.text_assets();
        let assets = (self.gui.lang(), font, atlas);
        // `Options.getBackgroundOpacity(0.25F)`.
        let opacity = self.controls.options.background_opacity(0.25);
        let (mut see_through, mut normal) =
            overlays::name_tags(&self.session, assets, camera, (view, opacity));
        let (shown_through, shown, mut offset) =
            displays::display_texts(&self.session, assets, (view, opacity));
        see_through.extend(shown_through);
        normal.extend(shown);
        let first_person = self.controls.camera_type == CameraType::FirstPerson;
        let (signs, outlined) = crate::world::signs::sign_texts(
            &self.session,
            assets,
            view,
            (
                first_person,
                self.controls
                    .sign_edit
                    .as_ref()
                    .map(|edit| (edit.pos, edit.front, edit.screen.messages())),
            ),
        );
        normal.extend(signs);
        offset.extend(outlined);
        (see_through, normal, offset)
    }
}
