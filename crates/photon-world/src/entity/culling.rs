//! What an entity asks of the camera before it is drawn: how far away it may be
//! (`Entity.shouldRenderAtSqrDistance` and the types that change it) and the box its renderer
//! tests against the view (`EntityRenderer.getBoundingBoxForCulling`, `affectedByCulling`).

use super::*;

/// The squared distance within which a projectile is not drawn at its first tick
/// (`MIN_CAMERA_DISTANCE_SQUARED`), so it does not start in the camera's face.
const NEAR_START_SQR: f64 = 12.25;

/// How each type reckons the distance it is drawn to.
enum Reach {
    /// `Entity`: the box's mean side times 64 and the view scale.
    Box,
    /// `RemotePlayer`, `AbstractArrow`: ten times the box's mean side.
    TenBoxes,
    /// `ThrowableProjectile`, `EyeOfEnder`, `Fireball`, `WindCharge`: not at its first tick near
    /// the camera, then four times the box's mean side, without the view scale.
    FourBoxesAfterStart,
    /// `AbstractHurtingProjectile`: four times the box's mean side, without the view scale.
    FourBoxes,
    /// `ArmorStand`: as `FourBoxes`, an empty box counting as one block.
    FourBoxesNotEmpty,
    /// Within a fixed squared distance.
    Within(f64),
    /// `LightningBolt` (one), `ItemFrame` (16): a length times 64 and the view scale.
    Scaled(f64),
    /// `Display`: its view range times 64 and the view scale.
    ViewRange,
    /// `Phantom`, and `ThrownTrident.shouldRender`: anywhere.
    Anywhere,
}

impl ClientEntity {
    fn reach(&self) -> Reach {
        let Some(name) = photon_data::entity_type::name(self.type_id) else {
            return Reach::Box;
        };
        match name {
            "minecraft:player" | "minecraft:arrow" | "minecraft:spectral_arrow" => Reach::TenBoxes,
            "minecraft:snowball"
            | "minecraft:egg"
            | "minecraft:ender_pearl"
            | "minecraft:splash_potion"
            | "minecraft:lingering_potion"
            | "minecraft:experience_bottle"
            | "minecraft:eye_of_ender"
            | "minecraft:fireball"
            | "minecraft:small_fireball"
            | "minecraft:wind_charge" => Reach::FourBoxesAfterStart,
            "minecraft:dragon_fireball"
            | "minecraft:wither_skull"
            | "minecraft:breeze_wind_charge" => Reach::FourBoxes,
            "minecraft:armor_stand" => Reach::FourBoxesNotEmpty,
            "minecraft:leash_knot" => Reach::Within(1024.0),
            "minecraft:shulker_bullet" => Reach::Within(16384.0),
            "minecraft:fishing_bobber" | "minecraft:firework_rocket" => Reach::Within(4096.0),
            "minecraft:lightning_bolt" => Reach::Scaled(1.0),
            "minecraft:item_frame" | "minecraft:glow_item_frame" => Reach::Scaled(16.0),
            "minecraft:block_display" | "minecraft:item_display" | "minecraft:text_display" => {
                Reach::ViewRange
            }
            "minecraft:phantom" | "minecraft:trident" => Reach::Anywhere,
            _ => Reach::Box,
        }
    }

