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
        let font = Arc::new(Font::load(root).ok_or("the default font could not be read")?);
        let alt_font = Arc::new(Font::load_alt(root).ok_or("the alt font could not be read")?);
        let (atlas, texture) =
            GuiAtlas::load(root, &[&font, &alt_font]).ok_or("the HUD sprites could not be read")?;
        backend.load_gui_texture(&texture)?;
        let started_at = std::time::Instant::now();
        let mut gui = Self {
            font,
            alt_font,
            atlas,
            atlas_image: texture,
            lang: Arc::new(Lang::load(root)),
            languages: photon_game::language::LanguageSource::open(root).map(Arc::new),
            registries: Arc::new(registries),
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
            assets_root: root.to_path_buf(),
            last_frame_ms: 0,
            layers: super::layers::Layers::new(started_at),
        };
        gui.hud
            .set_waypoint_styles(photon_gui::hud::locator::WaypointStyles::load(root));
        gui.set_language(language);
        Ok(gui)
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
            assets_root: self.assets_root.clone(),
        }
    }

    /// The client's language is another: texts are read again.
    pub fn set_language(&mut self, code: &str) {
        if let Some(source) = &self.languages {
            self.lang = Arc::new(source.lang(code));
        }
    }
}
