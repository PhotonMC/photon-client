use super::super::read_level_particles_packet;

fn payload(type_id: i32, extra: &[u8]) -> Vec<u8> {
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
    let packet = read_level_particles_packet(&mut input).unwrap().unwrap();
    assert!(input.is_empty());
    assert_eq!(packet.type_id, smoke);
    assert!(packet.override_limiter && !packet.always_show);
    assert_eq!(packet.at, (1.5, -64.25, 1.0e9));
    assert_eq!(packet.spread, (0.5, 0.25, -0.125));
    assert_eq!((packet.max_speed, packet.count), (2.0, 7));
}

#[test]
fn a_block_particle_carries_the_state_it_is_made_of() {
    let block = photon_data::particle_type::id("block").unwrap();
    let mut state = Vec::new();
    photon_core::wire::write_varint(1234, &mut state);
    let bytes = payload(block, &state);
    let mut input = bytes.as_slice();
    let packet = read_level_particles_packet(&mut input).unwrap().unwrap();
    assert!(input.is_empty());
    assert_eq!(
        (packet.type_id, packet.data.block_state()),
        (block, Some(1234))
    );
}

#[test]
fn a_type_with_options_is_skipped_with_what_follows_its_id() {
    let vibration = photon_data::particle_type::id("vibration").unwrap();
    let bytes = payload(vibration, &[1, 2, 3, 4, 5]);
    let mut input = bytes.as_slice();
    assert_eq!(read_level_particles_packet(&mut input).unwrap(), None);
    assert!(input.is_empty());
}

#[test]
fn an_unknown_type_or_a_short_packet_is_an_error() {
    let bytes = payload(9999, &[]);
    assert!(read_level_particles_packet(&mut bytes.as_slice()).is_err());
    let bytes = payload(0, &[]);
    assert!(read_level_particles_packet(&mut &bytes[..20]).is_err());
}
