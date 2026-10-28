//! The server list's work: the pings' answers, the forms, the delete question and what a press on
//! the list does.

use photon_render::window::WindowSurface;

use super::*;

impl Menu {
    /// The mouse, in GUI units, after a move.
    pub(super) fn track_mouse(&mut self, event: &WindowEvent, surface: &WindowSurface) {
        if let WindowEvent::CursorPos(x, y) = *event {
            let scale = surface.scale();
            let (width, height) = surface.window_size();
            self.mouse = (
                (x * f64::from(scale.gui_scaled_width()) / f64::from(width.max(1))) as i32,
                (y * f64::from(scale.gui_scaled_height()) / f64::from(height.max(1))) as i32,
            );
        }
    }

    /// An event for the server list: what it asks for.
    pub(super) fn servers_event(&mut self, event: WindowEvent) -> Option<ServersAction> {
        let mouse = self.mouse;
        let Screen::Servers(screen) = &mut self.screen else {
            return None;
        };
        match event {
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                let now = Instant::now();
                let double = self
                    .last_click
                    .is_some_and(|(at, place)| now - at < DOUBLE_CLICK && place == mouse);
                self.last_click = Some((now, mouse));
                screen.clicked(mouse, double)
            }
            WindowEvent::Scroll(_, notches) => {
                screen.scrolled((f64::from(mouse.0), f64::from(mouse.1)), notches);
                None
            }
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => screen.key_pressed(false),
            WindowEvent::Key(Key::Enter | Key::KpEnter, _, Action::Press, _) => {
                screen.key_pressed(true)
            }
            // `JoinMultiplayerScreen.keyPressed`: F5 refreshes.
            WindowEvent::Key(Key::F5, _, Action::Press, _) => Some(ServersAction::Refresh),
            _ => None,
        }
    }

    /// An event for a form of the list: its result, once it has one.
    pub(super) fn form_event(
        &mut self,
        event: WindowEvent,
        surface: &mut WindowSurface,
    ) -> Option<FormResult> {
        let mouse = self.mouse;
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let Screen::Form(screen, _) = &mut self.screen else {
            return None;
        };
        match event {
            WindowEvent::Char(character) => {
                screen.char_typed(font, character);
                None
            }
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => screen
                .key_pressed(
                    (font, lang),
                    key,
                    (
                        modifiers.contains(Modifiers::Control),
                        modifiers.contains(Modifiers::Shift),
                    ),
                    &mut crate::screens::WindowClipboard(surface),
                ),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, modifiers) => {
                screen.clicked((font, lang), mouse, modifiers.contains(Modifiers::Shift))
            }
            _ => None,
        }
    }

    /// The pings that have answered, shown on the list's entries of their addresses.
    pub(super) fn receive_pings(&mut self) {
        let Some(pings) = &self.pings else {
            return;
        };
        let Screen::Servers(screen) = &mut self.screen else {
            return;
        };
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let mut icons = Vec::new();
        while let Ok((ip, pinged)) = pings.try_recv() {
            if let Some(index) = self
                .servers
                .servers
                .iter()
                .position(|server| server.ip == ip)
            {
                screen.set_pinged(index, &pinged, font, lang);
                if let Some(favicon) = &pinged.favicon {
                    icons.push((index, favicon.clone()));
                }
            }
        }
        for (index, favicon) in icons {
            self.offer_ping_icon(index, &favicon);
        }
    }

    /// `deleteCallback`: Yes removes the server and saves; either answer shows the list again.
    pub(super) fn delete_answer(
        &mut self,
        answer: Option<ConfirmAction>,
        index: usize,
        size: (i32, i32),
    ) -> Result<Option<Choice>> {
        let Some(answer) = answer else {
            return Ok(None);
        };
        if answer == ConfirmAction::Yes && index < self.servers.servers.len() {
            self.servers.servers.remove(index);
            self.servers.save(&self.game_directory)?;
        }
        self.screen = self.server_list(size);
        Ok(None)
    }

    /// What leaving a form does (`addServerCallback`, `editServerCallback`, `directJoinCallback`):
    /// the list is saved and shown again, or the direct address joined.
    pub(super) fn form_result(
        &mut self,
        result: Option<FormResult>,
        size: (i32, i32),
    ) -> Result<Option<Choice>> {
        let Some(result) = result else {
            return Ok(None);
        };
        let Screen::Form(form, editing) = &self.screen else {
            return Ok(None);
        };
        let (kind, editing) = (form.kind(), *editing);
        if kind == FormKind::Direct {
            // `DirectJoinServerScreen.removed`: the address is kept for the next time.
            let mut options = crate::controls::load_options();
            options.last_server = form.address();
            options.save(&crate::controls::options_path())?;
        }
        let FormResult::Accepted { name, ip, pack } = result else {
            self.screen = self.server_list(size);
            return Ok(None);
        };
        if kind == FormKind::Direct {
            self.join_direct(&name, &ip, size)?;
            return Ok(None);
        }
        self.save_form(kind, editing, (name, ip, pack))?;
        self.screen = self.server_list(size);
        Ok(None)
    }

    /// `addServerCallback` and `editServerCallback`: the list with the form's server, saved.
    fn save_form(
        &mut self,
        kind: FormKind,
        editing: Option<usize>,
        (name, ip, pack): (String, String, PackStatus),
    ) -> Result<()> {
        if kind == FormKind::Add {
            match self.servers.unhide(&ip) {
                Some(server) => server.name = name,
                None => self.servers.add(
                    ServerData {
                        pack_status: pack,
                        ..ServerData::new(&name, &ip)
                    },
                    false,
                ),
            }
        } else if let Some(server) = editing.and_then(|index| self.servers.servers.get_mut(index)) {
            server.name = name;
            server.ip = ip;
            server.pack_status = pack;
        }
        self.servers.save(&self.game_directory)?;
        Ok(())
    }

    /// `directJoinCallback`: the address is added to the list when it is new, and joined.
    fn join_direct(&mut self, name: &str, ip: &str, size: (i32, i32)) -> Result<()> {
        if self.servers.find(ip).is_none() {
            self.servers.add(ServerData::new(name, ip), true);
            self.servers.save(&self.game_directory)?;
        }
        if let Some(address) = pings::resolve(ip) {
            self.connect(address, Some(ip), size);
        }
        Ok(())
    }

    /// A form of the server list.
    pub(super) fn form(&self, kind: FormKind, size: (i32, i32), editing: Option<usize>) -> Screen {
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        let server = editing.and_then(|index| self.servers.servers.get(index));
        let last = crate::controls::load_options().last_server;
        let fields = match (kind, server) {
            (FormKind::Direct, _) => ("", last.as_str(), PackStatus::Prompt),
            (_, Some(server)) => (server.name.as_str(), server.ip.as_str(), server.pack_status),
            _ => ("", "", PackStatus::Prompt),
        };
        Screen::Form(
            ServerFormScreen::new(font, lang, kind, size, fields),
            editing,
        )
    }

    /// What a press on the server list does.
    pub(super) fn servers_action(
        &mut self,
        action: Option<ServersAction>,
        size: (i32, i32),
    ) -> Result<Option<Choice>> {
        match action {
            Some(ServersAction::Join(index)) => {
                self.join_listed(index, size);
            }
            Some(ServersAction::Add) => self.screen = self.form(FormKind::Add, size, None),
            Some(ServersAction::Edit(index)) => {
                self.screen = self.form(FormKind::Edit, size, Some(index));
            }
            Some(ServersAction::Direct) => self.screen = self.form(FormKind::Direct, size, None),
            Some(ServersAction::Delete(index)) => {
                self.ask_delete(index, size);
            }
            Some(ServersAction::Swap(from, to)) => self.swap_servers(from, to)?,
            Some(ServersAction::Refresh) => self.screen = self.server_list(size),
            Some(ServersAction::Back) => self.screen = self.title(size),
            None => {}
        }
        Ok(None)
    }

    /// A join of the listed server.
    fn join_listed(&mut self, index: usize, size: (i32, i32)) {
        let Some(server) = self.servers.servers.get(index) else {
            return;
        };
        if let Some(address) = pings::resolve(&server.ip) {
            let ip = server.ip.clone();
            self.connect(address, Some(&ip), size);
        }
    }

    /// The question about deleting a server, shown.
    fn ask_delete(&mut self, index: usize, size: (i32, i32)) {
        if let Some(question) = self.delete_question(index, size) {
            self.screen = Screen::Confirm(question, index);
        }
    }

    /// The question about deleting a server.
    fn delete_question(&self, index: usize, size: (i32, i32)) -> Option<ConfirmScreen> {
        let server = self.servers.servers.get(index)?;
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        Some(ConfirmScreen::new(
            font,
            lang,
            size,
            &Component::translatable("selectServer.deleteQuestion"),
            &Component::translatable_with(
                "selectServer.deleteWarning",
                vec![Component::literal(&server.name)],
            ),
            (
                &Component::translatable("selectServer.deleteButton"),
                &Component::translatable("gui.cancel"),
            ),
        ))
    }

    /// Two entries of the list changed places (dragged), and the list is saved.
    fn swap_servers(&mut self, from: usize, to: usize) -> Result<()> {
        self.servers.swap(from, to);
        if from.max(to) < self.icons_shown.len() {
            self.icons_shown.swap(from, to);
        }
        self.servers.save(&self.game_directory)?;
        Ok(())
    }
}

