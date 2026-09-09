# P01.d — AABB intersect doğrulaması

`src/aabb.rs`, Java `AABB.intersect` metodunun eksen başına `Math.max`/`Math.min` endpoint hesaplarını ve sonuç kurucusunun normalizasyonunu uygular. `examples/behavior_adapter.rs` `intersect` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 382 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **382/382** math senaryosunda eşleşti.
- Altı yeni senaryoda iç içe/kısmi örtüşme, ayrık kutu normalizasyonu, yüz teması, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1407/1407** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `067168f58b7d3c483bc26a542577bb0191d5cf0956a168754322b8805241bae5`.

Birleşik referans kimliği: `a2c69ab98e59ffb82410fd45282bc6e8c110fd13877ba87471c7bfcc29f23318`.

Yerel raporlar: `compare-tgse4o_r` (math debug), `compare-t286vz_w` (math release), `compare-smgo22vh` (birleşik debug) ve `compare-h2kr9uj0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.intersect(AABB)` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
