//! The name tag of an item frame, which says the custom name of the item in it.

use photon_render::entity::tag_rules::NameRules;

use photon_net::connection::PlaySession;

/// What an entity's tag is made from, or none where it has no name to show.
pub(super) fn inputs_of(
    entity: &photon_world::ClientEntity,
    rules: NameRules,
    (lang, session): (&photon_game::text::Lang, &PlaySession),
) -> Option<photon_gui::entity_name::TagInputs> {
    let framed = rules == NameRules::ItemFrame;
    // Only an entity with a name of its own, or whose name shows, can have a tag; a frame has one
    // when its item has.
    if !framed && !entity.has_custom_name() && !entity.is_custom_name_visible() {
        return None;
    }
    let mut inputs = photon_gui::entity_name::tag_inputs(
        entity,
        None,
        (lang, session.scoreboard()),
        session.local_name(),
    );
    if framed {
        // `ItemFrameRenderer.getNameTag`: the item's name, undressed by any team. The score under
        // it is the base renderer's.
        inputs.runs = frame_item_runs(entity, lang);
        if inputs.runs.is_empty() {
            return None;
        }
    }
    Some(inputs)
}

/// The tag of an item frame: the custom name of the item in it (`shouldShowName` wants one), as
/// `getHoverName` gives it. Nothing for a frame that holds none.
fn frame_item_runs(
    entity: &photon_world::ClientEntity,
    lang: &photon_game::text::Lang,
) -> Vec<photon_render::gui::TextRun<'static>> {
    let Some(stack) = entity.item_stack() else {
        return Vec::new();
    };
    let values =
        photon_game::item::values::Values::new(stack.item_registry_id(), stack.patch_bytes());
    let Some(name) = photon_game::tooltip::custom_name(&values) else {
        return Vec::new();
    };
    let runs = lang.runs(&name, &photon_game::text::Style::default());
    photon_gui::draw_runs(&runs)
        .into_iter()
        .map(photon_render::gui::TextRun::into_owned)
        .collect()
}

/// `ItemFrameRenderer.getRenderOffset`, which the dispatcher moves the tag by as well.
pub(super) fn render_offset(entity: &photon_world::ClientEntity) -> photon_core::math::Vec3 {
    let (step_x, _, step_z) =
        photon_core::direction::Direction::from_3d_data_value(entity.data).step();
    photon_core::math::Vec3::new(
        f64::from(step_x as f32 * 0.3),
        -0.25,
        f64::from(step_z as f32 * 0.3),
    )
}
