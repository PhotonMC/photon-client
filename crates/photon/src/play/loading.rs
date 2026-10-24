//! `LevelLoadingScreen` in play: after a respawn the screen covers the world while the client
//! waits for the server's chunks and for the player's section to be compiled.

use crate::world::sections;
use photon_gui::screen::connect::{LoadingBackground, draw_level_loading_over};
use photon_net::connection::LoadingReason;
use photon_render::scene::Vertex;

use super::*;

/// The loading screen in view: when the panorama behind it last turned, and by how much.
pub(crate) struct LoadingView {
    spin: f32,
    last_spin: Instant,
}

impl Play {
    /// A window event while the level loads: the screen takes it, the game below takes none.
    pub(super) fn loading_event(&self, event: &glfw::WindowEvent) -> bool {
        self.loading.is_some() && is_input(event)
    }

    /// A frame of the loading screen in the place of the level. The first frame closes what was
    /// open over the old level and lets go of the mouse.
    pub(super) fn draw_loading(&mut self) -> Result<()> {
        let Some(reason) = self.session.level_loading() else {
            return Ok(());
        };
        if self.loading.is_none() {
            log::info!("level loading screen: {reason:?}");
            self.begin_loading()?;
        }
        // The sections around the player are meshed behind the screen; the load ends when the
        // one the camera is in has shown.
        if let Some(player) = self.session.player() {
            let frame = self.frame_camera(player);
            self.sections
                .forget_chunks(self.session.take_forgotten_chunks(), &mut self.backend)?;
            self.sections.update(
                &mut self.session,
                &mut self.backend,
                &sections::SectionView {
                    camera: [frame.camera.x, frame.camera.y, frame.camera.z],
                    frustum: frame.frustum,
                    view_distance: frame.render_distance,
                    origin: self.origin,
                },
                false,
            )?;
            if let Some(key) = self.session.wanted_player_section()
                && self.sections.compiled_and_visible(key)
            {
                self.session.player_section_compiled();
            }
        }
        // The nether portal's sprite animates.
        self.backend
            .update_atlas(&self.animated_atlas.changed_regions())?;
        self.spin_loading_panorama(reason);
        self.draw_end_portal_background(reason)?;
        let scale = self.surface.scale();
        let size = (scale.gui_scaled_width(), scale.gui_scaled_height());
        let lang = self.gui.lang();
        let background = self.loading_background(reason);
        self.gui
            .draw_menu_screen(&mut self.backend, &self.surface, |frame, atlas, font| {
                draw_level_loading_over(frame, atlas, font, lang, size, background);
            })?;
        self.gui.sync_glyphs(&mut self.backend)?;
        self.backend
            .draw_scene([0.0, 0.0, 0.0, 1.0], super::switch::IDENTITY)
            .map_err(Into::into)
    }

    /// `Reason.END_PORTAL`: the end portal's surface over the whole window, as the portal pipeline
    /// draws it for a block with the clip space as the model (15 layers, `END_PORTAL`).
    fn draw_end_portal_background(&mut self, reason: LoadingReason) -> Result<()> {
        let on = reason == LoadingReason::EndPortal;
        let corners = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
        let vertex = |[x, y]: [f32; 2]| Vertex {
            position: [x, y, 0.5],
            uv: [0.0; 2],
            shade: 1.0,
            tint: [crate::world::end_portals::PORTAL_LAYERS, 0.0, 0.0, 0.0],
            light: [0.0; 2],
        };
        let quad: Vec<Vertex> = [0, 2, 1, 0, 3, 2]
            .into_iter()
            .map(|corner| vertex(corners[corner]))
            .collect();
        self.backend.set_draw(
            photon_render::vulkan::DrawKind::EndPortal,
            if on { &quad } else { &[] },
        )?;
        if on {
            let ticks = self.session.world().game_time();
            self.backend.set_portal_time(ticks, 0.0);
        }
        Ok(())
    }

