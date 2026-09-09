# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **935/935** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1437/1437** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1437 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `b5430468b5bb806bec96f53c626329209858940d1a04da9bbfbedb16ab6b2dc0`. Birleşik referans kimliği: `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a`.

Yerel raporlar: `compare-96_s862r` (math debug), `compare-cbwrzmb5` (math release), `compare-fn36ocz_` (birleşik debug), `compare-a_hggp11` (birleşik release) ve `verify-k4za48of` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
