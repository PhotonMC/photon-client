//! An entity's name tag (`EntityRenderer.submitNameDisplay`): whether it shows, and the pose,
//! light and colours of what is submitted for it, with the team rules and the score line under
//! the name.

use photon_core::math::{Vec3, mth_cos_f64, mth_sin_f64};
use photon_world::{ClientEntity, ClientEntityPose};

use crate::gui::font::Font;
use crate::gui::frame::glyph_quads;
use crate::gui::{GuiAtlas, TextRun};
use crate::joml::{Matrix4, Pose, Quaternion};
use crate::scene::Vertex;

/// `Attributes.NAME_TAG_DISTANCE`'s default.
const NAME_TAG_DISTANCE: f64 = 64.0;
/// `Attributes.BELOW_NAME_DISTANCE`'s default.
const BELOW_NAME_DISTANCE: f64 = 10.0;

/// What the viewer contributes to the decision.
#[derive(Debug, Clone, Copy)]
pub struct Viewer {
    pub camera_position: Vec3,
    /// The local player watches from the spectator's seat.
    pub spectator: bool,
    /// The HUD is hidden (F1).
    pub hud_hidden: bool,
}

/// A name that shows, with where it hangs.
#[derive(Debug, Clone)]
pub struct NameTag {
    /// The name, one run per style.
    pub runs: Vec<TextRun<'static>>,
    /// `EntityAttachment.NAME_TAG` of the entity, turned by its facing.
    pub attachment: Vec3,
    /// The line's offset in the name tag's own units (`showExtraEars`).
    pub offset: i32,
    /// The name is drawn through blocks too (`!isDiscrete`).
    pub see_through: bool,
    /// `AvatarRenderer.getRenderOffset`: the crouching player is drawn lower.
    pub render_offset: Vec3,
    /// The score line under the name (`Entity.belowNameDisplay`), when one is near enough.
    pub score: Option<Vec<TextRun<'static>>>,
}

impl NameTag {
    /// The tag with the score line `score` if the entity is within the below-name distance of
    /// the viewer.
    pub fn with_score(
        mut self,
        entity: &ClientEntity,
        viewer: Viewer,
        score: Option<Vec<TextRun<'static>>>,
    ) -> Self {
        let reach = entity
            .attribute_value("minecraft:below_name_distance")
            .unwrap_or(BELOW_NAME_DISTANCE);
        if distance_sq(entity, viewer) < reach * reach {
            self.score = score;
        }
        self
    }

    /// The name without its styling (`Component.getString`).
    pub fn text(&self) -> String {
        self.runs.iter().map(|run| run.text.as_ref()).collect()
    }
}

/// `Entity.getYRot(partialTicks)`.
fn y_rot(entity: &ClientEntity, partial: f32) -> f32 {
    let (current, _, _) = entity.rotation();
    if partial == 1.0 {
        return current;
    }
    let (previous, _, _) = entity.previous_rotation();
    previous + partial * (current - previous)
}

/// `Entity.getAttachments().getNullable(NAME_TAG, 0, yRot)` of an avatar: the
/// point above its head the pose's dimensions give, turned by its facing.
fn attachment(pose: ClientEntityPose, y_rot: f32) -> Vec3 {
    let height = f64::from(pose.player_dimensions().height());
    // `Vec3.yRot(-rotY * (float) (Math.PI / 180.0))`.
    let angle = -y_rot * (std::f64::consts::PI / 180.0) as f32;
    let cos = f64::from(mth_cos_f64(f64::from(angle)));
    let sin = f64::from(mth_sin_f64(f64::from(angle)));
    Vec3::new(0.0 * cos + 0.0 * sin, height, 0.0 * cos - 0.0 * sin)
}

/// `Team.Visibility`: whom a team's name tags show for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

/// What a team says of the tags of its members.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeamRule {
    pub visibility: TagVisibility,
    /// `canSeeFriendlyInvisibles`.
    pub sees_friendly_invisibles: bool,
}

/// The teams a tag's decision looks at: the entity's, and whether the viewer is on one, and the
/// same one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Teams {
    pub entity: Option<TeamRule>,
    pub viewer_has_team: bool,
    /// Both are on a team and it is the same (`isAlliedTo` of the teams).
    pub same_team: bool,
}

/// `Entity.isInvisibleTo(player)`.
fn invisible_to_viewer(entity: &ClientEntity, viewer: Viewer, teams: Teams) -> bool {
    if viewer.spectator {
        return false;
    }
    let sees_friendly = teams.same_team
        && teams
            .entity
            .is_some_and(|rule| rule.sees_friendly_invisibles);
    !sees_friendly && entity.is_invisible()
}

