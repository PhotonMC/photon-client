# P01.d — AABB positions doğrulaması

`src/aabb.rs`, Java `AABB.getCenter()`, `getBottomCenter()`, `getMinPosition()` ve `getMaxPosition()` sorgularını uygular. `examples/behavior_adapter.rs` `positions` adımını, Java'nın Vec3 snapshot alanlarıyla birlikte JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 412 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **412/412** math senaryosunda eşleşti.
- Altı yeni senaryoda normalizasyon, midpoint aritmetiği, signed zero, sonsuzluk ve move sonrası sorgu sırası karşılaştırıldı.
- Birleşik altı aile corpus'u **884/884** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `130d0f8ac17bbfe7e8c5803d2c729ede2e7a2c581d12f4694d1e95cb513fcf99`.

Birleşik referans kimliği: `edc0c8336e5ffc79d1a744d6ddd301865a855ae2601c261b91b46cb5c453ddb9`.

Yerel raporlar: `compare-__ojm90z` (math debug), `compare-j97r39u3` (math release), `compare-cnmcrrgh` (birleşik debug) ve `compare-9l5dyku9` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB position getter'ı ve mevcut AABB adım adaptörüyle sınırlıdır. `collidedAlongVector`, diğer overload'lar ve çoklu AABB clip sonraki dilimlerdir.