impl Menu {
    /// The question about the server's resource packs, when one was raised while the join is
    /// going on (`PackConfirmScreen` over the connect screen). The packs' reloads are answered
    /// here too.
    pub(super) fn ask_about_packs(&mut self, client: &mut Client, size: (i32, i32)) {
        let packs = crate::packs::shared();
        crate::packs::serve_reload(&packs, &mut |loaded| self.reload_assets(client, loaded));
        let Screen::Connecting(_, _, question) = &mut self.screen else {
            return;
        };
        let Some(prompt) = packs.take_prompt() else {
            return;
        };
        let (font, lang) = (self.gui.text_assets().0, self.gui.lang());
        *question = Some(
            ConfirmScreen::server_pack(font, lang, size, prompt.required, prompt.text.as_ref())
                .with_background(photon_gui::screen::confirm::ConfirmBackground::Menu {
                    in_world: false,
                }),
        );
    }

    /// The player answered that question.
    pub(super) fn answer_pack_question(&mut self, accepted: bool) {
        if let Screen::Connecting(_, _, question) = &mut self.screen {
            *question = None;
        }
        crate::packs::shared().answer_prompt(accepted);
    }
}

impl Menu {
    /// One window event while a join goes on: it answers the question about the server's packs
    /// when one is up, and otherwise Cancel goes back to the list (the join's thread finishes
    /// alone and is dropped).
    pub(super) fn connecting_event(&mut self, event: WindowEvent, size: (i32, i32)) {
        let mouse = self.mouse;
        let Screen::Connecting(screen, task, question) = &mut self.screen else {
            return;
        };
        if let Some(question) = question {
            if let Some(action) = confirm_event(question, event, mouse) {
                self.answer_pack_question(action == ConfirmAction::Yes);
            }
            return;
        }
        if let WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) = event
            && !task.status.loading()
            && screen.cancelled(mouse)
        {
            crate::packs::shared().cleanup_after_disconnect();
            self.screen = self.server_list(size);
        }
    }
}

