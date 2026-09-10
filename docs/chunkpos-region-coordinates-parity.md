# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1903/1903** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2405/2405** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2405 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `0cc821a10c9af88c8563c153e55d57b691a8c69e33d90a8bd3022a5b3410bfd0`. Birleşik referans kimliği: `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa`.

Yerel raporlar: `compare-xiw125c3` (math debug), `compare-v5h63z7q` (math release), `compare-chdvlz2_` (birleşik debug), `compare-oywxz0ky` (birleşik release) ve `verify-da0drjib` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
