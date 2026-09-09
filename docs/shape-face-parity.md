# P01.d — VoxelShape getFaceShape doğrulaması

`src/shape.rs`, Java `VoxelShape.getFaceShape` sorgusunu `face_shape` ile uygular. Sınır voxel'inin hücrelerini seçilen yönde normalize edilmiş slice koordinatlarına taşır; `Shapes.block` sadeleştirmesi ve empty sonucu da aynı çağrı sınırında korunur. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [getFaceShape sözleşmesinde](shape-face-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Sekiz yeni yüz senaryosu empty/block sonuçlarını, cube-like eksen kısa yolunu, batı/doğu ve üst sınır slice'larını, eksik sınır sonucunu ve OR birleşimi yüzünü karşılaştırdı.
- Birleşik altı aile corpus'u **997/997** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `22be0a4cb9fc67f18a020a971ff97c40b8df3f00566b14266f8ed2f447053381`.

Birleşik referans kimliği: `d12db998718af31adab6351bbe129bcfd8d13fb33be8aa4270b7bc0546294c2d`.

Yerel raporlar: `compare-s1zm7hse` (math debug), `compare-nivpi0ij` (math release), `compare-m_cdl8w8` (birleşik debug) ve `compare-xp7qipgf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `getFaceShape` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-parity.md), diğer merger dalları ve world collider toplama sonraki dilimlerde doğrulanacaktır.
