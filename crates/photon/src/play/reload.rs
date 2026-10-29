//! `Minecraft.reloadResourcePacks` during a game: the packs the server pushed go in use and
//! everything the frames draw from assets is made again.

use super::ticks::share_registries;
use super::*;
use crate::packs::{Attempt, answer_attempt};
use photon_assets::ResourceStack;
use photon_gui::loading_overlay::LoadingOverlay;
use photon_render::model::DisplayContext;
use std::sync::atomic::{AtomicU8, Ordering};

/// The vanilla data pack the server runs, which sits next to the assets.
pub(super) fn data_root(root: &str) -> std::path::PathBuf {
    env::var("PHOTON_DATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(root).with_file_name("data"))
}

/// The entity texture atlas: every skin, mob, armor and particle picture, a shared one packed once.
pub(super) fn entity_textures(
    root: &str,
    armor: &Armor,
    particles: &crate::particles::Particles,
) -> Result<photon_render::entity::textures::EntityTextures> {
    photon_render::entity::textures::EntityTextures::build(
        photon_render::entity::player_renderer::skin_textures()
            .chain(mobs::textures())
            .chain(overlays::SHARED_TEXTURES)
            .map(str::to_owned)
            .chain(villagers::textures())
            .chain(armor.textures())
            .chain(particles.textures().iter().cloned())
            // A texture two layers share is packed once.
            .scan(std::collections::HashSet::new(), |seen, name| {
                Some(seen.insert(name.clone()).then_some(name))
            })
            .flatten()
            .map(|name| {
                let image = texture(root, &name, "entity texture")?;
                Ok((name, image))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .chain(Armor::trim_images(root))
            .chain(skin_slots())
            .collect(),
    )
    .ok_or_else(|| "the entity textures did not fit".into())
}

/// The places the downloaded skins and capes are drawn from, empty.
fn skin_slots() -> impl Iterator<Item = (String, photon_render::DecodedImage)> {
    (0..crate::skins::SLOT_COUNT)
        .map(|index| {
            let blank = photon_render::DecodedImage {
                width: 64,
                height: 64,
                rgba: vec![0; 64 * 64 * 4],
            };
            (crate::skins::slot_name(index), blank)
        })
        .chain((0..crate::skins::CAPE_SLOT_COUNT).map(|index| {
            let blank = photon_render::DecodedImage {
                width: 64,
                height: 32,
                rgba: vec![0; 64 * 32 * 4],
            };
            (crate::skins::cape_slot_name(index), blank)
        }))
}

/// The sun and the moon's phases in the block atlas.
pub(super) fn sky_sprites(block_renderer: &BlockRenderer) -> photon_render::sky::SkySprites {
    let sprite = |name: &str| {
        block_renderer
            .atlas
            .sprites
            .get(name)
            .copied()
            .unwrap_or([0.0; 4])
    };
    let moon_names: Vec<String> = photon_render::sky::moon_sprites().collect();
    photon_render::sky::SkySprites {
        sun: sprite(photon_render::sky::SUN_SPRITE),
        end_flash: sprite(photon_render::sky::END_FLASH_SPRITE),
        moon: std::array::from_fn(|phase| sprite(&moon_names[phase])),
    }
}

/// The time each step of a rebuild takes, logged as it goes (`RUST_LOG=info`).
struct Lap(std::time::Instant);

impl Lap {
    fn start() -> Self {
        Self(std::time::Instant::now())
    }

    fn mark(&mut self, step: &str) {
        log::info!("resource rebuild: {step} {:?}", self.0.elapsed());
        self.0 = std::time::Instant::now();
    }
}

/// What the preparation of a reload needs from the game.
pub(super) struct PrepareInputs {
    root: String,
    steps: Arc<AtomicU8>,
    trim_patterns: Vec<String>,
    trim_materials: Vec<String>,
}

/// The pictures the renderer takes besides the atlases.
pub(super) struct Images {
    rain: photon_render::DecodedImage,
    snow: photon_render::DecodedImage,
    cracks: photon_render::DecodedImage,
    forcefield: photon_render::DecodedImage,
    underwater: photon_render::DecodedImage,
    end_sky: photon_render::DecodedImage,
    end_portal: photon_render::DecodedImage,
}

impl Images {
    pub(super) fn read(root: &str) -> std::result::Result<Self, String> {
        let read = |name: &str, label: &str| texture(root, name, label).map_err(|e| e.to_string());
        Ok(Self {
            rain: read("environment/rain", "rain")?,
            snow: read("environment/snow", "snow")?,
            cracks: destroy_stages(root).map_err(|e| e.to_string())?,
            forcefield: read("misc/forcefield", "forcefield")?,
            underwater: read("misc/underwater", "underwater")?,
            end_sky: read("environment/end_sky", "end_sky")?,
            end_portal: read("entity/end_portal/end_portal", "end_portal")?,
        })
    }

    pub(super) fn upload(&self, backend: &mut photon_render::VulkanBackend) -> Result<()> {
        backend.load_weather_textures(&self.rain, &self.snow)?;
        backend.load_crack_texture(&self.cracks)?;
        backend.load_world_border_texture(&self.forcefield)?;
        backend.load_underwater_texture(&self.underwater)?;
        backend.load_end_portal_textures((&self.end_sky, &self.end_portal))?;
        Ok(())
    }
}

/// The entity pictures, the particle sprite lists and the armor, read from a stack.
pub(super) struct PreparedEntities {
    particles: crate::particles::Particles,
    armor: Armor,
    textures: photon_render::entity::textures::EntityTextures,
}

/// What a reload makes of a stack without touching the GPU or the game: everything the frames
/// draw from assets, as CPU data, ready to be applied.
pub(super) struct Prepared {
    stack: Arc<ResourceStack>,
    root: String,
    blocks: BlockRenderer,
    gui: crate::gui::ReadAssets,
    images: Images,
    entities: PreparedEntities,
}

/// Read and bake everything from `stack`, which only this thread sees until it is applied.
pub(super) fn prepare(
    stack: Arc<ResourceStack>,
    inputs: PrepareInputs,
) -> std::result::Result<Prepared, String> {
    let PrepareInputs {
        root,
        steps,
        trim_patterns,
        trim_materials,
    } = inputs;
    crate::resources::preparing(Arc::clone(&stack), || {
        let mut lap = Lap::start();
        let blocks = bake_renderer(&root).map_err(|e| e.to_string())?;
        lap.mark("block bake");
        steps.fetch_add(1, Ordering::Relaxed);
        let gui = Gui::read_assets(std::path::Path::new(&root))?;
        lap.mark("interface read");
        steps.fetch_add(1, Ordering::Relaxed);
        let images = Images::read(&root)?;
        steps.fetch_add(1, Ordering::Relaxed);
        let mut particles = crate::particles::Particles::load(&root);
        let mut armor = Armor::load(&root);
        armor.set_trim_registries(trim_patterns, trim_materials);
        let textures = entity_textures(&root, &armor, &particles).map_err(|e| e.to_string())?;
        // The sprite lists of the particles are the stack's: the engine takes them as they are.
        let _ = &mut particles;
        lap.mark("entity read");
        steps.fetch_add(1, Ordering::Relaxed);
        Ok(Prepared {
            stack: Arc::clone(&stack),
            root: root.clone(),
            blocks,
            gui,
            images,
            entities: PreparedEntities {
                particles,
                armor,
                textures,
            },
        })
    })
}

/// Which answer a running reload owes.
#[derive(Clone, Copy, PartialEq)]
enum Stage {
    /// The player chose other packs: nobody is told.
    Forced,
    /// The server's packs: loaded or failed is sent when it is applied.
    Server,
    /// The packs that were in use, after a failure.
    Recovery,
}

/// A reload being prepared on a worker thread. The game keeps drawing the old resources and
/// ticking the session meanwhile; the result is applied on the main thread once it is done.
pub(super) struct Job {
    handle: std::thread::JoinHandle<std::result::Result<Prepared, String>>,
    stage: Stage,
    /// How many of the reload's `STEPS` the worker has done.
    steps: Arc<AtomicU8>,
}

/// What a reload does besides applying it: the four reads of the worker and the apply, the
/// units its progress bar counts.
const STEPS: f32 = 5.0;

/// What a game keeps of its resource reloads.
#[derive(Default)]
pub(super) struct State {
    /// The reload being prepared on a worker thread.
    job: Option<Job>,
    /// A reload applied, waiting for the loading overlay to answer for it.
    applied: Option<Applied>,
}

/// A reload that is applied and waits for the overlay to answer for it (`LoadingOverlay.tick`).
pub(super) struct Applied {
    stage: Stage,
    outcome: std::result::Result<(), String>,
}

impl Play {
    /// Serve the reload the server's packs asked for, as `Minecraft.runTick` and
    /// `LoadingOverlay.tick` do: a finished worker is applied; the overlay answers once the reload
    /// is done and fades out; and a reload that came while an overlay was up starts after it.
    pub(super) fn serve_pack_reload(&mut self) {
        self.controls.pack_poll(&self.screen_assets);
        if self
            .reload
            .job
            .as_ref()
            .is_some_and(|job| job.handle.is_finished())
        {
            self.finish_reload();
        }
        self.tick_loading_overlay();
        // `pendingReload`: nothing starts while a loading overlay is up.
        if self.gui.is_loading() {
            return;
        }
        if self.controls.take_pack_reload() {
            // The player chose other packs: the server's go over them again.
            let packs = crate::resources::server_packs();
            self.start_reload(&packs, Stage::Forced);
            return;
        }
        let source = self.session.packs().clone();
        let Some(packs) = source.take_reload() else {
            return;
        };
        // Loading the packs already in use changes nothing.
        if crate::resources::holds(&packs) {
            source.reload_succeeded();
        } else {
            self.start_reload(&packs, Stage::Server);
        }
    }

    /// `LoadingOverlay.tick` and the end of `extractRenderState`: the finished reload is answered
    /// once the overlay is ready, which starts its fade-out, and the faded overlay goes.
    fn tick_loading_overlay(&mut self) {
        let now = self.gui.loading_clock();
        let progress = match &self.reload.job {
            Some(job) => f32::from(job.steps.load(Ordering::Relaxed)) / STEPS,
            None => 1.0,
        };
        self.gui.set_reload_progress(progress);
        let Some(overlay) = self.gui.loading_mut() else {
            return;
        };
        if overlay.finished(now) {
            self.gui.set_loading(None);
            return;
        }
        if self.reload.applied.is_some() && overlay.ready(true, now) {
            let Applied { stage, outcome } = self.reload.applied.take().expect("checked");
            self.answer_reload(stage, outcome);
            // A recovery put its own overlay in place of this one.
            if self.reload.job.is_none()
                && let Some(overlay) = self.gui.loading_mut()
            {
                overlay.start_fade_out(now);
            }
        }
    }

    /// Open `packs` over the selected ones and prepare what they make of the assets on a worker
    /// thread, behind a loading overlay. A failure to start is the reload's failure.
    fn start_reload(&mut self, packs: &crate::packs::LoadedPacks, stage: Stage) {
        let started = std::time::Instant::now();
        let prepared = env::var("PHOTON_ASSETS")
            .map_err(|_| "PHOTON_ASSETS must name the asset folder".to_owned())
            .and_then(|root| {
                let stack = crate::resources::with_packs(std::path::Path::new(&root), packs)?;
                Ok((Arc::new(stack), self.prepare_inputs(&root)))
            });
        let (stack, inputs) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => return self.answer_reload(stage, Err(error)),
        };
        let steps = Arc::clone(&inputs.steps);
        let handle = std::thread::Builder::new()
            .name("resource-reload".to_owned())
            .spawn(move || {
                let prepared = prepare(stack, inputs);
                log::info!("resources prepared in {:?}", started.elapsed());
                prepared
            });
        match handle {
            Ok(handle) => {
                self.reload.job = Some(Job {
                    handle,
                    stage,
                    steps,
                });
                self.gui
                    .set_loading(Some(LoadingOverlay::new(false, false)));
            }
            Err(error) => self.answer_reload(stage, Err(error.to_string())),
        }
    }

    /// Apply what the worker prepared; the overlay answers for it when it is ready.
    fn finish_reload(&mut self) {
        let Some(Job { handle, stage, .. }) = self.reload.job.take() else {
            return;
        };
        let started = std::time::Instant::now();
        let previous = env::var("PHOTON_ASSETS")
            .ok()
            .map(|root| crate::resources::shared(std::path::Path::new(&root)));
        let outcome = match handle.join() {
            Ok(Ok(prepared)) => self.apply_prepared(prepared).map_err(|error| {
                // The renderers may be half replaced: make them again from what was in use.
                let restored = previous.is_some_and(|stack| self.restore_resources(stack));
                format!("{error}{}", if restored { "" } else { " (not restored)" })
            }),
            Ok(Err(error)) => Err(error),
            Err(_) => Err("the reload thread panicked".to_owned()),
        };
        log::info!("resources applied in {:?}", started.elapsed());
        self.reload.applied = Some(Applied { stage, outcome });
    }

    /// Put `previous`, the stack that was in use, through the renderers again after an apply that
    /// failed midway.
    fn restore_resources(&mut self, previous: Arc<ResourceStack>) -> bool {
        let Ok(root) = env::var("PHOTON_ASSETS") else {
            return false;
        };
        let inputs = self.prepare_inputs(&root);
        matches!(
            prepare(previous, inputs).map(|prepared| self.apply_prepared(prepared)),
            Ok(Ok(()))
        )
    }

    /// Tell the pack source how the reload went; a failure starts the recovery it asks for.
    fn answer_reload(&mut self, stage: Stage, outcome: std::result::Result<(), String>) {
        if let Err(error) = &outcome {
            log::warn!("resource reload failed: {error}");
        }
        let attempt = match stage {
            Stage::Forced => return,
            Stage::Server => Attempt::First,
            Stage::Recovery => Attempt::Recovery,
        };
        let source = self.session.packs().clone();
        if let Some(recovery) = answer_attempt(&source, attempt, outcome.is_ok()) {
            self.start_reload(&recovery, Stage::Recovery);
        }
    }

    /// What the preparation needs from the game, owned.
    pub(super) fn prepare_inputs(&self, root: &str) -> PrepareInputs {
        PrepareInputs {
            root: root.to_owned(),
            steps: Arc::new(AtomicU8::new(0)),
            trim_patterns: self
                .session
                .registry_names("minecraft:trim_pattern")
                .to_vec(),
            trim_materials: self
                .session
                .registry_names("minecraft:trim_material")
                .to_vec(),
        }
    }

    /// What is made of the assets, made again from what was prepared. Making the block atlas
    /// anew makes the GPU scene anew, so every texture is sent again after it.
    pub(super) fn apply_prepared(&mut self, prepared: Prepared) -> Result<()> {
        let mut lap = Lap::start();
        let Prepared {
            stack,
            root,
            blocks,
            gui,
            images,
            entities,
        } = prepared;
        // Every reader sees the new stack from here on, with the renderers made from it.
        crate::resources::replace(stack);
        let data_root = data_root(&root);
        // The colour maps are read through the packs too.
        self.biome_tints = Arc::new(
            BiomeTints::load(
                &data_root,
                &*crate::resources::shared(std::path::Path::new(&root)),
                self.session.registry_names("minecraft:worldgen/biome"),
                self.session.hashed_seed(),
            )
            .ok_or("the biome colours could not be read")?,
        );
        lap.mark("biome colours");
        let (renderer, animated) = upload_blocks(blocks, &mut self.backend)?;
        self.block_renderer = renderer;
        self.animated_atlas = animated;
        lap.mark("block atlas upload");
        self.sections.set_renderer(
            Arc::clone(&self.block_renderer),
            Arc::clone(&self.biome_tints),
            &mut self.backend,
        )?;
        self.sky = Sky::load(
            &data_root,
            &root,
            &self.session,
            sky_sprites(&self.block_renderer),
        )?;
        lap.mark("sections and sky");
        images.upload(&mut self.backend)?;
        lap.mark("weather, cracks, overlays");
        self.apply_entities(entities, &data_root)?;
        lap.mark("entities");
        let applied = self.rebuild_interface(gui, &root);
        lap.mark("interface");
        applied
    }

    /// The entity pictures and the particles, which share the entity texture atlas.
    fn apply_entities(
        &mut self,
        entities: PreparedEntities,
        data_root: &std::path::Path,
    ) -> Result<()> {
        let PreparedEntities {
            mut particles,
            armor,
            textures,
        } = entities;
        particles.carry_over(&mut self.particles);
        particles.set_terrain(
            Arc::clone(&self.block_renderer),
            Arc::clone(&self.biome_tints),
        );
        self.backend.load_entity_texture(&textures.image)?;
        particles.bind_textures(&textures);
        self.skins.lay_out(&textures);
        let skin = photon_render::entity::player_renderer::default_skin(
            self.session
                .profile_id()
                .unwrap_or(photon_core::binary::Uuid { most: 0, least: 0 }),
        );
        self.skin_rect = textures.rect(skin.texture).ok_or("the player's skin")?;
        let registries = photon_game::registries::Registries::load(data_root, |registry| {
            self.session.registry_names(registry).to_vec()
        });
        share_registries(&mut self.session, &mut particles, &registries);
        (self.particles, self.armor, self.entity_textures) = (particles, armor, textures);
        Ok(())
    }

    /// The interface, the models' caches and the sounds.
    fn rebuild_interface(&mut self, read: crate::gui::ReadAssets, _root: &str) -> Result<()> {
        self.gui.apply_assets(
            read,
            (&mut self.backend, &self.controls.options.language_code),
        )?;
        self.gui.set_player_skin(self.skin_rect);
        self.session.set_lang(self.gui.lang_shared());
        self.screen_assets = self.gui.screen_assets();
        self.controls
            .rebuild_menus(&self.surface, &self.screen_assets);
        self.remote_models = overlays::RemoteModels::new();
        self.mob_models = mobs::MobModels::new();
        self.item_layers = WorldItemLayers::new();
        self.worn_layers = [
            DisplayContext::ThirdPersonRightHand,
            DisplayContext::ThirdPersonLeftHand,
            DisplayContext::Head,
            DisplayContext::Ground,
        ]
        .map(ItemLayerCache::new);
        self.held_item_layers = [
            DisplayContext::FirstPersonRightHand,
            DisplayContext::FirstPersonLeftHand,
        ]
        .map(ItemLayerCache::new);
        self.sound.reload_for_packs(&self.controls.options);
        Ok(())
    }
}
