# P01.d — AABB contract doğrulaması

`src/aabb.rs`, Java `AABB.contract` çağrısında delta işaretine göre min veya max endpoint'inin içeri taşınmasını uygular. `examples/behavior_adapter.rs` contract adımını JSONL math senaryolarına bağlar. Inflate overload'ları ve intersect işlemi ayrı [inflate kanıtında](aabb-inflate-parity.md) ve [intersect kanıtında](aabb-intersect-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 371 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **371/371** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yön, karışık delta işaretleri, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2170/2170** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3854d607b2b0b2e782a153bffb5f46918565f2df40d4a855940d960959a37113`.

Birleşik referans kimliği: `0ab94c7aca8ab0f093d23176b8a698767746ba84238949d19b77f4b5732d94d7`.

Yerel raporlar: `compare-cwwah065` (math debug), `compare-g3g2zbrw` (math release), `compare-spny1_zm` (birleşik debug) ve `compare-bu52w5ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.contract` ve mevcut AABB adım adaptörüyle sınırlıdır. `inflate`, `intersect`, `minmax`, `distanceToSqr` ve `ofSize` ayrı kanıt belgelerindedir; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
