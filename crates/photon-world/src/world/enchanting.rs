//! `EnchantingTableBlockEntity.bookAnimationTick` as the client runs it: the book turns to the
//! player near the table, opens as they come, and flips its pages now and then.

use photon_core::math::mth_atan2;
use photon_core::random::RandomSource;
use photon_data::block::blocks::id;

use crate::block::entity::BookState;
use crate::block::piston::Pos;
use crate::block::{self};
use crate::*;

/// `EnchantingTableBlockEntity.bookAnimationTick`'s reach for a player: three blocks.
const PLAYER_RANGE: f64 = 3.0;
const PI: f32 = std::f32::consts::PI;
const TWO_PI: f32 = std::f32::consts::PI * 2.0;

/// What the tables' ticks read of the level besides its blocks: the class's own random source
/// (shared by every table) and where the client's own player stands.
#[derive(Debug, Clone)]
pub(crate) struct Tables {
    random: RandomSource,
    /// The local player's place, and whether it is a spectator; none until the session says.
    local: Option<((f64, f64, f64), bool)>,
}

impl Default for Tables {
    fn default() -> Self {
        // `RandomSource.create()` seeds itself from the clock.
        let clock = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos() as i64);
        Self {
            random: RandomSource::create(clock),
            local: None,
        }
    }
}

/// What a table's book is drawn from (`EnchantTableRenderState`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnchantTableView {
    pub flip: f32,
    pub open: f32,
    pub time: f32,
    pub y_rot: f32,
}

impl ClientWorld {
    /// The class's random source, seeded.
    pub fn seed_book_random(&mut self, seed: i64) {
        self.tables.random = RandomSource::create(seed);
    }

    /// Where the client's own player stands after it moved this tick, and whether it spectates.
    pub fn set_local_player_spot(&mut self, at: (f64, f64, f64), spectator: bool) {
        self.tables.local = Some((at, spectator));
    }

    pub(super) fn is_enchanting_table_at(&self, (x, y, z): Pos) -> bool {
        block::state_or_air(self.block_state_or_air(x, y, z)).block == id::ENCHANTING_TABLE as u16
    }

    /// `Level.getNearestPlayer(x, y, z, range, false)`: where the nearest of the players in range
    /// who are not spectators stands.
    fn nearest_player(&self, at: (f64, f64, f64), range: f64) -> Option<(f64, f64, f64)> {
        let local = self
            .tables
            .local
            .filter(|(_, spectator)| !spectator)
            .map(|(place, _)| place);
        let remote = self.entities().players().map(|entity| entity.position());
        let mut best: Option<f64> = None;
        let mut nearest = None;
        for place in local.into_iter().chain(remote) {
            let (dx, dy, dz) = (place.0 - at.0, place.1 - at.1, place.2 - at.2);
            let distance = dx * dx + dy * dy + dz * dz;
            if distance < range * range && best.is_none_or(|current| distance < current) {
                best = Some(distance);
                nearest = Some(place);
            }
        }
        nearest
    }

    /// `EnchantingTableBlockEntity.bookAnimationTick`.
    pub(super) fn tick_enchanting_table(&mut self, pos: Pos) {
        let here = (
            f64::from(pos.0) + 0.5,
            f64::from(pos.1) + 0.5,
            f64::from(pos.2) + 0.5,
        );
        let player = self.nearest_player(here, PLAYER_RANGE);
        let Some(mut book) = self
            .block_entity_at(pos.0, pos.1, pos.2)
            .map(|entity| entity.events.book)
        else {
            self.block_tickers.unregister(pos);
            return;
        };
        let random = &mut self.tables.random;
        book.o_open = book.open;
        book.o_rot = book.rot;
        if let Some(place) = player {
            let (xd, zd) = (place.0 - here.0, place.2 - here.2);
            book.t_rot = mth_atan2(zd, xd) as f32;
            book.open += 0.1;
            if book.open < 0.5 || random.next_int_bound(40) == Ok(0) {
                let old = book.flip_t;
                while old == book.flip_t {
                    let step = random.next_int_bound(4).unwrap_or(0)
                        - random.next_int_bound(4).unwrap_or(0);
                    book.flip_t += step as f32;
                }
            }
        } else {
            book.t_rot += 0.02;
            book.open -= 0.1;
        }
        book.rot = wrapped(book.rot);
        book.t_rot = wrapped(book.t_rot);
        let direction = wrapped(book.t_rot - book.rot);
        book.rot += direction * 0.4;
        book.open = book.open.clamp(0.0, 1.0);
        book.time += 1;
        book.o_flip = book.flip;
        let difference = ((book.flip_t - book.flip) * 0.4).clamp(-0.2, 0.2);
        book.flip_a += (difference - book.flip_a) * 0.9;
        book.flip += book.flip_a;
        if let Some(entity) = self.block_entity_mut_at(pos) {
            entity.events.book = book;
        }
    }

    /// `EnchantTableRenderer.extractRenderState`: what the book is drawn from at a partial tick.
    pub fn enchant_table_view(&self, pos: Pos, partial: f32) -> Option<EnchantTableView> {
        if !self.is_enchanting_table_at(pos) {
            return None;
        }
        let book = self.block_entity_at(pos.0, pos.1, pos.2)?.events().book;
        Some(EnchantTableView::of(&book, partial))
    }
}

impl EnchantTableView {
    /// `EnchantTableRenderer.extractRenderState`: the book's numbers between two ticks.
    pub fn of(book: &BookState, partial: f32) -> Self {
        let lerp = |from: f32, to: f32| from + partial * (to - from);
        let turn = wrapped(book.rot - book.o_rot);
        Self {
            flip: lerp(book.o_flip, book.flip),
            open: lerp(book.o_open, book.open),
            time: book.time as f32 + partial,
            y_rot: book.o_rot + turn * partial,
        }
    }
}

/// An angle brought into [-pi, pi) the way the reference's loops do, a turn at a time.
fn wrapped(mut angle: f32) -> f32 {
    while angle >= PI {
        angle -= TWO_PI;
    }
    while angle < -PI {
        angle += TWO_PI;
    }
    angle
}
