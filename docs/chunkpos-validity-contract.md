# P01.c — ChunkPos validity sözleşmesi

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
