//! The server list's icons: the favicons the pings bring, kept with the saved servers and put
//! into the GUI atlas (`ServerSelectionList.OnlineServerEntry`'s `uploadServerIcon`).

use photon_gui::favicon::sync_server_icon;

use super::{Menu, Result, Screen};
use crate::play::Client;

impl Menu {
    /// A ping's favicon offered to the saved server of `index` (`handleStatusResponse`): a new one
    /// is kept, saved, and drawn from the next frame.
    pub(super) fn offer_ping_icon(&mut self, index: usize, favicon: &[u8]) {
        let Some(server) = self.servers.servers.get_mut(index) else {
            return;
        };
        if server.apply_ping_icon(favicon) {
            self.icons_dirty = true;
            if let Err(error) = self.servers.save(&self.game_directory) {
                log::warn!("Failed to save the server list: {error}");
            }
        }
    }

    /// The entries' check of their icons against the ones they show, after the list was made or an
    /// icon came: new icons go into the atlas, which is sent to the GPU again; one that cannot be
    /// used is forgotten and the list saved.
    pub(super) fn sync_icons(&mut self, client: &mut Client) -> Result<()> {
        if !std::mem::take(&mut self.icons_dirty) {
            return Ok(());
        }
        let Screen::Servers(screen) = &mut self.screen else {
            return Ok(());
        };
        self.icons_shown.resize(self.servers.servers.len(), None);
        let (mut image_changed, mut rejected) = (false, false);
        for (index, server) in self.servers.servers.iter_mut().enumerate() {
            let sync = sync_server_icon(
                &mut self.gui.atlas,
                &mut self.gui.atlas_image,
                server,
                &mut self.icons_shown[index],
            );
            if let Some(sync) = sync {
                screen.set_icon(index, sync.texture);
                image_changed |= sync.atlas_changed;
                rejected |= sync.rejected;
            }
        }
        if rejected {
            self.servers.save(&self.game_directory)?;
        }
        if image_changed {
            client.backend.replace_gui_texture(&self.gui.atlas_image)?;
        }
        Ok(())
    }
}
