//! Compiles `shaders/*.{vert,frag}` to SPIR-V in `OUT_DIR`, where `src/vulkan/shaders.rs`
//! embeds them. A shader that does not compile fails the build, so the gate catches it.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let shaders = Path::new("shaders");
    println!("cargo:rerun-if-changed=shaders");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));
    let compiler = shaderc::Compiler::new().expect("a shader compiler");

    let mut files: Vec<PathBuf> = fs::read_dir(shaders)
        .expect("the shaders directory")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("vert" | "frag")
            )
        })
        .collect();
    files.sort();

    for path in files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("a file name");
        let kind = match path.extension().and_then(|e| e.to_str()) {
            Some("vert") => shaderc::ShaderKind::Vertex,
            _ => shaderc::ShaderKind::Fragment,
        };
        let source = fs::read_to_string(&path).expect("a readable shader");
        let mut options = shaderc::CompileOptions::new().expect("compile options");
        options.set_include_callback(|requested, _, _, _| {
            let included = shaders.join(requested);
            let content =
                fs::read_to_string(&included).map_err(|error| format!("{requested}: {error}"))?;
            Ok(shaderc::ResolvedInclude {
                resolved_name: included.to_string_lossy().into_owned(),
                content,
            })
        });
        let artifact = compiler
            .compile_into_spirv(&source, kind, name, "main", Some(&options))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        fs::write(out.join(format!("{name}.spv")), artifact.as_binary_u8())
            .expect("SPIR-V written");
    }
}
