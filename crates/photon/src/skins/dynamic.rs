//! The profiles of heads that name their owner by name or id only (`ResolvableProfile.Dynamic`):
//! the tab list answers for the players in it, the account services for the rest, on threads of
//! their own. Until an answer is there, or where there is none, the head wears the default skin
//! of the partial profile.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

use photon_core::binary::Uuid;
use photon_net::connection::PlaySession;
use photon_net::skins::{ProfileProperty, fetch_by_id, fetch_by_name};

use super::profile::StaticProfile;
use super::{MAX_LOADING, RETRY};

/// The one thing a dynamic profile names its owner by.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Dynamic {
    Name(String),
    Id(Uuid),
}

/// A profile that is complete, or one that still has to be looked up.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Resolvable {
    Static(StaticProfile),
    Dynamic(Dynamic),
}

enum Entry {
    Loading,
    Ready(StaticProfile),
    Failed(Instant),
}

pub(crate) struct Lookups {
    entries: HashMap<Dynamic, Entry>,
    loading: usize,
    finished: Receiver<(Dynamic, Option<StaticProfile>)>,
    sender: Sender<(Dynamic, Option<StaticProfile>)>,
}

impl Default for Lookups {
    fn default() -> Self {
        let (sender, finished) = channel();
        Self {
            entries: HashMap::new(),
            loading: 0,
            finished,
            sender,
        }
    }
}

/// The profile a partial one stands for until it is looked up: its id if it has one.
fn partial(dynamic: &Dynamic) -> StaticProfile {
    StaticProfile {
        id: match dynamic {
            Dynamic::Id(id) => *id,
            Dynamic::Name(_) => Uuid::new(0, 0),
        },
        properties: Vec::new(),
    }
}

impl Lookups {
    /// A frame begins: the answers that came in are taken.
    pub(crate) fn begin_frame(&mut self) {
        while let Ok((dynamic, found)) = self.finished.try_recv() {
            self.loading = self.loading.saturating_sub(1);
            let entry = found.map_or_else(|| Entry::Failed(Instant::now()), Entry::Ready);
            self.entries.insert(dynamic, entry);
        }
    }

    /// The profile of a head: complete as it is, or what the lookup of its name or id gave.
    pub(crate) fn profile(&mut self, profile: Resolvable, session: &PlaySession) -> StaticProfile {
        let dynamic = match profile {
            Resolvable::Static(profile) => return profile,
            Resolvable::Dynamic(dynamic) => dynamic,
        };
        if let Some(found) = in_tab_list(&dynamic, session) {
            return found;
        }
        match self.entries.get(&dynamic) {
            Some(Entry::Ready(found)) => return found.clone(),
            Some(Entry::Loading) => return partial(&dynamic),
            Some(Entry::Failed(at)) if at.elapsed() < RETRY => return partial(&dynamic),
            _ => {}
        }
        if self.loading < MAX_LOADING {
            self.start(dynamic.clone());
        }
        partial(&dynamic)
    }

    fn start(&mut self, dynamic: Dynamic) {
        let sender = self.sender.clone();
        let asked = dynamic.clone();
        let spawned = std::thread::Builder::new()
            .name("Profile Lookup".to_owned())
            .spawn(move || {
                let timeout = std::time::Duration::from_secs(10);
                let fetched = match &asked {
                    Dynamic::Name(name) => fetch_by_name(name, timeout),
                    Dynamic::Id(id) => fetch_by_id(*id, timeout),
                };
                let found = fetched.map(|profile| StaticProfile {
                    id: profile.id,
                    properties: profile.properties,
                });
                let _ = sender.send((asked, found));
            });
        if spawned.is_ok() {
            self.loading += 1;
            self.entries.insert(dynamic, Entry::Loading);
        }
    }
}

/// `LocalPlayerResolver`: a player of the tab list is answered with the profile the server sent.
fn in_tab_list(dynamic: &Dynamic, session: &PlaySession) -> Option<StaticProfile> {
    let (id, properties): (Uuid, &[ProfileProperty]) = match dynamic {
        Dynamic::Name(name) => session.tab_profile_by_name(name)?,
        Dynamic::Id(id) => (*id, session.tab_profile_by_id(*id)?),
    };
    Some(StaticProfile {
        id,
        properties: properties.to_vec(),
    })
}
