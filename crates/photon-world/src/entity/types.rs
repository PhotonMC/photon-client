use super::*;

impl ClientEntity {
    /// A display entity's data and render state.
    pub fn display(&self) -> Option<&crate::objects::display::DisplayState> {
        self.kind.display()
    }
}

/// `Entity.blockPosition`: the block a position is in.
pub(super) fn block_of(position: (f64, f64, f64)) -> [i32; 3] {
    [position.0, position.1, position.2].map(|value| value.floor() as i32)
}

pub(crate) fn is_vanilla_shulker_entity_type(entity_type_id: EntityTypeId) -> bool {
    entity_type_id == VANILLA_SHULKER_ENTITY_TYPE_ID
}

/// Which display a type is, if it is one (`Display`).
pub(crate) fn display_kind_of(
    entity_type_id: EntityTypeId,
) -> Option<crate::objects::display::DisplayKind> {
    use crate::objects::display::DisplayKind;
    match photon_data::entity_type::name(entity_type_id)? {
        "minecraft:block_display" => Some(DisplayKind::Block),
        "minecraft:item_display" => Some(DisplayKind::Item),
        "minecraft:text_display" => Some(DisplayKind::Text),
        _ => None,
    }
}

pub(crate) fn is_vanilla_happy_ghast_entity_type(entity_type_id: EntityTypeId) -> bool {
    entity_type_id == VANILLA_HAPPY_GHAST_ENTITY_TYPE_ID
}

/// The state a kind of entity keeps, made as it spawns (the entity constructor in Java makes it
/// just as early). Types with no state of their own are `Plain`; a minecart's track steps of
/// the new behaviour are added by the caller, which knows which behaviour the world plays.
pub(crate) fn initial_kind(
    entity_type_id: EntityTypeId,
    entity_id: EntityId,
) -> crate::entity_kind::EntityKind {
    use crate::entity_kind::{EntityKind, MinecartSlot};
    if let Some(display_kind) = display_kind_of(entity_type_id) {
        let display = crate::objects::display::DisplayState::new(display_kind);
        return EntityKind::Display(Box::new(display));
    }
    if super::boat::is_vanilla_boat_entity_type(entity_type_id) {
        return EntityKind::Boat(Box::default());
    }
    if super::minecart::is_vanilla_minecart_entity_type(entity_type_id) {
        return EntityKind::Minecart(Box::<MinecartSlot>::default());
    }
    if crate::ride::mount::is_horse_type(entity_type_id) {
        return horse_kind(entity_id);
    }
    let Some(name) = photon_data::entity_type::name(entity_type_id) else {
        return EntityKind::Plain;
    };
    NAMED_KINDS
        .iter()
        .find(|(named, _)| *named == name)
        .map_or(EntityKind::Plain, |(_, make)| make(entity_id))
}

type MakeKind = fn(EntityId) -> crate::entity_kind::EntityKind;

fn horse_kind(_: EntityId) -> crate::entity_kind::EntityKind {
    let horse = crate::mobs::horse::HorseState::new();
    crate::entity_kind::EntityKind::Horse(Box::new(horse))
}

/// The types that keep a state of a kind of their own, by name. A kind several types share (a
/// squid and a glow squid, the llamas with the horses) appears once for each.
use crate::entity_kind::EntityKind;
use crate::{mobs, objects};

const NAMED_KINDS: &[(&str, MakeKind)] = &[
    ("minecraft:llama", horse_kind),
    ("minecraft:trader_llama", horse_kind),
    ("minecraft:rabbit", |_| {
        EntityKind::Rabbit(Box::new(mobs::rabbit::RabbitState::new()))
    }),
    ("minecraft:copper_golem", |_| {
        EntityKind::CopperGolem(Box::new(mobs::copper_golem::CopperGolemAnimations::new()))
    }),
    ("minecraft:squid", squid_kind),
    ("minecraft:glow_squid", squid_kind),
    ("minecraft:axolotl", |_| EntityKind::Axolotl(Box::default())),
    ("minecraft:bat", |_| EntityKind::Bat(Box::default())),
    ("minecraft:parrot", |_| EntityKind::Parrot(Box::default())),
    ("minecraft:allay", |_| EntityKind::Allay(Box::default())),
    ("minecraft:guardian", guardian_kind),
    ("minecraft:elder_guardian", guardian_kind),
    ("minecraft:breeze", |_| EntityKind::Breeze(Box::default())),
    ("minecraft:warden", |_| EntityKind::Warden(Box::default())),
    ("minecraft:creaking", |id| {
        EntityKind::Creaking(Box::new(mobs::creaking::CreakingState::new(i64::from(
            id.0,
        ))))
    }),
    ("minecraft:armadillo", |_| {
        EntityKind::Armadillo(Box::default())
    }),
    ("minecraft:sniffer", |_| EntityKind::Sniffer(Box::default())),
    ("minecraft:wither", |_| EntityKind::Wither(Box::default())),
    ("minecraft:ender_dragon", |id| {
        EntityKind::EnderDragon(Box::new(mobs::ender_dragon::EnderDragonState::new(
            i64::from(id.0),
        )))
    }),
    ("minecraft:end_crystal", |id| {
        EntityKind::EndCrystal(Box::new(objects::end_crystal::EndCrystalState::new(
            i64::from(id.0),
        )))
    }),
    ("minecraft:arrow", arrow_kind),
    ("minecraft:spectral_arrow", arrow_kind),
    ("minecraft:trident", arrow_kind),
    ("minecraft:evoker_fangs", |_| {
        EntityKind::EvokerFangs(Box::default())
    }),
    ("minecraft:tnt", |_| EntityKind::PrimedTnt(Box::default())),
    ("minecraft:lightning_bolt", |id| {
        EntityKind::Lightning(Box::new(objects::lightning::LightningState::new(
            i64::from(id.0),
        )))
    }),
    ("minecraft:fishing_bobber", |id| {
        EntityKind::FishingHook(Box::new(objects::fishing_hook::HookState::new(id)))
    }),
    ("minecraft:mannequin", |_| {
        EntityKind::Mannequin(Box::default())
    }),
    ("minecraft:frog", |_| EntityKind::Frog(Box::default())),
    ("minecraft:panda", |_| EntityKind::Panda(Box::default())),
    ("minecraft:ravager", |_| EntityKind::Ravager(Box::default())),
];

