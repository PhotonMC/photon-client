# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2140 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Yerel raporlar: `compare-ep8fiu6` (math debug), `compare-z9iwcisf` (math release), `compare-a21c2t4m` (birleşik debug), `compare-uwg2_ewv` (birleşik release) ve `verify-yscj0_fq` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