/// `LivingEntityRenderer.shouldShowName` (a viewer that is not the entity itself): the team's
/// visibility when the entity is on one, else the HUD, the invisibility and the riders.
pub(super) fn living_shows(
    entity: &ClientEntity,
    viewer: Viewer,
    teams: Teams,
    distance_sq: f64,
) -> bool {
    // `isDiscrete`: sneaking.
    if entity.is_shift_key_down() && distance_sq >= 1024.0 {
        return false;
    }
    let visible = !invisible_to_viewer(entity, viewer, teams);
    let Some(rule) = teams.entity else {
        return !viewer.hud_hidden && visible && entity.passenger_ids().is_empty();
    };
    team_allows(rule, teams, visible)
}

/// The switch over a team's name tag visibility.
fn team_allows(rule: TeamRule, teams: Teams, visible: bool) -> bool {
    match (rule.visibility, teams.viewer_has_team) {
        (TagVisibility::Always, _) => visible,
        (TagVisibility::Never, _) => false,
        (TagVisibility::HideForOtherTeams, true) => {
            teams.same_team && (rule.sees_friendly_invisibles || visible)
        }
        (TagVisibility::HideForOwnTeam, true) => !teams.same_team && visible,
        (TagVisibility::HideForOtherTeams | TagVisibility::HideForOwnTeam, false) => visible,
    }
}

/// The squared distance from the viewer's camera.
pub(super) fn distance_sq(entity: &ClientEntity, viewer: Viewer) -> f64 {
    let (x, y, z) = entity.position();
    let (dx, dy, dz) = (
        viewer.camera_position.x - x,
        viewer.camera_position.y - y,
        viewer.camera_position.z - z,
    );
    dx * dx + dy * dy + dz * dz
}

/// `EntityRenderer.extractNameTags` for an avatar, with the rules of
/// `LivingEntityRenderer.shouldShowName`.
pub fn name_tag(
    entity: &ClientEntity,
    name: &str,
    (viewer, teams): (Viewer, Teams),
    partial: f32,
) -> Option<NameTag> {
    let distance_sq = distance_sq(entity, viewer);
    let reach = entity
        .attribute_value("minecraft:name_tag_distance")
        .unwrap_or(NAME_TAG_DISTANCE);
    // `Mth.square(nameTagDistance)`.
    if distance_sq >= reach * reach || !living_shows(entity, viewer, teams, distance_sq) {
        return None;
    }
    let discrete = entity.is_shift_key_down();
    let pose = entity.pose();
    let crouching = pose == ClientEntityPose::Crouching;
    Some(NameTag {
        runs: vec![TextRun::plain(name).into_owned()],
        attachment: attachment(pose, y_rot(entity, partial)),
        offset: if name == "deadmau5" { -10 } else { 0 },
        see_through: !discrete,
        render_offset: if crouching {
            Vec3::new(0.0, f64::from(1.0_f32 * -2.0 / 16.0), 0.0)
        } else {
            Vec3::ZERO
        },
        score: None,
    })
}

/// How a submit is drawn (`Font.DisplayMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    Normal,
    SeeThrough,
}

/// `NameTagFeatureRenderer.Submit`: where the text sits and how it is lit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Submit {
    pub pose: Matrix4,
    /// The text's left edge, in the pose's units.
    pub x: f32,
    pub y: f32,
    pub light: i32,
    pub color: i32,
    pub background: i32,
    pub mode: DisplayMode,
    /// The line is the score under the name, not the name.
    pub score: bool,
}

/// `Camera.setRotation`'s orientation.
pub fn camera_orientation(y_rot: f32, x_rot: f32) -> Quaternion {
    let degrees = (std::f64::consts::PI / 180.0) as f32;
    Quaternion::rotation_yxz(
        std::f32::consts::PI - y_rot * degrees,
        -x_rot * degrees,
        0.0,
    )
}

/// `LightCoordsUtil.lightCoordsWithEmission`.
fn light_with_emission(light: i32, emission: i32) -> i32 {
    let sky = (light >> 20 & 15).max(emission);
    let block = (light >> 4 & 15).max(emission);
    block << 4 | sky << 20
}

