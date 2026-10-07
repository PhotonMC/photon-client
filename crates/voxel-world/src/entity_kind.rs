//! The state only some kinds of entity have. An entity is one kind for as long as it lives, so
//! what a rabbit keeps and what a warden keeps are variants of one value, not a field each that
//! is empty on every other entity. The state is made when something first reaches it (its data,
//! an event, its first tick), so an entity of a kind can still have none yet.

use std::borrow::{Borrow, BorrowMut};

/// A minecart's own state, and the steps the new behaviour (`NewMinecartBehavior`) plays along the
/// track; either can exist without the other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MinecartSlot {
    pub(crate) state: Option<crate::minecart::MinecartState>,
    pub(crate) track: Option<crate::minecart_track::TrackLerp>,
}

macro_rules! entity_kinds {
    ($($variant:ident($boxed:ty => $inner:ty) $get:ident $get_mut:ident $or_insert:ident;)*) => {
        /// What kind of entity this is, with the state that kind keeps.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub(crate) enum EntityKind {
            /// An entity with no state of its own, or one whose state is not made yet.
            #[default]
            Plain,
            $($variant($boxed),)*
        }

        // Each kind gets the same three accessors; not every kind needs all three.
        #[allow(dead_code)]
        impl EntityKind {
            $(
                pub(crate) fn $get(&self) -> Option<&$inner> {
                    match self {
                        Self::$variant(state) => Some(<$boxed as Borrow<$inner>>::borrow(state)),
                        _ => None,
                    }
                }

                pub(crate) fn $get_mut(&mut self) -> Option<&mut $inner> {
                    match self {
                        Self::$variant(state) => {
                            Some(<$boxed as BorrowMut<$inner>>::borrow_mut(state))
                        }
                        _ => None,
                    }
                }

                /// The state, made with `make` if there is none yet. An entity keeps one kind: it
                /// is a bug to ask for the state of another.
                pub(crate) fn $or_insert(&mut self, make: impl FnOnce() -> $boxed) -> &mut $inner {
                    if matches!(self, Self::Plain) {
                        *self = Self::$variant(make());
                    }
                    match self {
                        Self::$variant(state) => <$boxed as BorrowMut<$inner>>::borrow_mut(state),
                        other => panic!("the state of {other:?} asked for as {}", stringify!($variant)),
                    }
                }
            )*
        }
    };
}

entity_kinds! {
    Rabbit(Box<crate::rabbit::RabbitState> => crate::rabbit::RabbitState) rabbit rabbit_mut rabbit_or_insert_with;
    CopperGolem(Box<crate::copper_golem::CopperGolemAnimations> => crate::copper_golem::CopperGolemAnimations) copper_golem copper_golem_mut copper_golem_or_insert_with;
    Squid(Box<crate::squid::SquidState> => crate::squid::SquidState) squid squid_mut squid_or_insert_with;
    Axolotl(Box<crate::axolotl::AxolotlState> => crate::axolotl::AxolotlState) axolotl axolotl_mut axolotl_or_insert_with;
    Boat(Box<crate::boat::BoatState> => crate::boat::BoatState) boat boat_mut boat_or_insert_with;
    Bat(Box<crate::bat::BatState> => crate::bat::BatState) bat bat_mut bat_or_insert_with;
    Parrot(Box<crate::parrot::ParrotState> => crate::parrot::ParrotState) parrot parrot_mut parrot_or_insert_with;
    Allay(Box<crate::allay::AllayState> => crate::allay::AllayState) allay allay_mut allay_or_insert_with;
    Guardian(Box<crate::guardian::GuardianState> => crate::guardian::GuardianState) guardian guardian_mut guardian_or_insert_with;
    Breeze(Box<crate::breeze::BreezeState> => crate::breeze::BreezeState) breeze breeze_mut breeze_or_insert_with;
    Warden(Box<crate::warden::WardenState> => crate::warden::WardenState) warden warden_mut warden_or_insert_with;
    Creaking(Box<crate::creaking::CreakingState> => crate::creaking::CreakingState) creaking creaking_mut creaking_or_insert_with;
    Armadillo(Box<crate::armadillo::ArmadilloState> => crate::armadillo::ArmadilloState) armadillo armadillo_mut armadillo_or_insert_with;
    Sniffer(Box<crate::sniffer::SnifferState> => crate::sniffer::SnifferState) sniffer sniffer_mut sniffer_or_insert_with;
    Wither(Box<crate::wither::WitherState> => crate::wither::WitherState) wither wither_mut wither_or_insert_with;
    EnderDragon(Box<crate::ender_dragon::EnderDragonState> => crate::ender_dragon::EnderDragonState) ender_dragon ender_dragon_mut ender_dragon_or_insert_with;
    EndCrystal(Box<crate::end_crystal::EndCrystalState> => crate::end_crystal::EndCrystalState) end_crystal end_crystal_mut end_crystal_or_insert_with;
    Arrow(Box<crate::arrow::ArrowState> => crate::arrow::ArrowState) arrow arrow_mut arrow_or_insert_with;
    EvokerFangs(Box<crate::evoker_fangs::FangsState> => crate::evoker_fangs::FangsState) evoker_fangs evoker_fangs_mut evoker_fangs_or_insert_with;
    Display(Box<crate::display::DisplayState> => crate::display::DisplayState) display display_mut display_or_insert_with;
    PrimedTnt(Box<crate::primed_tnt::PrimedTntState> => crate::primed_tnt::PrimedTntState) primed_tnt primed_tnt_mut primed_tnt_or_insert_with;
    Lightning(Box<crate::lightning::LightningState> => crate::lightning::LightningState) lightning lightning_mut lightning_or_insert_with;
    Frog(Box<crate::frog::FrogState> => crate::frog::FrogState) frog frog_mut frog_or_insert_with;
    Panda(Box<crate::panda::PandaState> => crate::panda::PandaState) panda panda_mut panda_or_insert_with;
    Ravager(Box<crate::ravager::RavagerState> => crate::ravager::RavagerState) ravager ravager_mut ravager_or_insert_with;
    Horse(Box<crate::horse::HorseState> => crate::horse::HorseState) horse horse_mut horse_or_insert_with;
    Minecart(Box<MinecartSlot> => MinecartSlot) minecart_slot minecart_slot_mut minecart_slot_or_insert_with;
}
