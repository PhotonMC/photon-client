# P01.c — SectionPos blocksInside doğrulaması

`src/binary.rs`, SectionPos `blocksInside()` akışını Java ile aynı 16×16×16 inclusive BlockPos küpüne ve x/y/z traversal sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blocksInside için 3 özgün senaryo hazırlandı.
- Her senaryo 4096 BlockPos çıktısında origin, negatif section ve section→block sınırlarını doğrudan Java/Rust API'leriyle karşılaştırdı.
- Rust debug ve release adaptörleri güncel **1281/1281** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1783/1783** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `723f54fe26ff0dfbc47ce949faa4fe41b2692595cd36cc69c22a1e67078641e6`. Birleşik referans kimliği: `3003c22db35c2a7187f3f945e00cc4b2e164ad5b8fa5e1a499feb0d6d57eb928`.

Adapter hashleri: debug `f416a3dc75eb9f0867a64f3491324fd705ca6bb81111e3d3db3eb53edbb3b372`, release `ecc85f16c11ab8192117af94baf17849b1a9145a3957c325b239a401af8e812b`.

Yerel raporlar: `compare-gxfr9tv6` ve `compare-inyn44tc` (math debug/release), `compare-w1reqzv_` ve `compare-bxe0fyd7` (birleşik debug/release), `verify-hxxwgtez` (math) ve `verify-ep6ad6rk` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.blocksInside()` ve kullandığı `BlockPos.betweenClosedStream` yolunun sabit 4096 öğelik çıktısıyla sınırlıdır. Genel BlockPos range overload'ları, stream laziness/spliterator özellikleri ve `aroundAndAtBlockPos` akışı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
