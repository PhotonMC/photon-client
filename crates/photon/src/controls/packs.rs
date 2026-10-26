//! The Resource Packs screen over the options screen (`PackSelectionScreen`), and what choosing
//! packs changes: the lists in `options.txt` and a reload of the resources.

use std::time::{Duration, Instant};

use photon_assets::{
    PackEntry, Repository, SelectionModel, discover_built_in, discover_folder, options_lists,
    vanilla_entry,
};
use photon_game::text::Component;
use photon_gui::options::packs::{PackInput, PackScreen};
use photon_gui::screen::confirm::ConfirmBackground;

use super::unattended::ConfirmTarget;
use super::*;

/// `PackSelectionScreen.RELOAD_COOLDOWN` (20 ticks) after the folder was seen to change.
const RELOAD_COOLDOWN: Duration = Duration::from_secs(1);
/// How often the folder is looked at; the reference gets events from the system instead.
const POLL_EVERY: Duration = Duration::from_millis(250);

/// What the folder held when it was looked at: names, sizes and times of change.
type Snapshot = Vec<(std::ffi::OsString, u64, Option<std::time::SystemTime>)>;

/// The state of the folder watch (`PackSelectionScreen.Watcher` and `ticksToReload`).
pub(crate) struct FolderWatch {
    seen: Snapshot,
    polled: Instant,
    reload_at: Option<Instant>,
}

fn snapshot(folder: &std::path::Path) -> Snapshot {
    let mut found: Snapshot = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| {
            let meta = entry.metadata().ok();
            (
                entry.file_name(),
                meta.as_ref().map_or(0, std::fs::Metadata::len),
                meta.and_then(|meta| meta.modified().ok()),
            )
        })
        .collect();
    found.sort();
    found
}

/// The folder the packs are in, in the game directory.
fn pack_folder() -> std::path::PathBuf {
    photon_assets::pack_directory(&game_directory())
}

/// The icon of the game's own pack, if the assets are there.
fn vanilla_icon() -> Option<Vec<u8>> {
    let root = std::env::var("PHOTON_ASSETS").ok()?;
    crate::assets::vanilla_pack(std::path::Path::new(&root)).icon()
}

/// Every pack that can be chosen: the game's own, the built-in ones and the folder's.
fn all_entries(folder: &std::path::Path) -> Vec<PackEntry> {
    let mut entries = vec![vanilla_entry(vanilla_icon())];
    if let Ok(root) = std::env::var("PHOTON_ASSETS") {
        let zips = crate::resources::built_in_zips(std::path::Path::new(&root));
        entries.extend(discover_built_in(&zips, crate::resources::PACK_FORMAT));
    }
    entries.extend(discover_folder(folder, crate::resources::PACK_FORMAT));
    entries
}

impl Controls {
    /// The packs the screen offers, with the selection `options.txt` holds.
    fn pack_model(&self) -> SelectionModel {
        let entries = all_entries(&pack_folder());
        SelectionModel::new(Repository::new(entries, &self.options.resource_packs()))
    }

    /// `PackSelectionScreen`, over the options screen.
    pub(super) fn open_packs(&mut self, surface: &WindowSurface, assets: &ScreenAssets) {
        let folder = pack_folder();
        self.pack_watch = Some(FolderWatch {
            seen: snapshot(&folder),
            polled: Instant::now(),
            reload_at: None,
        });
        self.pack_screen = Some(PackScreen::new(
            &assets.creative.font,
            &assets.lang,
            Self::gui_size(surface),
            !self.title_host,
            self.pack_model(),
            folder,
        ));
    }

    /// The folder looked at again now and then: after a change it is read once it has stayed
    /// still for [`RELOAD_COOLDOWN`].
    pub(crate) fn pack_poll(&mut self, assets: &ScreenAssets) {
        let (Some(screen), Some(watch)) = (self.pack_screen.as_mut(), self.pack_watch.as_mut())
        else {
            return;
        };
        let now = Instant::now();
        if now - watch.polled >= POLL_EVERY {
            watch.polled = now;
            let seen = snapshot(screen.folder());
            if seen != watch.seen {
                watch.seen = seen;
                watch.reload_at = Some(now + RELOAD_COOLDOWN);
            }
        }
        if watch.reload_at.is_some_and(|at| now >= at) {
            watch.reload_at = None;
            let all = all_entries(screen.folder());
            screen.reload(&assets.lang, all);
        }
    }

