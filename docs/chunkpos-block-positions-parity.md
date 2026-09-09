# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1301/1301** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1803 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`. Birleşik referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Yerel raporlar: `compare-zq98bs3k` (math debug), `compare-zgg863t0` (math release), `compare-yz31bmcf` (birleşik debug), `compare-90dzj4qp` (birleşik release) ve `verify-l29xakrv` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
