# P01.d — VoxelShape min/max doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.min(axis)` ve `max(axis)` yollarını `VoxelShape::min` ve `VoxelShape::max` ile uygular. `tests/behavior/java/MathOracle.java` altı Java sorgusunu, `examples/behavior_adapter.rs` ise aynı shape kurulumu ve ham bit snapshot'ını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 859 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **859/859** math senaryosunda eşleşti.
- Empty shape'in sonsuz sınırları ile ayrık ve negatif koordinatlı birleşimlerin X/Y/Z min/max bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1313/1313** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2`.

Birleşik referans kimliği: `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2`.

Yerel raporlar: `compare-_2ksshx8` (math debug), `compare-z3kwikqe` (math release), `compare-6n7i4k79` (birleşik debug) ve `compare-se85bnok` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik shape birleşimiyle ve shape başına altı global axis sorgusuyla sınırlıdır. `min(axis, b, c)` ve `max(axis, b, c)` slice sorguları, doğrudan voxel coordinate listeleri ve world collider toplama ayrı kapsamdır.
