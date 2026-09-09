# P01.d — VoxelShape getFaceShape doğrulaması

`src/shape.rs`, Java `VoxelShape.getFaceShape` sorgusunu `face_shape` ile uygular. Sınır voxel'inin hücrelerini seçilen yönde normalize edilmiş slice koordinatlarına taşır; `Shapes.block` sadeleştirmesi ve empty sonucu da aynı çağrı sınırında korunur. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [getFaceShape sözleşmesinde](shape-face-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 511 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **511/511** eşleşti.
- Sekiz yeni yüz senaryosu empty/block sonuçlarını, cube-like eksen kısa yolunu, batı/doğu ve üst sınır slice'larını, eksik sınır sonucunu ve OR birleşimi yüzünü karşılaştırdı.
- Birleşik altı aile corpus'u **965/965** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `22be0a4cb9fc67f18a020a971ff97c40b8df3f00566b14266f8ed2f447053381`.

Birleşik referans kimliği: `2105d50b3ac69a505c2685a91ad23c0857f85a9601098d2f6ab2b966e92919ef`.

Yerel raporlar: `compare-s1zm7hse` (math debug), `compare-nivpi0ij` (math release), `compare-4xmx_z9_` (birleşik debug) ve `compare-y1pppv24` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `getFaceShape` sorgusuyla sınırlıdır. `mergedFaceOccludes`, `faceShapeOccludes`, rotation, diğer merger dalları ve world collider toplama sonraki dilimlerde doğrulanacaktır.
