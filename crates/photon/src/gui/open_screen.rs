use super::*;
use crate::objects::dropped_items::WorldItemLayers;
use photon_core::ids::EntityId;
use photon_render::entity::textures::EntityTextures;

/// What the figures and the maps of an open screen are drawn from.
pub struct ScreenFigures<'a> {
    pub player_model: &'a mut PlayerModel,
    pub dressing: super::draw::Dressing<'a>,
    pub entity_textures: &'a EntityTextures,
    pub layers: &'a mut WorldItemLayers,
}

impl Gui {
    /// The container screen open over the game, built over the frame, with the player and the
    /// mount in it as they stand now and the maps the client holds.
    pub(super) fn draw_open_screen(
        (atlas, font, lang, registries): (&GuiAtlas, &Arc<Font>, &Arc<Lang>, &Registries),
        (frame, draw_item): (
            &mut photon_render::gui::GuiFrame,
            &mut photon_render::gui::ItemDrawer<'_>,
        ),
        (backend, controls, session): (&mut VulkanBackend, &mut Controls, &PlaySession),
        (menu, skin, partial): (
            Option<photon_game::menus::container::ContainerMenu>,
            photon_render::entity::textures::TextureRect,
            f32,
        ),
        (figures, (renderer, tints)): (ScreenFigures<'_>, (&BlockRenderer, &BiomeTints)),
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (Some(menu), Some(screen)) = (menu, controls.screen.as_mut()) else {
            return Ok(());
        };
        let (mob_models, armor, held_layers) = figures.dressing;
        let (entity_textures, layers) = (figures.entity_textures, figures.layers);
        // Both previews dress what they draw from the same cache, one after the other.
        let armor = std::cell::RefCell::new(armor);
        let mut draw_player = preview::drawer(
            session,
            (session.player(), figures.player_model),
            (&armor, entity_textures),
            (skin, held_layers, (renderer, tints)),
        );
        // A mount in the box of its screen, between two ticks as it stands now.
        let mut draw_entity = |id: EntityId, area, size, offset_y, mouse| {
            preview::entity_vertices(
                (session, id),
                (&mut *mob_models, &armor),
                entity_textures,
                (area, size, offset_y, mouse),
            )
        };
        let advanced = controls.options.advanced_item_tooltips;
        let mut tooltip_lines = session_tooltips((lang, registries), advanced, session);
        // The map drawer holds the layers until the screen is built.
        {
            let mut draw_map = session_maps(session, &mut *layers, entity_textures);
            let mut draw = ScreenDraw {
                frame,
                atlas,
                font,
                item: draw_item,
                player: &mut draw_player,
                entity: &mut draw_entity,
                tooltip: &mut tooltip_lines,
                maps: &mut draw_map,
                lang,
                control_held: controls.mods.control,
                partial,
                widget_tooltips: true,
                viewer: crate::hud_input::screen_viewer(session),
                book: session.world().recipe_book(),
                tags: session.world().block_state_registry().item_tags(),
            };
            screen.build(&mut draw, &menu, controls.gui_mouse);
        }
        layers.hand_over_maps(backend)?;
        Ok(())
    }
}

/// A stack's tooltip in words, in the context of the level's maps, tags and tick rate.
fn session_tooltips<'a>(
    (lang, registries): (&'a Lang, &'a Registries),
    advanced: bool,
    session: &'a PlaySession,
) -> impl FnMut(&ClientItemStackState, bool) -> Vec<Vec<photon_game::text::Run>> + 'a {
    move |stack, creative| {
        let world = session.world();
        let maps = |id: i32| super::draw::map_info(world, id);
        let block_tags = |tag: &str| world.block_tag_members(tag);
        let context = photon_game::tooltip::Context {
            maps: &maps,
            block_tag_members: &block_tags,
            tick_rate: session.tick_rate(),
        };
        let flags = photon_game::tooltip::Flags { creative, advanced };
        photon_game::tooltip::lines(stack, true, flags, registries, context)
            .iter()
            .map(|line| lang.runs(line, &Style::default()))
            .collect()
    }
}

/// The maps the screen asks for: the client's data of each, put in the maps' image as it is asked
/// for.
fn session_maps<'a>(
    session: &'a PlaySession,
    layers: &'a mut WorldItemLayers,
    textures: &'a EntityTextures,
) -> impl FnMut(i32) -> Option<photon_render::gui::GuiMap> + 'a {
    move |id| {
        let data = session.world().map_data(id)?;
        Some(layers.gui_map(id, data, textures))
    }
}
