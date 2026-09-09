# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1113/1113** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`. Birleşik referans kimliği: `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461`.

Adapter hashleri: debug `50d15e54a8e4d9ee159108107a9950974c379fc5d72741265ebf2f9bccbac401`, release `cb77cf1f3a7ef94e14770a329f854e79896c6a2e8cca659b69c586f553d9fb9e`.

Yerel raporlar: `compare-y28xdodh` ve `compare-_7cfq3in` (math debug/release), `compare-cxv1an3r` ve `compare-1jjs0aun` (birleşik debug/release), `verify-uudhbkma` (math) ve `verify-71u5jh9x` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong`, `offset` ve range yardımcıları dışındaki node akışları ile `blocksInside`/`aroundAndAtBlockPos` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