impl Menu {
    /// The resource packs the connection settled on are put in use: the assets are read through
    /// the new stack and everything made of them is made again. A failure puts the stack that
    /// was in use back and says so.
    pub(super) fn reload_assets(
        &mut self,
        client: &mut Client,
        packs: &crate::packs::LoadedPacks,
    ) -> bool {
        let Ok(root) = env::var("PHOTON_ASSETS") else {
            return false;
        };
        let root = Path::new(&root);
        let previous = crate::resources::shared(root);
        let stack = match crate::resources::with_packs(root, packs) {
            Ok(stack) => stack,
            Err(error) => {
                log::warn!("resource packs: {error}");
                return false;
            }
        };
        crate::resources::replace(std::sync::Arc::new(stack));
        let started = Instant::now();
        let rebuilt = self.rebuild_assets(client, root);
        log::info!("resources rebuilt in {:?}", started.elapsed());
        match rebuilt {
            Ok(()) => true,
            Err(error) => {
                log::warn!("resource reload failed: {error}");
                crate::resources::replace(previous);
                false
            }
        }
    }

    /// What is made of the assets, made again: the world's atlas, the GUI, the panorama and the
    /// sounds. The GPU scene is made anew by the atlas, so the textures sent after it follow it.
    fn rebuild_assets(&mut self, client: &mut Client, root: &Path) -> Result<()> {
        let root_text = root.to_string_lossy();
        let (renderer, animated) = crate::play::bake_blocks(&root_text, &mut client.backend)?;
        client.block_renderer = renderer;
        client.animated_atlas = animated;
        self.gui
            .reload_assets(root, &mut client.backend, &self.options.language_code)?;
        client.backend.set_gui_only(true);
        client
            .backend
            .load_panorama(&photon_render::panorama::layers(&super::panorama::faces(
                &root_text,
            )?))?;
        self.assets = self.gui.screen_assets();
        // The new atlas holds no favicons: the list puts them in again.
        self.icons_shown.clear();
        self.icons_dirty = true;
        self.sound.reload_for_packs(&self.options);
        Ok(())
    }
}
