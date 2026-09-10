# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`. Birleşik referans kimliği: `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57`.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-nqxvpkjm` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
