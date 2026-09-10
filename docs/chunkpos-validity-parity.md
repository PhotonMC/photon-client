# P01.c — ChunkPos validity doğrulaması

`src/binary.rs`, Java `ChunkPos.isValid()` ve `isValid(int, int)` yollarını `Mth.absMax` ve `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sözleşmesiyle karşılar. Java oracle vanilla bootstrap gerektiren static sabiti kontrollü biçimde başlatır; Rust adaptörü aynı signed chunk girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos validity için 6 özgün senaryo hazırlandı.
- `±2_097_061` sınırı, sınır üstü değerler, signed i32 uçları ve Java `Integer.MIN_VALUE` mutlak değer kenarı instance/static sonuçlarıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2140 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Yerel raporlar: `compare-ep8fiu6` (math debug), `compare-z9iwcisf` (math release), `compare-a21c2t4m` (birleşik debug), `compare-uwg2_ewv` (birleşik release) ve `verify-yscj0_fq` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` validity instance/static yollarının sabit sınır karşılaştırmasını kapsar. `ChunkPyramid` üretim grafiğinin diğer davranışları, dünya sınırı, `rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
