# P01.d — VoxelShape optimize ve getCoords doğrulaması

`src/shape.rs`, Java `VoxelShape.optimize()` yolunu `VoxelShape::optimize` ile; public `getCoords` gözlemini de `VoxelShape::coords` ile uygular. Optimize, mevcut `to_aabbs` merge sırasındaki kutuları yeniden kurup OR ile birleştirir. Davranış sınırları [referans sözleşmesinde](shape-optimize-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1723 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1723/1723** math senaryosunda eşleşti.
- Üç yeni senaryoda empty sonuç, temas eden kutularda coordinate plane sadeleşmesi ve ayrık kutularda düzlem korunumu karşılaştırıldı.
- Birleşik altı aile corpus'u **2225/2225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161`.

Birleşik referans kimliği: `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161`.

Yerel raporlar: `compare-vdcnovbh` (math debug), `compare-4w3_lr0t` (math release), `compare-c5b8xhs4` (birleşik debug) ve `compare-mdqcgmw7` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "optimize"` shape senaryosuyla ve kutu/coordinate plane snapshot'larıyla sınırlıdır. [forAllEdges](shape-edges-parity.md), [forAllBoxes](shape-for-all-boxes-parity.md) ve [joinUnoptimized](shape-join-unoptimized-parity.md), doğrudan üretim shape'lerinin tüm index merger türleri ve world collider toplama sonraki kapsamdır.
