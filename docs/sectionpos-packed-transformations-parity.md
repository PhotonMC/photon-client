# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1025/1025** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`. Birleşik referans kimliği: `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc`.

Adapter hashleri: debug `7a355c6fac53d7c505e7f5dbf7ec0f7727bb2221b2f1c958f9b0281847b97deb`, release `c0dd231fcb30a82a482e257a42bbb9e9c76d5d978f7d81154cf4667af21af968`.

Yerel raporlar: `compare-0dsoh6yn` ve `compare-wdh5yokb` (math debug/release), `compare-7wace8y9` ve `compare-ohni39xi` (birleşik debug/release), `verify-4pgflsld` (math) ve `verify-a7iqdyvs` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong` ve `offset` dışındaki node akışları ile `blocksInside`/`cube` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
