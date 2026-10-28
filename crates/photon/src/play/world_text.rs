//! The text of the world a frame draws.

use super::*;
use crate::objects::displays;
use photon_render::scene::Vertex;

impl Play {
    /// The text of the world as triangles: name tags, text displays and signs, seen through
    /// blocks and tested against the depth.
    pub(super) fn world_texts(
        &self,
        camera: &overlays::TagCamera,
        view: overlays::FrameView,
    ) -> (Vec<Vertex>, Vec<Vertex>) {
        let (font, atlas) = self.gui.text_assets();
        let assets = (self.gui.lang(), font, atlas);
        // `Options.getBackgroundOpacity(0.25F)`.
        let opacity = self.controls.options.background_opacity(0.25);
        let (mut see_through, mut normal) =
            overlays::name_tags(&self.session, assets, camera, (view, opacity));
        let (shown_through, shown) =
            displays::display_texts(&self.session, assets, (view, opacity));
        see_through.extend(shown_through);
        normal.extend(shown);
        let first_person = self.controls.camera_type == CameraType::FirstPerson;
        normal.extend(crate::world::signs::sign_texts(
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
        ));
        (see_through, normal)
    }
}
