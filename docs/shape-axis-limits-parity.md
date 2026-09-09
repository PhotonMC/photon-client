# P01.d — VoxelShape min/max doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.min(axis)` ve `max(axis)` yollarını `VoxelShape::min` ve `VoxelShape::max` ile uygular. `tests/behavior/java/MathOracle.java` altı Java sorgusunu, `examples/behavior_adapter.rs` ise aynı shape kurulumu ve ham bit snapshot'ını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 627 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **627/627** math senaryosunda eşleşti.
- Empty shape'in sonsuz sınırları ile ayrık ve negatif koordinatlı birleşimlerin X/Y/Z min/max bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1081/1081** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Birleşik referans kimliği: `3a6e954c19b1248d5d7303650ed4ef77ee1f388c99efa24b8f5091c2bf1cf8ec`.

Yerel raporlar: `compare-tg0rl658` (math debug), `compare-713odvz2` (math release), `compare-pnvz2myv` (birleşik debug) ve `compare-_bmqdjjb` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik shape birleşimiyle ve shape başına altı global axis sorgusuyla sınırlıdır. `min(axis, b, c)` ve `max(axis, b, c)` slice sorguları, doğrudan voxel coordinate listeleri ve world collider toplama ayrı kapsamdır.
