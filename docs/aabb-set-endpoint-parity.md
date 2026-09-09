# P01.d — AABB endpoint setter doğrulaması

`src/aabb.rs`, Java `AABB.setMinX/Y/Z` ve `AABB.setMaxX/Y/Z` metotlarının tek endpoint'i değiştirip yeni AABB constructor'ı üzerinden normalize etmesini uygular. `examples/behavior_adapter.rs` altı setter adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 436 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **436/436** math senaryosunda eşleşti.
- Altı yeni senaryoda tüm min/max setter'ları, min ve max zincirleri, ters endpoint normalizasyonu, signed zero, sonsuzluk ve NaN karşılaştırıldı.
- Birleşik altı aile corpus'u **1061/1061** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `80065cae9313e93b5d45819cd549e3361f69569df75f00abcd445b0b2b65b506`.

Birleşik referans kimliği: `fa080fc05d94f4eec8044ce6e4989a288e586b7d5065e1926ba18b75170a3bdb`.

Yerel raporlar: `compare-53_472mo` (math debug), `compare-tta3azob` (math release), `compare-fgeq_2hx` (birleşik debug) ve `compare-b0c437h_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.setMinX/Y/Z` ve `AABB.setMaxX/Y/Z` setter'ları ile mevcut AABB adım adaptörüyle sınırlıdır. `AABB.move(BlockPos/Vec3/Vector3f)` için [ayrı kanıt](aabb-move-overloads-parity.md) vardır; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı, Çoklu AABB clip sonraki dilimdir.
