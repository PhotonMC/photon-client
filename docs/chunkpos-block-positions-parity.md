# P01.c — ChunkPos BlockPos yardımcıları doğrulaması

`src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1963/1963** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2465 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`. Birleşik referans kimliği: `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573`.

Yerel raporlar: `compare-hiau72j0` (math debug), `compare-bttpme0o` (math release), `compare-k17hah5m` (birleşik debug), `compare-hiiz6u35` (birleşik release) ve `verify-82qi_x56` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
