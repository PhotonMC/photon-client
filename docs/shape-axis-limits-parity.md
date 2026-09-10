# P01.d — VoxelShape min/max doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.min(axis)` ve `max(axis)` yollarını `VoxelShape::min` ve `VoxelShape::max` ile uygular. `tests/behavior/java/MathOracle.java` altı Java sorgusunu, `examples/behavior_adapter.rs` ise aynı shape kurulumu ve ham bit snapshot'ını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2216 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2216/2216** math senaryosunda eşleşti.
- Empty shape'in sonsuz sınırları ile ayrık ve negatif koordinatlı birleşimlerin X/Y/Z min/max bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2718/2718** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik shape birleşimiyle ve shape başına altı global axis sorgusuyla sınırlıdır. `min(axis, b, c)` ve `max(axis, b, c)` slice sorguları, doğrudan voxel coordinate listeleri ve world collider toplama ayrı kapsamdır.
