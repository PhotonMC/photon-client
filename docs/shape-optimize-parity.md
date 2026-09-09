# P01.d — VoxelShape optimize ve getCoords doğrulaması

`src/shape.rs`, Java `VoxelShape.optimize()` yolunu `VoxelShape::optimize` ile; public `getCoords` gözlemini de `VoxelShape::coords` ile uygular. Optimize, mevcut `to_aabbs` merge sırasındaki kutuları yeniden kurup OR ile birleştirir. Davranış sınırları [referans sözleşmesinde](shape-optimize-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 893 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **893/893** math senaryosunda eşleşti.
- Üç yeni senaryoda empty sonuç, temas eden kutularda coordinate plane sadeleşmesi ve ayrık kutularda düzlem korunumu karşılaştırıldı.
- Birleşik altı aile corpus'u **1375/1375** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `7c9538198afb189201c05520d1d9beb9d8b8ed1ecf818a5751ba7c9cf2774c30`.

Birleşik referans kimliği: `7c9538198afb189201c05520d1d9beb9d8b8ed1ecf818a5751ba7c9cf2774c30`.

Yerel raporlar: `compare-gnxkpf0j` (math debug), `compare-u696_8k6` (math release), `compare-wdb79blv` (birleşik debug) ve `compare-_hlw5kud` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "optimize"` shape senaryosuyla ve kutu/coordinate plane snapshot'larıyla sınırlıdır. [forAllEdges](shape-edges-parity.md), [forAllBoxes](shape-for-all-boxes-parity.md) ve [joinUnoptimized](shape-join-unoptimized-parity.md), doğrudan üretim shape'lerinin tüm index merger türleri ve world collider toplama sonraki kapsamdır.
