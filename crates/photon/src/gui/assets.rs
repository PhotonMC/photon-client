use super::*;

impl Gui {
    /// The default font, the GUI sprites (sent to the GPU) and the language
    /// from the asset folder.
    pub fn load(
        root: &Path,
        registries: Registries,
        backend: &mut VulkanBackend,
        language: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resources = crate::resources::shared(root);
        let font =
            Arc::new(Font::load_from(&*resources).ok_or("the default font could not be read")?);
        let alt_font =
            Arc::new(Font::load_alt_from(&*resources).ok_or("the alt font could not be read")?);
        let (atlas, texture) = GuiAtlas::load_from(&*resources, &[&font, &alt_font])
            .ok_or("the HUD sprites could not be read")?;
        backend.load_gui_texture(&texture)?;
        let lang = Arc::new(Lang::load_from(&*resources, root));
        let languages = photon_game::language::LanguageSource::open(root).map(Arc::new);
        Ok(Self::assembled(
            ((font, alt_font), (atlas, texture)),
            (resources, lang, languages),
            Arc::new(registries),
            language,
        ))
    }

    /// Whether the assets were read from the packs in use now, so that a game's `Gui` can serve
    /// the menus again without reading them.
    pub fn reads_from(&self, resources: &Arc<photon_assets::ResourceStack>) -> bool {
        Arc::ptr_eq(&self.resources, resources)
    }

    /// The assets of a finished game's `Gui` (and the sprites it left on the GPU) with the state
    /// of a new one, in `language`.
    #[must_use]
    pub fn recycled(self, language: &str) -> Self {
        let Self {
            font,
            alt_font,
            atlas,
            atlas_image,
            lang,
            languages,
            registries,
            resources,
            ..
        } = self;
        Self::assembled(
            ((font, alt_font), (atlas, atlas_image)),
            (resources, lang, languages),
            registries,
            language,
        )
    }

    /// A `Gui` over loaded assets, with the state of a start.
    fn assembled(
        ((font, alt_font), (atlas, atlas_image)): (
            (Arc<Font>, Arc<Font>),
            (GuiAtlas, photon_render::texture::png::DecodedImage),
        ),
        (resources, lang, languages): (
            Arc<photon_assets::ResourceStack>,
            Arc<Lang>,
            Option<Arc<photon_game::language::LanguageSource>>,
        ),
        registries: Arc<Registries>,
        language: &str,
    ) -> Self {
        let started_at = std::time::Instant::now();
        let mut gui = Self {
            font,
            alt_font,
            atlas,
            atlas_image,
            lang,
            resources,
            languages,
            registries,
            hud: Hud::default(),
            player_skin: [0.0, 0.0, 1.0, 1.0],
            titles: Titles::default(),
            boss_bars: BossBars::default(),
            tab_health: HealthStates::default(),
            gui_ticks: 0,
            tab_header: Vec::new(),
            tab_footer: Vec::new(),
            last_tick: std::time::Instant::now(),
            items: ItemLayerCache::new(photon_render::model::DisplayContext::Gui),
            started_at,
            chat_queue: Default::default(),
            last_frame_ms: 0,
            layers: super::layers::Layers::new(started_at),
        };
        gui.hud
            .set_waypoint_styles(photon_gui::hud::locator::WaypointStyles::load_from(
                &*gui.resources,
            ));
        gui.set_language(language);
        gui
    }

    /// The resource packs in use changed: the fonts, the sprites, the texts and the pictures made
    /// of the models are read again, and the sprites are sent to the GPU, whose scene must have
    /// been made anew.
    pub fn reload_assets(
        &mut self,
        root: &Path,
        backend: &mut VulkanBackend,
        language: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let resources = crate::resources::shared(root);
        let font =
            Arc::new(Font::load_from(&*resources).ok_or("the default font could not be read")?);
        let alt_font =
            Arc::new(Font::load_alt_from(&*resources).ok_or("the alt font could not be read")?);
        let (atlas, texture) = GuiAtlas::load_from(&*resources, &[&font, &alt_font])
            .ok_or("the HUD sprites could not be read")?;
        backend.load_gui_texture(&texture)?;
        self.font = font;
        self.alt_font = alt_font;
        self.atlas = atlas;
        self.atlas_image = texture;
        self.items = ItemLayerCache::new(photon_render::model::DisplayContext::Gui);
        self.resources = resources;
        self.hud
            .set_waypoint_styles(photon_gui::hud::locator::WaypointStyles::load_from(
                &*self.resources,
            ));
        self.lang = Arc::new(Lang::load_from(&*self.resources, root));
        self.set_language(language);
        Ok(())
    }

    /// Where the skin the inventory's player wears is in the entity textures.
    pub fn set_player_skin(&mut self, skin: photon_render::entity::textures::TextureRect) {
        self.player_skin = skin;
    }

    /// The default font and the sprites it is drawn from, for text in the world.
    /// The language the texts are worded in.
    pub fn lang(&self) -> &Lang {
        &self.lang
    }

    /// The same language, shared.
    pub fn lang_shared(&self) -> Arc<Lang> {
        Arc::clone(&self.lang)
    }

    pub fn text_assets(&self) -> (&Font, &GuiAtlas) {
        (&self.font, &self.atlas)
    }

    /// What the inventory screens open with: the search trees, the font and
    /// the titles in the client's language.
    pub fn screen_assets(&self) -> ScreenAssets {
        let lang = &self.lang;
        ScreenAssets {
            creative: photon_gui::screen::creative::CreativeAssets {
                trees: Arc::new(photon_game::menus::creative::SearchTrees::new(
                    lang.clone(),
                    self.registries.clone(),
                )),
                font: self.font.clone(),
                titles: photon_data::item::creative_tabs::CREATIVE_TABS
                    .iter()
                    .map(|tab| lang.text(tab.title))
                    .collect(),
            },
            inventory_title: lang.text("container.crafting"),
            registries: self.registries.clone(),
            alt_font: self.alt_font.clone(),
            lang: lang.clone(),
            languages: self.languages.clone(),
            resources: self.resources.clone(),
        }
    }

    /// The client's language is another: texts are read again.
    pub fn set_language(&mut self, code: &str) {
        if let Some(source) = &self.languages {
            self.lang = Arc::new(source.lang_from(code, &*self.resources));
        }
    }
}
