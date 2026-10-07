use super::*;

impl Default for SignatureCache {
    fn default() -> Self {
        Self {
            entries: vec![None; 128],
        }
    }
}

impl SignatureCache {
    /// `MessageSignature.Packed.unpack`: `None` for a number nothing holds.
    pub fn unpack(&self, packed: &PackedSignature) -> Option<ChatSignature> {
        match packed {
            PackedSignature::Full(signature) => Some(signature.clone()),
            PackedSignature::Cached(id) => usize::try_from(*id)
                .ok()
                .and_then(|id| self.entries.get(id))
                .and_then(Clone::clone),
        }
    }

    /// `MessageSignatureCache.push`: the new signatures go in at the front,
    /// the old ones they do not repeat shifted back.
    pub fn push(&mut self, mut queue: std::collections::VecDeque<ChatSignature>) {
        let new: std::collections::HashSet<ChatSignature> = queue.iter().cloned().collect();
        for index in 0..self.entries.len() {
            if queue.is_empty() {
                break;
            }
            let old = self.entries[index].take();
            self.entries[index] = queue.pop_back();
            if let Some(old) = old
                && !new.contains(&old)
            {
                queue.push_front(old);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PlayerPositionPacket {
    pub(crate) teleport_id: i32,
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) z: f64,
    pub(crate) delta_x: f64,
    pub(crate) delta_y: f64,
    pub(crate) delta_z: f64,
    pub(crate) y_rot: f32,
    pub(crate) x_rot: f32,
    pub(crate) relative_flags: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PaletteObservation {
    pub(super) bits_per_entry: u8,
    pub(super) palette_entry_count: usize,
    pub(super) palette_ids: Vec<i32>,
    pub(super) storage_word_count: usize,
    pub(super) first_values: Vec<i32>,
    pub(super) values: Vec<i32>,
    pub(super) distinct_value_count: usize,
}
