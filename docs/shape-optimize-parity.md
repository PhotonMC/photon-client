# P01.d — VoxelShape optimize ve getCoords doğrulaması

`src/shape.rs`, Java `VoxelShape.optimize()` yolunu `VoxelShape::optimize` ile; public `getCoords` gözlemini de `VoxelShape::coords` ile uygular. Optimize, mevcut `to_aabbs` merge sırasındaki kutuları yeniden kurup OR ile birleştirir. Davranış sınırları [referans sözleşmesinde](shape-optimize-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 619 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **619/619** math senaryosunda eşleşti.
- Üç yeni senaryoda empty sonuç, temas eden kutularda coordinate plane sadeleşmesi ve ayrık kutularda düzlem korunumu karşılaştırıldı.
- Birleşik altı aile corpus'u **1073/1073** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `6c664e0096c0ab12467c89aed7a906a49cbc22a2eb8b747dadf039bdd64c5f34`.

Birleşik referans kimliği: `a3f0878cf89549c06659e43bb88142eced597ada4cc7f8e7ceb2c84d2a0ef851`.

Yerel raporlar: `compare-vw9w8ekf` (math debug), `compare-p44xr16i` (math release), `compare-xj3gnj4z` (birleşik debug) ve `compare-u91oyin8` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "optimize"` shape senaryosuyla ve kutu/coordinate plane snapshot'larıyla sınırlıdır. [forAllEdges](shape-edges-parity.md), [forAllBoxes](shape-for-all-boxes-parity.md) ve [joinUnoptimized](shape-join-unoptimized-parity.md), doğrudan üretim shape'lerinin tüm index merger türleri ve world collider toplama sonraki kapsamdır.
