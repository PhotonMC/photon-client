# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1067/1067** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`. Birleşik referans kimliği: `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1`.

Adapter hashleri: debug `2e569fd5b282fc3c6202cf4a505416139c9e9cf90f6ebc711c0851970da8bc15`, release `72e7c71370f232b44263e273b357aab840c978acec80458d8e02a7b31aef41a1`.

Yerel raporlar: `compare-d3vh9fy0` ve `compare-cdl_uz9p` (math debug/release), `compare-b4u1_5xx` ve `compare-lz2txc82` (birleşik debug/release), `verify-cmxttwgw` (math) ve `verify-i4bptixz` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong`, `offset` ve range yardımcıları dışındaki node akışları ile `blocksInside`/`aroundAndAtBlockPos` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
