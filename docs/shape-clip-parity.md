# P01.d — VoxelShape clip doğrulaması

`src/shape.rs`, Java `VoxelShape.clip` çağrısındaki empty/kısa segment kontrollerini, test noktasıyla dolu hücre hızlı yolunu ve AABB fallback'ini uygular. `src/direction.rs` Java'nın float dot product yön seçimini taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [clip sözleşmesinde](shape-clip-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Altı yeni clip senaryosu ve içindeki 7 yön sorgusu; empty ve kısa segment sonuçlarını, doğrudan inside hit yönlerini, BlockPos ofsetini, kısmi kutu fallback'ini ve OR shape fallback'ini karşılaştırdı.
- Birleşik altı aile corpus'u **1070/1070** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `2ccf77d4a335bda75910148b74301ed5ceb8dd2e85877a9e63abbff58b164c50`.

Birleşik referans kimliği: `e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14`.

Yerel raporlar: `compare-al__d8s2` (math debug), `compare-q__13d9v` (math release), `compare-s3ki0jz5` (birleşik debug) ve `compare-fng7itjt` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `VoxelShape.clip` ve mevcut box/OR üretim yollarındaki hit sonuçlarıyla sınırlıdır. Occlusion, entity hedefleme, collider toplama, hit adaylarının dünya içindeki seçimi ve diğer shape factory/merger dalları sonraki dilimlerde doğrulanacaktır.
