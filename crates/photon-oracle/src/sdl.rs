//! The numbers SDL gives keys, buttons and modifiers, for the 26.3 cases. The cases name keys with
//! GLFW's numbers; 26.3 reads SDL's, so the oracle translates at the edge, as the Java side does.

/// Runs of GLFW keys that SDL numbers in the same order: first key, last key, first scancode.
const SCANCODE_RANGES: [(i32, i32, i32); 5] = [
    (65, 90, 4),
    (49, 57, 30),
    (290, 301, 58),
    (302, 313, 104),
    (321, 329, 89),
];

/// The other keys with a scancode: the GLFW key and the scancode.
const SCANCODES: [(i32, i32); 55] = [
    (48, 39),
    (32, 44),
    (39, 52),
    (44, 54),
    (45, 45),
    (46, 55),
    (47, 56),
    (59, 51),
    (61, 46),
    (91, 47),
    (92, 49),
    (93, 48),
    (96, 53),
    (256, 41),
    (257, 40),
    (258, 43),
    (259, 42),
    (260, 73),
    (261, 76),
    (262, 79),
    (263, 80),
    (264, 81),
    (265, 82),
    (266, 75),
    (267, 78),
    (268, 74),
    (269, 77),
    (280, 57),
    (281, 71),
    (282, 83),
    (283, 70),
    (284, 72),
    (320, 98),
    (330, 99),
    (331, 84),
    (332, 85),
    (333, 86),
    (334, 87),
    (335, 88),
    (336, 103),
    (340, 225),
    (341, 224),
    (342, 226),
    (343, 227),
    (344, 229),
    (345, 228),
    (346, 230),
    (347, 231),
    (348, 101),
    (256, 27),
    (257, 13),
    (258, 9),
    (259, 8),
    (261, 127),
    (0, 0),
];

/// The SDL scancode of the key GLFW numbers `key`; 0 for a key SDL has no scancode for.
pub(crate) fn scancode(key: i32) -> i32 {
    SCANCODE_RANGES
        .iter()
        .find(|(first, last, _)| (*first..=*last).contains(&key))
        .map(|(first, _, base)| base + key - first)
        .or_else(|| {
            SCANCODES
                .iter()
                .find(|(glfw, _)| *glfw == key)
                .map(|&(_, code)| code)
        })
        .unwrap_or(0)
}

/// The keys whose SDL keycode is not their scancode with the scancode flag.
const KEYCODES: [(i32, i32); 7] = [
    (256, 27),
    (257, 13),
    (258, 9),
    (259, 8),
    (261, 127),
    (335, 1_073_741_912),
    (348, 1_073_741_925),
];

/// The GLFW keys SDL gives the character as the keycode.
const CHARACTER_KEYS: [i32; 13] = [32, 39, 44, 45, 46, 47, 59, 61, 91, 92, 93, 96, 48];

/// The SDL keycode of the key GLFW numbers `key` (letters are lower case).
pub(crate) fn keycode(key: i32) -> i32 {
    if (65..=90).contains(&key) {
        return key + 32;
    }
    if (48..=57).contains(&key) || CHARACTER_KEYS.contains(&key) {
        return key;
    }
    KEYCODES.iter().find(|(glfw, _)| *glfw == key).map_or_else(
        || match scancode(key) {
            0 => 0,
            code => 0x4000_0000 | code,
        },
        |&(_, code)| code,
    )
}

/// SDL's modifier bits for GLFW's.
pub(crate) fn modifiers(glfw: i32) -> i32 {
    [(1, 1), (2, 64), (4, 256), (8, 1024)]
        .iter()
        .filter(|(bit, _)| glfw & bit != 0)
        .fold(0, |sdl, (_, mapped)| sdl | mapped)
}

/// The SDL mouse button for the GLFW button.
pub(crate) fn button(glfw: i32) -> i32 {
    match glfw {
        0 => 1,
        1 => 3,
        2 => 2,
        other => other.wrapping_add(1),
    }
}

/// The key event a case names, as the jar's version numbers it: GLFW's for 26.2, SDL's after.
pub(crate) fn event_numbers((key, scancode): (i32, i32)) -> (i32, i32) {
    if crate::jar_version() == "26.2" {
        (key, scancode)
    } else {
        (self::scancode(key), keycode(key))
    }
}

/// The mouse button a case names, as the jar's version numbers it.
pub(crate) fn mouse_button(glfw: i32) -> i32 {
    if crate::jar_version() == "26.2" {
        glfw
    } else {
        button(glfw)
    }
}
