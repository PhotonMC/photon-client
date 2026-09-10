# P01.c — SectionPos blocksInside doğrulaması

`src/binary.rs`, SectionPos `blocksInside()` akışını Java ile aynı 16×16×16 inclusive BlockPos küpüne ve x/y/z traversal sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blocksInside için 3 özgün senaryo hazırlandı.
- Her senaryo 4096 BlockPos çıktısında origin, negatif section ve section→block sınırlarını doğrudan Java/Rust API'leriyle karşılaştırdı.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Adapter hashleri: debug `4a9c082a05009b8ac6c37d12ee8814942a447b8e2e3a445f3ed95942b6a8356e`, release `750a8467b38e832bff19cfcaa43a5a4f94653270c98fd4a4d3f46248f0665f5d`.

Yerel raporlar: `compare-ep8fiu6` ve `compare-z9iwcisf` (math debug/release), `compare-a21c2t4m` ve `compare-uwg2_ewv` (birleşik debug/release), `verify-8mq2_8pm` (math) ve `verify-yscj0_fq` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.blocksInside()` ve kullandığı `BlockPos.betweenClosedStream` yolunun sabit 4096 öğelik çıktısıyla sınırlıdır. Genel BlockPos range overload'ları, stream laziness/spliterator özellikleri ve `aroundAndAtBlockPos` akışı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