fn squid_kind(_: EntityId) -> crate::entity_kind::EntityKind {
    let squid = crate::mobs::squid::SquidState::new();
    crate::entity_kind::EntityKind::Squid(Box::new(squid))
}

fn guardian_kind(_: EntityId) -> crate::entity_kind::EntityKind {
    let guardian = crate::mobs::guardian::GuardianState::new();
    crate::entity_kind::EntityKind::Guardian(Box::new(guardian))
}

fn arrow_kind(_: EntityId) -> crate::entity_kind::EntityKind {
    crate::entity_kind::EntityKind::Arrow(Box::default())
}

#[cfg(test)]
mod initial_kind_tests {
    use super::*;

    fn variant(kind: &crate::entity_kind::EntityKind) -> String {
        let text = format!("{kind:?}");
        text.split('(').next().unwrap_or_default().to_owned()
    }

    fn kinds_by_type() -> Vec<(&'static str, String)> {
        photon_data::entity_type::names()
            .map(|name| {
                let id = photon_data::entity_type::named(name);
                (name, variant(&initial_kind(id, EntityId(7))))
            })
            .collect()
    }

    #[test]
    fn every_kind_is_the_state_of_some_type() {
        let kinds = kinds_by_type();
        let made: std::collections::BTreeSet<&str> =
            kinds.iter().map(|(_, kind)| kind.as_str()).collect();
        for expected in [
            "Rabbit",
            "CopperGolem",
            "Squid",
            "Axolotl",
            "Boat",
            "Bat",
            "Parrot",
            "Allay",
            "Guardian",
            "Breeze",
            "Warden",
            "Creaking",
            "Armadillo",
            "Sniffer",
            "Wither",
            "EnderDragon",
            "EndCrystal",
            "Arrow",
            "EvokerFangs",
            "Display",
            "PrimedTnt",
            "Lightning",
            "FishingHook",
            "Mannequin",
            "Frog",
            "Panda",
            "Ravager",
            "Horse",
            "Minecart",
        ] {
            assert!(
                made.contains(expected),
                "no entity type makes the {expected} state"
            );
        }
    }

    #[test]
    fn the_shared_kinds_cover_all_their_types() {
        for name in photon_data::entity_type::names() {
            let id = photon_data::entity_type::named(name);
            let made = variant(&initial_kind(id, EntityId(7)));
            assert_eq!(
                super::super::boat::is_vanilla_boat_entity_type(id),
                made == "Boat",
                "{name}"
            );
            assert_eq!(
                super::super::minecart::is_vanilla_minecart_entity_type(id),
                made == "Minecart",
                "{name}"
            );
            let horse = crate::ride::mount::is_horse_type(id)
                || matches!(name, "minecraft:llama" | "minecraft:trader_llama");
            assert_eq!(horse, made == "Horse", "{name}");
        }
    }

    #[test]
    fn what_the_guards_admit_has_its_state() {
        for name in photon_data::entity_type::names() {
            let id = photon_data::entity_type::named(name);
            let made = variant(&initial_kind(id, EntityId(7)));
            // The minecart and boat guards are written over names and over id lists.
            let minecart = crate::objects::minecart::default_display_offset(name).is_some();
            assert_eq!(minecart, made == "Minecart", "{name}");
            let boat = super::boat::vanilla_boat_max_passengers(id).is_some();
            assert_eq!(boat, made == "Boat", "{name}");
            // The horse metadata readers admit exactly these (and the camels, in the table).
            for (guard, kind) in [
                (crate::ride::mount::is_horse_type(id), "Horse"),
                (
                    matches!(name, "minecraft:llama" | "minecraft:trader_llama"),
                    "Horse",
                ),
            ] {
                assert!(!guard || made == kind, "{name}");
            }
        }
    }
}
