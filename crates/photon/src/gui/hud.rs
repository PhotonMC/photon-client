use super::*;

impl Gui {
    /// A tick of the HUD, with the brightness of the light at the camera's eyes while there is a camera.
    pub fn tick(&mut self, camera_light: Option<f32>) {
        self.hud.tick();
        if let Some(light) = camera_light {
            self.hud.update_vignette(light);
        }
        self.titles.tick();
        self.gui_ticks += 1;
        self.last_tick = std::time::Instant::now();
    }

    /// `Hud.setNowPlaying`: "Now Playing:" and the name of the record the jukebox plays.
    pub fn show_now_playing(&mut self, song: usize) {
        use photon_game::text::{Component, Content};
        let Some(name) = self
            .registries
            .jukebox_songs
            .get(song)
            .and_then(|entry| entry.description.as_ref())
            .and_then(Component::from_json)
        else {
            return;
        };
        let message = Component {
            content: Content::Translatable {
                key: "record.nowPlaying".to_owned(),
                fallback: None,
                args: vec![name],
            },
            style: Style::default(),
            siblings: Vec::new(),
        };
        self.titles
            .set_now_playing(self.lang.runs(&message, &Style::default()));
    }

    /// The title, action bar and tab list packets.
    pub fn receive_hud(&mut self, events: Vec<HudEvent>) {
        let runs = |lang: &Lang, component: &photon_game::text::Component| {
            lang.runs(component, &Style::default())
        };
        for event in events {
            match event {
                HudEvent::Title(title) => self.titles.set_title(runs(&self.lang, &title)),
                HudEvent::Subtitle(subtitle) => {
                    self.titles.set_subtitle(runs(&self.lang, &subtitle));
                }
                HudEvent::ActionBar(message) => {
                    self.titles.set_overlay(runs(&self.lang, &message));
                }
                HudEvent::Times {
                    fade_in,
                    stay,
                    fade_out,
                } => self.titles.set_times(fade_in, stay, fade_out),
                HudEvent::Clear { reset_times } => self.titles.clear(reset_times),
                HudEvent::TabList { header, footer } => {
                    self.tab_header = runs(&self.lang, &header);
                    self.tab_footer = runs(&self.lang, &footer);
                }
                HudEvent::Boss { id, operation } => {
                    let now = self.started_at.elapsed().as_millis() as u64;
                    match operation {
                        BossOperation::Add {
                            name,
                            progress,
                            color,
                            overlay,
                            flags,
                        } => {
                            let name = runs(&self.lang, &name);
                            self.boss_bars
                                .add(id, name, progress, color, overlay, flags, now);
                        }
                        BossOperation::Remove => self.boss_bars.remove(id),
                        BossOperation::Progress(progress) => {
                            self.boss_bars.set_progress(id, progress, now);
                        }
                        BossOperation::Name(name) => {
                            let name = runs(&self.lang, &name);
                            self.boss_bars.set_name(id, name);
                        }
                        BossOperation::Style { color, overlay } => {
                            self.boss_bars.set_style(id, color, overlay);
                        }
                        BossOperation::Properties(flags) => self.boss_bars.set_flags(id, flags),
                    }
                }
            }
        }
    }

    /// The players of the tab list, in the order it lists them, and the
    /// objective whose scores it shows.
    pub(super) fn tab_entries(
        &self,
        session: &PlaySession,
    ) -> (Vec<TabEntry>, Option<TabObjective>) {
        tab_list::entries(
            session.listed_players().into_iter().map(|info| TabPlayer {
                name: &info.name,
                display_name: info.display_name.as_ref(),
                game_mode: info.game_mode,
                latency: info.latency,
                list_order: info.list_order,
            }),
            session.scoreboard(),
            &self.lang,
        )
    }
}
