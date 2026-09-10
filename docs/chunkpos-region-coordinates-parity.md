# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1378/1378** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1880 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`. Birleşik referans kimliği: `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81`.

Yerel raporlar: `compare-_ohp5wb8` (math debug), `compare-bezugwpw` (math release), `compare-i4o7vofo` (birleşik debug), `compare-9vhvfokc` (birleşik release) ve `verify-kfxghdw3` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
