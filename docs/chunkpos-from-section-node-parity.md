# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **1623/1623** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2125/2125** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2125 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `ad2753dc2b6d8419f59cf4874d459be1bbd0128324ea6d37e1afd0b11e7854cf`. Birleşik referans kimliği: `14062308d34164b4d537528e94d89e360ce28e02bc1488fc00ccd7a15390b0eb`.

Yerel raporlar: `compare-ny3j5bmv` (math debug), `compare-8wh6otww` (math release), `compare-us5kn_kw` (birleşik debug), `compare-qnl_96jt` (birleşik release) ve `verify-m1vz27n6` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
