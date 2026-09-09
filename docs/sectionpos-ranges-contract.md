# P01.c — SectionPos range yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos'ın inclusive `betweenClosedStream`, `cube` ve `aroundChunk` yollarının sonlu çıktı sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:234-264` ve `.data/minecraft/source/net/minecraft/core/Cursor3D.java:8-45` incelendi. Oracle Java stream'lerini doğrudan tüketir; beklenen sonuçlar Rust çıktısından türetilmez.

`betweenClosedStream`, min ve max uçlarını dahil eder. Cursor, x eksenini hızlı değiştirir; x satırı tamamlanınca y, y dilimi tamamlanınca z ilerler. `cube`, merkez section'ın üç bileşeninden radius çıkarıp ekleyerek bu akışa bağlanır. `aroundChunk`, ChunkPos'ın x/z bileşenlerinden yatay uçları ve verilen min/max section değerlerinden dikey uçları kurar.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `betweenClosedStream` | 3 | Tek nokta, x/y/z sırası ve negatif koordinatlar |
| `cube` | 2 | Origin çevresinde radius ve i32 sınır değerli tek section |
| `aroundChunk` | 1 | Yatay radius ve iki section yüksekliği |

Girdiler `family: "math"`, `operation: "sectionpos_ranges"` ve `mode` alanını kullanır. `between` modu min/max x/y/z; `cube` modu center x/y/z ve radius; `around-chunk` modu chunk x/z, radius ve min/max section alanlarını taşır. `result.value.positions`, x/y/z koordinatlarını ve `asLong()` packed değerini Java traversal sırasıyla içerir.

Bu sözleşme sonlu ve pozitif Cursor3D boyutlarına sahip aralıkları ölçer. Java stream tembelliği, spliterator estimate değeri, paralel tüketim ve taşarak devasa veya geçersiz aralık oluşturan i32 çarpımları bu dilime dahil değildir.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::between_closed`, `cube` ve `around_chunk`, Java'nın x-fastest/y-next/z-next sırasını ve merkez uçlarının i32 wrapping aritmetiğini üretir. `examples/behavior_adapter.rs` üç yolu JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream'lerini karşılaştırma oracle'ı olarak tüketir.
