# P01.c — ChunkPos region koordinatı doğrulaması

`src/binary.rs`, Java `ChunkPos` region ve region-local yardımcılarını signed shift, düşük 5 bit maskesi ve region min/max üretimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk/region girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Region koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Instance ve packed region sorguları, 0–31 local indeksleri, negatif chunk koordinatları ve region shift/add taşmaları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **941/941** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1443/1443** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1443 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7aaf16bdfc85f793bd25e0d4d234f6988f30874d554dc65c32ce3d0ec4f85363`. Birleşik referans kimliği: `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6`.

Yerel raporlar: `compare-9rd6f2k4` (math debug), `compare-azcslb8q` (math release), `compare-9epmwbb2` (birleşik debug), `compare-vdfamuam` (birleşik release) ve `verify-povrla2w` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, region x/z çıkarımını, packed region sorgularını, region-local x/z maskesini ve region'dan ilk/son chunk üretimini kapsar. ChunkPos packing'in tamamı, `isValid`, hash, block koordinatları, BlockPos dönüşümleri, range/distance yardımcıları ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
