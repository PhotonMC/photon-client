# P01.d — VoxelShape.singleEncompassing doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.singleEncompassing()` yolunu `VoxelShape::single_encompassing` ile uygular. `tests/behavior/java/MathOracle.java` Java yöntem çağrısını, `examples/behavior_adapter.rs` ise aynı OR kurulumu ve snapshot sınırını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-single-encompassing-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 685 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **685/685** math senaryosunda eşleşti.
- Empty, temas eden birleşim ve aralığı içine alan ayrık birleşim sonuçlarının shape snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1139/1139** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8`.

Birleşik referans kimliği: `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8`.

Yerel raporlar: `compare-2oxv9wmk` (math debug), `compare-54arzdz4` (math release), `compare-3tby77d3` (birleşik debug) ve `compare-nscg_s6g` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki box ile kurulan üç sentetik shape durumuyla sınırlıdır. Daha uzun birleşimler, doğrudan `min(axis, b, c)` ve `max(axis, b, c)` sorguları, `optimize` ve world collider toplama ayrı kapsamdır.
