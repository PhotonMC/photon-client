use super::super::*;

#[test]
fn attribute_effective_value_matches_java_operation_order_and_sanitization() {
    let attribute = ClientAttributeState::from_wire(
        26,
        0.25,
        vec![
            ClientAttributeModifierState::from_wire("minecraft:oracle_add".to_owned(), 0.1, 0),
            ClientAttributeModifierState::from_wire("minecraft:oracle_base".to_owned(), 0.2, 1),
            ClientAttributeModifierState::from_wire("minecraft:oracle_total".to_owned(), 0.5, 2),
        ],
    );
    assert_eq!(
        attribute.effective_value(0.0, 1024.0).to_bits(),
        0x3fe4_28f5_c28f_5c29
    );

    let unknown_operation = ClientAttributeState::from_wire(
        26,
        1.0,
        vec![ClientAttributeModifierState::from_wire(
            "minecraft:oracle_unknown".to_owned(),
            0.5,
            255,
        )],
    );
    assert_eq!(unknown_operation.effective_value(0.0, 1024.0), 1.5);

    let clamped = ClientAttributeState::from_wire(26, 2048.0, Vec::new());
    assert_eq!(clamped.effective_value(0.0, 1024.0), 1024.0);

    let nan = ClientAttributeState::from_wire(26, f64::NAN, Vec::new());
    assert_eq!(nan.effective_value(0.0, 1024.0), 0.0);
}
