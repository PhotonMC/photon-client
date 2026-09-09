# P01.d — VoxelShape getFaceShape doğrulaması

`src/shape.rs`, Java `VoxelShape.getFaceShape` sorgusunu `face_shape` ile uygular. Sınır voxel'inin hücrelerini seçilen yönde normalize edilmiş slice koordinatlarına taşır; `Shapes.block` sadeleştirmesi ve empty sonucu da aynı çağrı sınırında korunur. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [getFaceShape sözleşmesinde](shape-face-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Sekiz yeni yüz senaryosu empty/block sonuçlarını, cube-like eksen kısa yolunu, batı/doğu ve üst sınır slice'larını, eksik sınır sonucunu ve OR birleşimi yüzünü karşılaştırdı.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `22be0a4cb9fc67f18a020a971ff97c40b8df3f00566b14266f8ed2f447053381`.

Birleşik referans kimliği: `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55`.

Yerel raporlar: `compare-s1zm7hse` (math debug), `compare-nivpi0ij` (math release), `compare-ttkrbj5a` (birleşik debug) ve `compare-dz8xzceq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `getFaceShape` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-parity.md), diğer merger dalları ve world collider toplama sonraki dilimlerde doğrulanacaktır.
