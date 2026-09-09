# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1281/1281** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1783/1783** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1783 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `723f54fe26ff0dfbc47ce949faa4fe41b2692595cd36cc69c22a1e67078641e6`. Birleşik referans kimliği: `3003c22db35c2a7187f3f945e00cc4b2e164ad5b8fa5e1a499feb0d6d57eb928`.

Yerel raporlar: `compare-gxfr9tv6` (math debug), `compare-inyn44tc` (math release), `compare-w1reqzv_` (birleşik debug), `compare-bxe0fyd7` (birleşik release) ve `verify-ep6ad6rk` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
