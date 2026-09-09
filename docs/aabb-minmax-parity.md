# P01.d — AABB minmax doğrulaması

`src/aabb.rs`, Java `AABB.minmax` metodunun eksen başına `Math.min`/`Math.max` endpoint hesaplarını ve sonuç kurucusunun normalizasyonunu uygular. `examples/behavior_adapter.rs` `minmax` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 388 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **388/388** math senaryosunda eşleşti.
- Altı yeni senaryoda iç içe/kısmi örtüşme, ayrık kutu union'ı, yüz teması, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **916/916** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a5af108bd9af6e253a66e4f7d2ec37609cab22b050af5da64c032db59e8c71c2`.

Birleşik referans kimliği: `f3dac1b46227eaaf38e96f7c59cd9c377a91602c26a477ec26f348affc052a2f`.

Yerel raporlar: `compare-5tbvg_e5` (math debug), `compare-z90g_mxx` (math release), `compare-nmul5np1` (birleşik debug) ve `compare-1884mtz2` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.minmax(AABB)` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) ayrı dilimde tamamlandı ve çoklu AABB clip sonraki dilimdir.
