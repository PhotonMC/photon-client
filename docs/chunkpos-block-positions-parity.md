# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1593/1593** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2095 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`. Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Yerel raporlar: `compare-gqgspijr` (math debug), `compare-z8zpkwnx` (math release), `compare-94x3ynna` (birleşik debug), `compare-2cii6gte` (birleşik release) ve `verify-lu6dvbz4` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
