# P01.c — SectionPos spatial yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos'ın blok origin'i, merkez blok konumu ve ChunkPos dönüşümleridir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:202-215` incelendi. Oracle `SectionPos.origin()`, `center()` ve `chunk()` instance metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`origin`, section'ın minimum blok koordinatlarını döndürür. `center`, origin'e her eksende 8 ekleyen BlockPos offset yolunu kullanır. `chunk`, section x/z koordinatlarını ChunkPos'a aktarır; y düşürülür.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| Spatial dönüşümler | 6 | Köken, pozitif/negatif section'lar, shift sınırı ve i32 koordinat sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_spatial"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value` içinde `origin`, `center` BlockPos snapshot'ları ve `chunk` ChunkPos snapshot'ı bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::origin`, `center` ve `chunk`, daha önce doğrulanan min blok ve BlockPos offset yollarını kullanarak Java instance sonuçlarını taşır. `examples/behavior_adapter.rs` bu üç sonucu JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java metotlarını karşılaştırma oracle'ı olarak çağırır.
