# P01.d — AABB axis min/max doğrulaması

`src/aabb.rs`, Java `AABB.min(Direction.Axis)` ve `AABB.max(Direction.Axis)` seçimlerini uygular. `examples/behavior_adapter.rs` `min` ve `max` adımlarını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 418 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **418/418** math senaryosunda eşleşti.
- Altı yeni senaryoda üç eksen seçimi, min/max çağrı sırası, constructor normalizasyonu, signed zero ve NaN endpoint'leri karşılaştırıldı.
- Birleşik altı aile corpus'u **1010/1010** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `803f0124e49c7979da9fe752dd944ea1e0edce936d4d6afa25b3ab5358a5514c`.

Birleşik referans kimliği: `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c`.

Yerel raporlar: `compare-3zt58eiq` (math debug), `compare-pk22hysr` (math release), `compare-sq3wafy1` (birleşik debug) ve `compare-1i1dj3ib` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.min/max(Direction.Axis)` ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı; diğer overload'lar sonraki dilimlerdir; VoxelShape clip ayrı [kanıtta](shape-clip-parity.md) tamamlandı.
