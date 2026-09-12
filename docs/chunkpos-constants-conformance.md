# P01.c — ChunkPos public sabitleri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **4 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos` sınıfının public static sabitlerinin doğrudan değerleridir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:32-43` incelendi. Oracle sabitleri doğrudan okur; beklenen sonuçlar Rust çıktısından türetilmez.

- `ZERO`, x ve z koordinatları sıfır olan ChunkPos'tur.
- `INVALID_CHUNK_POS`, iki `1_875_066` koordinatının ChunkPos packed long biçimindeki sentinel değeridir ve unsigned lowercase hex olarak taşınır.
- `REGION_SIZE`, bir region içindeki chunk eksen uzunluğu olarak `32` değerini taşır.
- `REGION_MAX_INDEX`, region içindeki son local index olarak `31` değerini taşır.

Private katsayılar, sabitlerin başka Java metodlarındaki tüketicileri ve invalid sentinel'ın oyun akışındaki anlamı bu dilimin dışındadır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `zero` | `name: "zero"` | `result.value.chunk_pos` içindeki x/z decimal string alanları |
| `invalid_chunk_pos` | `name: "invalid_chunk_pos"` | `result.value.packed` içindeki unsigned lowercase hex packed long |
| `region_size` | `name: "region_size"` | `result.value.value` içindeki decimal string |
| `region_max_index` | `name: "region_max_index"` | `result.value.value` içindeki decimal string |

Girdiler `family: "math"` ve `operation: "chunkpos_constant"` alanlarını kullanır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` associated constant'ları Java değerlerini taşır. `examples/behavior_adapter.rs` sabiti doğrudan Rust API'sinden sorgular; `tests/behavior/java/MathOracle.java` aynı adı Java sabitine bağlar.


## Client-parity doğrulaması

`src/binary.rs`, Java `ChunkPos` public static `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` sabitlerini associated constant olarak karşılar. Rust adapter sabitleri Java ile aynı JSON şekline taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos public sabitleri için 4 özgün senaryo hazırlandı.
- `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` değerleri doğrudan Java/Rust API'lerinden karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hash'leri: debug `97f6542ce3ae6bdc92e45b21488b106ff68a3970f00c179def9efd5e3e488ad7`, release `f551909bf9a55fa2fc2aae95704f10f5210192c5c00eaaba1c0afa39126f64609`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört public sabit sorgusu ve ilgili JSON çıktı şekilleriyle sınırlıdır. Private katsayılar, invalid sentinel'ın oyun akışındaki tüketicileri ve sabitlerin diğer istemci davranışlarındaki sonuçları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
