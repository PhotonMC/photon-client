//! The state only some kinds of entity have. An entity is one kind for as long as it lives, so
//! what a rabbit keeps and what a warden keeps are variants of one value, not a field each that
//! is empty on every other entity. The state is made as the entity spawns, from its type.

use std::borrow::{Borrow, BorrowMut};

/// A minecart's own state, and the steps the new behaviour (`NewMinecartBehavior`) plays along the
/// track; either can exist without the other.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MinecartSlot {
    pub(crate) state: Option<crate::objects::minecart::MinecartState>,
    pub(crate) track: Option<crate::objects::minecart_track::TrackLerp>,
}

macro_rules! entity_kinds {
    ($($variant:ident($boxed:ty => $inner:ty) $get:ident $get_mut:ident $state:ident;)*) => {
        /// What kind of entity this is, with the state that kind keeps.
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub(crate) enum EntityKind {
            /// An entity with no state of its own.
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

                /// The state, which the entity makes as it spawns. An entity keeps one kind: it
                /// is a bug to ask for the state of another.
                pub(crate) fn $state(&mut self) -> &mut $inner {
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
    Rabbit(Box<crate::mobs::rabbit::RabbitState> => crate::mobs::rabbit::RabbitState) rabbit rabbit_mut rabbit_state;
    CopperGolem(Box<crate::mobs::copper_golem::CopperGolemAnimations> => crate::mobs::copper_golem::CopperGolemAnimations) copper_golem copper_golem_mut copper_golem_state;
    Squid(Box<crate::mobs::squid::SquidState> => crate::mobs::squid::SquidState) squid squid_mut squid_state;
    Axolotl(Box<crate::mobs::axolotl::AxolotlState> => crate::mobs::axolotl::AxolotlState) axolotl axolotl_mut axolotl_state;
    Boat(Box<crate::objects::boat::BoatState> => crate::objects::boat::BoatState) boat boat_mut boat_state;
    Bat(Box<crate::mobs::bat::BatState> => crate::mobs::bat::BatState) bat bat_mut bat_state;
    Parrot(Box<crate::mobs::parrot::ParrotState> => crate::mobs::parrot::ParrotState) parrot parrot_mut parrot_state;
    Allay(Box<crate::mobs::allay::AllayState> => crate::mobs::allay::AllayState) allay allay_mut allay_state;
    Guardian(Box<crate::mobs::guardian::GuardianState> => crate::mobs::guardian::GuardianState) guardian guardian_mut guardian_state;
    Breeze(Box<crate::mobs::breeze::BreezeState> => crate::mobs::breeze::BreezeState) breeze breeze_mut breeze_state;
    Warden(Box<crate::mobs::warden::WardenState> => crate::mobs::warden::WardenState) warden warden_mut warden_state;
    Creaking(Box<crate::mobs::creaking::CreakingState> => crate::mobs::creaking::CreakingState) creaking creaking_mut creaking_state;
    Armadillo(Box<crate::mobs::armadillo::ArmadilloState> => crate::mobs::armadillo::ArmadilloState) armadillo armadillo_mut armadillo_state;
    Sniffer(Box<crate::mobs::sniffer::SnifferState> => crate::mobs::sniffer::SnifferState) sniffer sniffer_mut sniffer_state;
    Wither(Box<crate::mobs::wither::WitherState> => crate::mobs::wither::WitherState) wither wither_mut wither_state;
    EnderDragon(Box<crate::mobs::ender_dragon::EnderDragonState> => crate::mobs::ender_dragon::EnderDragonState) ender_dragon ender_dragon_mut ender_dragon_state;
    EndCrystal(Box<crate::objects::end_crystal::EndCrystalState> => crate::objects::end_crystal::EndCrystalState) end_crystal end_crystal_mut end_crystal_state;
    Arrow(Box<crate::objects::arrow::ArrowState> => crate::objects::arrow::ArrowState) arrow arrow_mut arrow_state;
    EvokerFangs(Box<crate::objects::evoker_fangs::FangsState> => crate::objects::evoker_fangs::FangsState) evoker_fangs evoker_fangs_mut evoker_fangs_state;
    Display(Box<crate::objects::display::DisplayState> => crate::objects::display::DisplayState) display display_mut display_state;
    PrimedTnt(Box<crate::objects::primed_tnt::PrimedTntState> => crate::objects::primed_tnt::PrimedTntState) primed_tnt primed_tnt_mut primed_tnt_state;
    Lightning(Box<crate::objects::lightning::LightningState> => crate::objects::lightning::LightningState) lightning lightning_mut lightning_state;
    FishingHook(Box<crate::objects::fishing_hook::HookState> => crate::objects::fishing_hook::HookState) fishing_hook fishing_hook_mut fishing_hook_state;
    Mannequin(Box<crate::objects::mannequin::MannequinState> => crate::objects::mannequin::MannequinState) mannequin mannequin_mut mannequin_state;
    Frog(Box<crate::mobs::frog::FrogState> => crate::mobs::frog::FrogState) frog frog_mut frog_state;
    Panda(Box<crate::mobs::panda::PandaState> => crate::mobs::panda::PandaState) panda panda_mut panda_state;
    Ravager(Box<crate::mobs::ravager::RavagerState> => crate::mobs::ravager::RavagerState) ravager ravager_mut ravager_state;
    Horse(Box<crate::mobs::horse::HorseState> => crate::mobs::horse::HorseState) horse horse_mut horse_state;
    Minecart(Box<MinecartSlot> => MinecartSlot) minecart_slot minecart_slot_mut minecart_slot_state;
}