    /// An event for the open Resource Packs screen.
    pub(super) fn packs_event(
        &mut self,
        event: glfw::WindowEvent,
        surface: &mut WindowSurface,
        assets: &ScreenAssets,
    ) {
        use glfw::{Action, Key, MouseButton, WindowEvent};
        let font = &assets.creative.font;
        if let WindowEvent::CursorPos(x, y) = event {
            let before = self.gui_mouse.1;
            self.move_gui_mouse((x, y), surface);
            let mouse = self.gui_mouse;
            if let Some(screen) = self.pack_screen.as_mut() {
                screen.dragged(mouse, mouse.1 - before);
            }
            return;
        }
        let mouse = self.gui_mouse;
        let control = self.mods.control;
        let wheel = self.options.mouse_wheel_sensitivity;
        let Some(screen) = self.pack_screen.as_mut() else {
            return;
        };
        let mut input = PackInput::default();
        match event {
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => input.done = true,
            WindowEvent::Key(key, _, Action::Press | Action::Repeat, modifiers) => {
                let shift = modifiers.contains(glfw::Modifiers::Shift);
                input =
                    screen.key_pressed(font, key, (control, shift), &mut WindowClipboard(surface));
            }
            WindowEvent::Char(character) => screen.char_typed(font, character),
            WindowEvent::MouseButton(MouseButton::Button1, Action::Press, _) => {
                input = screen.clicked(font, mouse);
            }
            WindowEvent::MouseButton(MouseButton::Button1, Action::Release, _) => {
                screen.released();
            }
            WindowEvent::Scroll(_, y) => screen.scrolled(mouse, y * wheel),
            _ => {}
        }
        self.packs_input(input, surface, assets);
    }

    /// What the screen asked for.
    fn packs_input(&mut self, input: PackInput, surface: &WindowSurface, assets: &ScreenAssets) {
        if input.open_folder {
            let folder = pack_folder();
            let _ = std::fs::create_dir_all(&folder);
            open_uri(&folder.to_string_lossy());
        }
        if let Some((id, compatibility)) = input.confirm {
            let key = compatibility.key();
            let screen = ConfirmScreen::new(
                &assets.creative.font,
                &assets.lang,
                Self::gui_size(surface),
                &Component::translatable("pack.incompatible.confirm.title"),
                &Component::translatable(format!("pack.incompatible.confirm.{key}")),
                &Component::translatable("gui.yes"),
                &Component::translatable("gui.no"),
            )
            .with_background(ConfirmBackground::Menu {
                in_world: !self.title_host,
            });
            self.link_confirm = Some((screen, ConfirmTarget::Pack(id)));
        }
        if input.done {
            self.close_packs();
        }
    }

    /// The player agreed to a pack that does not fit.
    pub(super) fn select_confirmed_pack(&mut self, id: &str) {
        if let Some(screen) = self.pack_screen.as_mut() {
            screen.select_confirmed(id);
        }
    }

    /// `PackSelectionScreen.onClose`: the lists are written to `options.txt`, and the resources
    /// are loaded again when the selection is another.
    pub(super) fn close_packs(&mut self) {
        let Some(mut screen) = self.pack_screen.take() else {
            return;
        };
        self.pack_watch = None;
        let (packs, incompatible) = options_lists(screen.commit());
        // `PackEntry.toggleSelection` flips the option when the pack moves.
        let had = self
            .options
            .resource_packs()
            .iter()
            .any(|id| id == "high_contrast");
        if had != packs.iter().any(|id| id == "high_contrast") {
            let flipped = !self.options.high_contrast();
            self.options.set_high_contrast(flipped);
        }
        let changed = self.options.set_resource_packs(&packs, &incompatible);
        self.save_options();
        self.pack_reload |= changed;
    }

    /// `Options.highContrast`'s callback: the pack is put on top of the selection, or taken off
    /// it, and the resources are loaded again.
    pub(super) fn switch_high_contrast_pack(&mut self, on: bool) {
        const ID: &str = "high_contrast";
        let mut packs = self.options.resource_packs();
        let selected = packs.iter().any(|pack| pack == ID);
        if on && !selected {
            packs.push(ID.to_owned());
        } else if !on && selected {
            packs.retain(|pack| pack != ID);
        } else {
            return;
        }
        let incompatible = self.options.incompatible_resource_packs();
        self.options.set_resource_packs(&packs, &incompatible);
        // `updateResourcePacks` saves before it reloads, and the load reads the file.
        self.save_options();
        self.pack_reload = true;
    }

    /// Whether the resources are to be loaded again, once.
    pub fn take_pack_reload(&mut self) -> bool {
        std::mem::take(&mut self.pack_reload)
    }
}