    /// What the reason shows behind the text.
    fn loading_background(&self, reason: LoadingReason) -> LoadingBackground {
        match reason {
            LoadingReason::Other => LoadingBackground::Menu,
            LoadingReason::EndPortal => LoadingBackground::EndPortal,
            LoadingReason::NetherPortal => {
                let state =
                    photon_data::block::registry::default_state_named("minecraft:nether_portal");
                self.block_renderer
                    .particle_sprite(state)
                    .and_then(|name| self.block_renderer.sprite_rect(name))
                    .map_or(
                        LoadingBackground::EndPortal,
                        LoadingBackground::NetherPortal,
                    )
            }
        }
    }

    fn begin_loading(&mut self) -> Result<()> {
        self.close_screens();
        self.controls.captured = false;
        self.surface.set_cursor_captured(false);
        self.backend.set_gui_only(true);
        self.load_panorama();
        self.loading = Some(LoadingView {
            spin: 0.0,
            last_spin: Instant::now(),
        });
        Ok(())
    }

    /// The panorama's faces, into the renderer.
    fn load_panorama(&mut self) {
        let Ok(root) = env::var("PHOTON_ASSETS") else {
            return;
        };
        match crate::menu::panorama::faces(&root) {
            Ok(faces) => {
                if let Err(error) = self
                    .backend
                    .load_panorama(&photon_render::panorama::layers(&faces))
                {
                    log::warn!("panorama: {error:?}");
                }
            }
            Err(error) => log::warn!("panorama: {error}"),
        }
    }

    /// `Screen.extractPanorama` and the blur of `extractBlurredBackground`, for the plain reason.
    fn spin_loading_panorama(&mut self, reason: LoadingReason) {
        let Some(view) = self.loading.as_mut() else {
            return;
        };
        if reason != LoadingReason::Other {
            self.backend.set_panorama(None);
            return;
        }
        spin_view(
            view,
            &mut self.backend,
            self.controls.options.menu_background_blurriness,
        );
    }

    /// The panorama behind a screen with no level (`Screen.extractBackground` when
    /// `minecraft.level` is null), turning from frame to frame.
    pub(super) fn show_backdrop(&mut self) {
        if self.controls.backdrop.is_none() {
            self.backend.set_gui_only(true);
            // The surface of an end portal loading screen does not stay.
            let _ = self
                .backend
                .set_draw(photon_render::vulkan::DrawKind::EndPortal, &[]);
            self.load_panorama();
            self.controls.backdrop = Some(LoadingView {
                spin: 0.0,
                last_spin: Instant::now(),
            });
        }
        if let Some(view) = self.controls.backdrop.as_mut() {
            spin_view(
                view,
                &mut self.backend,
                self.controls.options.menu_background_blurriness,
            );
        }
    }

    /// The backdrop goes with its screen.
    pub(super) fn end_backdrop(&mut self) {
        if self.controls.backdrop.take().is_some() {
            self.backend.set_gui_only(false);
            self.backend.set_panorama(None);
        }
    }

    /// The level is loaded: the screen goes, and the mouse comes back to the game unless
    /// something else holds it.
    pub(super) fn end_loading(&mut self) {
        if self.loading.take().is_none() {
            return;
        }
        self.backend.set_gui_only(false);
        self.backend.set_panorama(None);
        if self.controls.disconnected.is_none() && self.controls.screen.is_none() {
            self.controls.captured = true;
            self.surface.set_cursor_captured(true);
        }
    }
}

/// One frame of the turning panorama behind a screen, with the blur of the options.
fn spin_view(
    view: &mut LoadingView,
    backend: &mut photon_render::VulkanBackend,
    blurriness: impl Into<f64>,
) {
    let now = Instant::now();
    let ticks =
        photon_render::panorama::real_delta_ticks(now.duration_since(view.last_spin).as_secs_f32());
    view.last_spin = now;
    view.spin = photon_render::panorama::spin_after(view.spin, ticks, 1.0);
    let extent = backend.description().extent;
    if let Err(error) = backend.set_panorama_blur(blurriness.into() as u32) {
        log::warn!("panorama blur: {error:?}");
    }
    backend.set_panorama(Some(photon_render::panorama::view_projection(
        -view.spin,
        extent.width,
        extent.height,
    )));
}
