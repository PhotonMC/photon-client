# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1703/1703** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2205 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`. Birleşik referans kimliği: `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162`.

Yerel raporlar: `compare-a8qhmnrl` (math debug), `compare-usm27y8z` (math release), `compare-8vwtvwwl` (birleşik debug), `compare-uey51w68` (birleşik release) ve `verify-l3irw60r` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
