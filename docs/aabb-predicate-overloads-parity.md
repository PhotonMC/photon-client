# P01.d — AABB predicate overload doğrulaması

`src/aabb.rs`, Java `AABB.contains(Vec3)`, `AABB.intersects(double...)`, `AABB.intersects(Vec3, Vec3)` ve `AABB.intersects(BlockPos)` yollarını uygular. `examples/behavior_adapter.rs` dört predicate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 448 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **448/448** math senaryosunda eşleşti.
- Altı yeni senaryoda Vec3 contains sınırları, ham endpoint karşılaştırması, Vec3 normalizasyonu, BlockPos `+1` overflow'u, signed zero, NaN ve sonsuzluk karşılaştırıldı.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `88113a66373c63d81952d5e2c26347bce90060d5e60c0b41a5a1ffc6127377a7`.

Birleşik referans kimliği: `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc`.

Yerel raporlar: `compare-rnhr_0c0` (math debug), `compare-0r8ijg22` (math release), `compare-7wace8y9` (birleşik debug) ve `compare-ohni39xi` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB predicate overload'u ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; Çoklu AABB clip sonraki dilimdir.
