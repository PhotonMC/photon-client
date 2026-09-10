# P01.d — AABB kutu kurucuları ve üreticileri doğrulaması

`src/aabb.rs`, Java `AABB(BlockPos)`, `AABB(Vec3, Vec3)`, `AABB.unitCubeFromLowerCorner(Vec3)` ve `AABB.encapsulatingFullBlocks(BlockPos, BlockPos)` yollarını uygular. `examples/behavior_adapter.rs` bu üreticileri JSONL math adımlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 462 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **462/462** math senaryosunda eşleşti.
- Altı yeni senaryoda BlockPos ve Vec3 kurucuları, unit cube ve full block kapsamı; ters endpoint, signed zero, NaN, sonsuzluk ve integer `+1` taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5dc02c58d7cf60cb11342c786c6a4d91d4d727099b7ac1faef9f51a5b5f192df`.

Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Yerel raporlar: `compare-a1guzop4` (math debug), `compare-40tsawyv` (math release), `compare-uf5y7y3a` (birleşik debug) ve `compare-wrj9a6ex` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört AABB kurucusu/üreticisi ve mevcut AABB adım adaptörüyle sınırlıdır. Çoklu AABB clip sonraki dilimdir.
