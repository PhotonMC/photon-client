//! Model and blockstate files: accepted shapes and the ones the codecs
//! refuse.

const FULL_ELEMENT: &str = concat!(
    r#"{"elements":[{"from":[0,0,0],"to":[16,16,16],"#,
    r##""faces":{"down":{"texture":"#all","cullface":"down"},"##,
    r##""up":{"texture":"#all","cullface":"up"},"##,
    r##""north":{"texture":"#all","cullface":"north"},"##,
    r##""south":{"texture":"#all","cullface":"south"},"##,
    r##""west":{"texture":"#all","cullface":"west"},"##,
    r##""east":{"texture":"#all","cullface":"east"}}}]}"##,
);

const DISPLAY_RIGHT: &str = concat!(
    r#"{"display":{"thirdperson_righthand":{"rotation":[75,45,0],"#,
    r#""translation":[0,2.5,0],"scale":[0.375,0.375,0.375]},"#,
    r#""firstperson_righthand":{"rotation":[0,45,0],"scale":[0.4,0.4,0.4]},"#,
    r#""gui":{"rotation":[30,225,0],"scale":[0.625,0.625,0.625]},"#,
    r#""ground":{"translation":[0,3,0],"scale":[0.25,0.25,0.25]}}}"#,
);

const DISPLAY_CLAMPED: &str = concat!(
    r#"{"display":{"fixed":{"translation":[100,-100,8],"scale":[9,-9,0.5]},"#,
    r#""head":{"rotation":[0,180,0],"translation":[0,13,7]},"#,
    r#""on_shelf":{"translation":[0,0,-1]},"#,
    r#""firstperson_lefthand":{"rotation":[0,-90,25]},"#,
    r#""thirdperson_lefthand":{"scale":[1,1,1]}}}"#,
);

/// One full-size element with `element` fields and an `up` face of `face`.
fn element(element: &str, face: &str) -> String {
    format!(
        r#"{{"elements":[{{"from":[0,0,0],"to":[16,16,16],{element}"faces":{{"up":{face}}}}}]}}"#
    )
}

fn textured(fields: &str) -> String {
    element(fields, r##"{"texture":"#t"}"##)
}

pub fn accepted_models() -> Vec<(&'static str, String)> {
    let full = &FULL_ELEMENT[1..];
    vec![
        ("display-right-hands", DISPLAY_RIGHT.to_owned()),
        ("display-clamped-and-left", DISPLAY_CLAMPED.to_owned()),
        ("empty-object", "{}".to_owned()),
        (
            "parent-only",
            r#"{"parent":"minecraft:block/cube_all","textures":{"all":"minecraft:block/stone"}}"#.to_owned(),
        ),
        ("full-cube", FULL_ELEMENT.to_owned()),
        ("ambient-occlusion-false", format!(r#"{{"ambientocclusion":false,{full}"#)),
        ("gui-light-front", format!(r#"{{"gui_light":"front",{full}"#)),
        // A face may carry its own uv rectangle, a tint and a rotation.
        (
            "face-detail",
            element("", r##"{"texture":"#t","uv":[1,2,15,14],"tintindex":3,"rotation":270}"##),
        ),
        // Rotation blocks, both spellings.
        (
            "rotation-axis",
            textured(r#""rotation":{"origin":[8,8,8],"axis":"y","angle":22.5,"rescale":true},"#),
        ),
        ("rotation-euler", textured(r#""rotation":{"origin":[0,8,16],"x":22.5,"y":45,"z":0},"#)),
        ("shade-false", textured(r#""shade":false,"#)),
        ("light-emission", textured(r#""light_emission":15,"#)),
        // The element grammar permits reaching outside the block, up to a
        // limit.
        (
            "bounds-edge",
            r##"{"elements":[{"from":[-16,-16,-16],"to":[32,32,32],"faces":{"up":{"texture":"#t"}}}]}"##.to_owned(),
        ),
        (
            "textures-mixed",
            r##"{"textures":{"all":"minecraft:block/stone","side":"#all","particle":{"sprite":"minecraft:block/dirt","force_translucent":true}}}"##.to_owned(),
        ),
        (
            "multiple-elements",
            concat!(
                r##"{"elements":[{"from":[0,0,0],"to":[16,8,16],"faces":{"up":{"texture":"#a"}}},"##,
                r##"{"from":[0,8,0],"to":[16,16,16],"faces":{"down":{"texture":"#b"}}}]}"##,
            )
            .to_owned(),
        ),
    ]
}

pub fn rejected_models() -> Vec<(&'static str, String)> {
    vec![
        (
            "bounds-too-low",
            r##"{"elements":[{"from":[-17,0,0],"to":[16,16,16],"faces":{"up":{"texture":"#t"}}}]}"##.to_owned(),
        ),
        (
            "bounds-too-high",
            r##"{"elements":[{"from":[0,0,0],"to":[33,16,16],"faces":{"up":{"texture":"#t"}}}]}"##.to_owned(),
        ),
        (
            "short-vector",
            r##"{"elements":[{"from":[0,0],"to":[16,16,16],"faces":{"up":{"texture":"#t"}}}]}"##.to_owned(),
        ),
        ("no-faces", r#"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{}}]}"#.to_owned()),
        (
            "unknown-facing",
            r##"{"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"sideways":{"texture":"#t"}}}]}"##.to_owned(),
        ),
        ("shade-not-boolean", textured(r#""shade":1,"#)),
        ("light-emission-high", textured(r#""light_emission":16,"#)),
        ("light-emission-negative", textured(r#""light_emission":-1,"#)),
        ("rotation-empty", textured(r#""rotation":{"origin":[8,8,8]},"#)),
        ("rotation-bad-axis", textured(r#""rotation":{"origin":[8,8,8],"axis":"w","angle":22.5},"#)),
        ("uv-short", element("", r##"{"texture":"#t","uv":[1,2,3]}"##)),
        ("face-rotation-invalid", element("", r##"{"texture":"#t","rotation":45}"##)),
        ("missing-texture", element("", "{}")),
        ("not-an-object", "[]".to_owned()),
        ("trailing-comma", r#"{"elements":[],}"#.to_owned()),
    ]
}
