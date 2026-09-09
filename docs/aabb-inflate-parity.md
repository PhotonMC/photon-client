# P01.d — AABB inflate doğrulaması

`src/aabb.rs`, Java `AABB.inflate` overload'larının endpoint hesaplarını ve scalar overload yönlendirmesini uygular. `examples/behavior_adapter.rs` iki inflate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 376 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **376/376** math senaryosunda eşleşti.
- Beş yeni senaryoda scalar pozitif/negatif miktar, üç eksenli karışık delta, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cc1e4e363fee8a21012ea0d56765f85c05556ced56a4bd8c330f603a71ce5656`.

Birleşik referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Yerel raporlar: `compare-7jscyfxl` (math debug), `compare-0cnexscj` (math release), `compare-p7m9abjy` (birleşik debug) ve `compare-vvlbkmvf` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.inflate` overload'ları ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) da ayrı dilimde tamamlandı, VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
