# P01.d — VoxelShape.singleEncompassing doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.singleEncompassing()` yolunu `VoxelShape::single_encompassing` ile uygular. `tests/behavior/java/MathOracle.java` Java yöntem çağrısını, `examples/behavior_adapter.rs` ise aynı OR kurulumu ve snapshot sınırını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-single-encompassing-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1450 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1450/1450** math senaryosunda eşleşti.
- Empty, temas eden birleşim ve aralığı içine alan ayrık birleşim sonuçlarının shape snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Yerel raporlar: `compare-qtqxwt9b` (math debug), `compare-ebava42g` (math release), `compare-x0hkkvkv` (birleşik debug) ve `compare-x5wgwswa` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki box ile kurulan üç sentetik shape durumuyla sınırlıdır. Daha uzun birleşimler, doğrudan `min(axis, b, c)` ve `max(axis, b, c)` sorguları, `optimize` ve world collider toplama ayrı kapsamdır.
