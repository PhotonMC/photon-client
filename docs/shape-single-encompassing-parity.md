# P01.d — VoxelShape.singleEncompassing doğrulaması

`src/shape.rs`, Java 26.2 `VoxelShape.singleEncompassing()` yolunu `VoxelShape::single_encompassing` ile uygular. `tests/behavior/java/MathOracle.java` Java yöntem çağrısını, `examples/behavior_adapter.rs` ise aynı OR kurulumu ve snapshot sınırını Rust API'sine bağlar. Davranış sınırları [referans sözleşmesinde](shape-single-encompassing-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Empty, temas eden birleşim ve aralığı içine alan ayrık birleşim sonuçlarının shape snapshot'ları karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Birleşik referans kimliği: `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654`.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug) ve `compare-4_se5ekn` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki box ile kurulan üç sentetik shape durumuyla sınırlıdır. Daha uzun birleşimler, doğrudan `min(axis, b, c)` ve `max(axis, b, c)` sorguları, `optimize` ve world collider toplama ayrı kapsamdır.
