# P01.d — AABB minmax doğrulaması

`src/aabb.rs`, Java `AABB.minmax` metodunun eksen başına `Math.min`/`Math.max` endpoint hesaplarını ve sonuç kurucusunun normalizasyonunu uygular. `examples/behavior_adapter.rs` `minmax` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 388 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **388/388** math senaryosunda eşleşti.
- Altı yeni senaryoda iç içe/kısmi örtüşme, ayrık kutu union'ı, yüz teması, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1104/1104** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a5af108bd9af6e253a66e4f7d2ec37609cab22b050af5da64c032db59e8c71c2`.

Birleşik referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Yerel raporlar: `compare-5tbvg_e5` (math debug), `compare-z90g_mxx` (math release), `compare-o61mnrf6` (birleşik debug) ve `compare-6fi73fsy` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.minmax(AABB)` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
