//! The chunks the client holds, as `ClientChunkCache` keeps them: a bounded ring of slots around
//! the view centre, a chunk in the slot its position maps to. A level without a cache (a test's or
//! a probe's) keeps what it is given by position.

use std::collections::BTreeMap;

use crate::*;

#[derive(Debug)]
struct Slot {
    pos: (i32, i32),
    chunk: ClientChunk,
}

/// The ring and the Java cache's bookkeeping beside it.
#[derive(Debug)]
struct ChunkCache {
    state: ClientChunkCacheState,
    ring: Vec<Option<Slot>>,
}

impl ChunkCache {
    fn new(server_view_radius: i32) -> Self {
        let state = ClientChunkCacheState::new(server_view_radius);
        let slots = (state.view_range as usize) * (state.view_range as usize);
        Self {
            state,
            ring: std::iter::repeat_with(|| None).take(slots).collect(),
        }
    }

    /// The chunk stored for a position, wherever the view centre is now.
    fn raw(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        let slot = self.ring[self.state.index(chunk_x, chunk_z)].as_ref()?;
        (slot.pos == (chunk_x, chunk_z)).then_some(&slot.chunk)
    }

    fn raw_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ClientChunk> {
        let index = self.state.index(chunk_x, chunk_z);
        let slot = self.ring[index].as_mut()?;
        (slot.pos == (chunk_x, chunk_z)).then_some(&mut slot.chunk)
    }
}

/// What replacing a chunk did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Replaced {
    /// The position is outside the cache's range: nothing was stored.
    OutOfRange,
    /// The chunk is stored; the chunk its slot held before, if it was another, was pushed out.
    Stored { evicted: Option<(i32, i32)> },
}

#[derive(Debug, Default, bevy_ecs::prelude::Resource)]
pub(crate) struct Chunks {
    cache: Option<ChunkCache>,
    loose: BTreeMap<(i32, i32), ClientChunk>,
}

impl Chunks {
    /// Drop every chunk, with a new cache for a server view radius, or none.
    pub(crate) fn reset(&mut self, server_view_radius: Option<i32>) {
        self.loose.clear();
        self.cache = server_view_radius.map(ChunkCache::new);
    }

    pub(crate) fn cache(&self) -> Option<&ClientChunkCacheState> {
        self.cache.as_ref().map(|cache| &cache.state)
    }

    pub(crate) fn cache_mut(&mut self) -> Option<&mut ClientChunkCacheState> {
        self.cache.as_mut().map(|cache| &mut cache.state)
    }

    /// The chunk a query finds: the one stored for the position, if the cache's range holds it.
    pub(crate) fn get(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        match &self.cache {
            Some(cache) => cache
                .state
                .in_range(chunk_x, chunk_z)
                .then(|| cache.raw(chunk_x, chunk_z))
                .flatten(),
            None => self.loose.get(&(chunk_x, chunk_z)),
        }
    }

    pub(crate) fn get_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut ClientChunk> {
        match &mut self.cache {
            Some(cache) => {
                if !cache.state.in_range(chunk_x, chunk_z) {
                    return None;
                }
                cache.raw_mut(chunk_x, chunk_z)
            }
            None => self.loose.get_mut(&(chunk_x, chunk_z)),
        }
    }

    /// The chunk stored for a position even if the view centre has moved away from it.
    pub(crate) fn get_stored(&self, chunk_x: i32, chunk_z: i32) -> Option<&ClientChunk> {
        match &self.cache {
            Some(cache) => cache.raw(chunk_x, chunk_z),
            None => self.loose.get(&(chunk_x, chunk_z)),
        }
    }

    pub(crate) fn len(&self) -> usize {
        match &self.cache {
            Some(cache) => cache.ring.iter().flatten().count(),
            None => self.loose.len(),
        }
    }

    /// The positions of the chunks held, in order.
    pub(crate) fn positions(&self) -> Vec<(i32, i32)> {
        match &self.cache {
            Some(cache) => {
                let mut positions: Vec<_> =
                    cache.ring.iter().flatten().map(|slot| slot.pos).collect();
                positions.sort_unstable();
                positions
            }
            None => self.loose.keys().copied().collect(),
        }
    }