    /// `Entity.shouldRenderAtSqrDistance` with the overrides of its type (`view_scale` is
    /// `Entity.viewScale`). An attached firework is never drawn this way, and an end crystal with a
    /// beam always is.
    pub fn should_render_at_sqr_distance(&self, distance: f64, view_scale: f64) -> bool {
        let size = self.bounding_box().size();
        let within = |size: f64| distance < size * size;
        let near = match self.reach() {
            Reach::Box => within(if size.is_nan() { 1.0 } else { size } * (64.0 * view_scale)),
            Reach::TenBoxes => {
                let size = size * 10.0;
                within(if size.is_nan() { 1.0 } else { size } * (64.0 * view_scale))
            }
            Reach::FourBoxesAfterStart if self.tick_count() < 2 && distance < NEAR_START_SQR => {
                false
            }
            Reach::FourBoxesAfterStart | Reach::FourBoxes => {
                let size = size * 4.0;
                within(if size.is_nan() { 4.0 } else { size } * 64.0)
            }
            Reach::FourBoxesNotEmpty => {
                let size = size * 4.0;
                within(
                    if size.is_nan() || size == 0.0 {
                        4.0
                    } else {
                        size
                    } * 64.0,
                )
            }
            Reach::Within(limit) => distance < limit,
            Reach::Scaled(length) => within(length * (64.0 * view_scale)),
            Reach::ViewRange => self
                .display()
                .is_some_and(|display| display.should_render_at_sqr_distance(distance, view_scale)),
            Reach::Anywhere => true,
        };
        match photon_data::entity_type::name(self.type_id) {
            Some("minecraft:firework_rocket") => near && self.attached_target.is_none(),
            Some("minecraft:end_crystal") => {
                near || self
                    .end_crystal()
                    .is_some_and(|crystal| crystal.beam_target().is_some())
            }
            _ => near,
        }
    }

    /// `Entity.shouldRender`: whether it is near enough to `camera` to be drawn.
    pub fn should_render(&self, camera: (f64, f64, f64), view_scale: f64) -> bool {
        let (x, y, z) = self.position();
        let (dx, dy, dz) = (x - camera.0, y - camera.1, z - camera.2);
        self.should_render_at_sqr_distance(dx * dx + dy * dy + dz * dz, view_scale)
    }

    /// `EntityRenderer.getBoundingBoxForCulling` with its renderer's overrides, or none when the
    /// renderer is not `affectedByCulling` (a bolt, the dragon, a fishing hook, a display with no
    /// width or height).
    pub fn culling_box(&self) -> Option<Aabb> {
        let name = photon_data::entity_type::name(self.type_id).unwrap_or_default();
        let bounds = self.bounding_box();
        Some(match name {
            "minecraft:lightning_bolt" | "minecraft:ender_dragon" | "minecraft:fishing_bobber" => {
                return None;
            }
            "minecraft:block_display" | "minecraft:item_display" | "minecraft:text_display" => {
                // `Display.updateCulling`: the box stands on the position.
                let (width, height) = self.display().map_or((0.0, 0.0), |display| display.size());
                if width == 0.0 || height == 0.0 {
                    return None;
                }
                let half = f64::from(width / 2.0);
                let (x, y, z) = self.position();
                Aabb::new(
                    x - half,
                    y,
                    z - half,
                    x + half,
                    y + f64::from(height),
                    z + half,
                )
            }
            "minecraft:illusioner" => bounds.inflate(3.0, 0.0, 3.0),
            "minecraft:sniffer" => bounds.inflate_all(f64::from(0.6_f32)),
            "minecraft:trident" => bounds.inflate_all(1.5),
            "minecraft:happy_ghast" => {
                let height = f64::from(self.dimensions().height() / 2.0);
                Aabb {
                    min_y: bounds.min_y - height,
                    ..bounds
                }
            }
            _ if self.is_minecart() => {
                match self.minecart_display_block().filter(|&state| state.0 != 0) {
                    Some(_) => bounds.expand_towards(
                        0.0,
                        f64::from(self.minecart_display_offset() as f32 * 0.75 / 16.0),
                        0.0,
                    ),
                    None => bounds,
                }
            }
            _ if self.is_living_entity() && self.wears_dragon_head() => bounds.inflate_all(0.5),
            _ => bounds,
        })
    }

    fn wears_dragon_head(&self) -> bool {
        self.equipment(EquipmentSlot::Head)
            .is_some_and(|stack| stack.item_registry_id() == photon_data::item::ids::DRAGON_HEAD)
    }
}
