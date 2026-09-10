# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **1578/1578** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2080/2080** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2080 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `845bee8faa9bd67215b4c4a330fee9163ef37c01793940d730ec7655ee66c654`. Birleşik referans kimliği: `e1fbdb47e2227938261f4e5bd3238926120301582d50b1af7c84fefac9844a94`.

Yerel raporlar: `compare-l4cf39d7` (math debug), `compare-jvzkn11s` (math release), `compare-oc9sa8gq` (birleşik debug), `compare-m9ue0q18` (birleşik release) ve `verify-np1asir2` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
