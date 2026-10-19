//! The toasts the server's packets ask for, kept in the order they came until the interface
//! makes them.

use crate::advancements::DisplayInfo;
use photon_core::ids::ItemId;

/// A toast a packet asked for: what it shows is taken when the packet is handled.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingToast {
    /// `ClientAdvancements.update`: an advancement was done.
    Advancement(Box<DisplayInfo>),
    /// `RecipeToast.addOrUpdate`: the first item of the recipe's crafting station and of its
    /// result; none draws nothing.
    Recipe {
        station: Option<ItemId>,
        result: Option<ItemId>,
    },
}
