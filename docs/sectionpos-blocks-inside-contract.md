# P01.c — SectionPos blocksInside sözleşmesi

Bu dilimde **3 Java davranış testi hazırlanmıştır**. Kapsam, bir SectionPos'ın içerdiği 16×16×16 BlockPos değerlerinin inclusive traversal sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:160-182,234-235` ve `.data/minecraft/source/net/minecraft/core/BlockPos.java:392-417` incelendi. Oracle `SectionPos.of(int, int, int)` ile section oluşturur, `blocksInside()` stream'ini tüketir ve her BlockPos'ı gözlemler; beklenen sonuçlar Rust çıktısından türetilmez.

`blocksInside`, section'ın minimum ve maksimum blok uçlarını `BlockPos.betweenClosedStream` yoluna verir. Her eksen 16 değer içerir. `BlockPos` cursor'ı x eksenini hızlı değiştirir; x satırı tamamlanınca y, y dilimi tamamlanınca z ilerler. Stream tüketilirken mutable cursor değerleri her öğe için JSON snapshot'ına kopyalanır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `blocksInside` | 3 | Köken, negatif section koordinatları ve section→block sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_blocks_inside"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value.positions`, 4096 BlockPos değerinin x/y/z signed decimal snapshot'larını Java traversal sırasıyla içerir.

Bu sözleşme sonlu 16×16×16 section çıktısına odaklanır. Java stream tembelliği, spliterator özellikleri ve mutable BlockPos nesnesinin kimliği bu dilime dahil değildir.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::blocks_inside`, `min_block_*` ve `max_block_*` uçlarını Java i32 wrapping aritmetiğiyle hesaplayıp aynı x-fastest/y-next/z-next sırada 4096 `BlockPos` üretir. `examples/behavior_adapter.rs` sonucu JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream'ini karşılaştırma oracle'ı olarak tüketir.
