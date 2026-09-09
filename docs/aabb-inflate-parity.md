# P01.d — AABB inflate doğrulaması

`src/aabb.rs`, Java `AABB.inflate` overload'larının endpoint hesaplarını ve scalar overload yönlendirmesini uygular. `examples/behavior_adapter.rs` iki inflate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 376 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **376/376** math senaryosunda eşleşti.
- Beş yeni senaryoda scalar pozitif/negatif miktar, üç eksenli karışık delta, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cc1e4e363fee8a21012ea0d56765f85c05556ced56a4bd8c330f603a71ce5656`.

Birleşik referans kimliği: `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55`.

Yerel raporlar: `compare-7jscyfxl` (math debug), `compare-0cnexscj` (math release), `compare-ttkrbj5a` (birleşik debug) ve `compare-dz8xzceq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.inflate` overload'ları ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-parity.md) da ayrı dilimde tamamlandı, VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
