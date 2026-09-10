# P01.c — ChunkPos validity doğrulaması

`src/binary.rs`, Java `ChunkPos.isValid()` ve `isValid(int, int)` yollarını `Mth.absMax` ve `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sözleşmesiyle karşılar. Java oracle vanilla bootstrap gerektiren static sabiti kontrollü biçimde başlatır; Rust adaptörü aynı signed chunk girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos validity için 6 özgün senaryo hazırlandı.
- `±2_097_061` sınırı, sınır üstü değerler, signed i32 uçları ve Java `Integer.MIN_VALUE` mutlak değer kenarı instance/static sonuçlarıyla eşleşti.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2345 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Yerel raporlar: `compare-5_avciij` (math debug), `compare-e6e6agr4` (math release), `compare-gvo3eind` (birleşik debug), `compare-wmabbl5t` (birleşik release) ve `verify-vzenwlxd` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` validity instance/static yollarının sabit sınır karşılaştırmasını kapsar. `ChunkPyramid` üretim grafiğinin diğer davranışları, dünya sınırı, `rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
