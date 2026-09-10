# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1534/1534** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2036 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`. Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug), `compare-t1bsi_04` (birleşik release) ve `verify-zwp9iban` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
