//! A value that can be set through a shared reference, for caches that a read fills in. Unlike
//! `std::cell::Cell` it can be shared between threads, which the entity components need.

use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct SyncCell<T: Copy>(Mutex<T>);

impl<T: Copy> SyncCell<T> {
    pub const fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    pub fn get(&self) -> T {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn set(&self, value: T) {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    }
}

impl<T: Copy> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get())
    }
}

impl<T: Copy + PartialEq> PartialEq for SyncCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Copy + Eq> Eq for SyncCell<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_is_set_through_a_shared_reference_and_copied_by_value() {
        let cell = SyncCell::new(1_u32);
        cell.set(5);
        let copy = cell.clone();
        cell.set(7);
        assert_eq!((cell.get(), copy.get()), (7, 5));
    }
}
