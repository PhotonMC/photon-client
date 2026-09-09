# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1001/1001** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1503/1503** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1503 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7d65abdf4967bf0f7585e849fc4e6783027eaef1ef313c8b7d22e85922955d4a`. Birleşik referans kimliği: `2411fed740f5d2548545a86cee882fb42daaf0d9fbededf1b224dce54a261a5d`.

Yerel raporlar: `compare-12czaqmk` (math debug), `compare-fm0qnyjh` (math release), `compare-sakmrbet` (birleşik debug), `compare-pd6md7hs` (birleşik release) ve `verify-5t2xrfy6` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
