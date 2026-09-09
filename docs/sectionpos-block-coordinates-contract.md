# P01.c — SectionPos blok koordinatı yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir SectionPos section koordinatının her eksendeki minimum ve maksimum blok koordinatlarına çevrilmesidir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:128-134,160-180` incelendi. Oracle `SectionPos.of(int, int, int)` ile SectionPos oluşturur ve `minBlockX/Y/Z` ile `maxBlockX/Y/Z` instance metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Minimum blok koordinatı, ilgili section koordinatının Java i32 dört bit sola kaydırılmasıdır. Maksimum koordinat, aynı minimum değere 15 eklenerek üretilir. x, y ve z eksenleri bağımsızdır; shift ve toplama i32 sarma davranışı korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Blok koordinatı sınırları | 6 | Köken, pozitif/negatif section'lar, shift sınırı ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_block_coordinates"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value` içinde `min_block_x/y/z` ve `max_block_x/y/z` signed decimal string sonuçları bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::min_block_x`, `min_block_y`, `min_block_z`, `max_block_x`, `max_block_y` ve `max_block_z`, ortak section-to-block dönüşümünü ve Java i32 wrapping `+15` davranışını taşır. `examples/behavior_adapter.rs` SectionPos girdisini bu API'lere bağlar; `tests/behavior/java/MathOracle.java` Java instance yardımcılarını karşılaştırma oracle'ı olarak çağırır.
