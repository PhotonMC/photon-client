//! `Minecraft.reloadResourcePacks` during a game: the packs the server pushed go in use and
//! everything the frames draw from assets is made again.

use super::ticks::{load_overlay_textures, share_registries};
use super::*;
use photon_render::model::DisplayContext;

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

/// The places the downloaded skins are drawn from, empty.
fn skin_slots() -> impl Iterator<Item = (String, photon_render::DecodedImage)> {
    (0..crate::skins::SLOT_COUNT).map(|index| {
        let blank = photon_render::DecodedImage {
            width: 64,
            height: 64,
            rgba: vec![0; 64 * 64 * 4],
        };
        (crate::skins::slot_name(index), blank)
    })
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

impl Play {
    /// Answer the reload the server's packs asked for: the packs go in use, or are reported as
    /// failed with the stack that was in use put back.
    pub(super) fn serve_pack_reload(&mut self) {
        let source = self.session.packs().clone();
        crate::packs::serve_reload(&source, &mut |packs| self.reload_packs(packs));
    }

    fn reload_packs(&mut self, packs: &crate::packs::LoadedPacks) -> bool {
        // Loading the packs already in use changes nothing.
        if crate::resources::holds(packs) {
            return true;
        }
        let Ok(root) = env::var("PHOTON_ASSETS") else {
            return false;
        };
        let root = std::path::Path::new(&root);
        let previous = crate::resources::shared(root);
        let stack = match crate::resources::with_packs(root, packs) {
            Ok(stack) => stack,
            Err(error) => {
                log::warn!("resource packs: {error}");
                return false;
            }
        };
        crate::resources::replace(Arc::new(stack));
        let rebuilt = self.rebuild_assets(&root.to_string_lossy());
        match rebuilt {
            Ok(()) => true,
            Err(error) => {
                log::warn!("resource reload failed: {error}");
                crate::resources::replace(previous);
                if let Err(error) = self.rebuild_assets(&root.to_string_lossy()) {
                    log::warn!("the previous resources could not be restored: {error}");
                }
                false
            }
        }
    }

    /// What is made of the assets, made again. Making the block atlas anew makes the GPU scene
    /// anew, so every texture is sent again after it.
    fn rebuild_assets(&mut self, root: &str) -> Result<()> {
        let data_root = data_root(root);
        // The colour maps are read through the packs too.
        self.biome_tints = Arc::new(
            BiomeTints::load(
                &data_root,
                &*crate::resources::shared(std::path::Path::new(root)),
                self.session.registry_names("minecraft:worldgen/biome"),
                self.session.hashed_seed(),
            )
            .ok_or("the biome colours could not be read")?,
        );
        let (renderer, animated) = bake_blocks(root, &mut self.backend)?;
        self.block_renderer = renderer;
        self.animated_atlas = animated;
        self.sections.set_renderer(
            Arc::clone(&self.block_renderer),
            Arc::clone(&self.biome_tints),
            &mut self.backend,
        )?;
        self.sky = Sky::load(
            &data_root,
            root,
            &self.session,
            sky_sprites(&self.block_renderer),
        )?;
        self.backend.load_weather_textures(
            &texture(root, "environment/rain", "rain")?,
            &texture(root, "environment/snow", "snow")?,
        )?;
        self.backend.load_crack_texture(&destroy_stages(root)?)?;
        load_overlay_textures(&mut self.backend, root)?;
        self.rebuild_entities(root, &data_root)?;
        self.rebuild_interface(root)
    }

    /// The entity pictures and the particles, which share the entity texture atlas.
    fn rebuild_entities(&mut self, root: &str, data_root: &std::path::Path) -> Result<()> {
        let mut particles = crate::particles::Particles::load(root);
        particles.set_terrain(
            Arc::clone(&self.block_renderer),
            Arc::clone(&self.biome_tints),
        );
        let mut armor = Armor::load(root);
        armor.set_trim_registries(
            self.session
                .registry_names("minecraft:trim_pattern")
                .to_vec(),
            self.session
                .registry_names("minecraft:trim_material")
                .to_vec(),
        );
        let textures = entity_textures(root, &armor, &particles)?;
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
    fn rebuild_interface(&mut self, root: &str) -> Result<()> {
        self.gui.reload_assets(
            std::path::Path::new(root),
            &mut self.backend,
            &self.controls.options.language_code,
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
