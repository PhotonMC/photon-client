use super::*;

#[test]
fn the_first_tick_moves_the_cloak_to_the_avatar() {
    // It starts at the origin, ten blocks or more away.
    let mut cloak = Cloak::default();
    cloak.tick((100.0, 64.0, -50.0));
    assert_eq!(cloak.interpolated(0.0), [100.0, 64.0, -50.0]);
    assert_eq!(cloak.interpolated(1.0), [100.0, 64.0, -50.0]);
}

#[test]
fn the_cloak_follows_a_quarter_of_the_way_each_tick() {
    let mut cloak = Cloak::default();
    cloak.tick((100.0, 64.0, -50.0));
    cloak.tick((101.0, 64.0, -50.0));
    assert_eq!(cloak.interpolated(0.0), [100.0, 64.0, -50.0]);
    assert_eq!(cloak.interpolated(1.0), [100.25, 64.0, -50.0]);
    assert_eq!(cloak.interpolated(0.5), [100.125, 64.0, -50.0]);
}

#[test]
fn a_jump_of_more_than_ten_blocks_is_followed_at_once_on_that_axis_only() {
    let mut cloak = Cloak::default();
    cloak.tick((0.0, 0.0, 0.0));
    cloak.tick((20.0, 4.0, 0.0));
    assert_eq!(cloak.interpolated(0.0), [20.0, 0.0, 0.0]);
    assert_eq!(cloak.interpolated(1.0), [20.0, 1.0, 0.0]);
}

#[test]
fn the_bob_eases_four_tenths_of_the_way_to_its_target() {
    let mut cloak = Cloak::default();
    cloak.update_bob(0.1);
    assert!((cloak.interpolated_bob(1.0) - 0.04).abs() < 1e-7);
    cloak.update_bob(0.1);
    assert!((cloak.interpolated_bob(1.0) - 0.064).abs() < 1e-7);
    assert!((cloak.interpolated_bob(0.0) - 0.04).abs() < 1e-7);
    cloak.reset_bob();
    assert_eq!(cloak.interpolated_bob(1.0), 0.0);
}
