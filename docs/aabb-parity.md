# P01.d — AABB contract doğrulaması

`src/aabb.rs`, Java `AABB.contract` çağrısında delta işaretine göre min veya max endpoint'inin içeri taşınmasını uygular. `examples/behavior_adapter.rs` contract adımını JSONL math senaryolarına bağlar. Inflate overload'ları ve intersect işlemi ayrı [inflate kanıtında](aabb-inflate-parity.md) ve [intersect kanıtında](aabb-intersect-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 371 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **371/371** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yön, karışık delta işaretleri, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1010/1010** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3795d607b2b0b2e782a153bffb5f46917835f2df40d4a855940d960953a37113`.

Birleşik referans kimliği: `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c`.

Yerel raporlar: `compare-cwwah065` (math debug), `compare-g3g2zbrw` (math release), `compare-sq3wafy1` (birleşik debug) ve `compare-1i1dj3ib` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.contract` ve mevcut AABB adım adaptörüyle sınırlıdır. `inflate`, `intersect`, `minmax`, `distanceToSqr` ve `ofSize` ayrı kanıt belgelerindedir; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
