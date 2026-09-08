# P01.d — AABB contract doğrulaması

`src/aabb.rs`, Java `AABB.contract` çağrısında delta işaretine göre min veya max endpoint'inin içeri taşınmasını uygular. `examples/behavior_adapter.rs` contract adımını JSONL math senaryolarına bağlar. Inflate overload'ları ve intersect işlemi ayrı [inflate kanıtında](aabb-inflate-parity.md) ve [intersect kanıtında](aabb-intersect-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 371 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **371/371** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yön, karışık delta işaretleri, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **878/878** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3795d607b2b0b2e782a153bffb5f46917835f2df40d4a855940d960953a37113`.

Birleşik referans kimliği: `f5bf07c5ccc2b4ef8770f606fffd6c42755bcd78a9a333cc0919f86821fb2a3b`.

Yerel raporlar: `compare-cwwah065` (math debug), `compare-g3g2zbrw` (math release), `compare-6u6le94v` (birleşik debug) ve `compare-v7d9e5ad` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.contract` ve mevcut AABB adım adaptörüyle sınırlıdır. `inflate`, `intersect`, `minmax` ve `distanceToSqr` ayrı kanıt belgelerindedir; `ofSize`, `collidedAlongVector` ve çoklu AABB clip sonraki dilimlerdir.