/// `SubmitNodeCollection.submitNameTag`: the pose at the entity (its place
/// from the camera, with the render offset), then the tag's attachment turned
/// to face the camera and scaled to the font's units. A tag seen through
/// blocks is submitted twice: lit and plain, then the translucent copy.
pub fn submits(
    tag: &NameTag,
    placement: Vec3,
    widths: (i32, i32),
    (light, background_opacity): (i32, f32),
    orientation: &Quaternion,
) -> Vec<Submit> {
    let mut pose = Pose::default();
    pose.translate(
        (placement.x + tag.render_offset.x) as f32,
        (placement.y + tag.render_offset.y) as f32,
        (placement.z + tag.render_offset.z) as f32,
    );
    let line = |pose: &Pose, width: i32, score: bool| {
        line_submits(
            tag,
            (pose, width, score),
            (light, background_opacity),
            orientation,
        )
    };
    let mut out = Vec::new();
    if tag.score.is_some() {
        out.extend(line(&pose, widths.1, true));
        // `EntityRenderer.submitNameDisplay`: the name sits a line above the score.
        pose.translate(0.0, 9.0_f32 * 1.15_f32 * 0.025_f32, 0.0);
    }
    out.extend(line(&pose, widths.0, false));
    out
}

/// One line of a tag (`SubmitNodeCollection.submitNameTag`) at the pose of the entity.
fn line_submits(
    tag: &NameTag,
    (entity_pose, width, score): (&Pose, i32, bool),
    (light, background_opacity): (i32, f32),
    orientation: &Quaternion,
) -> Vec<Submit> {
    let mut pose = *entity_pose;
    pose.translate(
        tag.attachment.x as f32,
        (tag.attachment.y + 0.5) as f32,
        tag.attachment.z as f32,
    );
    pose.rotate(orientation);
    pose.scale(0.025, -0.025, 0.025);
    let x = -(width as f32) / 2.0;
    let y = tag.offset as f32;
    // `ARGB.color(opacity, 0xFF000000)`: the alpha channel of the opacity.
    let background = ((background_opacity * 255.0).floor() as i32) << 24;
    let translucent = -2_130_706_433;
    let submit = |light, color, background, mode| Submit {
        pose: pose.pose,
        x,
        y,
        light,
        color,
        background,
        mode,
        score,
    };
    if tag.see_through {
        vec![
            submit(light_with_emission(light, 2), -1, 0, DisplayMode::Normal),
            submit(light, translucent, background, DisplayMode::SeeThrough),
        ]
    } else {
        vec![submit(light, translucent, background, DisplayMode::Normal)]
    }
}

/// `ARGB` as a vertex colour.
fn argb(color: i32) -> [f32; 4] {
    let color = color as u32;
    [16, 8, 0, 24].map(|shift| ((color >> shift) & 0xFF) as f32 / 255.0)
}

/// The triangles of one submit's text (`Font.prepareText` with a background):
/// the box behind the line a hundredth back, then each glyph in the pose,
/// shifted by where the pose's origin lies from the world's origin.
pub fn submit_vertices(
    submit: &Submit,
    runs: &[TextRun],
    font: &Font,
    atlas: &GuiAtlas,
    shift: [f32; 3],
) -> Vec<Vertex> {
    let light = crate::block::light::light_uv(submit.light);
    let place = |x: f32, y: f32, z: f32, uv: [f32; 2], tint: [f32; 4]| {
        let [px, py, pz] = submit.pose.transform_position(x, y, z);
        Vertex {
            position: [px + shift[0], py + shift[1], pz + shift[2]],
            uv,
            shade: 1.0,
            tint,
            light,
        }
    };
    let mut vertices = Vec::new();
    let mut push = |corners: [Vertex; 4]| {
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(corners[index]);
        }
    };
    if submit.background >> 24 & 0xFF != 0 {
        // `markBackground`: from a unit before the line's start to its end,
        // a unit above the line to the nine its cells take.
        let end = submit.x + font.width_of_text_runs(runs) as f32;
        let (left, top, right, bottom) = (submit.x - 1.0, submit.y - 1.0, end, submit.y + 9.0);
        let tint = argb(submit.background);
        // No texel: a negative coordinate tells the shader.
        let none = [-1.0, -1.0];
        push([
            place(left, top, -0.01, none, tint),
            place(left, bottom, -0.01, none, tint),
            place(right, bottom, -0.01, none, tint),
            place(right, top, -0.01, none, tint),
        ]);
    }
    for corners in glyph_quads(
        atlas,
        font,
        runs,
        (submit.x, submit.y),
        submit.color as u32,
        false,
    ) {
        push(corners.map(|corner| {
            place(
                corner.position[0],
                corner.position[1],
                0.0,
                corner.uv,
                corner.colour,
            )
        }));
    }
    vertices
}
