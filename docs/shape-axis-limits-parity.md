# P01.d — VoxelShape min/max doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.min(axis)` ve `max(axis)` yollarını `VoxelShape::min` ve `VoxelShape::max` ile uygular. `tests/behavior/java/MathOracle.java` altı Java sorgusunu, `examples/behavior_adapter.rs` ise aynı shape kurulumu ve ham bit snapshot'ını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 619 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **619/619** math senaryosunda eşleşti.
- Empty shape'in sonsuz sınırları ile ayrık ve negatif koordinatlı birleşimlerin X/Y/Z min/max bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1073/1073** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `6c664e0096c0ab12467c89aed7a906a49cbc22a2eb8b747dadf039bdd64c5f34`.

Birleşik referans kimliği: `a3f0878cf89549c06659e43bb88142eced597ada4cc7f8e7ceb2c84d2a0ef851`.

Yerel raporlar: `compare-vw9w8ekf` (math debug), `compare-p44xr16i` (math release), `compare-xj3gnj4z` (birleşik debug) ve `compare-u91oyin8` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik shape birleşimiyle ve shape başına altı global axis sorgusuyla sınırlıdır. `min(axis, b, c)` ve `max(axis, b, c)` slice sorguları, doğrudan voxel coordinate listeleri ve world collider toplama ayrı kapsamdır.
