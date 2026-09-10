# P01.d — VoxelShape min/max doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.min(axis)` ve `max(axis)` yollarını `VoxelShape::min` ve `VoxelShape::max` ile uygular. `tests/behavior/java/MathOracle.java` altı Java sorgusunu, `examples/behavior_adapter.rs` ise aynı shape kurulumu ve ham bit snapshot'ını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1534 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1534/1534** math senaryosunda eşleşti.
- Empty shape'in sonsuz sınırları ile ayrık ve negatif koordinatlı birleşimlerin X/Y/Z min/max bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug) ve `compare-t1bsi_04` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik shape birleşimiyle ve shape başına altı global axis sorgusuyla sınırlıdır. `min(axis, b, c)` ve `max(axis, b, c)` slice sorguları, doğrudan voxel coordinate listeleri ve world collider toplama ayrı kapsamdır.
