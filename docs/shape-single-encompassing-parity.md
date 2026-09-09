# P01.d — VoxelShape.singleEncompassing doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.singleEncompassing()` yolunu `VoxelShape::single_encompassing` ile uygular. `tests/behavior/java/MathOracle.java` Java yöntem çağrısını, `examples/behavior_adapter.rs` ise aynı OR kurulumu ve snapshot sınırını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-single-encompassing-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 650 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **650/650** math senaryosunda eşleşti.
- Empty, temas eden birleşim ve aralığı içine alan ayrık birleşim sonuçlarının shape snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1104/1104** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Birleşik referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Yerel raporlar: `compare-krjmldv0` (math debug), `compare-ufbmlufj` (math release), `compare-o61mnrf6` (birleşik debug) ve `compare-6fi73fsy` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki box ile kurulan üç sentetik shape durumuyla sınırlıdır. Daha uzun birleşimler, doğrudan `min(axis, b, c)` ve `max(axis, b, c)` sorguları, `optimize` ve world collider toplama ayrı kapsamdır.
