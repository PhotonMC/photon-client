//! The packets the connection counted, as `Connection` counts them: the averages per second that
//! the debug overlay prints as `tx` and `rx`.

/// `Connection.tick`, `tickSecond` and the two packet counters.
#[derive(Debug, Default)]
pub(super) struct Traffic {
    tick_count: u32,
    received: u32,
    average_sent: f32,
    average_received: f32,
    /// Between a bundle's two delimiters: the reference decodes the bundle into one packet.
    in_bundle: bool,
}

impl Traffic {
    /// A packet handled (`Connection.channelRead0`). A bundle is one packet, counted where it ends.
    pub(super) fn received_packet(&mut self, delimiter: bool) {
        if delimiter {
            self.in_bundle = !self.in_bundle;
            if !self.in_bundle {
                self.received += 1;
            }
        } else if !self.in_bundle {
            self.received += 1;
        }
    }

    /// `Connection.tick`: every twentieth tick, the first included, the second is over. `sent` is the
    /// packets written since the last one.
    pub(super) fn tick(&mut self, take_sent: impl FnOnce() -> u32) {
        let second = self.tick_count.is_multiple_of(20);
        self.tick_count = self.tick_count.wrapping_add(1);
        if second {
            // `Mth.lerp(0.75F, count, average)`.
            let lerp = |count: f32, average: f32| count + 0.75 * (average - count);
            self.average_sent = lerp(take_sent() as f32, self.average_sent);
            self.average_received = lerp(self.received as f32, self.average_received);
            self.received = 0;
        }
    }

    pub(super) fn average_sent(&self) -> f32 {
        self.average_sent
    }

    pub(super) fn average_received(&self) -> f32 {
        self.average_received
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_averages_weigh_the_old_one_three_quarters() {
        let mut traffic = Traffic::default();
        for _ in 0..4 {
            traffic.received_packet(false);
        }
        traffic.tick(|| 8);
        assert_eq!(
            (traffic.average_sent(), traffic.average_received()),
            (2.0, 1.0)
        );
        for _ in 0..19 {
            traffic.tick(|| unreachable!());
        }
        traffic.received_packet(false);
        traffic.tick(|| 0);
        assert_eq!(
            (traffic.average_sent(), traffic.average_received()),
            (1.5, 1.0)
        );
    }

    #[test]
    fn a_bundle_counts_once_where_it_ends() {
        let mut traffic = Traffic::default();
        traffic.received_packet(true);
        traffic.received_packet(false);
        traffic.received_packet(false);
        traffic.received_packet(true);
        traffic.received_packet(false);
        traffic.tick(|| 0);
        assert_eq!(traffic.average_received(), 0.5);
    }
}
