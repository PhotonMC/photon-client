//! `LastSeenMessagesTracker`: the last 20 messages the player was shown, which the server asks the
//! chat packets and the acknowledgement packet to account for.

/// How many messages the tracker keeps (`new LastSeenMessagesTracker(20)`).
const COUNT: usize = 20;

/// `LastSeenTrackedEntry`: a signature and whether the server has not yet been told of it.
#[derive(Clone)]
struct Entry {
    signature: Vec<u8>,
    pending: bool,
}

/// `LastSeenMessages.Update`: how many messages came since the last update, which of the kept
/// ones are acknowledged (bit `i` for the `i`th from the oldest) and their checksum.
#[derive(Debug, PartialEq, Eq)]
pub struct LastSeenUpdate {
    pub offset: i32,
    pub acknowledged: u32,
    pub checksum: u8,
    /// The signatures of the acknowledged messages, oldest first (`LastSeenMessages.entries`).
    pub seen: Vec<Vec<u8>>,
}

pub(super) struct LastSeenTracker {
    tracked: [Option<Entry>; COUNT],
    tail: usize,
    offset: i32,
    last_tracked: Option<Vec<u8>>,
}

impl Default for LastSeenTracker {
    fn default() -> Self {
        Self {
            tracked: std::array::from_fn(|_| None),
            tail: 0,
            offset: 0,
            last_tracked: None,
        }
    }
}

/// `Arrays.hashCode(byte[])`, which `MessageSignature.checksum` is.
fn signature_checksum(bytes: &[u8]) -> i32 {
    bytes.iter().fold(1_i32, |hash, &byte| {
        hash.wrapping_mul(31).wrapping_add(i32::from(byte as i8))
    })
}

impl LastSeenTracker {
    /// `addPending`: a message shown or not. False for the message the tracker took last.
    pub(super) fn add_pending(&mut self, signature: &[u8], was_shown: bool) -> bool {
        if self.last_tracked.as_deref() == Some(signature) {
            return false;
        }
        self.last_tracked = Some(signature.to_vec());
        let index = self.tail;
        self.tail = (index + 1) % COUNT;
        self.offset += 1;
        self.tracked[index] = was_shown.then(|| Entry {
            signature: signature.to_vec(),
            pending: true,
        });
        true
    }

    /// `ignorePending`: a message that was deleted before the server heard of it.
    pub(super) fn ignore_pending(&mut self, signature: &[u8]) {
        let found = self.tracked.iter_mut().find(|slot| {
            slot.as_ref()
                .is_some_and(|entry| entry.pending && entry.signature == signature)
        });
        if let Some(slot) = found {
            *slot = None;
        }
    }

    pub(super) fn offset(&self) -> i32 {
        self.offset
    }

    pub(super) fn take_offset(&mut self) -> i32 {
        std::mem::take(&mut self.offset)
    }

    /// `generateAndApplyUpdate`: what a chat packet carries, and every kept message counts as
    /// acknowledged from now on.
    pub(super) fn generate_update(&mut self) -> LastSeenUpdate {
        let offset = self.take_offset();
        let mut acknowledged = 0_u32;
        let mut checksum = 1_i32;
        let mut seen = Vec::new();
        for i in 0..COUNT {
            let index = (self.tail + i) % COUNT;
            if let Some(entry) = self.tracked[index].as_mut() {
                acknowledged |= 1 << i;
                checksum = checksum
                    .wrapping_mul(31)
                    .wrapping_add(signature_checksum(&entry.signature));
                seen.push(entry.signature.clone());
                entry.pending = false;
            }
        }
        let byte = checksum as u8;
        LastSeenUpdate {
            offset,
            acknowledged,
            checksum: if byte == 0 { 1 } else { byte },
            seen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_checksum_of_nothing_is_one() {
        let mut tracker = LastSeenTracker::default();
        assert_eq!(
            tracker.generate_update(),
            LastSeenUpdate {
                offset: 0,
                acknowledged: 0,
                checksum: 1,
                seen: Vec::new()
            }
        );
    }

    #[test]
    fn a_signature_hashes_like_arrays_hash_code() {
        // Arrays.hashCode(new byte[] {1, -1}) = (31 + 1) * 31 + -1.
        assert_eq!(signature_checksum(&[1, 0xFF]), 991);
        assert_eq!(signature_checksum(&[]), 1);
    }

    #[test]
    fn a_shown_message_is_acknowledged_in_the_order_it_came() {
        let mut tracker = LastSeenTracker::default();
        assert!(tracker.add_pending(&[1], true));
        assert!(tracker.add_pending(&[2], false));
        assert!(tracker.add_pending(&[3], true));
        assert_eq!(tracker.offset(), 3);
        let update = tracker.generate_update();
        // The window starts at the tail, so the three newest are the last three slots' worth:
        // bit 0 is the oldest slot of the ring, which holds the first message.
        assert_eq!(update.offset, 3);
        assert_eq!(update.acknowledged.count_ones(), 2);
        let expected = (31 + signature_checksum(&[1])) * 31 + signature_checksum(&[3]);
        assert_eq!(update.checksum, expected as u8);
        assert_eq!(tracker.offset(), 0);
    }

    #[test]
    fn the_message_taken_last_is_not_taken_twice() {
        let mut tracker = LastSeenTracker::default();
        assert!(tracker.add_pending(&[7], true));
        assert!(!tracker.add_pending(&[7], true));
        assert_eq!(tracker.offset(), 1);
    }

    #[test]
    fn a_deleted_message_leaves_the_update_while_pending_only() {
        let mut tracker = LastSeenTracker::default();
        tracker.add_pending(&[1], true);
        tracker.add_pending(&[2], true);
        tracker.ignore_pending(&[1]);
        assert_eq!(tracker.generate_update().acknowledged.count_ones(), 1);
        // Once acknowledged, deleting it changes nothing.
        tracker.ignore_pending(&[2]);
        assert_eq!(tracker.generate_update().acknowledged.count_ones(), 1);
    }
}