    /// The chunks held, in no particular order.
    pub(crate) fn values(&self) -> impl Iterator<Item = &ClientChunk> {
        let ring = self
            .cache
            .iter()
            .flat_map(|cache| cache.ring.iter().flatten().map(|slot| &slot.chunk));
        ring.chain(self.loose.values())
    }

    /// `ClientChunkCache.drop`: the chunk goes if the cache holds it; whether one went.
    pub(crate) fn unload(&mut self, chunk_x: i32, chunk_z: i32) -> bool {
        let Some(cache) = self.cache.as_mut() else {
            return self.loose.remove(&(chunk_x, chunk_z)).is_some();
        };
        if !cache.state.in_range(chunk_x, chunk_z) {
            return false;
        }
        let index = cache.state.index(chunk_x, chunk_z);
        if cache.ring[index]
            .as_ref()
            .is_none_or(|slot| slot.pos != (chunk_x, chunk_z))
        {
            return false;
        }
        let Some(slot) = cache.ring[index].take() else {
            return false;
        };
        let section_ys = slot.chunk.sections.keys().collect::<Vec<_>>();
        cache.state.on_chunk_removed(chunk_x, chunk_z, section_ys);
        true
    }

    /// `ClientChunkCache.replaceWithPacketData`'s storage: the chunk takes the slot of its
    /// position, and `sections` (each section's number and whether it holds only air) tell the
    /// cache what changed.
    pub(crate) fn replace(
        &mut self,
        (chunk_x, chunk_z): (i32, i32),
        chunk: ClientChunk,
        sections: Vec<(i32, bool)>,
    ) -> Replaced {
        let Some(cache) = self.cache.as_mut() else {
            self.loose.insert((chunk_x, chunk_z), chunk);
            return Replaced::Stored { evicted: None };
        };
        if !cache.state.in_range(chunk_x, chunk_z) {
            return Replaced::OutOfRange;
        }
        let index = cache.state.index(chunk_x, chunk_z);
        let slot = Slot {
            pos: (chunk_x, chunk_z),
            chunk,
        };
        let held = cache.ring[index].replace(slot);
        match held {
            Some(old) if old.pos == (chunk_x, chunk_z) => {
                cache
                    .state
                    .refresh_empty_sections(chunk_x, chunk_z, sections);
                Replaced::Stored { evicted: None }
            }
            held => {
                let evicted = held.map(|old| {
                    let section_ys = old.chunk.sections.keys().collect::<Vec<_>>();
                    cache
                        .state
                        .on_chunk_removed(old.pos.0, old.pos.1, section_ys);
                    old.pos
                });
                cache.state.on_chunk_added(chunk_x, chunk_z, sections);
                Replaced::Stored { evicted }
            }
        }
    }

    /// `ClientChunkCache.updateViewRadius`: a cache of the new size, keeping the chunks the new
    /// range holds. Nothing changes if the storage is the size it was.
    pub(crate) fn resize(&mut self, server_view_radius: i32) {
        let Some(cache) = self.cache.as_mut() else {
            return;
        };
        let storage_radius = server_view_radius.max(2) + 3;
        cache.state.server_view_radius = server_view_radius;
        if cache.state.storage_radius == storage_radius {
            return;
        }
        let mut resized = ChunkCache::new(server_view_radius);
        resized.state.view_center_x = cache.state.view_center_x;
        resized.state.view_center_z = cache.state.view_center_z;
        for slot in cache.ring.iter_mut().filter_map(Option::take) {
            let (chunk_x, chunk_z) = slot.pos;
            if !resized.state.in_range(chunk_x, chunk_z) {
                continue;
            }
            let sections = slot
                .chunk
                .sections
                .iter()
                .map(|(section_y, section)| (section_y, section.has_only_air()))
                .collect::<Vec<_>>();
            resized.state.on_chunk_added(chunk_x, chunk_z, sections);
            let index = resized.state.index(chunk_x, chunk_z);
            resized.ring[index] = Some(slot);
        }
        *cache = resized;
    }
}
