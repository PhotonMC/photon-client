# P01.c — ChunkPos validity doğrulaması

`src/binary.rs`, Java `ChunkPos.isValid()` ve `isValid(int, int)` yollarını `Mth.absMax` ve `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sözleşmesiyle karşılar. Java oracle vanilla bootstrap gerektiren static sabiti kontrollü biçimde başlatır; Rust adaptörü aynı signed chunk girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos validity için 6 özgün senaryo hazırlandı.
- `±2_097_061` sınırı, sınır üstü değerler, signed i32 uçları ve Java `Integer.MIN_VALUE` mutlak değer kenarı instance/static sonuçlarıyla eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` validity instance/static yollarının sabit sınır karşılaştırmasını kapsar. `ChunkPyramid` üretim grafiğinin diğer davranışları, dünya sınırı, `rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
