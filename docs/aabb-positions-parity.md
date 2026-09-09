# P01.d — AABB positions doğrulaması

`src/aabb.rs`, Java `AABB.getCenter()`, `getBottomCenter()`, `getMinPosition()` ve `getMaxPosition()` sorgularını uygular. `examples/behavior_adapter.rs` `positions` adımını, Java'nın Vec3 snapshot alanlarıyla birlikte JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 412 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **412/412** math senaryosunda eşleşti.
- Altı yeni senaryoda normalizasyon, midpoint aritmetiği, signed zero, sonsuzluk ve move sonrası sorgu sırası karşılaştırıldı.
- Birleşik altı aile corpus'u **1347/1347** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `130d0f8ac17bbfe7e8c5803d2c729ede2e7a2c581d12f4694d1e95cb513fcf99`.

Birleşik referans kimliği: `2ab36c787d438222d85bab3a2d9318b031512b21c923892daf5b2c0ad183f52c`.

Yerel raporlar: `compare-__ojm90z` (math debug), `compare-j97r39u3` (math release), `compare-xswxfvez` (birleşik debug) ve `compare-n8easogi` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB position getter'ı ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; diğer overload'lar sonraki dilimlerdir; VoxelShape clip ayrı [kanıtta](shape-clip-parity.md) tamamlandı.
