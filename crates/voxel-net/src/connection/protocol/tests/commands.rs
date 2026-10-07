#[test]
fn a_commands_packet_links_into_a_tree() {
    use voxel_game::commands::NodeKind;
    // Node 0 is the root with children 1; node 1 literal "a" (executable)
    // with child 2; node 2 an integer argument "n" from 1 to 5.
    let mut packet = vec![3];
    packet.extend([0, 1, 1]);
    packet.extend([1 | 4, 1, 2, 1, b'a']);
    packet.extend([2 | 4, 0, 1, b'n', 3, 0b11]);
    packet.extend(1i32.to_be_bytes());
    packet.extend(5i32.to_be_bytes());
    packet.push(0);
    let tree = super::super::parse_commands_packet(&packet).unwrap();
    let root = tree.node(tree.root());
    assert_eq!(root.children.len(), 1);
    assert_eq!(
        tree.node(root.children[0]).kind,
        NodeKind::Literal("a".to_owned())
    );
    let parse = tree.parse("a 3", 0, true);
    assert!(!parse.can_read() && parse.exceptions.is_empty());
    let parse = tree.parse("a 9", 0, true);
    assert_eq!(parse.exceptions.len(), 1);
    // A child past the end is refused.
    assert!(super::super::parse_commands_packet(&[1, 0, 1, 7, 0]).is_err());
}
