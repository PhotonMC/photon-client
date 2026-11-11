use super::super::read_level_particles_packet;
use super::registries;

/// A 26.3 packet: the particle, the flags and place, the spread, a speed for each axis, the
/// count and how the spread is drawn.
fn payload(type_id: i32, extra: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    photon_core::wire::write_varint(type_id, &mut bytes);
    bytes.extend_from_slice(extra);
    bytes.extend_from_slice(&[1, 0]);
    for value in [1.5_f64, -64.25, 1.0e9] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    for value in [0.5_f32, 0.25, -0.125, 2.0, 3.0, -4.0] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    photon_core::wire::write_varint(7, &mut bytes);
    photon_core::wire::write_varint(2, &mut bytes);
    bytes
}

/// A 26.2 packet: the flags and place, the spread, one speed, a four-byte count, then the
/// particle.
fn older_payload(type_id: i32, extra: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, 0];
    for value in [1.5_f64, -64.25, 1.0e9] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    for value in [0.5_f32, 0.25, -0.125, 2.0] {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes.extend_from_slice(&7_i32.to_be_bytes());
    photon_core::wire::write_varint(type_id, &mut bytes);
    bytes.extend_from_slice(extra);
    bytes
}

#[test]
fn a_level_particles_packet_of_a_plain_type_is_read_whole() {
    let smoke = photon_data::particle_type::id("smoke").unwrap();
    let bytes = payload(smoke, &[]);
    let mut input = bytes.as_slice();
    let packet = read_level_particles_packet(&mut input, &registries())
        .unwrap()
        .unwrap();
    assert!(input.is_empty());
    assert_eq!(packet.type_id, smoke);
    assert!(packet.override_limiter && !packet.always_show);
    assert_eq!(packet.at, (1.5, -64.25, 1.0e9));
    assert_eq!(packet.spread, (0.5, 0.25, -0.125));
    assert_eq!(packet.max_speed, (2.0, 3.0, -4.0));
    assert_eq!(packet.count, 7);
    assert_eq!(
        packet.randomization,
        photon_data::particle_values::Randomization::AlternativeWithSpeed
    );
}

#[test]
fn a_26_2_packet_has_one_speed_for_every_axis_and_the_default_spread() {
    let smoke = photon_data::particle_type::id("smoke").unwrap();
    let older = photon_data::version_registries::VersionRegistries::of(
        photon_core::version::Version::V26_2,
    );
    let bytes = older_payload(smoke, &[]);
    let mut input = bytes.as_slice();
    let packet = read_level_particles_packet(&mut input, &older)
        .unwrap()
        .unwrap();
    assert!(input.is_empty());
    assert_eq!(packet.max_speed, (2.0, 2.0, 2.0));
    assert_eq!(packet.count, 7);
    assert_eq!(
        packet.randomization,
        photon_data::particle_values::Randomization::Default
    );
}

#[test]
fn a_randomization_id_out_of_range_reads_as_the_default() {
    let smoke = photon_data::particle_type::id("smoke").unwrap();
    let mut bytes = payload(smoke, &[]);
    *bytes.last_mut().unwrap() = 9;
    let packet = read_level_particles_packet(&mut bytes.as_slice(), &registries())
        .unwrap()
        .unwrap();
    assert_eq!(
        packet.randomization,
        photon_data::particle_values::Randomization::Default
    );
}

#[test]
fn a_block_particle_carries_the_state_it_is_made_of() {
    let block = photon_data::particle_type::id("block").unwrap();
    let mut state = Vec::new();
    photon_core::wire::write_varint(1234, &mut state);
    let bytes = payload(block, &state);
    let mut input = bytes.as_slice();
    let packet = read_level_particles_packet(&mut input, &registries())
        .unwrap()
        .unwrap();
    assert!(input.is_empty());
    assert_eq!(
        (packet.type_id, packet.data.block_state()),
        (block, Some(photon_core::ids::BlockStateId(1234)))
    );
}

#[test]
fn the_dragons_breath_reads_its_power() {
    let dragon_breath = photon_data::particle_type::id("dragon_breath").unwrap();
    let bytes = payload(dragon_breath, &1.5_f32.to_be_bytes());
    let mut input = bytes.as_slice();
    let packet = read_level_particles_packet(&mut input, &registries())
        .unwrap()
        .unwrap();
    assert!(input.is_empty());
    assert_eq!(
        packet.data,
        photon_data::particle_values::ParticleData::Power(1.5)
    );
}

#[test]
fn an_unknown_type_or_a_short_packet_is_an_error() {
    let bytes = payload(9999, &[]);
    assert!(read_level_particles_packet(&mut bytes.as_slice(), &registries()).is_err());
    let bytes = payload(0, &[]);
    assert!(read_level_particles_packet(&mut &bytes[..20], &registries()).is_err());
}
