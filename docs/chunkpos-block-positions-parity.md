# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1723/1723** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2225/2225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2225 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `efa8006dabc939417f5dae848830a2b524d131c4014d44a262a55fc501c1ba27`. Birleşik referans kimliği: `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161`.

Yerel raporlar: `compare-vdcnovbh` (math debug), `compare-4w3_lr0t` (math release), `compare-c5b8xhs4` (birleşik debug), `compare-mdqcgmw7` (birleşik release) ve `verify-r95rgxgt` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
