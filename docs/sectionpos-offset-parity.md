# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1025/1025** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`. Birleşik referans kimliği: `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc`.

Adapter hashleri: debug `7a355c6fac53d7c505e7f5dbf7ec0f7727bb2221b2f1c958f9b0281847b97deb`, release `c0dd231fcb30a82a482e257a42bbb9e9c76d5d978f7d81154cf4667af21af968`.

Yerel raporlar: `compare-0dsoh6yn` ve `compare-wdh5yokb` (math debug/release), `compare-7wace8y9` ve `compare-ohni39xi` (birleşik debug/release), `verify-4pgflsld` (math) ve `verify-a7iqdyvs` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside`, `cube`, `aroundChunk`, `betweenClosedStream` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
