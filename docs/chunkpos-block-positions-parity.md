# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **947/947** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1449 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `173698891dc93629abbcd5ca3642ca5af04a5ddb10ffcbaa0610cf03306c320e`. Birleşik referans kimliği: `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363`.

Yerel raporlar: `compare-auuk33g2` (math debug), `compare-w_2b2455` (math release), `compare-tjsmsqxw` (birleşik debug), `compare-sks6l8sa` (birleşik release) ve `verify-egp3gvx_` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
