//! `ClientWorld` metadata layer split out of the former single-file module.

use super::*;
pub(crate) use player::*;

mod fields;
pub use fields::*;
mod living;
pub(crate) use living::*;
mod mobs;
pub(crate) use mobs::*;
mod player;
mod vehicles;
pub(crate) use vehicles::*;
