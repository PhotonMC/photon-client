//! The delay `ChatListener` puts between the messages shown (the `chatDelay` option): a message
//! that comes before the delay since the last one was shown waits in a queue.

use std::collections::VecDeque;

/// `ChatListener`'s queue and clock. What waits is the caller's own record of a message; whether
/// it was shown is what the caller answers when the queue hands it back.
pub struct ChatQueue<T> {
    queue: VecDeque<(Option<Vec<u8>>, T)>,
    /// `messageDelay`, in milliseconds.
    delay: u64,
    /// `previousMessageTime`: when a message was last shown, in the caller's milliseconds; none
    /// is the zero Java starts from, which is long before any clock reads.
    previous: Option<u64>,
}

impl<T> Default for ChatQueue<T> {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            delay: 0,
            previous: None,
        }
    }
}

impl<T> ChatQueue<T> {
    /// `queueSize`: how many messages wait.
    pub fn size(&self) -> usize {
        self.queue.len()
    }

    /// `setMessageDelay`: a delay that goes to none lets the waiting messages through.
    pub fn set_delay(&mut self, seconds: f64) -> Vec<T> {
        let delay = (seconds * 1000.0) as u64;
        let flushed = if delay == 0 && self.delay > 0 {
            self.flush()
        } else {
            Vec::new()
        };
        self.delay = delay;
        flushed
    }

    /// `willDelayMessages`: the last message shown is too recent. The queue being empty does not
    /// matter, so a message after the delay is shown ahead of the ones that wait.
    pub fn will_delay(&self, now: u64) -> bool {
        self.delay > 0
            && self
                .previous
                .is_some_and(|previous| now < previous + self.delay)
    }

    /// `previousMessageTime = Util.getMillis()`: a message was shown.
    pub fn shown(&mut self, now: u64) {
        self.previous = Some(now);
    }

    /// A message that waits (`handleMessage` while a message is delayed).
    pub fn push(&mut self, signature: Option<Vec<u8>>, item: T) {
        self.queue.push_back((signature, item));
    }

    /// `flushQueue`: every message that waits, in order.
    pub fn flush(&mut self) -> Vec<T> {
        self.previous = None;
        self.queue.drain(..).map(|(_, item)| item).collect()
    }

    /// `removeFromDelayedMessageQueue`: whether a message with the signature was waiting.
    pub fn remove(&mut self, signature: &[u8]) -> bool {
        let before = self.queue.len();
        self.queue
            .retain(|(own, _)| own.as_deref() != Some(signature));
        self.queue.len() != before
    }

    /// The part of `tick` that lets a message go: with no delay everything goes; otherwise, once
    /// the delay has passed, messages are taken until one is shown.
    pub fn due(&mut self, now: u64, mut show: impl FnMut(T) -> bool) {
        if self.delay == 0 {
            self.flush().into_iter().for_each(|item| {
                show(item);
            });
        } else if self
            .previous
            .is_none_or(|previous| now >= previous + self.delay)
        {
            while let Some((_, item)) = self.queue.pop_front() {
                if show(item) {
                    self.previous = Some(now);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_inside_the_delay_waits_and_one_after_it_does_not() {
        let mut queue = ChatQueue::<u32>::default();
        queue.set_delay(1.0);
        assert!(!queue.will_delay(0));
        queue.shown(100);
        assert!(queue.will_delay(1099));
        assert!(!queue.will_delay(1100));
    }

    #[test]
    fn no_delay_never_waits() {
        let mut queue = ChatQueue::<u32>::default();
        queue.shown(100);
        assert!(!queue.will_delay(100));
    }

    #[test]
    fn the_queue_lets_one_shown_message_go_a_delay() {
        let mut queue = ChatQueue::default();
        queue.set_delay(1.0);
        queue.shown(0);
        queue.push(None, 1);
        queue.push(None, 2);
        let mut seen = Vec::new();
        queue.due(500, |item| {
            seen.push(item);
            true
        });
        assert!(seen.is_empty());
        queue.due(1000, |item| {
            seen.push(item);
            true
        });
        assert_eq!(seen, [1]);
        assert_eq!(queue.size(), 1);
        // The next waits for a delay from the one just shown.
        queue.due(1500, |item| {
            seen.push(item);
            true
        });
        assert_eq!(seen, [1]);
        queue.due(2000, |item| {
            seen.push(item);
            true
        });
        assert_eq!(seen, [1, 2]);
    }

    #[test]
    fn messages_that_are_not_shown_are_passed_over() {
        let mut queue = ChatQueue::default();
        queue.set_delay(1.0);
        for item in [1, 2, 3] {
            queue.push(None, item);
        }
        let mut seen = Vec::new();
        queue.due(1000, |item| {
            seen.push(item);
            item == 3
        });
        assert_eq!(seen, [1, 2, 3]);
        assert_eq!(queue.size(), 0);
        assert!(queue.will_delay(1500));
    }

    #[test]
    fn no_delay_lets_the_waiting_through_in_order() {
        let mut queue = ChatQueue::default();
        queue.set_delay(1.0);
        queue.push(None, 1);
        queue.push(None, 2);
        assert_eq!(queue.set_delay(0.0), [1, 2]);
        assert_eq!(queue.size(), 0);
    }

    #[test]
    fn a_deleted_message_leaves_the_queue_by_its_signature() {
        let mut queue = ChatQueue::default();
        queue.set_delay(1.0);
        queue.push(Some(vec![1]), 10);
        queue.push(None, 20);
        assert!(queue.remove(&[1]));
        assert!(!queue.remove(&[1]));
        assert_eq!(queue.size(), 1);
    }
}
