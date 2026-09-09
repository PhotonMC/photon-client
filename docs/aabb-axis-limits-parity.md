# P01.d — AABB axis min/max doğrulaması

`src/aabb.rs`, Java `AABB.min(Direction.Axis)` ve `AABB.max(Direction.Axis)` seçimlerini uygular. `examples/behavior_adapter.rs` `min` ve `max` adımlarını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 418 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **418/418** math senaryosunda eşleşti.
- Altı yeni senaryoda üç eksen seçimi, min/max çağrı sırası, constructor normalizasyonu, signed zero ve NaN endpoint'leri karşılaştırıldı.
- Birleşik altı aile corpus'u **1013/1013** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `803f0124e49c7979da9fe752dd944ea1e0edce936d4d6afa25b3ab5358a5514c`.

Birleşik referans kimliği: `8bd229a580fc01d7c478c33923afb04c0fee7803869b7423f2d3ad4587821b9f`.

Yerel raporlar: `compare-3zt58eiq` (math debug), `compare-pk22hysr` (math release), `compare-n_x78ct2` (birleşik debug) ve `compare-pzl7bv8b` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.min/max(Direction.Axis)` ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; diğer overload'lar sonraki dilimlerdir; VoxelShape clip ayrı [kanıtta](shape-clip-parity.md) tamamlandı.
