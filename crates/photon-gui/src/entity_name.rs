//! The name an entity shows over its head (`Entity.getDisplayName` without a team): its custom
//! name, or the name of its type.

use photon_game::text::{Component, Lang, Style};
use photon_render::gui::TextRun;
use photon_world::ClientEntity;

/// `Entity.getName`: the custom name the server sent, else the type's name (`getTypeName`;
/// the types that name themselves otherwise, like a villager by its profession, are not told
/// apart). Styled the way the component says, in words of `lang`.
pub fn entity_name_runs(entity: &ClientEntity, lang: &Lang) -> Vec<TextRun<'static>> {
    let component = entity
        .custom_name()
        .and_then(|tag| Component::from_nbt(&tag))
        .or_else(|| {
            let name = photon_data::entity_type::name(entity.type_id)?;
            // `Util.makeDescriptionId("entity", id)`.
            let (namespace, path) = name.split_once(':')?;
            Some(Component::translatable(format!(
                "entity.{namespace}.{}",
                path.replace('/', ".")
            )))
        });
    let Some(component) = component else {
        return Vec::new();
    };
    let runs = lang.runs(&component, &Style::default());
    crate::draw_runs(&runs)
        .into_iter()
        .map(TextRun::into_owned)
        .collect()
}
