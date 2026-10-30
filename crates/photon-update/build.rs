//! Hands the target triple the client is built for to the code: it names the release files of
//! this platform on the host.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rustc-env=PHOTON_TARGET={target}");
    println!("cargo:rerun-if-changed=build.rs");
}
