# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`. Birleşik referans kimliği: `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57`.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-nqxvpkjm` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
