# P01.d — VoxelShape optimize ve getCoords doğrulaması

`src/shape.rs`, Java `VoxelShape.optimize()` yolunu `VoxelShape::optimize` ile; public `getCoords` gözlemini de `VoxelShape::coords` ile uygular. Optimize, mevcut `to_aabbs` merge sırasındaki kutuları yeniden kurup OR ile birleştirir. Davranış sınırları [referans sözleşmesinde](shape-optimize-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 929 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **929/929** math senaryosunda eşleşti.
- Üç yeni senaryoda empty sonuç, temas eden kutularda coordinate plane sadeleşmesi ve ayrık kutularda düzlem korunumu karşılaştırıldı.
- Birleşik altı aile corpus'u **1431/1431** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88`.

Birleşik referans kimliği: `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88`.

Yerel raporlar: `compare-eiqs7rub` (math debug), `compare-t3kloypj` (math release), `compare-wfqzv7gl` (birleşik debug) ve `compare-_to6641m` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "optimize"` shape senaryosuyla ve kutu/coordinate plane snapshot'larıyla sınırlıdır. [forAllEdges](shape-edges-parity.md), [forAllBoxes](shape-for-all-boxes-parity.md) ve [joinUnoptimized](shape-join-unoptimized-parity.md), doğrudan üretim shape'lerinin tüm index merger türleri ve world collider toplama sonraki kapsamdır.
