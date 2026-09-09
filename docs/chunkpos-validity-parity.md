# P01.c — ChunkPos validity doğrulaması

`src/binary.rs`, Java `ChunkPos.isValid()` ve `isValid(int, int)` yollarını `Mth.absMax` ve `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sözleşmesiyle karşılar. Java oracle vanilla bootstrap gerektiren static sabiti kontrollü biçimde başlatır; Rust adaptörü aynı signed chunk girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos validity için 6 özgün senaryo hazırlandı.
- `±2_097_061` sınırı, sınır üstü değerler, signed i32 uçları ve Java `Integer.MIN_VALUE` mutlak değer kenarı instance/static sonuçlarıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1067/1067** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1569 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`. Birleşik referans kimliği: `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1`.

Yerel raporlar: `compare-d3vh9fy0` (math debug), `compare-cdl_uz9p` (math release), `compare-b4u1_5xx` (birleşik debug), `compare-lz2txc82` (birleşik release) ve `verify-i4bptixz` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` validity instance/static yollarının sabit sınır karşılaştırmasını kapsar. `ChunkPyramid` üretim grafiğinin diğer davranışları, dünya sınırı, `rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
