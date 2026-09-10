# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release) ve `verify-5ju3ok97` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
