use super::*;

impl MobModel {
    /// `SpinAttackEffectModel.setupAnim`: the two boxes turn about the vertical, the second a
    /// little faster, as the age in ticks counts up.
    pub fn setup_spin_attack(&mut self, age_in_ticks: f32) {
        self.root.reset();
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        for (index, name) in ["box0", "box1"].into_iter().enumerate() {
            let angle = age_in_ticks * -(45 + (index as i32 + 1) * 5) as f32;
            if let Some(part) = self.root.child_mut(name) {
                part.y_rot = photon_core::math::mth_wrap_degrees_f32(angle) * degrees;
            }
        }
    }
}

#[cfg(test)]
mod spin_attack_tests {
    use super::*;

    #[test]
    fn the_spin_boxes_turn_at_fifty_and_fifty_five_degrees_a_tick() {
        let mut model = MobModel::bake(
            photon_data::entity_model::layer(concat!("minecraft:", "spin_attack#main"))
                .expect("the spin layer"),
        );
        // `SpinAttackEffectModel.setupAnim` at an age of two ticks: -100 and -110 degrees.
        model.setup_spin_attack(2.0);
        let turn = |name: &str| {
            model
                .root
                .children()
                .iter()
                .find(|part| part.name() == name)
                .map(|part| part.y_rot)
        };
        let degrees = (std::f64::consts::PI / 180.0) as f32;
        assert_eq!(turn("box0"), Some(-100.0 * degrees));
        assert_eq!(turn("box1"), Some(-110.0 * degrees));
        // At an age of five ticks the second box has gone round: -275 wraps to 85.
        model.setup_spin_attack(5.0);
        let turn = |name: &str| {
            model
                .root
                .children()
                .iter()
                .find(|part| part.name() == name)
                .map(|part| part.y_rot)
        };
        assert_eq!(turn("box1"), Some(85.0 * degrees));
    }
}
