# P01.c — ChunkPos validity sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos.isValid()` instance yolunu ve `isValid(int, int)` static yolunu Java'nın dünya koordinatı sınırıyla karşılaştırmaktır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:61-67`, `world/level/chunk/status/ChunkPyramid.java:57-58`, `core/BlockPos.java:48-56`, `core/SectionPos.java:81-83` ve `util/Mth.java:126-127` yolları incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Kaynak zincirindeki `BlockPos.MAX_HORIZONTAL_COORDINATE` değeri `33_554_431`, section dönüşümü `2_097_151` ve generation safety margin `90` chunk'tır; bu nedenle referans `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sınırı `2_097_061` olur. `ChunkPos.isValid` static alanı vanilla registry başlangıcına bağlı olduğundan oracle bu test yolunda `SharedConstants` sürümünü ayarlayıp vanilla bootstrap'ı başlatır; JSONL stdout akışı bootstrap sonrasında korunur.

Bu dilimde `ChunkPyramid` üretim davranışı, dünya sınırı uygulaması, chunk yaşam döngüsü ve `rangeClosed` ölçülmez.

## Davranış sözleşmesi

- Instance `isValid()`, kendi x/z değerlerini static `isValid(int, int)` yoluna verir.
- Static yol, `Mth.absMax(x, z) <= 2_097_061` karşılaştırmasını kullanır.
- `Mth.absMax`, Java `Math.abs(int)` ve `Math.max(int, int)` davranışını korur. Bu nedenle `Integer.MIN_VALUE` mutlak değeri negatif kalır ve diğer eksen sıfırken sonuç geçerli olabilir.
- Sınır karşılaştırması signed i32 değerleriyle yapılır; koordinatların `±2_097_061` sınırındaki davranışı ve sınır dışı sonucu ayrı gözlenir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos validity | 6 | Köken, pozitif/negatif sınır, sınır üstü, `Integer.MIN_VALUE` kenarı ve signed maximum |

Girdiler `family: "math"`, `operation: "chunkpos_validity"` ve signed decimal i32 `x`, `z` alanlarını kullanır. `value` altında instance ve static yolların boolean sonuçları `instance` ve `static` alanlarında bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::is_valid` ve `is_valid_coordinates`, `src/math.rs` içindeki `abs_max` ile sabitlenmiş Java 26.2 sınırını kullanır. `examples/behavior_adapter.rs` iki yolu JSONL'ye bağlar; `tests/behavior/java/MathOracle.java` Java instance/static çağrılarını doğrudan gözlemler.


## Client-parity doğrulaması

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
