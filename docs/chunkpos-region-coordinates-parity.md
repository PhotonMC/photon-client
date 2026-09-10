# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Yerel raporlar: `compare-qtqxwt9b` (math debug), `compare-ebava42g` (math release), `compare-x0hkkvkv` (birleşik debug), `compare-x5wgwswa` (birleşik release) ve `verify-rc21vyy9` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
