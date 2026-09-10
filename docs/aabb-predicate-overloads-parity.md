# P01.d — AABB predicate overload doğrulaması

`src/aabb.rs`, Java `AABB.contains(Vec3)`, `AABB.intersects(double...)`, `AABB.intersects(Vec3, Vec3)` ve `AABB.intersects(BlockPos)` yollarını uygular. `examples/behavior_adapter.rs` dört predicate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 448 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **448/448** math senaryosunda eşleşti.
- Altı yeni senaryoda Vec3 contains sınırları, ham endpoint karşılaştırması, Vec3 normalizasyonu, BlockPos `+1` overflow'u, signed zero, NaN ve sonsuzluk karşılaştırıldı.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `88113a66373c63d82285d5e2c26347bce90060d5e60c0b41a5a1ffc6127377a7`.

Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Yerel raporlar: `compare-rnhr_0c0` (math debug), `compare-0r8ijg22` (math release), `compare-qmmqnzwk` (birleşik debug) ve `compare-ogma6668` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB predicate overload'u ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; Çoklu AABB clip sonraki dilimdir.
